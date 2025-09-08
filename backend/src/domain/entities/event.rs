use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

pub struct EventInvariants;

impl EventInvariants {
    pub fn check_time_span(start: OffsetDateTime, end: OffsetDateTime) -> Result<(), &'static str> {
        if end <= start {
            return Err("end_time must be after start_time");
        }
        Ok(())
    }

    pub fn check_deadline(deadline: OffsetDateTime, start: OffsetDateTime) -> Result<(), &'static str> {
        if deadline > start {
            return Err("registration_deadline must be <= start_time");
        }
        Ok(())
    }
}

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

impl EventRow {
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
        if let Some(capacity) = self.capacity {
            if capacity < 0 {
                return Err(EventValidationError::NegativeCapacity);
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
    #[error("capacity must be >= 0")]
    NegativeCapacity,
}