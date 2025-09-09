use uuid::Uuid;

use crate::api::models::event::EventOut;
use crate::api::requests::event::CreateEventIn;
use crate::domain::entities::event::{Event, EventValidationError};
use crate::domain::entities::event_row::EventRow;

impl TryFrom<CreateEventIn> for EventRow {
    type Error = EventValidationError;
    fn try_from(v: CreateEventIn) -> Result<Self, Self::Error> {
        let d = Event::from(v);
        Ok(d.into())
    }
}

pub struct EventWithCount {
    pub id: Uuid,
    pub company_id: Uuid,
    pub manager_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub starts_at: time::OffsetDateTime,
    pub ends_at: Option<time::OffsetDateTime>,
    pub signup_deadline: Option<time::OffsetDateTime>,
    pub capacity: Option<i32>,
    pub is_published: bool,
    pub registered_count: Option<i64>,
}

impl From<EventWithCount> for EventOut {
    fn from(v: EventWithCount) -> Self {
        Self {
            id: v.id,
            company_id: v.company_id,
            manager_id: v.manager_id,
            title: v.title,
            short_desc: v.description,
            location: v.location,
            starts_at: v.starts_at,
            ends_at: v.ends_at,
            signup_deadline: v.signup_deadline,
            registered_count: v.registered_count,
            capacity: v.capacity,
            is_published: v.is_published,
        }
    }
}

impl From<EventRow> for EventOut {
    fn from(r: EventRow) -> Self {
        Self {
            id: r.id,
            company_id: r.company_id,
            manager_id: r.manager_id,
            title: r.title,
            short_desc: r.description,
            location: r.location,
            starts_at: r.starts_at,
            ends_at: r.ends_at,
            signup_deadline: r.signup_deadline,
            registered_count: None,
            capacity: r.capacity,
            is_published: r.is_published,
        }
    }
}