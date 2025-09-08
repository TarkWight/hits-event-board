use axum::{extract::FromRequestParts};
use http::{request::Parts, StatusCode};
use crate::infra::security::jwt::decode_token;

#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: String,
    pub role: String,
}

#[axum::async_trait]
impl<S> FromRequestParts<S> for AuthUser
where S: Send + Sync {
    type Rejection = (StatusCode, String);
    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let Some(header) = parts.headers.get(http::header::AUTHORIZATION) else {
            return Err((StatusCode::UNAUTHORIZED, "Missing Authorization".into()));
        };
        let s = header.to_str().map_err(|_| (StatusCode::UNAUTHORIZED, "Bad header".into()))?;
        let token = s.strip_prefix("Bearer ").ok_or((StatusCode::UNAUTHORIZED, "Bad scheme".into()))?;
        let claims = decode_token(token).map_err(|_| (StatusCode::UNAUTHORIZED, "Invalid token".into()))?;
        Ok(AuthUser { user_id: claims.sub, role: claims.role })
    }
}
