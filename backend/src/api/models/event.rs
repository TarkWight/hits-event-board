use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::entities::event::EventRow;

#[derive(Debug, Deserialize, Serialize)]
pub struct EventCommonFields {
    pub title: String,
    pub short_desc: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub starts_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub ends_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub signup_deadline: Option<OffsetDateTime>,
    pub location: Option<String>,
    pub capacity: Option<i32>,
    pub is_published: bool,
}

#[derive(Debug, Serialize)]
pub struct EventOut {
    pub id: Uuid,
    pub company_id: Uuid,
    pub manager_id: Uuid,
    pub title: String,
    pub short_desc: Option<String>,
    #[serde(with = "time::serde::rfc3339")]
    pub starts_at: OffsetDateTime,
    #[serde(with = "time::serde::rfc3339::option")]
    pub ends_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub signup_deadline: Option<OffsetDateTime>,
    pub location: Option<String>,
    pub registered_count: Option<i64>,
    pub capacity: Option<i32>,
    pub is_published: bool,
}

impl From<(EventRow, Option<i64>)> for EventOut {
    fn from((row, count): (EventRow, Option<i64>)) -> Self {
        Self {
            id: row.id,
            company_id: row.company_id,
            manager_id: row.manager_id,
            title: row.title,
            short_desc: row.description,
            starts_at: row.starts_at,
            ends_at: row.ends_at,
            signup_deadline: row.signup_deadline,
            location: row.location,
            registered_count: count,
            capacity: row.capacity,
            is_published: row.is_published,
        }
    }
}