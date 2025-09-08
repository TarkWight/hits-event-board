use axum::{Router, routing::get, routing::post, extract::{Path, Query, State}, Json};
use serde::Deserialize;
use uuid::Uuid;
use crate::{state::AppState, error::{ApiError, ApiResult}};
use crate::api::models::{User, Paged, Meta};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/users", get(list_users))
        .route("/api/v1/users/:id/approve", post(approve_user))
        .route("/api/v1/users/:id/reject", post(reject_user))
        .route("/api/v1/users/:id/block", post(block_user))
        .route("/api/v1/users/:id/unblock", post(unblock_user))
        .with_state(state)
}

#[derive(Deserialize)]
struct ListQ { page: Option<i32>, limit: Option<i32>, q: Option<String> }

async fn list_users(_st: State<AppState>, _q: Query<ListQ>) -> Json<Paged<User>> {
    let data = vec![];
    Json(Paged { data, meta: Meta { page: 1, limit: 20, total: 0 } })
}

async fn approve_user(State(st): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<()> {
    st.users.approve(id, Uuid::nil()).await?;
    Ok(())
}
async fn reject_user(Path(_id): Path<Uuid>) -> ApiResult<()> { Err(ApiError::NotImplemented) }
async fn block_user(Path(_id): Path<Uuid>) -> ApiResult<()> { Err(ApiError::NotImplemented) }
async fn unblock_user(Path(_id): Path<Uuid>) -> ApiResult<()> { Err(ApiError::NotImplemented) }
