use uuid::Uuid;

use crate::error::{ApiError, ApiResult};
use crate::auth::extractor::AuthUser;
use crate::auth::roles::{ManagerStatus, Role};

#[inline]
pub fn require_role(user: &AuthUser, allowed: &[Role]) -> ApiResult<()> {
    if allowed.iter().any(|r| *r == user.role) {
        Ok(())
    } else {
        Err(ApiError::Forbidden)
    }
}

#[inline]
pub fn require_dean(user: &AuthUser) -> ApiResult<()> {
    require_role(user, &[Role::Dean])
}

#[inline]
pub fn require_manager_confirmed(user: &AuthUser) -> ApiResult<()> {
    match (user.role, user.manager_status) {
        (Role::Dean, _) => Ok(()),
        (Role::Manager, Some(ManagerStatus::Confirmed)) => Ok(()),
        _ => Err(ApiError::Forbidden),
    }
}

#[inline]
pub fn require_manager_confirmed_of_company(user: &AuthUser, company_id: Uuid) -> ApiResult<()> {
    match (user.role, user.manager_status, user.company_id) {
        (Role::Dean, _, _) => Ok(()),
        (Role::Manager, Some(ManagerStatus::Confirmed), Some(cid)) if cid == company_id => Ok(()),
        _ => Err(ApiError::Forbidden),
    }
}

#[inline]
pub fn require_dean_or_company_manager(user: &AuthUser, company_id: Uuid) -> ApiResult<()> {
    if user.role == Role::Dean {
        return Ok(());
    }
    require_manager_confirmed_of_company(user, company_id)
}

#[inline]
pub fn require_student_confirmed(user: &AuthUser) -> ApiResult<()> {
    if user.role == Role::Dean {
        return Ok(());
    }
    if user.role == Role::Student && user.student_confirmed.unwrap_or(false) {
        return Ok(());
    }
    Err(ApiError::Forbidden)
}