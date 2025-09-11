use serde::Deserialize;
use time::OffsetDateTime;

use crate::domain::entities::event::{Event, EventPatch};

#[derive(Debug, Deserialize)]
pub struct CreateEventIn {
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
    pub is_published: Option<bool>,
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