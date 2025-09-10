use sqlx::FromRow;
use uuid::Uuid;
use crate::domain::entities::company::{Company, CompanyValidationError};

#[derive(Debug, Clone, FromRow)]
pub struct CompanyRow {
    pub id: Uuid,
    pub name: String,
}

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[sqlx(type_name = "company_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CompanyStatus {
    Active,
    Archived,
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