use uuid::Uuid;
use crate::infra::repositories::event_repo::{EventRepository, EventCreate, EventUpdate};
use crate::domain::entities::event::EventInvariants;
use crate::api::models::Event;
use crate::error::{ApiError, ApiResult};

#[derive(Clone)]
pub struct EventService<R: EventRepository + Send + Sync + 'static> { repo: R }

impl<R: EventRepository + Send + Sync + 'static> EventService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn list(&self, page: i32, limit: i32) -> ApiResult<Vec<Event>> {
        self.repo.list(page, limit, None, None, None, None, None).await
    }

    pub async fn create(&self, mut payload: EventCreate, manager_id: Uuid) -> ApiResult<Event> {
        EventInvariants::check_time_span(payload.start_time, payload.end_time).map_err(|e| ApiError::Unprocessable(e.into()))?;
        EventInvariants::check_deadline(payload.registration_deadline, payload.start_time).map_err(|e| ApiError::Unprocessable(e.into()))?;
        payload.manager_id = manager_id;
        self.repo.create(payload).await
    }

    pub async fn publish(&self, id: Uuid, _manager_id: Uuid) -> ApiResult<()> {
        self.repo.publish(id, true).await
    }

    pub async fn update(&self, id: Uuid, payload: EventUpdate) -> ApiResult<Event> {
        if let (Some(s), Some(e)) = (payload.start_time, payload.end_time) {
            EventInvariants::check_time_span(s, e).map_err(|e| ApiError::Unprocessable(e.into()))?;
        }
        if let (Some(d), Some(s)) = (payload.registration_deadline, payload.start_time) {
            EventInvariants::check_deadline(d, s).map_err(|e| ApiError::Unprocessable(e.into()))?;
        }
        self.repo.update(id, payload).await
    }
}
