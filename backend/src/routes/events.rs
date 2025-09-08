use axum::{Router, routing::{get, post, patch, delete}, extract::{Path, Query, State}, Json};
use uuid::Uuid;
use serde::Deserialize;
use crate::{state::AppState, error::{ApiError, ApiResult}};
use crate::api::models::{Event, Paged, Meta, Registration};
use crate::infra::repositories::event_repo::{EventCreate, EventUpdate};

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
struct ListQ { page: Option<i32>, limit: Option<i32> }

async fn list_events(State(st): State<AppState>, q: Query<ListQ>) -> Json<Paged<Event>> {
    let data = st.events.list(q.page.unwrap_or(1), q.limit.unwrap_or(20)).await.unwrap_or_default();
    Json(Paged { data, meta: Meta { page: q.page.unwrap_or(1), limit: q.limit.unwrap_or(20), total: 0 } })
}

async fn create_event(State(st): State<AppState>, Json(body): Json<EventCreate>) -> (axum::http::StatusCode, Json<Event>) {
    let created = st.events.create(body, Uuid::nil()).await.expect("todo");
    (axum::http::StatusCode::CREATED, Json(created))
}

async fn get_event(Path(_id): Path<Uuid>) -> ApiResult<Json<Event>> { Err(ApiError::NotImplemented) }
async fn update_event(State(st): State<AppState>, Path(id): Path<Uuid>, Json(body): Json<EventUpdate>) -> ApiResult<Json<Event>> { Ok(Json(st.events.update(id, body).await.expect("todo"))) }
async fn delete_event(Path(_id): Path<Uuid>) -> ApiResult<()> { Err(ApiError::NotImplemented) }

async fn publish_event(State(st): State<AppState>, Path(id): Path<Uuid>) -> ApiResult<()> { st.events.publish(id, Uuid::nil()).await?; Ok(()) }
async fn unpublish_event(Path(_id): Path<Uuid>) -> ApiResult<()> { Err(ApiError::NotImplemented) }
async fn update_deadline(Path(_id): Path<Uuid>, Json(_body): Json<serde_json::Value>) -> ApiResult<()> { Err(ApiError::NotImplemented) }

async fn list_registrations(Path(_id): Path<Uuid>) -> ApiResult<Json<Vec<Registration>>> { Err(ApiError::NotImplemented) }
async fn register_event(Path(_id): Path<Uuid>, Json(_body): Json<serde_json::Value>) -> ApiResult<Json<Registration>> { Err(ApiError::NotImplemented) }
async fn cancel_registration(Path(_id): Path<Uuid>) -> ApiResult<()> { Err(ApiError::NotImplemented) }
