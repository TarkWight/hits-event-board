use sqlx::FromRow;
use uuid::Uuid;

use crate::domain::entities::company::{Company, CompanyValidationError};

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