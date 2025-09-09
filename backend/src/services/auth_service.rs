use crate::infra::repositories::user_repo::UserRepository;
use crate::infra::security::password;
use crate::infra::security::password_policy;
use crate::error::{ApiError, ApiResult};

use crate::api::requests::student_register::StudentRegisterRequest;
use crate::infra::repositories::user_repo::UserRow;

#[derive(Clone)]
pub struct AuthService<R: UserRepository + Send + Sync + 'static> { repo: R }

impl<R: UserRepository + Send + Sync + 'static> AuthService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn register_student(&self, req: StudentRegisterRequest) -> ApiResult<UserRow> {
        if req.name.trim().is_empty() {
            return Err(ApiError::Unprocessable("name must not be empty".into()));
        }
        if !req.email.contains('@') {
            return Err(ApiError::Unprocessable("email is invalid".into()));
        }
        password_policy::validate(&req.password).map_err(|m| ApiError::Unprocessable(m.into()))?;

        let hash = password::hash_password(&req.password)
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let created = self.repo
            .create_student(&req.name, &req.email, &hash)
            .await?;

        Ok(created)
    }
}