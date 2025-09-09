use serde::Serialize;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct CompanyOut {
    pub id: Uuid,
    pub name: String,
    pub manager_count: Option<i64>,
    pub event_count: Option<i64>,
}