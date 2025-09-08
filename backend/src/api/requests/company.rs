use serde::Deserialize;
use uuid::Uuid;

use crate::domain::entities::company::{Company, CompanyValidationError};

#[derive(Debug, Deserialize)]
pub struct CreateCompanyIn {
    pub name: String,
}

impl From<CreateCompanyIn> for Company {
    fn from(value: CreateCompanyIn) -> Self {
        Company::new(Uuid::new_v4(), value.name).unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateCompanyIn {
    pub name: Option<String>,
}

impl Company {
    pub fn apply_update(&mut self, upd: UpdateCompanyIn) -> Result<(), CompanyValidationError> {
        if let Some(name) = upd.name {
            self.rename(name)?;
        }
        Ok(())
    }
}