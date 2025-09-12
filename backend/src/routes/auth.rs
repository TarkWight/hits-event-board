use crate::api::requests::{login::LoginRequest, refresh::RefreshRequest};
use axum::{Router, routing::post, extract::State, Json};
use crate::{state::AppState, error::ApiResult};
use crate::api::requests::student_register::StudentRegisterRequest;
use crate::api::models::auth::{RegisterOut, LoginOut};
use crate::api::requests::manager_register::ManagerRegisterRequest;
use crate::utils::token::TokenDTO;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/auth/login",    post(login))
        .route("/api/v1/auth/logout",   post(logout))
        .route("/api/v1/auth/refresh",  post(refresh))
        .route("/api/v1/auth/register/student", post(register_student))
        .route("/api/v1/auth/register/manager", post(register_manager))
        .with_state(state)
}

async fn login(State(st): State<AppState>, Json(body): Json<LoginRequest>)
               -> ApiResult<Json<LoginOut>>
{
    let out = st.auth_service.login(body).await?;
    Ok(Json(out))
}

async fn logout(State(st): State<AppState>, user: crate::auth::extractor::AuthUser)
                -> ApiResult<()>
{
    st.auth_service.logout(user.user_id).await?;
    Ok(())
}

async fn refresh(State(st): State<AppState>, Json(body): Json<RefreshRequest>)
                 -> ApiResult<Json<TokenDTO>>
{
    let out = st.auth_service.refresh_by_token(body).await?;
    Ok(Json(out))
}

async fn register_student(
    State(st): State<AppState>,
    Json(body): Json<StudentRegisterRequest>
) -> ApiResult<(axum::http::StatusCode, Json<RegisterOut>)>
{
    let out = st.auth_service.register_student(body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(out)))
}

async fn register_manager(
    State(st): State<AppState>,
    Json(body): Json<ManagerRegisterRequest>
) -> ApiResult<(axum::http::StatusCode, Json<RegisterOut>)>
{
    let out = st.auth_service.register_manager(body).await?;
    Ok((axum::http::StatusCode::CREATED, Json(out)))
}