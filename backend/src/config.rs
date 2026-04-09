use std::env;
use std::net::SocketAddr;
use std::time::Duration;

use crate::error::AppError;

#[derive(Debug, Clone)]
pub struct Config {
    pub bind_addr: SocketAddr,
    pub database_url: String,
    /// Separate log database URL. If unset, falls back to `database_url`.
    pub log_database_url: String,
    pub cache_ttl: Duration,
    pub base_path: String,
}

impl Config {
    pub fn from_env() -> Result<Self, AppError> {
        let bind_addr = env::var("WORK_DASHBOARD_BIND")
            .unwrap_or_else(|_| "0.0.0.0:18088".to_string())
            .parse::<SocketAddr>()
            .map_err(|err| AppError::Config(format!("invalid WORK_DASHBOARD_BIND: {err}")))?;

        let database_url = env::var("WORK_DASHBOARD_DATABASE_URL")
            .or_else(|_| env::var("NEWAPI_DB_DSN"))
            .map_err(|_| {
                AppError::Config(
                    "missing WORK_DASHBOARD_DATABASE_URL (or NEWAPI_DB_DSN)".to_string(),
                )
            })?;

        if !database_url.starts_with("postgres://") && !database_url.starts_with("postgresql://") {
            return Err(AppError::Config(
                "this service currently supports PostgreSQL only; set a postgres DSN".to_string(),
            ));
        }

        let log_database_url = env::var("WORK_DASHBOARD_LOG_DATABASE_URL")
            .or_else(|_| env::var("LOG_SQL_DSN"))
            .unwrap_or_else(|_| database_url.clone());

        if !log_database_url.starts_with("postgres://")
            && !log_database_url.starts_with("postgresql://")
        {
            return Err(AppError::Config(
                "log database must also be PostgreSQL; check WORK_DASHBOARD_LOG_DATABASE_URL"
                    .to_string(),
            ));
        }

        let cache_ttl_seconds = env::var("WORK_DASHBOARD_CACHE_TTL_SECONDS")
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok())
            .unwrap_or(60);

        let base_path = normalize_base_path(
            env::var("WORK_DASHBOARD_BASE_PATH").unwrap_or_else(|_| "/".to_string()),
        )?;

        Ok(Self {
            bind_addr,
            database_url,
            log_database_url,
            cache_ttl: Duration::from_secs(cache_ttl_seconds.max(1)),
            base_path,
        })
    }
}

fn normalize_base_path(raw: String) -> Result<String, AppError> {
    let mut path = raw.trim().to_string();
    if path.is_empty() {
        path = "/".to_string();
    }

    if !path.starts_with('/') {
        path = format!("/{path}");
    }

    if path.len() > 1 {
        path = path.trim_end_matches('/').to_string();
        if path.is_empty() {
            path = "/".to_string();
        }
    }

    if !path
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '-' || c == '_' || c == '.')
    {
        return Err(AppError::Config(
            "WORK_DASHBOARD_BASE_PATH contains invalid characters; allowed: a-z A-Z 0-9 / - _ ."
                .to_string(),
        ));
    }

    Ok(path)
}
