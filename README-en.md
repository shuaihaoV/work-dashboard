# Work-Dashboard

[![Rust](https://img.shields.io/badge/Rust-1.75%2B-orange.svg)](https://www.rust-lang.org/)
[![Frontend](https://img.shields.io/badge/React-18-blue.svg)](https://react.dev/)
[![License](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

English | **[中文](README.md)**

A standalone statistics dashboard for [new-api](https://github.com/Calcium-Ion/new-api), providing API call analytics and quality monitoring across user, model, and channel dimensions. Frontend assets are embedded into the backend binary at compile time — deploy with a single executable.

## Features

- **Overview Panel** — Request count, success rate, input/output/cached token usage at a glance
- **User Stats** — Aggregated by user: request count, token usage, average latency, cache hit rate
- **Model Stats** — Aggregated by model name: call quality and consumption
- **Channel Stats** — Per-channel: type, enabled status, success rate, latency
- **Raw Model Stats** — Model x channel cross-aggregation for pinpointing problematic channels
- **Top Rankings** — Top 5 high-throughput channels and top 5 most-requested models
- **Multi-dimensional Filtering** — Combined filtering by user, model, and channel
- **Flexible Time Ranges** — Presets (today / this week / this month / last 7 days / last 30 days) + custom precision down to the minute
- **Auto Refresh** — Automatic polling when the time range extends into the future; silent for historical ranges
- **Dark Mode** — Follow system / manual toggle
- **Sub-path Deployment** — Reverse proxy sub-path support (e.g., `/work-dashboard`)

## Tech Stack

| Layer | Technology |
| --- | --- |
| Backend | Rust, Axum, SQLx (PostgreSQL), rust-embed |
| Frontend | React 18, TypeScript, Vite, TailwindCSS, shadcn/ui |
| Data Fetching | TanStack Query (React Query) |
| Caching | In-memory cache, default 60s TTL |
| Build | Cargo build.rs auto-builds frontend and embeds into binary |

## Getting Started

### Prerequisites

- Rust 1.75+ (with Cargo)
- Node.js 18+ & pnpm
- PostgreSQL (shared with new-api or standalone)
- Accessible new-api database

### Local Development

```bash
# 1. Install frontend dependencies
pnpm --dir frontend install

# 2. Start frontend dev server (hot reload)
pnpm --dir frontend dev

# 3. Start backend
WORK_DASHBOARD_DATABASE_URL="postgres://user:pass@host:5432/newapi" \
cargo run --manifest-path backend/Cargo.toml
```

Frontend runs at `http://localhost:5173`, backend at `http://localhost:18088`.

### Production Build

```bash
cargo build --release --manifest-path backend/Cargo.toml
```

Output: `backend/target/release/work-dashboard` (single binary with embedded frontend).

> `build.rs` automatically runs `pnpm build` for the frontend. To skip, set `WORK_DASHBOARD_SKIP_FRONTEND_BUILD=true`.

## Environment Variables

| Variable | Required | Default | Description |
| --- | --- | --- | --- |
| `WORK_DASHBOARD_DATABASE_URL` | Yes | — | Primary database PostgreSQL DSN |
| `NEWAPI_DB_DSN` | No | — | Fallback when `WORK_DASHBOARD_DATABASE_URL` is not set; compatible with new-api's native env var |
| `WORK_DASHBOARD_LOG_DATABASE_URL` | No | Same as primary | Log database DSN; use when new-api is configured with `LOG_SQL_DSN` for a separate database |
| `LOG_SQL_DSN` | No | — | Log database fallback; compatible with new-api's native env var |
| `WORK_DASHBOARD_BIND` | No | `0.0.0.0:18088` | Server listen address |
| `WORK_DASHBOARD_CACHE_TTL_SECONDS` | No | `60` | API cache TTL in seconds |
| `WORK_DASHBOARD_BASE_PATH` | No | `/` | Sub-path deployment prefix, e.g., `/work-dashboard` |
| `WORK_DASHBOARD_SKIP_FRONTEND_BUILD` | No | `false` | Skip frontend build (build.rs only) |

## API Endpoints

All endpoints are mounted under `{base_path}/api/v1/`.

| Method | Path | Description |
| --- | --- | --- |
| GET | `/overview` | Overview stats (requests, success rate, token usage) |
| GET | `/stats/users` | User-dimension stats |
| GET | `/stats/channels` | Channel-dimension stats |
| GET | `/stats/models` | Model-dimension stats |
| GET | `/stats/raw-models` | Model x channel cross stats |
| GET | `/stats/extra` | Top rankings (throughput channels, most-requested models) |
| GET | `/users/search?q=` | User search (dropdown options) |
| GET | `/models/search?q=` | Model search |
| GET | `/channels/search?q=` | Channel search |
| GET | `/healthz` | Health check |

Stats endpoint query parameters:

| Parameter | Type | Description |
| --- | --- | --- |
| `from` | RFC3339 | Start time (required) |
| `to` | RFC3339 | End time (required) |
| `userId` | int64 | Filter by user |
| `modelId` | string | Filter by model name |
| `channelId` | int64 | Filter by channel |

Examples:

```
GET /api/v1/overview?from=2026-03-01T00:00:00Z&to=2026-03-31T23:59:59Z
GET /api/v1/stats/users?from=2026-03-01T00:00:00%2B08:00&to=2026-03-31T23:59:59%2B08:00&userId=1
```

## Deployment

### Systemd

```bash
# Edit the config file with your actual database DSN
cp deploy/work-dashboard.service /etc/systemd/system/work-dashboard.service

# Upload the binary
cp backend/target/release/work-dashboard /opt/work-dashboard/

systemctl daemon-reload
systemctl enable --now work-dashboard
```

### Nginx Reverse Proxy

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

Set `WORK_DASHBOARD_BASE_PATH=/work-dashboard` when starting the server.

## Project Structure

```
.
├── backend/
│   ├── src/
│   │   ├── main.rs      # Entry point, routing, server startup
│   │   ├── api.rs        # HTTP handlers & AppState
│   │   ├── repo.rs       # Database queries (SQLx)
│   │   ├── models.rs     # Data models & serialization
│   │   ├── cache.rs      # In-memory cache (RwLock HashMap)
│   │   ├── config.rs     # Environment variable config
│   │   ├── period.rs     # Time range parsing
│   │   ├── assets.rs     # SPA static asset serving
│   │   └── error.rs      # Error types & responses
│   ├── build.rs          # Auto frontend build
│   └── Cargo.toml
├── frontend/
│   ├── src/
│   │   ├── App.tsx       # Main application component
│   │   ├── api.ts        # API request wrapper
│   │   ├── format.ts     # Number formatting utilities
│   │   ├── theme.tsx     # Theme provider
│   │   ├── main.tsx      # React entry point
│   │   ├── components/   # UI components
│   │   └── styles.css    # Global styles & CSS variables
│   ├── vite.config.ts
│   ├── tailwind.config.ts
│   └── package.json
├── deploy/
│   └── work-dashboard.service  # systemd example
└── CLAUDE.md                   # Claude Code project guide
```

## Security Notes

- **Database Credentials**: Never commit real credentials to version control. Use `EnvironmentFile=` or a secrets management service.
- **Network Isolation**: This version has no built-in authentication. Protect via Nginx auth, VPN, or internal network access controls.
- **Database Connection**: Enable SSL in production (remove `sslmode=disable`).
- **SQL Injection**: All database queries use parameterized bindings — no SQL injection risk.

## Acknowledgements

- [new-api](https://github.com/Calcium-Ion/new-api) — The API gateway this project integrates with
- [Axum](https://github.com/tokio-rs/axum) — Rust web framework
- [shadcn/ui](https://ui.shadcn.com/) — Frontend UI component style

## License

MIT
