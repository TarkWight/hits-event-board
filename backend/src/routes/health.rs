use axum::{Router, routing::get, Json};
use serde::Serialize;

#[derive(Serialize)]
struct HealthOut {
    status: &'static str,
}

pub fn router() -> Router {
    Router::new().route("/health", get(health))
}

async fn health() -> Json<HealthOut> {
    Json(HealthOut { status: "ok" })
}