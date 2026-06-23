# PROJECT KNOWLEDGE BASE

**Generated:** 2026-06-22
**Commit:** 342c627 (main)

## OVERVIEW

面向 new-api 的 API 调用统计看板。Rust (Axum + SQLx) 后端 + React 18 + TypeScript (Vite) 前端，通过 `build.rs` + `rust-embed` 编译为**单个二进制文件**部署。

## STRUCTURE

```
.
├── backend/                # Rust 后端（含 build.rs 自动构建前端）
│   ├── build.rs            # [非标准] 构建时调用 pnpm build + 占位 index.html 逻辑
│   ├── Cargo.toml          # release: LTO=fat, strip, panic=abort
│   └── src/                # → 见 backend/src/AGENTS.md
├── frontend/               # React + Vite 前端
│   └── src/                # → 见 frontend/src/AGENTS.md
├── deploy/
│   └── work-dashboard.service  # systemd 示例（无真实凭据）
├── .github/workflows/
│   ├── ci.yml              # PR: frontend tsc-check + backend cargo-check
│   └── release.yml         # tag v*: 5 平台交叉编译 → GitHub Release
└── 见各目录下的 AGENTS.md    # AI 协作知识库
└── README.md / README-en.md
```

## WHERE TO LOOK

| Task | Location | Notes |
|------|----------|-------|
| 添加 HTTP 端点 | `backend/src/main.rs` (路由注册) + `api.rs` (handler) + `repo.rs` (SQL) | 三处协同 |
| 修改 SQL 查询 | `backend/src/repo.rs` | 所有 SQL 集中于此，用 `format!` 插入常量片段 + `$N` 绑定参数 |
| 调整缓存策略 | `backend/src/api.rs::respond_cached` + `cache.rs` | 缓存键格式见 CONVENTIONS |
| 修改 UI | `frontend/src/App.tsx` | 所有页面组件集中在此（734 行） |
| 添加 API 类型 | `frontend/src/api.ts` + `backend/src/models.rs` | 双向同步，camelCase 对齐 |
| 子路径部署 | `backend/src/assets.rs` + `WORK_DASHBOARD_BASE_PATH` | 运行时 HTML 重写，非 Vite base |
| 改渠道类型映射 | `backend/src/repo.rs::channel_type_name` | 硬编码 match 表 |
| Token 计算逻辑 | `backend/src/repo.rs::REAL_INPUT_EXPR` | Claude/OpenRouter/标准 三分支处理 |
| 部署配置 | `deploy/work-dashboard.service` | systemd，需自填凭据 |

## CODE MAP

核心符号（LSP 不可用，源码精读）：

| Symbol | Type | Location | Role |
|--------|------|----------|------|
| `main` | fn | `backend/src/main.rs:25` | 入口：路由注册 + 服务启动 |
| `AppState` | struct | `backend/src/api.rs:22` | 共享状态：pool, log_pool, cache, cache_ttl |
| `respond_cached` | fn | `backend/src/api.rs:171` | 缓存中间件：查缓存→执行 fetcher→写缓存 |
| `get_overview` 等 9 个 | fn | `backend/src/api.rs` | HTTP handler，全部走 `respond_cached` |
| `fetch_overview` 等 6 个 | fn | `backend/src/repo.rs` | 数据库聚合查询（logs 表） |
| `search_users/models/channels` | fn | `backend/src/repo.rs` | 下拉搜索（LIMIT 20） |
| `fetch_channel_map` | fn | `backend/src/repo.rs:120` | 主库 channels 全量加载到 HashMap |
| `REAL_INPUT_EXPR` | const | `backend/src/repo.rs:57` | Token 真实输入计算（Claude/OpenRouter/标准） |
| `CACHE_TOKENS_EXPR` | const | `backend/src/repo.rs:31` | 从 `other` JSON 提取 cache_tokens |
| `AppError` | enum | `backend/src/error.rs:8` | 5 变体，`IntoResponse` 自动转 HTTP |
| `ApiCache` | struct | `backend/src/cache.rs:18` | `RwLock<HashMap<String, CacheEntry>>` |
| `serve_spa` | fn | `backend/src/assets.rs:11` | SPA fallback：静态资源 + index.html 重写 |
| `App` | component | `frontend/src/App.tsx` | 单体主组件，6 个 TanStack Query |
| `fetchOverview` 等 | fn | `frontend/src/api.ts:140` | API 请求封装 |
| `ThemeProvider` | component | `frontend/src/theme.tsx:24` | system/light/dark，localStorage 持久化 |

## CONVENTIONS

### Rust 后端
- **SQL 安全**：所有用户输入通过 `$1, $2, ...` + `.bind()` 绑定。`format!` 仅用于插入**编译期常量**（`CACHE_TOKENS_EXPR`, `REAL_INPUT_EXPR`），不含用户数据
- **错误统一**：所有 `Result` 用 `AppError`，通过 `IntoResponse` 自动转 HTTP。禁止裸字符串错误
- **并行查询**：独立查询用 `tokio::try_join!`（见 `repo.rs:497,689,873`）
- **缓存键格式**：`{prefix}:custom:{start_ts_floor_minute}:{end_ts_floor_minute}:user:{id|all}|model:{name|all}|channel:{id|all}`
- **new-api 日志类型**：`type=2` 成功消费，`type=5` 错误
- **分库**：`pool`（主库 channels）+ `log_pool`（日志库 logs），可同库或分库

### TypeScript 前端
- **路径别名**：`@/` → `frontend/src/`
- **类型对齐**：后端 `#[serde(rename_all = "camelCase")]` ↔ 前端 TS interface camelCase 字段
- **API 类型集中**：所有 API 类型定义在 `frontend/src/api.ts`
- **格式化集中**：所有数值格式化在 `frontend/src/format.ts`
- **TanStack Query**：queryKey 包含全部筛选维度（from/to/userId/modelId/channelId），自动刷新仅在时间范围含未来时启用
- **中文 UI 文案**

### 通用
- **环境变量前缀**：`WORK_DASHBOARD_`，兼容 new-api 原生 `NEWAPI_DB_DSN` / `LOG_SQL_DSN` 回退
- **无认证**：依赖网络层访问控制
- **无 linter/formatter**：项目中无 ESLint/Prettier/Biome 配置

## ANTI-PATTERNS (THIS PROJECT)

- ❌ **SQL 字符串拼接用户输入** — 必须用 `$N` + `.bind()`
- ❌ **`any` 类型 / `as any` / `@ts-ignore` / `@ts-expect-error`** — 严格执行
- ❌ **裸 `unwrap()` / `panic!`** — 用 `AppError` 或 `unwrap_or_else`
- ❌ **绕过 `AppError` 的错误处理** — 不允许散落的 `Result::Err(String)`
- ❌ **TODO/FIXME/HACK 注释** — 当前代码库零遗留，维持此标准

## UNIQUE STYLES

- **单二进制部署**：`build.rs` 编译期调用 `pnpm build`，`rust-embed` 嵌入 `frontend/dist/` 到二进制。部署只需一个文件
- **运行时子路径重写**：非 Vite `base` 配置，而是 `assets.rs` 在请求时替换 `src="/assets/"` → `src="{base_path}/assets/"` + 注入 `window.__WORK_DASHBOARD_BASE_PATH__`
- **Token 双计修复**：`REAL_INPUT_EXPR` 区分 Claude（prompt_tokens 不含 cache_read）、OpenRouter（已减 cache）、标准 OpenAI 三种情况
- **缓存键分钟级取整**：`floor_to_minute_ts` 避免相同查询的缓存碎片
- **shadcn/ui 源码复制**：`components/ui/` 是 shadcn 组件源码（非 npm 包），由 `components.json` 配置
- **App.tsx 单体**：所有页面组件集中在 734 行单文件（项目刻意选择，页面数量少）

## COMMANDS

```bash
# 前端开发
pnpm --dir frontend install
pnpm --dir frontend dev                    # Vite HMR @ :5173

# 后端开发（需数据库）
WORK_DASHBOARD_DATABASE_URL="postgres://..." cargo run --manifest-path backend/Cargo.toml

# 完整生产构建（前端 → 编译 → 嵌入 → 单二进制）
cargo build --release --manifest-path backend/Cargo.toml

# 跳过前端构建（仅后端改动 / dist 已就绪）
WORK_DASHBOARD_SKIP_FRONTEND_BUILD=true cargo build --release --manifest-path backend/Cargo.toml

# CI 类型检查（无前端构建）
WORK_DASHBOARD_SKIP_FRONTEND_BUILD=true cargo check --manifest-path backend/Cargo.toml
pnpm --dir frontend exec tsc -b
```

## NOTES

- **LSP 限制**：`rust-analyzer` 不在 stable 工具链，TypeScript LSP 未安装。CODE MAP 基于源码精读
- **零测试**：项目当前无任何测试（无 `#[cfg(test)]`、无 vitest/jest、无测试文件）。新功能建议引入 `rstest`（后端）+ `vitest`（前端）
- **release CI**：tag `v*` 触发 5 平台构建（linux-x86_64/aarch64 musl, darwin-aarch64/x86_64, windows-x86_64）。CI 先 `pnpm build` 再设 `SKIP_FRONTEND_BUILD=true` 避免重复构建
- **`vite.config.js` / `tailwind.config.js` 等生成物被版本控制**（.gitignore 列出但已跟踪）
- **占位 index.html**：`build.rs` 在 `frontend/dist/` 不存在时创建占位页，确保 `rust-embed` 编译不报错
- **数据库依赖**：读取 new-api 的 PostgreSQL（channels 主库 + logs 日志库，可分库）
