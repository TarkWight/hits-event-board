use uuid::Uuid;
use crate::api::requests::company::CreateCompanyIn;
use crate::domain::entities::company::{Company, CompanyValidationError};
use crate::domain::entities::company_row::CompanyRow;

impl TryFrom<CreateCompanyIn> for CompanyRow {
    type Error = CompanyValidationError;
    fn try_from(v: CreateCompanyIn) -> Result<Self, Self::Error> {
        let company = Company::new(Uuid::new_v4(), v.name)?;
        Ok(company.into())
    }
}