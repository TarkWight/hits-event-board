use serde::Serialize;
use uuid::Uuid;

use crate::domain::entities::company::Company;

#[derive(Debug, Serialize)]
pub struct CompanyOut {
    pub id: Uuid,
    pub name: String,
    pub manager_count: Option<i64>,
    pub event_count: Option<i64>,
}

impl From<(Company, Option<i64>, Option<i64>)> for CompanyOut {
    fn from((company, manager_count, event_count): (Company, Option<i64>, Option<i64>)) -> Self {
        Self {
            id: company.id,
            name: company.name,
            manager_count,
            event_count,
        }
    }
}