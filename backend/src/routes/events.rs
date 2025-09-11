use axum::{
    Router,
    routing::{get, post},
    extract::{Path, Query, State},
    Json,
};
use serde::Deserialize;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::state::AppState;
use crate::api::models::event::EventOut;
use crate::api::requests::event::{CreateEventIn, UpdateEventIn};
use crate::infra::repositories::event_repo::EventListFilter;
use crate::infra::security::rbac;
use crate::auth::extractor::AuthUser;
use crate::auth::roles::{ManagerStatus, Role};
use crate::error::ApiResult;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/events", get(list_events).post(create_event))
        .route("/api/v1/events/:id", get(get_event).patch(update_event).delete(delete_event))
        .route("/api/v1/events/:id/publish", post(publish_event))
        .route("/api/v1/events/:id/unpublish", post(unpublish_event))
        .route("/api/v1/events/:id/deadline", post(update_deadline))
        .route("/api/v1/events/:id/registrations", get(list_registrations))
        .route("/api/v1/events/:id/register", post(register_event))
        .route("/api/v1/events/:id/cancel", post(cancel_registration))
        .with_state(state)
}

#[derive(Deserialize)]
struct ListQ {
    page: Option<i32>,
    limit: Option<i32>,
    q: Option<String>,
    company_id: Option<Uuid>,
    manager_id: Option<Uuid>,
    published: Option<bool>,
    #[serde(with = "time::serde::rfc3339::option")] from: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")] to: Option<OffsetDateTime>,
}


async fn list_events(
    State(st): State<AppState>,
    user: Option<AuthUser>,
    q: Query<ListQ>,
) -> ApiResult<Json<Vec<EventOut>>> {
    let mut f = EventListFilter {
        company_id: q.company_id,
        manager_id: q.manager_id,
        published: q.published,
        q: q.q.clone(),
        from: q.from,
        to: q.to,
    };

    if let Some(company_id) = f.company_id {
        if let Some(u) = &user {
            let is_allowed = u.role == Role::Dean
                || (u.role == Role::Manager
                && u.company_id == Some(company_id)
                && matches!(u.manager_status, Some(ManagerStatus::Confirmed)));
            if !is_allowed {
                f.published = Some(true);
            }
        } else {
            f.published = Some(true);
        }
    } else {
        if user.is_none() {
            f.published = Some(true);
        }
    }

    let page = q.page.unwrap_or(1);
    let limit = q.limit.unwrap_or(20);
    Ok(Json(st.events.list(page, limit, f).await?))
}

async fn create_event(
    State(st): State<AppState>,
    user: AuthUser,
    Json(body): Json<CreateEventIn>,
) -> ApiResult<(http::StatusCode, Json<EventOut>)> {
    rbac::require_manager_confirmed(&user)?;

    let company_id = user.company_id.ok_or(crate::error::ApiError::Forbidden)?;
    let manager_id = user.user_id;

    let e = st.events.create(body, company_id, manager_id).await?;
    Ok((http::StatusCode::CREATED, Json(e)))
}

async fn get_event(State(st): State<AppState>, user: Option<AuthUser>, Path(id): Path<Uuid>)
                   -> ApiResult<Json<EventOut>> {
    let e = st.events.get(id).await?;

    if !e.is_published {
        if let Some(u) = user {
            let allowed = u.role == Role::Dean
                || (u.role == Role::Manager
                && u.company_id == Some(e.company_id)
                && matches!(u.manager_status, Some(ManagerStatus::Confirmed)));
            if !allowed {
                return Err(crate::error::ApiError::Forbidden);
            }
        } else {
            return Err(crate::error::ApiError::Forbidden);
        }
    }
    Ok(Json(e))
}

async fn update_event(
    State(st): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<UpdateEventIn>,
) -> ApiResult<Json<EventOut>> {
    let e = st.events.get(id).await?;
    rbac::require_dean_or_company_manager(&user, e.company_id)?;
    Ok(Json(st.events.update(id, body).await?))
}

async fn delete_event(State(st): State<AppState>, user: AuthUser, Path(id): Path<Uuid>) -> ApiResult<()> {
    let e = st.events.get(id).await?;
    rbac::require_dean_or_company_manager(&user, e.company_id)?;
    st.events.delete(id).await
}

#[derive(serde::Deserialize)]
struct DeadlineIn {
    #[serde(with = "time::serde::rfc3339::option")]
    deadline: Option<OffsetDateTime>,
}

async fn publish_event(State(st): State<AppState>, user: AuthUser, Path(id): Path<Uuid>)
                       -> ApiResult<Json<EventOut>> {
    let e = st.events.get(id).await?;
    rbac::require_dean_or_company_manager(&user, e.company_id)?;
    Ok(Json(st.events.set_published(id, true).await?))
}

async fn unpublish_event(State(st): State<AppState>, user: AuthUser, Path(id): Path<Uuid>)
                         -> ApiResult<Json<EventOut>> {
    let e = st.events.get(id).await?;
    rbac::require_dean_or_company_manager(&user, e.company_id)?;
    Ok(Json(st.events.set_published(id, false).await?))
}

async fn update_deadline(
    State(st): State<AppState>,
    user: AuthUser,
    Path(id): Path<Uuid>,
    Json(body): Json<DeadlineIn>,
) -> ApiResult<Json<EventOut>> {
    let e = st.events.get(id).await?;
    rbac::require_dean_or_company_manager(&user, e.company_id)?;
    Ok(Json(st.events.set_deadline(id, body.deadline).await?))
}

#[derive(serde::Serialize)]
pub struct RegistrationOut {
    pub student_id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub registered_at: OffsetDateTime,
}

async fn list_registrations(
    State(st): State<AppState>,
    user: AuthUser,
    Path(event_id): Path<Uuid>,
) -> ApiResult<Json<Vec<RegistrationOut>>> {
    let e = st.events.get(event_id).await?;
    rbac::require_dean_or_company_manager(&user, e.company_id)?;
    let rows = st.events.list_registrations(event_id).await?;
    Ok(Json(rows))
}

async fn register_event(
    State(st): State<AppState>,
    user: AuthUser,
    Path(event_id): Path<Uuid>,
) -> ApiResult<()> {
    rbac::require_student_confirmed(&user)?;
    st.events.register(event_id, user.user_id).await
}

async fn cancel_registration(
    State(st): State<AppState>,
    user: AuthUser,
    Path(event_id): Path<Uuid>,
) -> ApiResult<()> {
    rbac::require_student_confirmed(&user)?;
    st.events.cancel_registration(event_id, user.user_id).await
}