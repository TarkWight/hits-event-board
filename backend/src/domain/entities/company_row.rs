use sqlx::FromRow;
use uuid::Uuid;
use crate::domain::entities::company::{Company, CompanyValidationError};
use crate::api::requests::company::CreateCompanyIn;

#[derive(Debug, Clone, FromRow)]
pub struct CompanyRow {
    pub id: Uuid,
    pub name: String,
}

impl TryFrom<CompanyRow> for Company {
    type Error = CompanyValidationError;
    fn try_from(row: CompanyRow) -> Result<Self, Self::Error> {
        Company::new(row.id, row.name)
    }
}

impl From<Company> for CompanyRow {
    fn from(domain: Company) -> Self {
        Self { id: domain.id, name: domain.name }
    }
}

impl TryFrom<CreateCompanyIn> for CompanyRow {
    type Error = String;

    fn try_from(value: CreateCompanyIn) -> Result<Self, Self::Error> {
        if value.name.trim().is_empty() {
            return Err("Company name must not be empty".to_string());
        }

        Ok(CompanyRow {
            id: uuid::Uuid::new_v4(),
            name: value.name,
        })
    }
}