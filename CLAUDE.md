# CLAUDE.md — Work-Dashboard 项目指引

## 项目概述

面向 new-api 的 API 使用统计看板。后端 Rust (Axum + SQLx)，前端 React + TypeScript (Vite)。前后端编译为单个二进制文件。

## 技术栈

- 后端: Rust, Axum 0.7, SQLx 0.8 (PostgreSQL), rust-embed, tokio
- 前端: React 18, TypeScript, Vite 7, TailwindCSS 3, TanStack Query 5, shadcn/ui 风格组件
- 包管理: pnpm (前端), Cargo (后端)
- 构建: `build.rs` 自动调用 `pnpm build`，产物通过 rust-embed 嵌入二进制

## 常用命令

```bash
# 前端开发
pnpm --dir frontend install
pnpm --dir frontend dev

# 后端开发（需要数据库）
WORK_DASHBOARD_DATABASE_URL="postgres://..." cargo run --manifest-path backend/Cargo.toml

# 生产构建（自动构建前端）
cargo build --release --manifest-path backend/Cargo.toml

# 跳过前端构建（仅后端改动时）
WORK_DASHBOARD_SKIP_FRONTEND_BUILD=true cargo build --release --manifest-path backend/Cargo.toml
```

## 项目结构

- `backend/src/` — Rust 源码
  - `main.rs` 入口和路由
  - `api.rs` HTTP handler 和 AppState
  - `repo.rs` 数据库查询（所有 SQL 在此）
  - `models.rs` 数据模型
  - `cache.rs` 内存缓存
  - `config.rs` 环境变量配置
  - `period.rs` 时间范围解析
  - `assets.rs` SPA 静态资源服务
  - `error.rs` 错误类型
- `frontend/src/` — React 前端
  - `App.tsx` 主组件（所有页面组件）
  - `api.ts` API 请求封装和类型定义
  - `format.ts` 数值格式化工具
  - `theme.tsx` 主题系统 (system/light/dark)
  - `components/` UI 组件（date-time-range-picker, filter-combobox, theme-toggle）
  - `components/ui/` shadcn/ui 基础组件
- `deploy/` — 部署配置（systemd service）

## 编码约定

### Rust 后端
- 所有 SQL 使用 sqlx 参数绑定 (`$1`, `$2` + `.bind()`)，禁止字符串拼接
- 错误统一走 `AppError` 枚举，通过 `IntoResponse` 自动转换为 HTTP 响应
- 使用 `tokio::try_join!` 并行执行多个独立数据库查询
- 缓存键格式：`{prefix}:custom:{start_ts}:{end_ts}:user:{id}|model:{name}|channel:{id}`
- SQL 中 `other` JSON 字段的解析使用 `CACHE_TOKENS_EXPR` 和 `REAL_INPUT_EXPR` 常量
- new-api 日志类型: `type=2` 为成功消费, `type=5` 为错误

### TypeScript 前端
- 路径别名: `@/` → `frontend/src/`
- API 类型定义集中在 `api.ts`，前后端通过 `camelCase` serde rename 对齐
- 数值格式化集中在 `format.ts`
- 组件使用函数式组件 + hooks
- TanStack Query 管理 server state，queryKey 包含所有筛选维度
- 不使用 `any` 类型

### 通用
- 环境变量前缀: `WORK_DASHBOARD_`
- 同时兼容 new-api 原生环境变量 (`NEWAPI_DB_DSN`, `LOG_SQL_DSN`) 作为回退
- 中文 UI 文案

## 数据库依赖

本项目读取 new-api 的 PostgreSQL 数据库：
- **主库**: `channels` 表（渠道信息）
- **日志库**: `logs` 表（调用日志，字段包括 user_id, model_name, channel_id, prompt_tokens, completion_tokens, quota, use_time, type, other 等）

## 注意事项

- 前端 `App.tsx` 包含所有页面组件（约 560 行），修改 UI 时直接编辑此文件
- `build.rs` 会在 `frontend/dist` 不存在时创建占位 index.html
- `deploy/work-dashboard.service` 中不包含真实凭据，部署时需自行配置
- 当前无内置认证机制，依赖网络层访问控制
