use std::future::Future;
use std::sync::Arc;
use std::time::Duration;

use axum::extract::{Query, State};
use axum::Json;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::json;
use sqlx::PgPool;

use crate::cache::ApiCache;
use crate::error::AppError;
use crate::models::{
    ApiResponse, ChannelOptionItem, ChannelStatsItem, ExtraStats, ModelOptionItem, ModelStatsItem,
    OverviewStats, RawModelStatsItem, UserOptionItem, UserStatsItem,
};
use crate::period::{parse_custom_window, PeriodWindow};
use crate::repo::{self, StatsFilter};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
    pub log_pool: PgPool,
    pub cache: Arc<ApiCache>,
    pub cache_ttl: Duration,
}

#[derive(Debug, Deserialize)]
pub struct RangeQuery {
    from: Option<String>,
    to: Option<String>,
    period: Option<String>,
    #[serde(alias = "userId")]
    user_id: Option<i64>,
    #[serde(alias = "modelId")]
    model_id: Option<String>,
    #[serde(alias = "channelId")]
    channel_id: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct UserSearchQuery {
    q: Option<String>,
}

pub async fn healthz() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}

pub async fn get_overview(
    State(state): State<AppState>,
    Query(query): Query<RangeQuery>,
) -> Result<Json<ApiResponse<OverviewStats>>, AppError> {
    let log_pool = state.log_pool.clone();
    let filter = query.stats_filter();
    respond_cached(&state, "overview", &query, move |window| async move {
        repo::fetch_overview(&log_pool, window.start_utc, window.end_utc, filter).await
    })
    .await
}

pub async fn get_user_stats(
    State(state): State<AppState>,
    Query(query): Query<RangeQuery>,
) -> Result<Json<ApiResponse<Vec<UserStatsItem>>>, AppError> {
    let log_pool = state.log_pool.clone();
    let filter = query.stats_filter();
    respond_cached(&state, "stats-users", &query, move |window| async move {
        repo::fetch_user_stats(&log_pool, window.start_utc, window.end_utc, filter).await
    })
    .await
}

pub async fn get_channel_stats(
    State(state): State<AppState>,
    Query(query): Query<RangeQuery>,
) -> Result<Json<ApiResponse<Vec<ChannelStatsItem>>>, AppError> {
    let pool = state.pool.clone();
    let log_pool = state.log_pool.clone();
    let filter = query.stats_filter();
    respond_cached(&state, "stats-channels", &query, move |window| async move {
        repo::fetch_channel_stats(&pool, &log_pool, window.start_utc, window.end_utc, filter).await
    })
    .await
}

pub async fn get_model_stats(
    State(state): State<AppState>,
    Query(query): Query<RangeQuery>,
) -> Result<Json<ApiResponse<Vec<ModelStatsItem>>>, AppError> {
    let log_pool = state.log_pool.clone();
    let filter = query.stats_filter();
    respond_cached(&state, "stats-models", &query, move |window| async move {
        repo::fetch_model_stats(&log_pool, window.start_utc, window.end_utc, filter).await
    })
    .await
}

pub async fn get_raw_model_stats(
    State(state): State<AppState>,
    Query(query): Query<RangeQuery>,
) -> Result<Json<ApiResponse<Vec<RawModelStatsItem>>>, AppError> {
    let pool = state.pool.clone();
    let log_pool = state.log_pool.clone();
    let filter = query.stats_filter();
    respond_cached(
        &state,
        "stats-raw-models",
        &query,
        move |window| async move {
            repo::fetch_raw_model_stats(&pool, &log_pool, window.start_utc, window.end_utc, filter)
                .await
        },
    )
    .await
}

pub async fn get_extra_stats(
    State(state): State<AppState>,
    Query(query): Query<RangeQuery>,
) -> Result<Json<ApiResponse<ExtraStats>>, AppError> {
    let pool = state.pool.clone();
    let log_pool = state.log_pool.clone();
    let filter = query.stats_filter();
    respond_cached(&state, "stats-extra", &query, move |window| async move {
        repo::fetch_extra_stats(&pool, &log_pool, window.start_utc, window.end_utc, filter).await
    })
    .await
}

pub async fn search_users(
    State(state): State<AppState>,
    Query(query): Query<UserSearchQuery>,
) -> Result<Json<ApiResponse<Vec<UserOptionItem>>>, AppError> {
    let keyword = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let users = repo::search_users(&state.log_pool, keyword).await?;
    Ok(Json(ApiResponse::new(state.cache_ttl.as_secs(), users)))
}

pub async fn search_models(
    State(state): State<AppState>,
    Query(query): Query<UserSearchQuery>,
) -> Result<Json<ApiResponse<Vec<ModelOptionItem>>>, AppError> {
    let keyword = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let models = repo::search_models(&state.log_pool, keyword).await?;
    Ok(Json(ApiResponse::new(state.cache_ttl.as_secs(), models)))
}

pub async fn search_channels(
    State(state): State<AppState>,
    Query(query): Query<UserSearchQuery>,
) -> Result<Json<ApiResponse<Vec<ChannelOptionItem>>>, AppError> {
    let keyword = query
        .q
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty());
    let channels = repo::search_channels(&state.pool, keyword).await?;
    Ok(Json(ApiResponse::new(state.cache_ttl.as_secs(), channels)))
}

async fn respond_cached<T, F, Fut>(
    state: &AppState,
    prefix: &str,
    query: &RangeQuery,
    fetcher: F,
) -> Result<Json<ApiResponse<T>>, AppError>
where
    T: Clone + Serialize + DeserializeOwned,
    F: FnOnce(PeriodWindow) -> Fut,
    Fut: Future<Output = Result<T, AppError>>,
{
    let window = resolve_window(query)?;
    let cache_key = format!(
        "{prefix}:custom:{}:{}:{}",
        floor_to_minute_ts(window.start_utc.timestamp()),
        floor_to_minute_ts(window.end_utc.timestamp()),
        cache_filter_segment(query),
    );

    if let Some(cached) = state.cache.get::<ApiResponse<T>>(&cache_key).await {
        return Ok(Json(cached));
    }

    let payload = fetcher(window).await?;
    let response = ApiResponse::new(state.cache_ttl.as_secs(), payload);
    state
        .cache
        .set(cache_key, state.cache_ttl, &response)
        .await?;

    Ok(Json(response))
}

fn resolve_window(query: &RangeQuery) -> Result<PeriodWindow, AppError> {
    if query.period.is_some() {
        return Err(AppError::BadRequest(
            "period query parameter is no longer supported; use from/to instead".to_string(),
        ));
    }

    match (&query.from, &query.to) {
        (Some(from), Some(to)) => parse_custom_window(from, to),
        (Some(_), None) | (None, Some(_)) => Err(AppError::BadRequest(
            "from and to must be provided together".to_string(),
        )),
        (None, None) => Err(AppError::BadRequest(
            "from and to query parameters are required".to_string(),
        )),
    }
}

fn floor_to_minute_ts(unix_seconds: i64) -> i64 {
    (unix_seconds / 60) * 60
}

fn cache_filter_segment(query: &RangeQuery) -> String {
    let filter = query.stats_filter();
    format!(
        "user:{}|model:{}|channel:{}",
        filter
            .user_id
            .map(|value| value.to_string())
            .unwrap_or_else(|| "all".to_string()),
        filter.model_name.unwrap_or_else(|| "all".to_string()),
        filter
            .channel_id
            .map(|value| value.to_string())
            .unwrap_or_else(|| "all".to_string())
    )
}

impl RangeQuery {
    fn stats_filter(&self) -> StatsFilter {
        StatsFilter {
            user_id: self.user_id,
            model_name: self
                .model_id
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string),
            channel_id: self.channel_id,
        }
    }
}
