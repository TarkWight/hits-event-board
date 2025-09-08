// src/requests/event.rs
use serde::Deserialize;
use time::OffsetDateTime;
use uuid::Uuid;

use crate::api::models::event::EventCommonFields;
use crate::domain::entities::event::{EventRow, EventValidationError};

// ---------------------------
// DTO In (API requests)
// ---------------------------
#[derive(Debug, Deserialize)]
pub struct CreateEventIn {
    pub company_id: Uuid,
    pub manager_id: Uuid,
    #[serde(flatten)]
    pub common: EventCommonFields,
}

impl TryFrom<CreateEventIn> for EventRow {
    type Error = EventValidationError;

    fn try_from(value: CreateEventIn) -> Result<Self, Self::Error> {
        let row = EventRow {
            id: Uuid::new_v4(),
            company_id: value.company_id,
            manager_id: value.manager_id,
            title: value.common.title,
            description: value.common.short_desc,
            location: value.common.location,
            starts_at: value.common.starts_at,
            ends_at: value.common.ends_at,
            signup_deadline: value.common.signup_deadline,
            capacity: value.common.capacity,
            is_published: value.common.is_published,
        };
        row.validate()?;
        Ok(row)
    }
}

// #[derive(Debug, Deserialize)]
pub struct UpdateEventIn {
    pub title: Option<String>,
    pub short_desc: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub starts_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub ends_at: Option<OffsetDateTime>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub signup_deadline: Option<OffsetDateTime>,
    pub location: Option<String>,
    pub capacity: Option<i32>,
    pub is_published: Option<bool>,
}

impl EventRow {
    pub fn apply_update(&mut self, upd: UpdateEventIn) -> Result<(), EventValidationError> {
        if let Some(title) = upd.title {
            self.title = title;
        }
        if let Some(desc) = upd.short_desc {
            self.description = Some(desc);
        }
        if let Some(loc) = upd.location {
            self.location = Some(loc);
        }
        if let Some(starts) = upd.starts_at {
            self.starts_at = starts;
        }
        if let Some(ends) = upd.ends_at {
            self.ends_at = Some(ends);
        }
        if let Some(deadline) = upd.signup_deadline {
            self.signup_deadline = Some(deadline);
        }
        if let Some(cap) = upd.capacity {
            self.capacity = Some(cap);
        }
        if let Some(pub_flag) = upd.is_published {
            self.is_published = pub_flag;
        }
        self.validate()
    }
}