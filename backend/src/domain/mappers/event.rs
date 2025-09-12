use uuid::Uuid;
use time::OffsetDateTime;

use crate::api::models::event::EventOut;
use crate::api::requests::event::CreateEventIn;
use crate::domain::entities::event::{Event, EventValidationError};
use crate::domain::entities::event_row::EventRow;

impl EventRow {
    pub fn from_manager_input(
        input: CreateEventIn,
        company_id: Uuid,
        manager_id: Uuid,
    ) -> Result<Self, EventValidationError> {
        let e = Event::new(
            Uuid::new_v4(),
            company_id,
            manager_id,
            input.title,
            input.short_desc,
            input.location,
            input.starts_at,
            input.ends_at,
            input.signup_deadline,
            input.capacity,
            input.is_published.unwrap_or(false),
        )?;
        Ok(e.into())
    }
}

// УБРАНО:
// impl TryFrom<CreateEventIn> for EventRow { ... } — больше не используем,
// т.к. company_id/manager_id берём из JWT, а не из тела запроса.

pub struct EventWithCount {
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