use thiserror::Error;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Event {
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

#[derive(Debug, Error)]
pub enum EventValidationError {
    #[error("title must not be empty")]
    EmptyTitle,
    #[error("ends_at is before starts_at")]
    EndsBeforeStart,
    #[error("signup_deadline is after starts_at")]
    DeadlineAfterStart,
    #[error("capacity must be >= 0")]
    NegativeCapacity,
}

impl Event {
    pub fn new(
        id: Uuid,
        company_id: Uuid,
        manager_id: Uuid,
        title: String,
        description: Option<String>,
        location: Option<String>,
        starts_at: OffsetDateTime,
        ends_at: Option<OffsetDateTime>,
        signup_deadline: Option<OffsetDateTime>,
        capacity: Option<i32>,
        is_published: bool,
    ) -> Result<Self, EventValidationError> {
        let e = Self {
            id, company_id, manager_id, title, description, location,
            starts_at, ends_at, signup_deadline, capacity, is_published
        };
        e.validate()?;
        Ok(e)
    }

    pub fn validate(&self) -> Result<(), EventValidationError> {
        if self.title.trim().is_empty() {
            return Err(EventValidationError::EmptyTitle);
        }
        if let Some(ends) = self.ends_at {
            if ends < self.starts_at {
                return Err(EventValidationError::EndsBeforeStart);
            }
        }
        if let Some(dl) = self.signup_deadline {
            if dl > self.starts_at {
                return Err(EventValidationError::DeadlineAfterStart);
            }
        }
        if let Some(cap) = self.capacity {
            if cap < 0 {
                return Err(EventValidationError::NegativeCapacity);
            }
        }
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct EventPatch {
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub starts_at: Option<OffsetDateTime>,
    pub ends_at: Option<OffsetDateTime>,
    pub signup_deadline: Option<OffsetDateTime>,
    pub capacity: Option<i32>,
    pub is_published: Option<bool>,
}

impl Event {
    pub fn apply(&mut self, p: EventPatch) -> Result<(), EventValidationError> {
        if let Some(v) = p.title { self.title = v; }
        if let Some(v) = p.description { self.description = Some(v); }
        if let Some(v) = p.location { self.location = Some(v); }
        if let Some(v) = p.starts_at { self.starts_at = v; }
        if let Some(v) = p.ends_at { self.ends_at = Some(v); }
        if let Some(v) = p.signup_deadline { self.signup_deadline = Some(v); }
        if let Some(v) = p.capacity { self.capacity = Some(v); }
        if let Some(v) = p.is_published { self.is_published = v; }
        self.validate()
    }
}