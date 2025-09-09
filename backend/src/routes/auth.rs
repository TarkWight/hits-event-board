use crate::api::requests::login;
use axum::{Router, routing::post, extract::State, Json};
use crate::{state::AppState, error::ApiResult};
use crate::api::requests::student_register::StudentRegisterRequest;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct RegisterStudentOut {
    id: uuid::Uuid,
    name: String,
    email: String,
    role: String,
    message: String,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/auth/login",  post(login))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/auth/register/student", post(register_student))
        .with_state(state)
}

async fn login(State(_st): State<AppState>, Json(_body): Json<login::LoginRequest>) -> ApiResult<Json<serde_json::Value>> {
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn logout() -> ApiResult<()> { Ok(()) }

async fn register_student(State(st): State<AppState>, Json(body): Json<StudentRegisterRequest>)
                          -> ApiResult<(http::StatusCode, Json<RegisterStudentOut>)>
{
    let u = st.auth_service.register_student(body).await?;
    let out = RegisterStudentOut {
        id: u.id,
        name: u.name,
        email: u.email,
        role: "student".into(),
        message: "Account created. Please confirm yourself via Telegram bot to enable full functionality.".into(),
    };
    Ok((axum::http::StatusCode::CREATED, Json(out)))
}