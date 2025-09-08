use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Company {
    pub id: Uuid,
    pub name: String,
}

impl Company {
    pub fn new(id: Uuid, name: String) -> Result<Self, CompanyValidationError> {
        let c = Self { id, name };
        c.validate()?;
        Ok(c)
    }

    pub fn validate(&self) -> Result<(), CompanyValidationError> {
        if self.name.trim().is_empty() {
            return Err(CompanyValidationError::EmptyName);
        }
        Ok(())
    }

    pub fn rename(&mut self, new_name: String) -> Result<(), CompanyValidationError> {
        self.name = new_name;
        self.validate()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CompanyValidationError {
    #[error("company name must not be empty")]
    EmptyName,
}