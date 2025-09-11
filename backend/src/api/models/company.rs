use serde::Serialize;
use uuid::Uuid;
use crate::domain::entities::company_row::CompanyStatus;
use crate::domain::entities::company::CompanyWithCounts;

#[derive(Debug, Serialize)]
pub struct CompanyOut {
    pub id: Uuid,
    pub name: String,
    pub status: CompanyStatus,
    pub manager_count: Option<i64>,
    pub event_count: Option<i64>,
}

impl From<CompanyWithCounts> for CompanyOut {
    fn from(v: CompanyWithCounts) -> Self {
        Self {
            id: v.id,
            name: v.name,
            status: v.status,
            manager_count: v.manager_count,
            event_count: v.event_count,
        }
    }
}