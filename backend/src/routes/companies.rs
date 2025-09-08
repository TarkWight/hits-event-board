use axum::{Router, routing::{get, post, patch}, extract::{Path, Query, State}, Json};
use serde::Deserialize;
use uuid::Uuid;
use crate::{state::AppState, error::{ApiError, ApiResult}};
use crate::api::models::{Company, Paged, Meta};
use crate::infra::repositories::company_repo::{CompanyCreate, CompanyUpdate};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/companies", get(list_companies).post(create_company))
        .route("/api/v1/companies/:id", get(get_company).patch(update_company))
        .route("/api/v1/companies/:id/managers", get(get_company_managers))
        .route("/api/v1/companies/:id/managers/invite", post(invite_manager))
        .route("/api/v1/companies/:id/managers/:userId/approve", post(approve_manager))
        .with_state(state)
}

#[derive(Deserialize)]
struct ListQ { page: Option<i32>, limit: Option<i32>, q: Option<String> }

async fn list_companies(State(st): State<AppState>, q: Query<ListQ>) -> Json<Paged<Company>> {
    let data = st.companies.list(q.page.unwrap_or(1), q.limit.unwrap_or(20), q.q.clone()).await.unwrap_or_default();
    Json(Paged { data, meta: Meta { page: q.page.unwrap_or(1), limit: q.limit.unwrap_or(20), total: 0 } })
}

async fn create_company(State(st): State<AppState>, Json(body): Json<CompanyCreate>) -> (axum::http::StatusCode, Json<Company>) {
    let c = st.companies.create(body, Uuid::nil()).await.expect("todo");
    (axum::http::StatusCode::CREATED, Json(c))
}

async fn get_company(State(st): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<Json<Company>> { Ok(Json(st.companies.get(id).await.expect("todo"))) }
async fn update_company(State(st): State<AppState>, Path(id): Path<Uuid>, Json(body): Json<CompanyUpdate>) -> ApiResult<Json<Company>> { Ok(Json(st.companies.update(id, body).await.expect("todo"))) }

async fn get_company_managers(Path(_id): Path<Uuid>) -> ApiResult<Json<serde_json::Value>> { Err(ApiError::NotImplemented) }
async fn invite_manager(Path(_id): Path<Uuid>, Json(_body): Json<serde_json::Value>) -> ApiResult<()> { Err(ApiError::NotImplemented) }
async fn approve_manager(Path((_id, _user_id)): Path<(Uuid, Uuid)>) -> ApiResult<()> { Err(ApiError::NotImplemented) }
