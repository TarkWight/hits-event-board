use serde::Serialize;
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;
use crate::domain::mappers::event::EventWithCount;

#[derive(Debug, Serialize)]
pub struct EventOut {
    pub id: Uuid,
    pub company_id: Uuid,
    pub manager_id: Uuid,
    pub title: String,
    pub short_desc: Option<String>,
    pub location: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub starts_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub ends_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub signup_deadline: Option<OffsetDateTime>,
    pub registered_count: Option<i64>,
    pub capacity: Option<i32>,
    pub is_published: bool,
}


#[derive(Debug, FromRow)]
pub struct EventRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub manager_id: Uuid,
    pub title: String,
    pub short_desc: Option<String>,
    pub location: Option<String>,
    pub starts_at: OffsetDateTime,
    pub ends_at: Option<OffsetDateTime>,
    pub signup_deadline: Option<OffsetDateTime>,
    pub registered_count: Option<i64>,
    pub capacity: Option<i32>,
    pub is_published: bool,
}


impl From<EventRow> for EventOut {
    fn from(r: EventRow) -> Self {
        Self {
            id: r.id,
            company_id: r.company_id,
            manager_id: r.manager_id,
            title: r.title,
            short_desc: r.short_desc,
            location: r.location,
            starts_at: r.starts_at,
            ends_at: r.ends_at,
            signup_deadline: r.signup_deadline,
            registered_count: r.registered_count,
            capacity: r.capacity,
            is_published: r.is_published,
        }
    }
}