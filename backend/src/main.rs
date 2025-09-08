use axum::Router;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod app;
mod state;
mod error;
mod config;
mod api;
mod routes;
mod middleware;
mod domain;
mod infra;
mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let env_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info,tsu=debug".into());
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(env_filter))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cfg = config::Config::from_env();

    let app_state = state::AppState::init_with(cfg.clone()).await?;
    let app: Router = app::build_router(app_state);

    let addr = format!("{}:{}", cfg.host, cfg.port);
    tracing::info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
