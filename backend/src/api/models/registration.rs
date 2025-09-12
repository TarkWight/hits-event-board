use serde::Serialize;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct RegistrationOut {
    pub student_id: Uuid,
    #[serde(with = "time::serde::rfc3339")]
    pub registered_at: OffsetDateTime,
}
