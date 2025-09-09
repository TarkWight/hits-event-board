use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::entities::event::{Event, EventValidationError};

#[derive(Debug, Clone, FromRow)]
pub struct EventRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub manager_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub starts_at: OffsetDateTime,
    pub ends_at: Option<OffsetDateTime>,
    pub signup_deadline: Option<OffsetDateTime>,
    pub capacity: Option<i32>,
    pub is_published: bool,
}

impl TryFrom<EventRow> for Event {
    type Error = EventValidationError;
    fn try_from(r: EventRow) -> Result<Self, Self::Error> {
        Event::new(
            r.id, r.company_id, r.manager_id, r.title, r.description, r.location,
            r.starts_at, r.ends_at, r.signup_deadline, r.capacity, r.is_published,
        )
    }
}

impl From<Event> for EventRow {
    fn from(d: Event) -> Self {
        Self {
            id: d.id,
            company_id: d.company_id,
            manager_id: d.manager_id,
            title: d.title,
            description: d.description,
            location: d.location,
            starts_at: d.starts_at,
            ends_at: d.ends_at,
            signup_deadline: d.signup_deadline,
            capacity: d.capacity,
            is_published: d.is_published,
        }
    }
}