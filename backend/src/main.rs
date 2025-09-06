mod config;
mod db;
mod error;
mod models;
mod services;

use crate::{
    config::Config,
    db::PgPool,
    error::{AppError, AppResult},
};

use axum::{
    extract::{rejection::JsonRejection, rejection::QueryRejection, Path, Query, State},
    routing::{delete, get},
    Json, Router,
};
use http::StatusCode;
use serde::{Deserialize, Serialize};
// use time::OffsetDateTime;
use tokio::net::TcpListener;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    hits: u64,
}

#[derive(Deserialize)]
struct HealthQuery {
    fail: Option<bool>,
}

#[derive(Clone)]
struct AppState {
    hits: Arc<AtomicU64>,
    cfg: Config,
    pool: Option<PgPool>,
    events: Option<services::EventService>,
}

// ================================
// main
// ================================

#[tokio::main]
async fn main() {
    // logs
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // config
    let cfg = Config::from_env();
    tracing::info!(?cfg, "loaded config");

    // db pool (optional at this stage)
    let pool = if let Some(url) = &cfg.database_url {
        let pool = db::make_pool(url)
            .await
            .expect("failed to connect to Postgres");

        // if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        //     panic!("failed to run migrations: {e}");
        // }
        Some(pool)
    } else {
        None
    };

    // services (optional)
    let events_service = pool.as_ref().cloned().map(services::EventService::new);

    // state
    let state = AppState {
        hits: Arc::new(AtomicU64::new(0)),
        cfg,
        pool,
        events: events_service,
    };

    // router
    let app = Router::new()
        .route("/api/health", get(health))
        // .route("/events", get(list_events).post(create_event))
        // .route("/events/{id}", get(get_event).delete(delete_event))
        .with_state(state.clone());

    // serve
    let addr = format!("0.0.0.0:{}", state.cfg.port);
    let listener = TcpListener::bind(&addr).await.unwrap();
    tracing::info!("listening on {addr}"); // dev: http; prod: behind reverse proxy (TLS)

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

// ================================
// Handlers
// ================================

/// Health check endpoint
///
/// # Args
/// * `State(state)`: глобальное состояние приложения
/// * `q: Result<Query<HealthQuery>, QueryRejection>`: query-параметры, при ошибке вернём 400
///
/// # Returns
/// `AppResult<(StatusCode, Json<HealthResponse>)>`
///
/// # Examples
/// ```no_run
/// // GET /health
/// // → 200 OK
/// // { "status": "ok", "hits": 1 }
///
/// // GET /health?fail=true
/// // → 400 Bad Request
/// // { "error": "bad_request", "details": "forced failure for testing" }
///
/// // GET /health?fail=1  (невалидный bool)
/// // → 400 Bad Request
/// // { "error": "bad_request", "details": "invalid query parameters" }
/// ```
async fn health(
    State(state): State<AppState>,
    q: Result<Query<HealthQuery>, QueryRejection>,
) -> AppResult<(StatusCode, Json<HealthResponse>)> {
    let q = match q {
        Ok(q) => q.0,
        Err(e) => {
            tracing::warn!("query rejection: {e}");
            return Err(AppError::bad("invalid query parameters"));
        }
    };

    if q.fail.unwrap_or(false) {
        return Err(AppError::bad("forced failure for testing"));
    }

    let n = state.hits.fetch_add(1, Ordering::Relaxed) + 1;
    Ok((
        StatusCode::OK,
        Json(HealthResponse {
            status: "ok",
            hits: n,
        }),
    ))
}

// ================================
// Helpers
// ================================

#[allow(dead_code)]
async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    tracing::info!("signal received, starting graceful shutdown");
}
