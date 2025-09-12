use uuid::Uuid;
use thiserror::Error;
use crate::auth::roles::ManagerStatus;

#[derive(Debug, Clone)]
pub struct Manager {
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub name: String,
    pub email: String,
    pub status: ManagerStatus,
}

#[derive(Debug, Error)]
pub enum ManagerValidationError {
    #[error("manager name must not be empty")]
    EmptyName,
    #[error("email must not be empty")]
    EmptyEmail,
}

impl Manager {
    pub fn validate(&self) -> Result<(), ManagerValidationError> {
        if self.name.trim().is_empty() {
            return Err(ManagerValidationError::EmptyName);
        }
        if self.email.trim().is_empty() {
            return Err(ManagerValidationError::EmptyEmail);
        }
        Ok(())
    }
}