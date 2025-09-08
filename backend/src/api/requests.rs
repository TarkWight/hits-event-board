use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct LoginRequest { pub email: String, pub password: String }

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateCompanyIn { pub name: String }

#[derive(Debug, Deserialize)]
pub struct UpdateCompanyIn { pub name: Option<String> }

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
    pub capacity: Option<i32>,
    pub is_published: bool,
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
    pub capacity: Option<i32>,
    pub is_published: Option<bool>,
}
