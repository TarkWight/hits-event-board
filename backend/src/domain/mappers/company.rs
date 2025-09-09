use uuid::Uuid;

use crate::api::models::company::CompanyOut;
use crate::api::requests::company::CreateCompanyIn;
use crate::domain::entities::company::{Company, CompanyValidationError};
use crate::domain::entities::company_row::CompanyRow;

/// CreateCompanyIn -> CompanyRow (валидация на доменной сущности)
impl TryFrom<CreateCompanyIn> for CompanyRow {
    type Error = CompanyValidationError;
    fn try_from(v: CreateCompanyIn) -> Result<Self, Self::Error> {
        let company = Company::new(Uuid::new_v4(), v.name)?;
        Ok(company.into())
    }
}

/// Маппер infra-проекции с аггрегатами -> CompanyOut (используется в репозитории)
pub struct CompanyWithCounts {
    pub id: Uuid,
    pub name: String,
    pub manager_count: Option<i64>,
    pub event_count: Option<i64>,
}

impl From<CompanyWithCounts> for CompanyOut {
    fn from(v: CompanyWithCounts) -> Self {
        CompanyOut {
            id: v.id,
            name: v.name,
            manager_count: v.manager_count,
            event_count: v.event_count,
        }
    }
}

/// Базовый маппер Row->Out (без счётчиков)
impl From<CompanyRow> for CompanyOut {
    fn from(row: CompanyRow) -> Self {
        CompanyOut { id: row.id, name: row.name, manager_count: None, event_count: None }
    }
}