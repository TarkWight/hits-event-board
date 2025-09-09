use crate::api::requests::login;
use axum::{Router, routing::post, extract::State, Json};
use crate::{state::AppState, error::ApiResult};
use crate::api::requests::student_register::StudentRegisterRequest;
use crate::api::models::auth::RegisterOut;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/auth/login",  post(login))
        .route("/api/v1/auth/logout", post(logout))
        .route("/api/v1/auth/register/student", post(register_student))
        .with_state(state)
}

async fn login(State(_st): State<AppState>, Json(_body): Json<login::LoginRequest>)
               -> ApiResult<Json<serde_json::Value>>
{
    // TODO: реальная аутентификация (сверка пароля, выдача токенов)
    Ok(Json(serde_json::json!({ "ok": true })))
}

async fn logout() -> ApiResult<()> { Ok(()) }

async fn register_student(
    State(st): State<AppState>,
    Json(body): Json<StudentRegisterRequest>
) -> ApiResult<(http::StatusCode, Json<RegisterOut>)>
{
    let out = st.auth_service.register_student(body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(out)))
}