use axum::{
    Router,
    routing::{get, post, patch},
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;
use crate::api::models::company::CompanyOut;
use crate::api::requests::company::{CreateCompanyIn, UpdateCompanyIn};
use crate::error::{ApiResult, ApiError};

// ---- Router ----
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/companies", get(list_companies).post(create_company))
        .route("/api/v1/companies/:id", get(get_company).patch(update_company))
        .route("/api/v1/companies/:id/managers", get(get_company_managers))
        .route("/api/v1/companies/:id/managers/invite", post(invite_manager))
        .route("/api/v1/companies/:id/managers/:user_id/approve", post(approve_manager))
        .with_state(state)
}

// ---- Query Params ----
#[derive(Deserialize)]
struct ListQ { page: Option<i32>, limit: Option<i32>, q: Option<String> }

// ---- Handlers ----
async fn list_companies(State(st): State<AppState>, q: Query<ListQ>) -> ApiResult<Json<Vec<CompanyOut>>> {
    let data = st.companies
        .list(q.page.unwrap_or(1), q.limit.unwrap_or(20), q.q.clone())
        .await?;
    Ok(Json(data))
}

async fn create_company(State(st): State<AppState>, Json(body): Json<CreateCompanyIn>) -> ApiResult<(axum::http::StatusCode, Json<CompanyOut>)> {
    let c = st.companies.create(body, Uuid::nil()).await?;
    Ok((axum::http::StatusCode::CREATED, Json(c)))
}

async fn get_company(State(st): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<Json<CompanyOut>> {
    let c = st.companies.get(id).await?;
    Ok(Json(c))
}

async fn update_company(State(st): State<AppState>, Path(id): Path<Uuid>, Json(body): Json<UpdateCompanyIn>) -> ApiResult<Json<CompanyOut>> {
    let c = st.companies.update(id, body).await?;
    Ok(Json(c))
}

// ---- Managers ----

// Заглушка структуры менеджера
#[derive(serde::Serialize)]
struct ManagerOut {
    id: Uuid,
    name: String,
}

async fn get_company_managers(State(st): State<AppState>, Path(company_id): Path<Uuid>) -> ApiResult<Json<Vec<ManagerOut>>> {
    // Здесь можно вызвать st.companies.get_managers(company_id).await
    // Пока заглушка:
    let managers = vec![];
    Ok(Json(managers))
}

#[derive(serde::Deserialize)]
struct InviteManagerIn {
    email: String,
}

async fn invite_manager(State(st): State<AppState>, Path(company_id): Path<Uuid>, Json(body): Json<InviteManagerIn>) -> ApiResult<()> {
    // st.companies.invite_manager(company_id, body.email).await?;
    Ok(())
}

async fn approve_manager(State(st): State<AppState>, Path((company_id, user_id)): Path<(Uuid, Uuid)>) -> ApiResult<()> {
    // st.companies.approve_manager(company_id, user_id).await?;
    Ok(())
}