mod event;

use serde::{Serialize, Deserialize};
use uuid::Uuid;
use time::OffsetDateTime;

#[derive(Debug, Serialize)]
pub struct CompanyOut {
    pub id: Uuid,
    pub name: String,
    pub manager_count: Option<i64>,
    pub event_count: Option<i64>,
}

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
    pub registered_count: Option<i64>,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Meta { pub page: i32, pub limit: i32, pub total: i32 }

#[derive(Serialize, Deserialize, Clone)]
pub struct Paged<T> {
    pub data: Vec<T>,
    pub meta: Meta,
}
