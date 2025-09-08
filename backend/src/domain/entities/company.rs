use uuid::Uuid;
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct Company {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Error)]
pub enum CompanyValidationError {
    #[error("company name must not be empty")]
    EmptyName,
}

impl Company {
    pub fn new(id: Uuid, name: String) -> Result<Self, CompanyValidationError> {
        if name.trim().is_empty() { return Err(CompanyValidationError::EmptyName); }
        Ok(Self { id, name })
    }

    pub fn apply_name(&mut self, name: String) -> Result<(), CompanyValidationError> {
        if name.trim().is_empty() { return Err(CompanyValidationError::EmptyName); }
        self.name = name;
        Ok(())
    }

    pub fn apply_update(&mut self, upd: crate::api::requests::company::UpdateCompanyIn) -> Result<(), CompanyValidationError> {
        if let Some(name) = upd.name {
            self.apply_name(name)?;
        }
        Ok(())
    }
}