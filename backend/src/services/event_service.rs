use uuid::Uuid;
use time::OffsetDateTime;

use crate::api::models::event::EventOut;
use crate::api::requests::event::{CreateEventIn, UpdateEventIn};
use crate::domain::entities::event::{Event, EventPatch};
use crate::domain::mappers::event::EventWithCount;
use crate::infra::repositories::event_repo::{EventRepository, EventListFilter};
use crate::error::{ApiResult, ApiError};

#[derive(Clone)]
pub struct EventService<R: EventRepository + Send + Sync + 'static> {
    repo: R,
}

impl<R: EventRepository + Send + Sync + 'static> EventService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn list(&self, page: i32, limit: i32, f: EventListFilter) -> ApiResult<Vec<EventOut>> {
        let rows: Vec<EventWithCount> = self.repo.list(page, limit, f).await?;
        Ok(rows.into_iter().map(EventOut::from).collect())
    }

    /// Создание события с явным контекстом менеджера/компании
    pub async fn create(&self, body: CreateEventIn, company_id: Uuid, manager_id: Uuid) -> ApiResult<EventOut> {
        let d = Event::new(
            Uuid::new_v4(),
            company_id,
            manager_id,
            body.title,
            body.short_desc,
            body.location,
            body.starts_at,
            body.ends_at,
            body.signup_deadline,
            body.capacity,
            body.is_published.unwrap_or(false),
        ).map_err(|e| ApiError::Unprocessable(e.to_string()))?;

        let saved = self.repo.create(d.into()).await?;
        Ok(EventOut::from(saved))
    }

    pub async fn get(&self, id: Uuid) -> ApiResult<EventOut> {
        Ok(self.repo.get(id).await?.into())
    }

    pub async fn update(&self, id: Uuid, patch_in: UpdateEventIn) -> ApiResult<EventOut> {
        let current = self.repo.get(id).await?;
        let mut d = Event::new(
            current.id, current.company_id, current.manager_id, current.title,
            current.description, current.location, current.starts_at, current.ends_at,
            current.signup_deadline, current.capacity, current.is_published
        ).expect("already validated");

        let patch: EventPatch = patch_in.into();
        d.apply(patch).map_err(|e| ApiError::Unprocessable(e.to_string()))?;
        Ok(self.repo.update_all(d.into()).await?.into())
    }

    pub async fn delete(&self, id: Uuid) -> ApiResult<()> {
        self.repo.delete(id).await?;
        Ok(())
    }

    pub async fn set_published(&self, id: Uuid, flag: bool) -> ApiResult<EventOut> {
        Ok(self.repo.set_published(id, flag).await?.into())
    }

    pub async fn set_deadline(&self, id: Uuid, deadline: Option<OffsetDateTime>) -> ApiResult<EventOut> {
        if let Some(dl) = deadline {
            let e = self.repo.get(id).await?;
            if dl > e.starts_at {
                return Err(ApiError::Unprocessable("deadline must be <= starts_at".into()));
            }
        }
        Ok(self.repo.set_deadline(id, deadline).await?.into())
    }

    pub async fn list_registrations(&self, event_id: Uuid) -> ApiResult<Vec<crate::routes::events::RegistrationOut>> {
        let regs = self.repo.list_registrations(event_id).await?;
        Ok(regs.into_iter()
            .map(|(student_id, registered_at)| crate::routes::events::RegistrationOut { student_id, registered_at })
            .collect())
    }

    pub async fn register(&self, event_id: Uuid, student_id: Uuid) -> ApiResult<()> {
        let now = OffsetDateTime::now_utc();
        let e = self.repo.get(event_id).await?;
        if !e.is_published {
            return Err(ApiError::PreconditionFailed("event not published".into()));
        }
        if let Some(dl) = e.signup_deadline {
            if now > dl {
                return Err(ApiError::PreconditionFailed("deadline passed".into()));
            }
        }
        if let Some(cap) = e.capacity {
            let used = self.repo.count_registrations(event_id).await?;
            if used >= cap as i64 {
                return Err(ApiError::PreconditionFailed("no seats available".into()));
            }
        }
        self.repo.register(event_id, student_id, now).await?;
        Ok(())
    }

    pub async fn cancel_registration(&self, event_id: Uuid, student_id: Uuid) -> ApiResult<()> {
        self.repo.cancel_registration(event_id, student_id, OffsetDateTime::now_utc()).await?;
        Ok(())
    }
}