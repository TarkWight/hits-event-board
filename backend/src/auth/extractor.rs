use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use uuid::Uuid;
use crate::auth::roles::{ManagerStatus, Role};
use crate::error::ApiError;
use crate::infra::security::jwt::{Claims, TokenService};
use crate::state::AppState;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub role: Role,
    pub manager_status: Option<ManagerStatus>,
    pub company_id: Option<Uuid>,
    pub student_confirmed: Option<bool>,
    pub raw: Claims,
}

#[derive(Clone)]
pub struct AuthState {
    pub token_service: TokenService,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, Self::Rejection> {
        let auth = state.auth.clone();

        let authz = parts.headers
            .get(http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;
        let token = authz.strip_prefix("Bearer ").ok_or(ApiError::Unauthorized)?;

        let claims = auth.token_service
            .validate_token(token)
            .map_err(|_| ApiError::Unauthorized)?;

        let role = match claims.role.as_str() {
            "student" => Role::Student,
            "manager" => Role::Manager,
            "dean"    => Role::Dean,
            _ => return Err(ApiError::Forbidden),
        };

        let manager_status = claims.manager_status.as_deref().map(|s| match s {
            "pending"   => ManagerStatus::Pending,
            "confirmed" => ManagerStatus::Confirmed,
            "rejected"  => ManagerStatus::Rejected,
            _ => ManagerStatus::Rejected,
        });

        let student_confirmed: Option<bool> = match role {
            Role::Student => Some(claims.student_confirmed.unwrap_or(false)),
            _ => None,
        };

        Ok(AuthUser {
            user_id: claims.user_id,
            role,
            manager_status,
            company_id: claims.company_id,
            student_confirmed,
            raw: claims,
        })
    }
}