use axum::{Router, routing::{get, post, delete}, Json};
use crate::error::ApiResult;
use crate::auth::extractor::AuthUser;
use crate::auth::roles::{ManagerStatus, UserRole, StudentStatus};
use crate::state::AppState;

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
struct MeOut {
    user_id: uuid::Uuid,
    role: UserRole,
    manager_status: Option<ManagerStatus>,
    company_id: Option<uuid::Uuid>,
    student_status: Option<StudentStatus>,
    email: String,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/me", get(me))
        .route("/api/v1/me/google/connect", post(google_connect))
        .route("/api/v1/me/google", delete(google_disconnect))
        .with_state(state)
}

async fn me(user: AuthUser) -> ApiResult<Json<MeOut>> {
    let out = MeOut {
        user_id: user.user_id,
        role: user.role,
        manager_status: user.manager_status,
        company_id: user.company_id,
        student_status: user.student_status,
        email: user.raw.sub.clone(),
    };
    Ok(Json(out))
}

async fn google_connect() -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({
        "redirect_url": "https://accounts.google.com/o/oauth2/v2/auth?..."
    })))
}

async fn google_disconnect() -> ApiResult<()> { Ok(()) }