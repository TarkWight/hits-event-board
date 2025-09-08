use crate::api::requests::company::CreateCompanyIn;
use crate::api::models::company::CompanyOut;
use crate::domain::entities::company::{CompanyRow, Company, CompanyValidationError};

impl TryFrom<CreateCompanyIn> for CompanyRow {
    type Error = CompanyValidationError;
    fn try_from(value: CreateCompanyIn) -> Result<Self, Self::Error> {
        let row = CompanyRow { id: uuid::Uuid::new_v4(), name: value.name };
        Company::new(row.id, row.name.clone())?;
        Ok(row)
    }
}

impl From<CompanyRow> for CompanyOut {
    fn from(row: CompanyRow) -> Self {
        Self { id: row.id, name: row.name, manager_count: None, event_count: None }
    }
}