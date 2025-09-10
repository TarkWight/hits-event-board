use uuid::Uuid;
use crate::auth::extractor::{AuthUser, Role, ManagerStatus};
use crate::error::ApiError;

impl AuthUser {
    #[inline] pub fn is_dean(&self) -> bool { self.role == Role::Dean }

    #[inline]
    pub fn is_manager_confirmed_for(&self, company_id: Uuid) -> bool {
        self.role == Role::Manager
            && self.company_id == Some(company_id)
            && self.manager_status == Some(ManagerStatus::Confirmed)
    }

    #[inline]
    pub fn require_dean(&self) -> Result<(), ApiError> {
        if self.is_dean() { Ok(()) } else { Err(ApiError::Forbidden) }
    }

    #[inline]
    pub fn require_dean_or_confirmed_manager_of(&self, company_id: Uuid) -> Result<(), ApiError> {
        if self.is_dean() || self.is_manager_confirmed_for(company_id) {
            Ok(())
        } else {
            Err(ApiError::Forbidden)
        }
    }
}