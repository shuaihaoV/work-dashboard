mod api;
mod assets;
mod cache;
mod config;
mod error;
mod models;
mod period;
mod repo;

use std::sync::Arc;

use axum::http::StatusCode;
use axum::http::Uri;
use axum::routing::get;
use axum::Router;
use sqlx::postgres::PgPoolOptions;
use tracing::{error, info};

use crate::api::AppState;
use crate::cache::ApiCache;
use crate::config::Config;
use crate::error::AppError;

#[tokio::main]
async fn main() -> Result<(), AppError> {
    init_tracing();

    let config = Config::from_env()?;

    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&config.database_url)
        .await?;

    let log_pool = if config.log_database_url == config.database_url {
        pool.clone()
    } else {
        PgPoolOptions::new()
            .max_connections(10)
            .connect(&config.log_database_url)
            .await?
    };

    let separate_log_db = config.log_database_url != config.database_url;
    info!(
        bind = %config.bind_addr,
        cache_ttl_sec = config.cache_ttl.as_secs(),
        base_path = %config.base_path,
        separate_log_db,
        "work-dashboard starting"
    );

    let base_path = config.base_path.clone();

    let state = AppState {
        pool,
        log_pool,
        cache: Arc::new(ApiCache::new()),
        cache_ttl: config.cache_ttl,
    };

    let api_router = Router::new()
        .route("/overview", get(api::get_overview))
        .route("/users/search", get(api::search_users))
        .route("/models/search", get(api::search_models))
        .route("/channels/search", get(api::search_channels))
        .route("/stats/users", get(api::get_user_stats))
        .route("/stats/channels", get(api::get_channel_stats))
        .route("/stats/models", get(api::get_model_stats))
        .route("/stats/raw-models", get(api::get_raw_model_stats))
        .route("/stats/extra", get(api::get_extra_stats));

    let base_path_for_fallback = base_path.clone();
    let scoped_router = Router::new()
        .route("/healthz", get(api::healthz))
        .nest("/api/v1", api_router)
        .fallback(get(move |uri: Uri| {
            let path = base_path_for_fallback.clone();
            async move { assets::serve_spa(uri, &path).await }
        }));

    let app = if base_path == "/" {
        scoped_router
    } else {
        Router::new()
            .route("/healthz", get(api::healthz))
            .nest(&base_path, scoped_router)
            .fallback(|| async { StatusCode::NOT_FOUND })
    }
    .with_state(state);

    let listener = tokio::net::TcpListener::bind(config.bind_addr)
        .await
        .map_err(|err| AppError::Internal(format!("failed to bind listener: {err}")))?;

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|err| AppError::Internal(format!("server error: {err}")))?;

    Ok(())
}
fn init_tracing() {
    let filter = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "work_dashboard=info,tower_http=info,axum=info".into());

    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_target(false)
        .compact()
        .init();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            error!(error = %err, "failed to install ctrl+c handler");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut sigterm) => {
                sigterm.recv().await;
            }
            Err(err) => {
                error!(error = %err, "failed to install SIGTERM handler");
            }
        }
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    info!("shutdown signal received");
}
