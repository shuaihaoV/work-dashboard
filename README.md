# Work-Dashboard

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Frontend](https://img.shields.io/badge/React-18-blue.svg)](https://react.dev/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

**[English](README-en.md)** | 中文

面向 [new-api](https://github.com/Calcium-Ion/new-api) 的独立统计看板，提供用户、模型、渠道维度的 API 调用统计与质量监控。前端静态资源在编译期嵌入后端二进制，部署时仅需一个可执行文件。

## 功能特性

- **总览面板** — 请求量、成功率、输入/输出/缓存 Token 消耗一览
- **用户统计** — 按用户聚合请求量、Token 消耗、平均延迟、缓存命中率
- **模型统计** — 按模型名称聚合调用质量与消耗
- **渠道统计** — 按渠道展示类型、启用状态、成功率、延迟
- **原始模型统计** — 模型 × 渠道交叉聚合，精确定位问题渠道
- **Top 排行** — 高吞吐渠道 Top 5、高频模型 Top 5
- **多维筛选** — 用户、模型、渠道三维度组合过滤
- **灵活时间范围** — 预设快捷项（今天/本周/本月/过去7天/过去30天）+ 自定义精确到分钟
- **自动刷新** — 当时间范围包含未来时刻时自动轮询，历史区间静默
- **深色模式** — 跟随系统 / 手动切换
- **子路径部署** — 支持反向代理子路径（如 `/work-dashboard`）

## 技术栈

| 层级 | 技术 |
| --- | --- |
| 后端 | Rust, Axum, SQLx (PostgreSQL), rust-embed |
| 前端 | React 18, TypeScript, Vite, TailwindCSS, shadcn/ui |
| 数据获取 | TanStack Query (React Query) |
| 缓存 | 进程内内存缓存，默认 60s TTL |
| 构建 | Cargo build.rs 自动构建前端并嵌入二进制 |

## 快速开始

### 前提条件

- Rust 1.75+ (with Cargo)
- Node.js 18+ & pnpm
- PostgreSQL（与 new-api 共用或独立）
- 可访问的 new-api 数据库

### 本地开发

```bash
# 1. 安装前端依赖
pnpm --dir frontend install

# 2. 启动前端开发服务（热更新）
pnpm --dir frontend dev

# 3. 启动后端
WORK_DASHBOARD_DATABASE_URL="postgres://user:pass@host:5432/newapi" \
cargo run --manifest-path backend/Cargo.toml
```

前端默认运行在 `http://localhost:5173`，后端默认运行在 `http://localhost:18088`。

### 生产构建

```bash
cargo build --release --manifest-path backend/Cargo.toml
```

构建产物：`backend/target/release/work-dashboard`（单个二进制文件，内含前端）。

> `build.rs` 会自动执行 `pnpm build` 构建前端。如需跳过，设置环境变量 `WORK_DASHBOARD_SKIP_FRONTEND_BUILD=true`。

## 环境变量

| 变量名 | 必填 | 默认值 | 说明 |
| --- | --- | --- | --- |
| `WORK_DASHBOARD_DATABASE_URL` | 是 | — | 主数据库 PostgreSQL DSN |
| `NEWAPI_DB_DSN` | 否 | — | 当未设置 `WORK_DASHBOARD_DATABASE_URL` 时的回退，兼容 new-api 原生环境变量 |
| `WORK_DASHBOARD_LOG_DATABASE_URL` | 否 | 同主库 | 日志数据库 DSN；new-api 配置 `LOG_SQL_DSN` 分库时使用 |
| `LOG_SQL_DSN` | 否 | — | 日志数据库回退，兼容 new-api 原生环境变量 |
| `WORK_DASHBOARD_BIND` | 否 | `0.0.0.0:18088` | 服务监听地址 |
| `WORK_DASHBOARD_CACHE_TTL_SECONDS` | 否 | `60` | API 缓存过期秒数 |
| `WORK_DASHBOARD_BASE_PATH` | 否 | `/` | 子路径部署前缀，如 `/work-dashboard` |
| `WORK_DASHBOARD_SKIP_FRONTEND_BUILD` | 否 | `false` | 是否跳过前端构建（仅 build.rs 使用） |

## API 端点

所有端点挂载在 `{base_path}/api/v1/` 下。

| 方法 | 路径 | 说明 |
| --- | --- | --- |
| GET | `/overview` | 总览统计（请求量、成功率、Token 消耗） |
| GET | `/stats/users` | 用户维度统计 |
| GET | `/stats/channels` | 渠道维度统计 |
| GET | `/stats/models` | 模型维度统计 |
| GET | `/stats/raw-models` | 模型 × 渠道交叉统计 |
| GET | `/stats/extra` | Top 排行（高吞吐渠道、高频模型） |
| GET | `/users/search?q=` | 用户搜索（下拉选项） |
| GET | `/models/search?q=` | 模型搜索 |
| GET | `/channels/search?q=` | 渠道搜索 |
| GET | `/healthz` | 健康检查 |

统计端点查询参数：

| 参数 | 类型 | 说明 |
| --- | --- | --- |
| `from` | RFC3339 | 开始时间（必填） |
| `to` | RFC3339 | 结束时间（必填） |
| `userId` | int64 | 按用户过滤 |
| `modelId` | string | 按模型名称过滤 |
| `channelId` | int64 | 按渠道过滤 |

示例：

```
GET /api/v1/overview?from=2026-03-01T00:00:00Z&to=2026-03-31T23:59:59Z
GET /api/v1/stats/users?from=2026-03-01T00:00:00%2B08:00&to=2026-03-31T23:59:59%2B08:00&userId=1
```

## 部署

### Systemd

```bash
# 编辑配置文件，填入实际的数据库 DSN
cp deploy/work-dashboard.service /etc/systemd/system/work-dashboard.service

# 上传二进制
cp backend/target/release/work-dashboard /opt/work-dashboard/

systemctl daemon-reload
systemctl enable --now work-dashboard
```

### Nginx 反向代理

```nginx
server {
    listen 80;
    server_name example.com;

    location = /work-dashboard {
        proxy_pass http://127.0.0.1:18088;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    location ^~ /work-dashboard/ {
        proxy_pass http://127.0.0.1:18088;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

启动时设置 `WORK_DASHBOARD_BASE_PATH=/work-dashboard`。

## 项目结构

```
.
├── backend/
│   ├── src/
│   │   ├── main.rs      # 入口、路由、服务启动
│   │   ├── api.rs        # HTTP handler & AppState
│   │   ├── repo.rs       # 数据库查询（SQLx）
│   │   ├── models.rs     # 数据模型 & 序列化
│   │   ├── cache.rs      # 内存缓存（RwLock HashMap）
│   │   ├── config.rs     # 环境变量配置
│   │   ├── period.rs     # 时间范围解析
│   │   ├── assets.rs     # 前端静态资源 SPA 服务
│   │   └── error.rs      # 错误类型 & 响应
│   ├── build.rs          # 前端自动构建
│   └── Cargo.toml
├── frontend/
│   ├── src/
│   │   ├── App.tsx       # 主应用组件
│   │   ├── api.ts        # API 请求封装
│   │   ├── format.ts     # 数值格式化工具
│   │   ├── theme.tsx     # 主题 Provider
│   │   ├── main.tsx      # React 入口
│   │   ├── components/   # UI 组件
│   │   └── styles.css    # 全局样式 & CSS 变量
│   ├── vite.config.ts
│   ├── tailwind.config.ts
│   └── package.json
├── deploy/
│   └── work-dashboard.service  # systemd 示例
└── CLAUDE.md                   # Claude Code 项目指引
```

## 安全注意事项

- **数据库凭据**: 请勿在版本控制中提交真实凭据，使用 `EnvironmentFile=` 或密钥管理服务
- **网络隔离**: 当前版本无内置认证，建议通过 Nginx auth、VPN 或内网访问控制保护
- **数据库连接**: 生产环境建议启用 SSL（移除 `sslmode=disable`）
- **SQL 注入**: 所有数据库查询使用参数绑定，不存在 SQL 注入风险

## 致谢

- [new-api](https://github.com/Calcium-Ion/new-api) — 本项目适配的 API 网关
- [Axum](https://github.com/tokio-rs/axum) — Rust Web 框架
- [shadcn/ui](https://ui.shadcn.com/) — 前端 UI 组件风格

## License

MIT
