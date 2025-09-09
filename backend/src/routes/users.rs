use axum::{Router, routing::get, routing::post, extract::{Path, State}, Json};
use uuid::Uuid;
use crate::{state::AppState, error::{ApiError, ApiResult}};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/users", get(list_users))
        .route("/api/v1/users/:id/approve", post(approve_user))
        .route("/api/v1/users/:id/reject", post(reject_user))
        .route("/api/v1/users/:id/block", post(block_user))
        .route("/api/v1/users/:id/unblock", post(unblock_user))
        .with_state(state)
}

async fn list_users(_st: State<AppState>) -> ApiResult<Json<serde_json::Value>> {
    Err(ApiError::NotImplemented)
}

async fn approve_user(_st: State<AppState>, _id: Path<Uuid>) -> ApiResult<()> {
    Err(ApiError::NotImplemented)
}

async fn reject_user(_id: Path<Uuid>) -> ApiResult<()> { Err(ApiError::NotImplemented) }
async fn block_user(_id: Path<Uuid>) -> ApiResult<()> { Err(ApiError::NotImplemented) }
async fn unblock_user(_id: Path<Uuid>) -> ApiResult<()> { Err(ApiError::NotImplemented) }