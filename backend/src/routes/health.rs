use axum::{Router, routing::get, response::IntoResponse};
pub fn router() -> Router { Router::new().route("/api/v1/health", get(health)) }
async fn health() -> impl IntoResponse { "ok" }
