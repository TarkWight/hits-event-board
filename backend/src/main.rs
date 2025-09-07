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

    let app_state = state::AppState::init().await?;
    let app: Router = app::build_router(app_state);

    let host = std::env::var("APP_HOST").unwrap_or_else(|_| "127.0.0.1".into());
    let port = std::env::var("APP_PORT").unwrap_or_else(|_| "8080".into()).parse::<u16>()?;
    let addr = format!("{}:{}", host, port);
    tracing::info!("Listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
