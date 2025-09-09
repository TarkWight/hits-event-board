use axum::{
    Router,
    routing::{get, post, patch, delete},
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
    page: Option<i32>, limit: Option<i32>, q: Option<String>,
    company_id: Option<Uuid>, manager_id: Option<Uuid>, published: Option<bool>,
    #[serde(with = "time::serde::rfc3339::option")] from: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")] to: Option<OffsetDateTime>,
}

async fn list_events(State(st): State<AppState>, q: Query<ListQ>) -> ApiResult<Json<Vec<EventOut>>> {
    let f = EventListFilter {
        company_id: q.company_id,
        manager_id: q.manager_id,
        published: q.published,
        q: q.q.clone(),
        from: q.from,
        to: q.to,
    };
    Ok(Json(st.events.list(q.page.unwrap_or(1), q.limit.unwrap_or(20), f).await?))
}

async fn create_event(State(st): State<AppState>, Json(body): Json<CreateEventIn>)
                      -> ApiResult<(axum::http::StatusCode, Json<EventOut>)>
{
    let e = st.events.create(body, Uuid::nil()).await?;
    Ok((axum::http::StatusCode::CREATED, Json(e)))
}

async fn get_event(State(st): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<Json<EventOut>> {
    Ok(Json(st.events.get(id).await?))
}

async fn update_event(State(st): State<AppState>, Path(id): Path<Uuid>, Json(body): Json<UpdateEventIn>)
                      -> ApiResult<Json<EventOut>>
{
    Ok(Json(st.events.update(id, body).await?))
}

async fn delete_event(State(st): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<()> {
    st.events.delete(id).await
}

#[derive(serde::Deserialize)]
struct DeadlineIn {
    #[serde(with = "time::serde::rfc3339::option")]
    deadline: Option<OffsetDateTime>,
}

async fn publish_event(State(st): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<Json<EventOut>> {
    Ok(Json(st.events.set_published(id, true).await?))
}

async fn unpublish_event(State(st): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<Json<EventOut>> {
    Ok(Json(st.events.set_published(id, false).await?))
}

async fn update_deadline(State(st): State<AppState>, Path(id): Path<Uuid>, Json(body): Json<DeadlineIn>)
                         -> ApiResult<Json<EventOut>>
{
    Ok(Json(st.events.set_deadline(id, body.deadline).await?))
}

#[derive(serde::Serialize)]
pub struct RegistrationOut {
    pub student_id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub registered_at: OffsetDateTime,
}

async fn list_registrations(State(st): State<AppState>, Path(event_id): Path<Uuid>)
                            -> ApiResult<Json<Vec<RegistrationOut>>>
{
    let rows = st.events.list_registrations(event_id).await?;
    Ok(Json(rows))
}

#[derive(serde::Deserialize)]
struct RegisterIn { student_id: Uuid }

async fn register_event(State(st): State<AppState>, Path(event_id): Path<Uuid>, Json(body): Json<RegisterIn>)
                        -> ApiResult<()>
{
    st.events.register(event_id, body.student_id).await
}

async fn cancel_registration(State(st): State<AppState>, Path(event_id): Path<Uuid>, Json(body): Json<RegisterIn>)
                             -> ApiResult<()>
{
    st.events.cancel_registration(event_id, body.student_id).await
}