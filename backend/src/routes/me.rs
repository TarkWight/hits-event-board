use axum::{Router, routing::get, Json, extract::State};
use uuid::Uuid;
use time::OffsetDateTime;
use crate::{state::AppState, error::ApiResult};
use crate::api::models::{User, Role, UserStatus};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/me", get(me))
        .route("/api/v1/me/google/connect", axum::routing::post(google_connect))
        .route("/api/v1/me/google", axum::routing::delete(google_disconnect))
        .with_state(state)
}

async fn me() -> ApiResult<Json<User>> {
    let u = User {
        id: Uuid::new_v4(),
        role: Role::Student,
        status: UserStatus::Approved,
        email: Some("student@example.com".into()),
        full_name: Some("Student Sample".into()),
        telegram_user_id: None,
        approved_by: None,
        created_at: OffsetDateTime::now_utc(),
        updated_at: OffsetDateTime::now_utc(),
    };
    Ok(Json(u))
}

async fn google_connect() -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "redirect_url": "https://accounts.google.com/o/oauth2/v2/auth?..." })))
}

async fn google_disconnect() -> ApiResult<()> { Ok(()) }
