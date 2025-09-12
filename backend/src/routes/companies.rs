use axum::{
    Router,
    routing::{get, post, patch},
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::state::AppState;
use crate::error::ApiResult;
use crate::api::models::company::CompanyOut;
use crate::api::requests::company::{CreateCompanyIn, UpdateCompanyIn};
use crate::infra::security::rbac;
use crate::auth::extractor::AuthUser;

use crate::api::models::manager::ManagerOut;
use crate::auth::roles::ManagerStatus as DManagerStatus;

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ListQ {
    page: Option<i32>,
    limit: Option<i32>,
    q: Option<String>,
    include_archived: Option<bool>,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/companies", get(list_companies).post(create_company))
        .route("/api/v1/companies/admin", get(list_companies_admin))
        .route("/api/v1/companies/:id", get(get_company).patch(update_company))
        .route("/api/v1/companies/:id/status/:status", post(set_company_status))
        .route("/api/v1/companies/:id/managers", get(get_company_managers))
        .route(
            "/api/v1/companies/:id/managers/:user_id/status/:status",
            post(set_manager_status),
        )
        .with_state(state)
}

// ---------- списки ----------

async fn list_companies(
    State(st): State<AppState>,
    q: Query<ListQ>,
) -> ApiResult<Json<Vec<CompanyOut>>> {
    let data = st
        .companies
        .list(q.page.unwrap_or(1), q.limit.unwrap_or(20), q.q.clone())
        .await?;
    Ok(Json(data))
}

async fn list_companies_admin(
    State(st): State<AppState>,
    user: AuthUser,
    q: Query<ListQ>,
) -> ApiResult<Json<Vec<CompanyOut>>> {
    rbac::require_dean(&user)?;
    let data = st
        .companies
        .list_admin(
            q.page.unwrap_or(1),
            q.limit.unwrap_or(20),
            q.q.clone(),
            q.include_archived.unwrap_or(false),
        )
        .await?;
    Ok(Json(data))
}

// ---------- CRUD ----------

async fn create_company(
    State(st): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateCompanyIn>,
) -> ApiResult<(axum::http::StatusCode, Json<CompanyOut>)> {
    rbac::require_dean(&user)?;
    let c = st.companies.create(body, user.user_id).await?;
    Ok((axum::http::StatusCode::CREATED, Json(c)))
}

async fn get_company(
    State(st): State<AppState>,
    Path(id): Path<Uuid>,
) -> ApiResult<Json<CompanyOut>> {
    Ok(Json(st.companies.get(id).await?))
}

async fn update_company(
    State(st): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateCompanyIn>,
) -> ApiResult<Json<CompanyOut>> {
    rbac::require_dean(&user)?;
    Ok(Json(st.companies.update(id, body).await?))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum StatusParam {
    Active,
    Archived,
}

async fn set_company_status(
    State(st): State<AppState>,
    user: AuthUser,
    Path((id, status)): Path<(Uuid, StatusParam)>,
) -> ApiResult<Json<CompanyOut>> {
    rbac::require_dean(&user)?;
    use crate::domain::entities::company_row::CompanyStatus;
    let target = match status {
        StatusParam::Active => CompanyStatus::Active,
        StatusParam::Archived => CompanyStatus::Archived,
    };
    let out = st.companies.set_status(id, target).await?;
    Ok(Json(out))
}

// ---------- менеджеры компании ----------

async fn get_company_managers(
    State(st): State<AppState>,
    user: AuthUser,
    Path(company_id): Path<Uuid>,
) -> ApiResult<Json<Vec<ManagerOut>>> {
    rbac::require_dean_or_company_manager(&user, company_id)?;
    let rows = st.managers.list_for_company(company_id).await?;
    Ok(Json(rows))
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "lowercase")]
enum ManagerStatusParam {
    Pending,
    Confirmed,
    Rejected,
}

async fn set_manager_status(
    State(st): State<AppState>,
    user: AuthUser,
    Path((company_id, user_id, status)): Path<(Uuid, Uuid, ManagerStatusParam)>,
) -> ApiResult<()> {
    rbac::require_dean_or_company_manager(&user, company_id)?;
    let target = match status {
        ManagerStatusParam::Pending => DManagerStatus::Pending,
        ManagerStatusParam::Confirmed => DManagerStatus::Confirmed,
        ManagerStatusParam::Rejected => DManagerStatus::Rejected,
    };
    st.managers.set_status(company_id, user_id, target).await?;
    Ok(())
}