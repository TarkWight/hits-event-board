use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RegistrationRow {
    pub event_id: Uuid,
    pub student_id: Uuid,
    pub student_name: String,
    pub student_email: String,
    #[serde(with = "time::serde::rfc3339")]
    pub registered_at: OffsetDateTime,
    pub gcal_event_id: Option<String>,
}

#[derive(sqlx::FromRow)]
struct RegistrationWithUserRow {
    student_id: Uuid,
    registered_at: OffsetDateTime,
    student_name: String,
    student_email: String,
}