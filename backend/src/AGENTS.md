# backend/src — Rust 后端

## OVERVIEW

Axum 0.7 + SQLx 0.8 (PostgreSQL) + rust-embed，9 个模块，888+255+142 行核心逻辑，单文件 9 个 .rs。

## STRUCTURE

```
backend/src/
├── main.rs       # 入口：路由注册 + 服务启动 + 优雅关闭
├── api.rs        # HTTP handler + AppState + 缓存中间件
├── repo.rs       # 所有 SQL 查询（占 ~45% 代码量）
├── models.rs     # 数据模型（serde camelCase 序列化）
├── cache.rs      # 内存缓存（RwLock<HashMap>）
├── config.rs     # 环境变量配置 + base_path 归一化
├── period.rs     # 时间范围解析（RFC3339）
├── assets.rs     # rust-embed SPA 静态资源 + 子路径重写
└── error.rs      # AppError 枚举 + IntoResponse
```

## WHERE TO LOOK

| Task | File | Key Symbol |
|------|------|------------|
| 添加 API 端点 | `main.rs` (路由) + `api.rs` (handler) + `repo.rs` (SQL) | 三处协同 |
| 修改 SQL 聚合 | `repo.rs` | `fetch_overview/user_stats/channel_stats/model_stats/raw_model_stats/extra_stats` |
| 修改缓存逻辑 | `api.rs::respond_cached` (171) + `cache.rs` | TTL 默认 60s |
| 添加错误变体 | `error.rs` | `AppError` enum + `IntoResponse` match |
| 修改配置项 | `config.rs::Config::from_env` | 环境变量解析 + new-api 回退 |
| 时间范围校验 | `period.rs::parse_custom_window` | RFC3339 + from < to |
| SPA 静态资源 | `assets.rs::serve_spa` | rust-embed + HTML 重写 |
| 渠道类型映射 | `repo.rs::channel_type_name` (73) | 硬编码 20+ 类型 match |
| 渠道状态映射 | `repo.rs::channel_status_name` (100) | 1/2/3 → enabled/disabled/auto_disabled |
| Token 双计处理 | `repo.rs::REAL_INPUT_EXPR` (57) | Claude/OpenRouter/标准 三分支 |

## CODE MAP

### main.rs (142 行)
- `main()` :25 — `#[tokio::main]` 入口，初始化 tracing/config/pools/router
- `init_tracing()` :103 — EnvFilter + compact 格式
- `shutdown_signal()` :114 — Ctrl+C + SIGTERM (Unix)
- 路由注册 :62-90 — 9 个 API 路由 + `/healthz` + SPA fallback；base_path≠/ 时嵌套

### api.rs (255 行)
- `AppState` :22 — `pool, log_pool, cache: Arc<ApiCache>, cache_ttl`
- `RangeQuery` :30 — `from/to/userId/modelId/channelId`，`period` 已废弃会报错
- `healthz` :47 — 健康检查
- `get_overview` :51 — 走 `respond_cached`
- `get_user_stats` :63 / `get_channel_stats` :75 / `get_model_stats` :88 / `get_raw_model_stats` :100 / `get_extra_stats` :119 — 6 个统计 handler
- `search_users` :132 / `search_models` :145 / `search_channels` :158 — 3 个下拉搜索（不缓存，直接返回）
- `respond_cached` :171 — 缓存中间件：`resolve_window` → 构造 cache_key → 查缓存 → 执行 fetcher → 写缓存
- `resolve_window` :204 — 校验 from/to 必填、period 已废弃
- `floor_to_minute_ts` :222 — 缓存键时间戳取整
- `cache_filter_segment` :226 — `user:{id|all}|model:{name|all}|channel:{id|all}`

### repo.rs (888 行 — 核心)
- `StatsFilter` :13 — `{user_id, model_name, channel_id}`
- `LOG_TYPE_CONSUME=2` / `LOG_TYPE_ERROR=5` :27-28 — new-api 日志类型常量
- `CACHE_TOKENS_EXPR` :31 — `COALESCE((NULLIF(other,'')::json->>'cache_tokens')::bigint, 0)`
- `REAL_INPUT_EXPR` :57 — 三分支 CASE：Claude(+cache_tokens) / 非 Claude 含 cache_creation / 标准(prompt_tokens)
- `channel_type_name` :73 — 20+ 渠道类型硬编码映射
- `channel_status_name` :100 — 状态映射
- `fetch_channel_map` :120 — 主库 channels 全量加载到 `HashMap<i64, ChannelInfoRow>`
- `fetch_overview` :140 — 单行聚合（COUNT/SUM/FILTER）
- `fetch_user_stats` :212 — GROUP BY user_id，LIMIT 200，`ARRAY_AGG(username ORDER BY created_at DESC)` 取最新用户名
- `search_users` :292 — CTE + 模糊匹配 + 排序权重（精确=0, 前缀=1, 包含=2, 其他=3），LIMIT 20
- `search_models` :351 — 类似 search_users
- `search_channels` :405 — 从主库 channels 表查询
- `fetch_channel_stats` :459 — `tokio::try_join!` 并行 logs 查询 + channels 主库
- `fetch_model_stats` :565 — GROUP BY model_name，LIMIT 300
- `fetch_raw_model_stats` :650 — GROUP BY model_name, channel_id，LIMIT 500
- `fetch_top_throughput_channels` :744 — `tokens_per_second = SUM(completion_tokens)/SUM(use_time)`，HAVING COUNT>=5，LIMIT 5
- `fetch_top_requested_models` :821 — LIMIT 5
- `fetch_extra_stats` :866 — `tokio::try_join!` 并行两个 Top 查询

### models.rs (133 行)
- `ApiResponse<T>` :6 — `{generated_at, cache_ttl_sec, data}`，统一响应包装
- `OverviewStats` :24 / `UserStatsItem` :35 / `ChannelStatsItem` :69 / `ModelStatsItem` :85 / `RawModelStatsItem` :98 / `ExtraStats` :130 — 全部 `#[serde(rename_all = "camelCase")]`
- `TopThroughputChannel` :114 / `TopRequestedModel` :123

### cache.rs (66 行)
- `CacheEntry` :12 — `{expires_at: Instant, payload: Value}`
- `ApiCache` :18 — `RwLock<HashMap<String, CacheEntry>>`
- `get<T>` :29 — 读锁查过期 → 未过期返回反序列化；过期升级写锁删除
- `set<T>` :49 — 序列化为 Value 存入

### error.rs (47 行)
- `AppError` :8 — 5 变体：`BadRequest`/`Config`/`Database(#[from] sqlx::Error)`/`Serialization(#[from] serde_json::Error)`/`Internal`
- `IntoResponse` :31 — `BadRequest→400`，其余→`500`，payload `{error: String}`

### config.rs (98 行)
- `Config` :8 — `{bind_addr, database_url, log_database_url, cache_ttl, base_path}`
- `Config::from_env` :18 — 解析环境变量，`NEWAPI_DB_DSN`/`LOG_SQL_DSN` 作为回退
- `normalize_base_path` :70 — 路径归一化 + 字符校验（仅允许 `[a-zA-Z0-9/-_.]`）

### period.rs (29 行)
- `PeriodWindow` :6 — `{start_utc, end_utc}`
- `parse_custom_window` :11 — RFC3339 解析 + from < to 校验

### assets.rs (91 行)
- `FrontendAssets` :9 — `#[derive(RustEmbed)] #[folder = "../frontend/dist"]`
- `serve_spa` :11 — fallback handler：静态资源 → SPA index.html → 404
- `normalize_path` :34 — 剥离 base_path 前缀
- `load_index_response` :61 — **运行时** HTML 重写：`src/href="/assets/"` → `"{base_path}/assets/"` + 注入 `window.__WORK_DASHBOARD_BASE_PATH__`

## CONVENTIONS (backend/src 特有)

- **SQL 构建模式**：`format!(r#"SELECT ... {cached} ... {real_input} ..."#, cached = CACHE_TOKENS_EXPR, real_input = REAL_INPUT_EXPR)` — 仅插常量，用户数据走 `$1..$7` + `.bind()`
- **过滤模式**：`AND ($5::bigint IS NULL OR user_id = $5)` — 通过 NULL 短路实现可选过滤
- **LIMIT 策略**：下拉搜索 20，stats 列表 200-500，Top 排行 5
- **success_rate 精度**：`((success_count / total) * 10000.0).round() / 100.0` — 两位小数
- **avg_latency_ms**：`AVG(use_time * 1000.0) FILTER (WHERE use_time > 0)` — 毫秒，过滤 0
- **用户名解析**：`ARRAY_AGG(username ORDER BY created_at DESC) FILTER (WHERE NULLIF(username,'') IS NOT NULL))[1]` — 取最新非空用户名，无则 `#{user_id}`

## ANTI-PATTERNS

- ❌ 在 `format!` SQL 模板中插入任何用户可控变量
- ❌ `unwrap()` / `expect()` / `panic!()`（用 `AppError` 或 `unwrap_or_else`）
- ❌ 散落的 `Result::Err(String)` — 必须用 `AppError` 变体
- ❌ 新增 `#[allow(...)]` 抑制警告
- ❌ `unsafe` 块
