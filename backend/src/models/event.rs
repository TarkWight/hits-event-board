use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

// ---------------------------
// DB row (TPT/TPH-агностично)
// ---------------------------
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
}

impl EventRow {
    /// Простая валидация бизнес-инвариантов. Вызывай перед INSERT/UPDATE.
    pub fn validate(&self) -> Result<(), EventValidationError> {
        if let Some(ends) = self.ends_at {
            if ends < self.starts_at {
                return Err(EventValidationError::EndsBeforeStart);
            }
        }
        if let Some(deadline) = self.signup_deadline {
            if deadline > self.starts_at {
                return Err(EventValidationError::DeadlineAfterStart);
            }
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum EventValidationError {
    #[error("ends_at is before starts_at")]
    EndsBeforeStart,
    #[error("signup_deadline is after starts_at")]
    DeadlineAfterStart,
}

// ---------------------------
// API DTOs (внешние контракты)
// ---------------------------
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
    // Часто полезно вернуть количество записавшихся, а не весь список
    pub registered_count: Option<i64>,
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
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateEventIn {
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
}

impl TryFrom<CreateEventIn> for EventRow {
    type Error = EventValidationError;
    fn try_from(value: CreateEventIn) -> Result<Self, Self::Error> {
        let row = EventRow {
            id: Uuid::new_v4(),
            company_id: value.company_id,
            manager_id: value.manager_id,
            title: value.title,
            description: value.short_desc,
            location: value.location,
            starts_at: value.starts_at,
            ends_at: value.ends_at,
            signup_deadline: value.signup_deadline,
        };
        row.validate()?;
        Ok(row)
    }
}

#[derive(Debug, Deserialize)]
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
}

impl EventRow {
    /// Применить частичное обновление + провалидировать инварианты.
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
        self.validate()
    }
}

// -------------------------------------
// Связи (регистрации студентов на эвент)
// -------------------------------------
#[derive(Debug, Clone, FromRow)]
pub struct EventRegistrationRow {
    pub event_id: Uuid,
    pub student_id: Uuid,
    pub registered_at: OffsetDateTime,
}

/// Утилита для сборки EventOut + count из двух запросов
pub struct EventWithCount {
    pub event: EventRow,
    pub registered_count: Option<i64>,
}

impl From<EventWithCount> for EventOut {
    fn from(value: EventWithCount) -> Self {
        (value.event, value.registered_count).into()
    }
}
