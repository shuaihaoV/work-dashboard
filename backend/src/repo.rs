use std::collections::HashMap;

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};

use crate::error::AppError;
use crate::models::{
    ChannelOptionItem, ChannelStatsItem, ExtraStats, ModelOptionItem, ModelStatsItem,
    OverviewStats, RawModelStatsItem, TopRequestedModel, TopThroughputChannel, UserOptionItem,
    UserStatsItem,
};

#[derive(Debug, Clone, Default)]
pub struct StatsFilter {
    pub user_id: Option<i64>,
    pub model_name: Option<String>,
    pub channel_id: Option<i64>,
}

impl StatsFilter {
    fn model_name(&self) -> Option<&str> {
        self.model_name.as_deref()
    }
}

// new-api log types
const LOG_TYPE_CONSUME: i64 = 2;
const LOG_TYPE_ERROR: i64 = 5;

// SQL fragment to extract cache_tokens (read) from the `other` JSON text field
const CACHE_TOKENS_EXPR: &str = "COALESCE((NULLIF(other, '')::json->>'cache_tokens')::bigint, 0)";

// Real total input tokens, handling provider differences:
//   Anthropic: prompt_tokens excludes cache; total = prompt + creation + cache_read
//   OpenAI:    prompt_tokens already includes cache; total = prompt
const REAL_INPUT_EXPR: &str = concat!(
    "CASE WHEN ",
    "(NULLIF(other, '')::json->>'claude')::boolean IS TRUE",
    " OR (NULLIF(other, '')::json->>'cache_creation_tokens') IS NOT NULL",
    " OR COALESCE((NULLIF(other, '')::json->>'cache_tokens')::bigint, 0) > prompt_tokens",
    " OR (COALESCE((NULLIF(other, '')::json->>'cache_ratio')::numeric, 1) < 0.5",
    "     AND COALESCE((NULLIF(other, '')::json->>'cache_tokens')::bigint, 0) > 0)",
    " THEN prompt_tokens",
    "   + COALESCE((NULLIF(other, '')::json->>'cache_creation_tokens')::bigint, 0)",
    "   + COALESCE((NULLIF(other, '')::json->>'cache_tokens')::bigint, 0)",
    " ELSE prompt_tokens",
    " END",
);

fn channel_type_name(type_id: i64) -> &'static str {
    match type_id {
        1 => "OpenAI",
        2 => "Midjourney",
        3 => "Azure",
        4 => "Ollama",
        8 => "Custom",
        14 => "Anthropic",
        15 => "Baidu",
        16 | 26 => "Zhipu",
        17 => "Ali",
        20 => "OpenRouter",
        24 => "Gemini",
        25 => "Moonshot",
        27 => "Perplexity",
        31 => "LingYi",
        33 => "AWS",
        34 => "Cohere",
        35 => "MiniMax",
        40 => "SiliconFlow",
        41 => "VertexAI",
        42 => "Mistral",
        43 => "DeepSeek",
        _ => "Other",
    }
}

fn channel_status_name(status: i64) -> &'static str {
    match status {
        1 => "enabled",
        2 => "disabled",
        3 => "auto_disabled",
        _ => "unknown",
    }
}

// ── Channel info helper (from main DB) ──

#[derive(Debug, FromRow)]
struct ChannelInfoRow {
    id: i64,
    name: String,
    #[sqlx(rename = "type")]
    type_id: i64,
    status: i64,
}

async fn fetch_channel_map(pool: &PgPool) -> Result<HashMap<i64, ChannelInfoRow>, AppError> {
    let rows = sqlx::query_as::<_, ChannelInfoRow>("SELECT id, name, type, status FROM channels")
        .fetch_all(pool)
        .await?;

    Ok(rows.into_iter().map(|r| (r.id, r)).collect())
}

// ── Overview ──

#[derive(Debug, FromRow)]
struct OverviewRow {
    total_requests: i64,
    success_count: i64,
    total_input_tokens: i64,
    total_output_tokens: i64,
    total_cached_tokens: i64,
    total_quota: i64,
}

pub async fn fetch_overview(
    log_pool: &PgPool,
    period_start_utc: DateTime<Utc>,
    period_end_utc: DateTime<Utc>,
    filter: StatsFilter,
) -> Result<OverviewStats, AppError> {
    let start_ts = period_start_utc.timestamp();
    let end_ts = period_end_utc.timestamp();

    let sql = format!(
        r#"
SELECT
    COUNT(*) FILTER (WHERE type IN ($3, $4))::bigint AS total_requests,
    COUNT(*) FILTER (WHERE type = $3)::bigint AS success_count,
    COALESCE(SUM({real_input}) FILTER (WHERE type = $3), 0)::bigint AS total_input_tokens,
    COALESCE(SUM(completion_tokens) FILTER (WHERE type = $3), 0)::bigint AS total_output_tokens,
    COALESCE(SUM({cached}) FILTER (WHERE type = $3), 0)::bigint AS total_cached_tokens,
    COALESCE(SUM(quota) FILTER (WHERE type = $3), 0)::bigint AS total_quota
FROM logs
WHERE created_at >= $1
  AND created_at < $2
  AND type IN ($3, $4)
  AND ($5::bigint IS NULL OR user_id = $5)
  AND ($6::text IS NULL OR model_name = $6)
  AND ($7::bigint IS NULL OR channel_id = $7)
"#,
        cached = CACHE_TOKENS_EXPR,
        real_input = REAL_INPUT_EXPR
    );

    let row = sqlx::query_as::<_, OverviewRow>(&sql)
        .bind(start_ts)
        .bind(end_ts)
        .bind(LOG_TYPE_CONSUME)
        .bind(LOG_TYPE_ERROR)
        .bind(filter.user_id)
        .bind(filter.model_name())
        .bind(filter.channel_id)
        .fetch_one(log_pool)
        .await?;

    let success_rate = if row.total_requests > 0 {
        (row.success_count as f64 / row.total_requests as f64) * 100.0
    } else {
        0.0
    };

    Ok(OverviewStats {
        total_requests: row.total_requests,
        success_rate,
        total_input_tokens: row.total_input_tokens,
        total_output_tokens: row.total_output_tokens,
        total_cached_tokens: row.total_cached_tokens,
        total_quota: row.total_quota,
    })
}

// ── User Stats ──

#[derive(Debug, FromRow)]
struct UserStatsRow {
    user_id: i64,
    user_name: String,
    total_requests: i64,
    success_count: i64,
    input_tokens: i64,
    output_tokens: i64,
    cached_tokens: i64,
    quota_used: i64,
    avg_latency_ms: Option<f64>,
}

pub async fn fetch_user_stats(
    log_pool: &PgPool,
    period_start_utc: DateTime<Utc>,
    period_end_utc: DateTime<Utc>,
    filter: StatsFilter,
) -> Result<Vec<UserStatsItem>, AppError> {
    let start_ts = period_start_utc.timestamp();
    let end_ts = period_end_utc.timestamp();

    let sql = format!(
        r#"
SELECT
    user_id,
    COALESCE(MAX(username), '#' || user_id::text) AS user_name,
    COUNT(*)::bigint AS total_requests,
    COUNT(*) FILTER (WHERE type = $3)::bigint AS success_count,
    COALESCE(SUM({real_input}) FILTER (WHERE type = $3), 0)::bigint AS input_tokens,
    COALESCE(SUM(completion_tokens) FILTER (WHERE type = $3), 0)::bigint AS output_tokens,
    COALESCE(SUM({cached}) FILTER (WHERE type = $3), 0)::bigint AS cached_tokens,
    COALESCE(SUM(quota) FILTER (WHERE type = $3), 0)::bigint AS quota_used,
    AVG(use_time * 1000.0) FILTER (WHERE type = $3 AND use_time > 0)::double precision AS avg_latency_ms
FROM logs
WHERE created_at >= $1
  AND created_at < $2
  AND type IN ($3, $4)
  AND ($5::bigint IS NULL OR user_id = $5)
  AND ($6::text IS NULL OR model_name = $6)
  AND ($7::bigint IS NULL OR channel_id = $7)
GROUP BY user_id
HAVING COUNT(*) > 0
ORDER BY total_requests DESC, output_tokens DESC
LIMIT 200
"#,
        cached = CACHE_TOKENS_EXPR,
        real_input = REAL_INPUT_EXPR
    );

    let rows = sqlx::query_as::<_, UserStatsRow>(&sql)
        .bind(start_ts)
        .bind(end_ts)
        .bind(LOG_TYPE_CONSUME)
        .bind(LOG_TYPE_ERROR)
        .bind(filter.user_id)
        .bind(filter.model_name())
        .bind(filter.channel_id)
        .fetch_all(log_pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let success_rate = if row.total_requests > 0 {
                ((row.success_count as f64 / row.total_requests as f64) * 10000.0).round() / 100.0
            } else {
                0.0
            };
            UserStatsItem {
                user_id: row.user_id,
                user_name: row.user_name,
                total_requests: row.total_requests,
                success_rate,
                input_tokens: row.input_tokens,
                output_tokens: row.output_tokens,
                cached_tokens: row.cached_tokens,
                quota_used: row.quota_used,
                avg_latency_ms: row.avg_latency_ms,
            }
        })
        .collect())
}

#[derive(Debug, FromRow)]
struct UserOptionRow {
    user_id: i64,
    user_name: String,
}

pub async fn search_users(
    log_pool: &PgPool,
    keyword: Option<&str>,
) -> Result<Vec<UserOptionItem>, AppError> {
    let rows = sqlx::query_as::<_, UserOptionRow>(
        r#"
WITH ranked_users AS (
    SELECT
        user_id,
        COALESCE(MAX(NULLIF(username, '')), '#' || user_id::text) AS user_name,
        COUNT(*)::bigint AS request_count,
        MAX(created_at)::bigint AS last_seen_at
    FROM logs
    WHERE user_id IS NOT NULL
      AND (
        $1::text IS NULL
        OR user_id::text ILIKE '%' || $1 || '%'
        OR COALESCE(username, '') ILIKE '%' || $1 || '%'
      )
    GROUP BY user_id
)
SELECT
    user_id,
    user_name
FROM ranked_users
ORDER BY
    CASE
        WHEN $1::text IS NOT NULL AND user_id::text = $1 THEN 0
        WHEN $1::text IS NOT NULL AND user_name ILIKE $1 || '%' THEN 1
        WHEN $1::text IS NOT NULL AND user_name ILIKE '%' || $1 || '%' THEN 2
        ELSE 3
    END,
    last_seen_at DESC,
    request_count DESC,
    user_id DESC
LIMIT 20
"#,
    )
    .bind(keyword)
    .fetch_all(log_pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| UserOptionItem {
            user_id: row.user_id,
            user_name: row.user_name,
        })
        .collect())
}

#[derive(Debug, FromRow)]
struct ModelOptionRow {
    model_name: String,
}

pub async fn search_models(
    log_pool: &PgPool,
    keyword: Option<&str>,
) -> Result<Vec<ModelOptionItem>, AppError> {
    let rows = sqlx::query_as::<_, ModelOptionRow>(
        r#"
WITH ranked_models AS (
    SELECT
        model_name,
        COUNT(*)::bigint AS request_count,
        MAX(created_at)::bigint AS last_seen_at
    FROM logs
    WHERE model_name IS NOT NULL
      AND model_name <> ''
      AND (
        $1::text IS NULL
        OR model_name ILIKE '%' || $1 || '%'
      )
    GROUP BY model_name
)
SELECT
    model_name
FROM ranked_models
ORDER BY
    CASE
        WHEN $1::text IS NOT NULL AND model_name = $1 THEN 0
        WHEN $1::text IS NOT NULL AND model_name ILIKE $1 || '%' THEN 1
        WHEN $1::text IS NOT NULL AND model_name ILIKE '%' || $1 || '%' THEN 2
        ELSE 3
    END,
    last_seen_at DESC,
    request_count DESC,
    model_name ASC
LIMIT 20
"#,
    )
    .bind(keyword)
    .fetch_all(log_pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| ModelOptionItem {
            model_name: row.model_name,
        })
        .collect())
}

#[derive(Debug, FromRow)]
struct ChannelOptionRow {
    channel_id: i64,
    channel_name: String,
}

pub async fn search_channels(
    pool: &PgPool,
    keyword: Option<&str>,
) -> Result<Vec<ChannelOptionItem>, AppError> {
    let rows = sqlx::query_as::<_, ChannelOptionRow>(
        r#"
SELECT
    id AS channel_id,
    name AS channel_name
FROM channels
WHERE
    $1::text IS NULL
    OR id::text ILIKE '%' || $1 || '%'
    OR name ILIKE '%' || $1 || '%'
ORDER BY
    CASE
        WHEN $1::text IS NOT NULL AND id::text = $1 THEN 0
        WHEN $1::text IS NOT NULL AND name ILIKE $1 || '%' THEN 1
        WHEN $1::text IS NOT NULL AND name ILIKE '%' || $1 || '%' THEN 2
        ELSE 3
    END,
    (status = 1) DESC,
    id DESC
LIMIT 20
"#,
    )
    .bind(keyword)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| ChannelOptionItem {
            channel_id: row.channel_id,
            channel_name: row.channel_name,
        })
        .collect())
}

// ── Channel Stats ──
// logs from log_pool, channel info from main pool, merged in Rust

#[derive(Debug, FromRow)]
struct ChannelLogStatsRow {
    channel_id: i64,
    total_requests: i64,
    success_count: i64,
    input_tokens: i64,
    output_tokens: i64,
    cached_tokens: i64,
    quota_used: i64,
    avg_latency_ms: Option<f64>,
}

pub async fn fetch_channel_stats(
    pool: &PgPool,
    log_pool: &PgPool,
    period_start_utc: DateTime<Utc>,
    period_end_utc: DateTime<Utc>,
    filter: StatsFilter,
) -> Result<Vec<ChannelStatsItem>, AppError> {
    let start_ts = period_start_utc.timestamp();
    let end_ts = period_end_utc.timestamp();

    let sql = format!(
        r#"
SELECT
    channel_id,
    COUNT(*)::bigint AS total_requests,
    COUNT(*) FILTER (WHERE type = $3)::bigint AS success_count,
    COALESCE(SUM({real_input}) FILTER (WHERE type = $3), 0)::bigint AS input_tokens,
    COALESCE(SUM(completion_tokens) FILTER (WHERE type = $3), 0)::bigint AS output_tokens,
    COALESCE(SUM({cached}) FILTER (WHERE type = $3), 0)::bigint AS cached_tokens,
    COALESCE(SUM(quota) FILTER (WHERE type = $3), 0)::bigint AS quota_used,
    AVG(use_time * 1000.0) FILTER (WHERE type = $3 AND use_time > 0)::double precision AS avg_latency_ms
FROM logs
WHERE created_at >= $1
  AND created_at < $2
  AND type IN ($3, $4)
  AND ($5::bigint IS NULL OR user_id = $5)
  AND ($6::text IS NULL OR model_name = $6)
  AND ($7::bigint IS NULL OR channel_id = $7)
  AND channel_id IS NOT NULL
GROUP BY channel_id
HAVING COUNT(*) > 0
ORDER BY total_requests DESC, output_tokens DESC
LIMIT 300
"#,
        cached = CACHE_TOKENS_EXPR,
        real_input = REAL_INPUT_EXPR
    );

    let (log_rows, channel_map) = tokio::try_join!(
        async {
            sqlx::query_as::<_, ChannelLogStatsRow>(&sql)
                .bind(start_ts)
                .bind(end_ts)
                .bind(LOG_TYPE_CONSUME)
                .bind(LOG_TYPE_ERROR)
                .bind(filter.user_id)
                .bind(filter.model_name())
                .bind(filter.channel_id)
                .fetch_all(log_pool)
                .await
                .map_err(AppError::from)
        },
        fetch_channel_map(pool),
    )?;

    Ok(log_rows
        .into_iter()
        .map(|row| {
            let success_rate = if row.total_requests > 0 {
                ((row.success_count as f64 / row.total_requests as f64) * 10000.0).round() / 100.0
            } else {
                0.0
            };
            let (channel_name, channel_type, status) = match channel_map.get(&row.channel_id) {
                Some(ch) => (
                    ch.name.clone(),
                    channel_type_name(ch.type_id).to_string(),
                    channel_status_name(ch.status).to_string(),
                ),
                None => (
                    format!("#{}", row.channel_id),
                    "Other".to_string(),
                    "unknown".to_string(),
                ),
            };
            ChannelStatsItem {
                channel_id: row.channel_id,
                channel_name,
                channel_type,
                status,
                total_requests: row.total_requests,
                success_rate,
                input_tokens: row.input_tokens,
                output_tokens: row.output_tokens,
                cached_tokens: row.cached_tokens,
                quota_used: row.quota_used,
                avg_latency_ms: row.avg_latency_ms,
            }
        })
        .collect())
}

// ── Model Stats ──

#[derive(Debug, FromRow)]
struct ModelStatsRow {
    model_name: String,
    total_requests: i64,
    success_count: i64,
    input_tokens: i64,
    output_tokens: i64,
    cached_tokens: i64,
    quota_used: i64,
    avg_latency_ms: Option<f64>,
}

pub async fn fetch_model_stats(
    log_pool: &PgPool,
    period_start_utc: DateTime<Utc>,
    period_end_utc: DateTime<Utc>,
    filter: StatsFilter,
) -> Result<Vec<ModelStatsItem>, AppError> {
    let start_ts = period_start_utc.timestamp();
    let end_ts = period_end_utc.timestamp();

    let sql = format!(
        r#"
SELECT
    model_name,
    COUNT(*)::bigint AS total_requests,
    COUNT(*) FILTER (WHERE type = $3)::bigint AS success_count,
    COALESCE(SUM({real_input}) FILTER (WHERE type = $3), 0)::bigint AS input_tokens,
    COALESCE(SUM(completion_tokens) FILTER (WHERE type = $3), 0)::bigint AS output_tokens,
    COALESCE(SUM({cached}) FILTER (WHERE type = $3), 0)::bigint AS cached_tokens,
    COALESCE(SUM(quota) FILTER (WHERE type = $3), 0)::bigint AS quota_used,
    AVG(use_time * 1000.0) FILTER (WHERE type = $3 AND use_time > 0)::double precision AS avg_latency_ms
FROM logs
WHERE created_at >= $1
  AND created_at < $2
  AND type IN ($3, $4)
  AND ($5::bigint IS NULL OR user_id = $5)
  AND ($6::text IS NULL OR model_name = $6)
  AND ($7::bigint IS NULL OR channel_id = $7)
  AND model_name IS NOT NULL AND model_name <> ''
GROUP BY model_name
ORDER BY total_requests DESC, output_tokens DESC
LIMIT 300
"#,
        cached = CACHE_TOKENS_EXPR,
        real_input = REAL_INPUT_EXPR
    );

    let rows = sqlx::query_as::<_, ModelStatsRow>(&sql)
        .bind(start_ts)
        .bind(end_ts)
        .bind(LOG_TYPE_CONSUME)
        .bind(LOG_TYPE_ERROR)
        .bind(filter.user_id)
        .bind(filter.model_name())
        .bind(filter.channel_id)
        .fetch_all(log_pool)
        .await?;

    Ok(rows
        .into_iter()
        .map(|row| {
            let success_rate = if row.total_requests > 0 {
                ((row.success_count as f64 / row.total_requests as f64) * 10000.0).round() / 100.0
            } else {
                0.0
            };
            ModelStatsItem {
                model_name: row.model_name,
                total_requests: row.total_requests,
                success_rate,
                input_tokens: row.input_tokens,
                output_tokens: row.output_tokens,
                cached_tokens: row.cached_tokens,
                quota_used: row.quota_used,
                avg_latency_ms: row.avg_latency_ms,
            }
        })
        .collect())
}

// ── Raw Model Stats ──
// logs from log_pool, channel info from main pool, merged in Rust

#[derive(Debug, FromRow)]
struct RawModelLogStatsRow {
    model_name: String,
    channel_id: i64,
    total_requests: i64,
    success_count: i64,
    input_tokens: i64,
    output_tokens: i64,
    cached_tokens: i64,
    quota_used: i64,
    avg_latency_ms: Option<f64>,
}

pub async fn fetch_raw_model_stats(
    pool: &PgPool,
    log_pool: &PgPool,
    period_start_utc: DateTime<Utc>,
    period_end_utc: DateTime<Utc>,
    filter: StatsFilter,
) -> Result<Vec<RawModelStatsItem>, AppError> {
    let start_ts = period_start_utc.timestamp();
    let end_ts = period_end_utc.timestamp();

    let sql = format!(
        r#"
SELECT
    model_name,
    channel_id,
    COUNT(*)::bigint AS total_requests,
    COUNT(*) FILTER (WHERE type = $3)::bigint AS success_count,
    COALESCE(SUM({real_input}) FILTER (WHERE type = $3), 0)::bigint AS input_tokens,
    COALESCE(SUM(completion_tokens) FILTER (WHERE type = $3), 0)::bigint AS output_tokens,
    COALESCE(SUM({cached}) FILTER (WHERE type = $3), 0)::bigint AS cached_tokens,
    COALESCE(SUM(quota) FILTER (WHERE type = $3), 0)::bigint AS quota_used,
    AVG(use_time * 1000.0) FILTER (WHERE type = $3 AND use_time > 0)::double precision AS avg_latency_ms
FROM logs
WHERE created_at >= $1
  AND created_at < $2
  AND type IN ($3, $4)
  AND ($5::bigint IS NULL OR user_id = $5)
  AND ($6::text IS NULL OR model_name = $6)
  AND ($7::bigint IS NULL OR channel_id = $7)
  AND model_name IS NOT NULL AND model_name <> ''
  AND channel_id IS NOT NULL
GROUP BY model_name, channel_id
ORDER BY total_requests DESC, output_tokens DESC
LIMIT 500
"#,
        cached = CACHE_TOKENS_EXPR,
        real_input = REAL_INPUT_EXPR
    );

    let (log_rows, channel_map) = tokio::try_join!(
        async {
            sqlx::query_as::<_, RawModelLogStatsRow>(&sql)
                .bind(start_ts)
                .bind(end_ts)
                .bind(LOG_TYPE_CONSUME)
                .bind(LOG_TYPE_ERROR)
                .bind(filter.user_id)
                .bind(filter.model_name())
                .bind(filter.channel_id)
                .fetch_all(log_pool)
                .await
                .map_err(AppError::from)
        },
        fetch_channel_map(pool),
    )?;

    Ok(log_rows
        .into_iter()
        .map(|row| {
            let success_rate = if row.total_requests > 0 {
                ((row.success_count as f64 / row.total_requests as f64) * 10000.0).round() / 100.0
            } else {
                0.0
            };
            let (channel_name, channel_type) = match channel_map.get(&row.channel_id) {
                Some(ch) => (ch.name.clone(), channel_type_name(ch.type_id).to_string()),
                None => (format!("#{}", row.channel_id), "Other".to_string()),
            };
            RawModelStatsItem {
                model_name: row.model_name,
                channel_id: row.channel_id,
                channel_name,
                channel_type,
                total_requests: row.total_requests,
                success_rate,
                input_tokens: row.input_tokens,
                output_tokens: row.output_tokens,
                cached_tokens: row.cached_tokens,
                quota_used: row.quota_used,
                avg_latency_ms: row.avg_latency_ms,
            }
        })
        .collect())
}

// ── Extra Stats ──

#[derive(Debug, FromRow)]
struct TopThroughputLogRow {
    channel_id: i64,
    tokens_per_second: f64,
    request_count: i64,
}

async fn fetch_top_throughput_channels(
    pool: &PgPool,
    log_pool: &PgPool,
    period_start_utc: DateTime<Utc>,
    period_end_utc: DateTime<Utc>,
    filter: StatsFilter,
) -> Result<Vec<TopThroughputChannel>, AppError> {
    let start_ts = period_start_utc.timestamp();
    let end_ts = period_end_utc.timestamp();

    let log_rows = sqlx::query_as::<_, TopThroughputLogRow>(
        r#"
WITH channel_perf AS (
    SELECT
        l.channel_id,
        COUNT(*)::bigint AS request_count,
        (
            SUM(l.completion_tokens)::double precision
            / NULLIF(SUM(l.use_time)::double precision, 0)
        ) AS tokens_per_second
    FROM logs l
    WHERE l.created_at >= $1
      AND l.created_at < $2
      AND l.type = $3
      AND ($4::bigint IS NULL OR l.user_id = $4)
      AND ($5::text IS NULL OR l.model_name = $5)
      AND ($6::bigint IS NULL OR l.channel_id = $6)
      AND l.channel_id IS NOT NULL
      AND l.use_time > 0
      AND l.completion_tokens > 0
    GROUP BY l.channel_id
    HAVING COUNT(*) >= 5
)
SELECT
    cp.channel_id,
    ROUND(cp.tokens_per_second::numeric, 2)::double precision AS tokens_per_second,
    cp.request_count
FROM channel_perf cp
WHERE cp.tokens_per_second IS NOT NULL
ORDER BY cp.tokens_per_second DESC, cp.request_count DESC
LIMIT 5
"#,
    )
    .bind(start_ts)
    .bind(end_ts)
    .bind(LOG_TYPE_CONSUME)
    .bind(filter.user_id)
    .bind(filter.model_name())
    .bind(filter.channel_id)
    .fetch_all(log_pool)
    .await?;

    let channel_map = fetch_channel_map(pool).await?;

    Ok(log_rows
        .into_iter()
        .map(|row| {
            let channel_name = channel_map
                .get(&row.channel_id)
                .map(|ch| ch.name.clone())
                .unwrap_or_else(|| format!("#{}", row.channel_id));
            TopThroughputChannel {
                channel_id: row.channel_id,
                channel_name,
                tokens_per_second: row.tokens_per_second,
                request_count: row.request_count,
            }
        })
        .collect())
}

#[derive(Debug, FromRow)]
struct TopRequestedModelRow {
    model_name: String,
    total_requests: i64,
}

async fn fetch_top_requested_models(
    log_pool: &PgPool,
    period_start_utc: DateTime<Utc>,
    period_end_utc: DateTime<Utc>,
    filter: StatsFilter,
) -> Result<Vec<TopRequestedModel>, AppError> {
    let start_ts = period_start_utc.timestamp();
    let end_ts = period_end_utc.timestamp();

    let rows = sqlx::query_as::<_, TopRequestedModelRow>(
        r#"
SELECT
    model_name,
    COUNT(*)::bigint AS total_requests
FROM logs
WHERE created_at >= $1
  AND created_at < $2
  AND type = $3
  AND ($4::bigint IS NULL OR user_id = $4)
  AND ($5::text IS NULL OR model_name = $5)
  AND ($6::bigint IS NULL OR channel_id = $6)
  AND model_name IS NOT NULL AND model_name <> ''
GROUP BY model_name
ORDER BY total_requests DESC
LIMIT 5
"#,
    )
    .bind(start_ts)
    .bind(end_ts)
    .bind(LOG_TYPE_CONSUME)
    .bind(filter.user_id)
    .bind(filter.model_name())
    .bind(filter.channel_id)
    .fetch_all(log_pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|row| TopRequestedModel {
            model_name: row.model_name,
            total_requests: row.total_requests,
        })
        .collect())
}

pub async fn fetch_extra_stats(
    pool: &PgPool,
    log_pool: &PgPool,
    period_start_utc: DateTime<Utc>,
    period_end_utc: DateTime<Utc>,
    filter: StatsFilter,
) -> Result<ExtraStats, AppError> {
    let (top_throughput_channels, top_requested_models) = tokio::try_join!(
        fetch_top_throughput_channels(
            pool,
            log_pool,
            period_start_utc,
            period_end_utc,
            filter.clone()
        ),
        fetch_top_requested_models(log_pool, period_start_utc, period_end_utc, filter),
    )?;

    Ok(ExtraStats {
        top_throughput_channels,
        top_requested_models,
    })
}
