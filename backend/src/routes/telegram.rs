use axum::{Router, routing::post, Json, extract::State};
use serde_json::Value;
use crate::{state::AppState, error::ApiResult};

pub fn router(state: AppState) -> Router {
    Router::new().route("/api/v1/telegram/webhook", post(webhook)).with_state(state)
}

async fn webhook(State(_st): State<AppState>, Json(_update): Json<Value>) -> ApiResult<Json<Value>> {
    Ok(Json(serde_json::json!({ "ok": true })))
}
