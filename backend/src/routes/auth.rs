use axum::{Router, routing::post, Json, extract::State};
use crate::{state::AppState, error::ApiResult};
use crate::api::requests::LoginRequest;
use crate::infra::security::jwt;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/auth/login", post(login))
        .route("/api/v1/auth/logout", post(logout))
        .with_state(state)
}

async fn login(State(_st): State<AppState>, Json(_body): Json<LoginRequest>) -> ApiResult<Json<serde_json::Value>> {
    let _token_example = jwt::encode_demo()?;
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn logout() -> ApiResult<()> { Ok(()) }
