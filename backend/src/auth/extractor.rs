use axum::{
    async_trait,
    extract::FromRequestParts,
    http::request::Parts,
};
use uuid::Uuid;

use crate::error::ApiError;
use crate::infra::security::jwt::{Claims, TokenService};
use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Role { Student, Manager, Dean }

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum ManagerStatus { Pending, Confirmed, Rejected }

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub role: Role,
    pub manager_status: Option<ManagerStatus>,
    pub company_id: Option<Uuid>,
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

        let _auth = parts.headers
            .get(http::header::AUTHORIZATION)
            .and_then(|h| h.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;
        let token = _auth.strip_prefix("Bearer ").ok_or(ApiError::Unauthorized)?;

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

        Ok(AuthUser {
            user_id: claims.user_id,
            role,
            manager_status,
            company_id: claims.company_id,
            raw: claims,
        })
    }
}