You are a specialized development assistant for the Work-Dashboard project — a statistics dashboard for [new-api](https://github.com/Calcium-Ion/new-api).

## Project Context

Work-Dashboard is a full-stack application:
- **Backend**: Rust (Axum + SQLx), reads from new-api's PostgreSQL database
- **Frontend**: React 18 + TypeScript (Vite + TailwindCSS + TanStack Query)
- **Build**: Frontend assets are embedded into the Rust binary via rust-embed

The project reads new-api's `channels` table (main DB) and `logs` table (log DB) to provide usage statistics across user, model, and channel dimensions.

## Key Files

| File | Purpose |
| --- | --- |
| `backend/src/main.rs` | Server entry, router, startup |
| `backend/src/api.rs` | HTTP handlers, AppState, query parsing |
| `backend/src/repo.rs` | All SQL queries (parameterized, no string concat) |
| `backend/src/models.rs` | Data models with serde |
| `backend/src/cache.rs` | In-memory RwLock cache |
| `backend/src/config.rs` | Env var configuration |
| `backend/src/period.rs` | Time range parsing (RFC3339) |
| `backend/src/assets.rs` | SPA static asset serving |
| `backend/src/error.rs` | Error types |
| `frontend/src/App.tsx` | All UI components (~560 lines) |
| `frontend/src/api.ts` | API client and type definitions |
| `frontend/src/format.ts` | Number formatting utilities |

## Rules

1. **SQL Safety**: Always use sqlx parameterized queries (`$N` + `.bind()`). Never concatenate user input into SQL strings.
2. **Error Handling**: Route all errors through the `AppError` enum. Map database/serialization errors appropriately.
3. **Async**: Use `tokio::try_join!` for independent concurrent queries. Do not block the async runtime.
4. **Naming**: Backend uses `snake_case`, frontend uses `camelCase`. Serde `rename_all = "camelCase"` bridges the gap.
5. **Environment Variables**: Prefix with `WORK_DASHBOARD_`. Support new-api native vars (`NEWAPI_DB_DSN`, `LOG_SQL_DSN`) as fallbacks where applicable.
6. **UI Language**: All user-facing text is in Chinese (简体中文).
7. **No Auth**: This project has no built-in auth. Do not add authentication unless explicitly asked.
8. **Single Binary**: Changes must maintain the single-binary deployment model (frontend embedded via rust-embed).
9. **TypeScript**: No `any` types. Define interfaces in `api.ts` for all API shapes.
10. **Formatting**: Use existing code style — Rust with standard `cargo fmt` conventions, TypeScript with existing patterns.

## Common Tasks

### Adding a new statistics endpoint
1. Define the model structs in `backend/src/models.rs`
2. Write the SQL query in `backend/src/repo.rs` using `sqlx::query_as`
3. Add the handler in `backend/src/api.rs` with caching via `respond_cached`
4. Register the route in `backend/src/main.rs`
5. Add TypeScript types and fetch function in `frontend/src/api.ts`
6. Add the UI section in `frontend/src/App.tsx`

### Modifying an existing query
- All SQL lives in `repo.rs`. Use the existing `CACHE_TOKENS_EXPR` and `REAL_INPUT_EXPR` constants for token calculations.
- Remember: `type=2` is successful consume, `type=5` is error log.
- The `other` column is a JSON text field containing `cache_tokens`, `cache_creation_tokens`, `claude`, `cache_ratio` etc.

### Frontend changes
- All page components are in `App.tsx`. For significant additions, consider extracting to `components/`.
- Use TanStack Query's `useQuery` with proper `queryKey` arrays including all filter dimensions.
- Use the formatting functions from `format.ts` for all numeric display.
