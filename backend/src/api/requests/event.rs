use serde::Deserialize;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::entities::event::{Event, EventPatch};

#[derive(Debug, Deserialize)]
pub struct CreateEventIn {
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
    pub capacity: Option<i32>,
    pub is_published: bool,
}

impl From<CreateEventIn> for Event {
    fn from(v: CreateEventIn) -> Self {
        Event::new(
            Uuid::new_v4(),
            v.company_id,
            v.manager_id,
            v.title,
            v.short_desc,
            v.location,
            v.starts_at,
            v.ends_at,
            v.signup_deadline,
            v.capacity,
            v.is_published,
        ).expect("validated")
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateEventIn {
    pub title: Option<String>,
    pub short_desc: Option<String>,
    pub location: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub starts_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub ends_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub signup_deadline: Option<OffsetDateTime>,
    pub capacity: Option<i32>,
    pub is_published: Option<bool>,
}

impl From<UpdateEventIn> for EventPatch {
    fn from(v: UpdateEventIn) -> Self {
        EventPatch {
            title: v.title,
            description: v.short_desc,
            location: v.location,
            starts_at: v.starts_at,
            ends_at: v.ends_at,
            signup_deadline: v.signup_deadline,
            capacity: v.capacity,
            is_published: v.is_published,
        }
    }
}