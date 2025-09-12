use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RegistrationRow {
    pub event_id: Uuid,
    pub student_id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub registered_at: OffsetDateTime,
    pub gcal_event_id: Option<String>,
}
