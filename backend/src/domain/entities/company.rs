use thiserror::Error;
use uuid::Uuid;

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
        if name.trim().is_empty() {
            return Err(CompanyValidationError::EmptyName);
        }
        Ok(Self { id, name })
    }

    pub fn apply_name(&mut self, name: String) -> Result<(), CompanyValidationError> {
        if name.trim().is_empty() {
            return Err(CompanyValidationError::EmptyName);
        }
        self.name = name;
        Ok(())
    }
}

#[derive(Debug, Default, Clone)]
pub struct CompanyPatch {
    pub name: Option<String>,
}

impl Company {
    pub fn apply(&mut self, patch: CompanyPatch) -> Result<(), CompanyValidationError> {
        if let Some(name) = patch.name {
            self.apply_name(name)?;
        }
        Ok(())
    }
}