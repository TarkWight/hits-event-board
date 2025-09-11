use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::auth::extractor::AuthUser;
use crate::auth::roles::{ManagerStatus, UserRole, StudentStatus};

#[inline]
pub fn require_role(user: &AuthUser, allowed: &[UserRole]) -> ApiResult<()> {
    if allowed.iter().any(|r| *r == user.role) {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

#[inline]
pub fn require_dean(user: &AuthUser) -> ApiResult<()> {
    require_role(user, &[UserRole::Dean])
}

#[inline]
pub fn require_manager_confirmed(user: &AuthUser) -> ApiResult<()> {
    match (user.role, user.manager_status) {
        (UserRole::Dean, _) => Ok(()),
        (UserRole::Manager, Some(ManagerStatus::Confirmed)) => Ok(()),
        _ => Err(ApiError::Forbidden),
    }
}

#[inline]
pub fn require_manager_confirmed_of_company(user: &AuthUser, company_id: Uuid) -> ApiResult<()> {
    match (user.role, user.manager_status, user.company_id) {
        (UserRole::Dean, _, _) => Ok(()),
        (UserRole::Manager, Some(ManagerStatus::Confirmed), Some(cid)) if cid == company_id => Ok(()),
        _ => Err(ApiError::Forbidden),
    }
}

#[inline]
pub fn require_dean_or_company_manager(user: &AuthUser, company_id: Uuid) -> ApiResult<()> {
    if user.role == UserRole::Dean {
        return Ok(());
    }
    require_manager_confirmed_of_company(user, company_id)
}

#[inline]
pub fn require_student_confirmed(user: &AuthUser) -> ApiResult<()> {
    if user.role == UserRole::Dean { return Ok(()); }
    match (user.role, user.student_status) {
        (UserRole::Student, Some(StudentStatus::Confirmed)) => Ok(()),
        _ => Err(ApiError::Forbidden),
    }
}