use base64::{engine::general_purpose::STANDARD as b64, Engine};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

#[derive(Clone)]
pub struct TokenConfig {
    pub issuer: String,
    pub audience: String,
    pub lifetime_minutes: i64,
    pub hmac_secret: String,
}

impl TokenConfig {
    pub fn from_env() -> Self {
        let issuer = std::env::var("JWT_ISSUER").unwrap_or_else(|_| "TSUHITs".into());
        let audience = std::env::var("JWT_AUDIENCE").unwrap_or_else(|_| "User".into());
        let lifetime_minutes = std::env::var("JWT_LIFETIME_MINUTES")
            .ok().and_then(|s| s.parse::<i64>().ok()).unwrap_or(60);
        let hmac_secret =
            std::env::var("JWT_HS256_SECRET").unwrap_or_else(|_| "dev_insecure_secret_change_me".into());
        Self { issuer, audience, lifetime_minutes, hmac_secret }
    }
    fn enc(&self) -> EncodingKey { EncodingKey::from_secret(self.hmac_secret.as_bytes()) }
    fn dec(&self) -> DecodingKey { DecodingKey::from_secret(self.hmac_secret.as_bytes()) }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub iss: String,
    pub aud: String,
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub jti: String,
    // custom
    pub user_id: Uuid,
    pub role: String,                 // "student" | "manager" | "dean"
    pub student_confirmed: Option<bool>,
    pub manager_status: Option<String>,
    pub company_id: Option<Uuid>,
}

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("jwt error: {0}")]
    Jwt(String),
    #[error("expired or invalid")]
    Invalid,
}

#[derive(Clone)]
pub struct TokenService {
    pub(crate) cfg: TokenConfig,
}

impl TokenService {
    pub fn lifetime_minutes(&self) -> i64 {
        self.cfg.lifetime_minutes
    }

    pub fn new(cfg: TokenConfig) -> Self { Self { cfg } }

    pub fn generate_token(
        &self,
        sub_email: &str,
        user_id: Uuid,
        role: &str,
        student_confirmed: Option<bool>,
        manager_status: Option<&str>,
        company_id: Option<Uuid>,
    ) -> Result<String, TokenError> {
        let now = OffsetDateTime::now_utc();
        let iat = now.unix_timestamp();
        let exp = (now + Duration::minutes(self.cfg.lifetime_minutes)).unix_timestamp();

        let claims = Claims {
            iss: self.cfg.issuer.clone(),
            aud: self.cfg.audience.clone(),
            sub: sub_email.to_string(),
            iat, exp,
            jti: Uuid::new_v4().to_string(),
            user_id,
            role: role.to_string(),
            student_confirmed,
            manager_status: manager_status.map(|s| s.to_string()),
            company_id,
        };
        let mut header = Header::new(Algorithm::HS256);
        header.kid = None;
        encode(&header, &claims, &self.cfg.enc()).map_err(|e| TokenError::Jwt(e.to_string()))
    }

    pub fn generate_student_token(
        &self,
        email: &str,
        user_id: Uuid,
        confirmed: bool,
    ) -> Result<String, TokenError> {
        self.generate_token(email, user_id, "student", Some(confirmed), None, None)
    }

    pub fn generate_manager_token(
        &self,
        email: &str,
        user_id: Uuid,
        status: Option<&str>,        // "pending" | "confirmed" | "rejected"
        company_id: Option<Uuid>,
    ) -> Result<String, TokenError> {
        self.generate_token(email, user_id, "manager", None, status, company_id)
    }

    pub fn generate_dean_token(
        &self,
        email: &str,
        user_id: Uuid,
    ) -> Result<String, TokenError> {
        self.generate_token(email, user_id, "dean", None, None, None)
    }

    pub fn reissue_with_student_confirmed(
        &self,
        old_token: &str,
        confirmed: bool,
    ) -> Result<String, TokenError> {
        let mut claims = self.validate_token(old_token)?;
        let now = OffsetDateTime::now_utc();
        claims.iat = now.unix_timestamp();
        claims.exp = (now + Duration::minutes(self.cfg.lifetime_minutes)).unix_timestamp();
        claims.student_confirmed = Some(confirmed);

        let mut header = Header::new(Algorithm::HS256);
        header.kid = None;
        encode(&header, &claims, &self.cfg.enc()).map_err(|e| TokenError::Jwt(e.to_string()))
    }

    pub fn validate_token(&self, token: &str) -> Result<Claims, TokenError> {
        let mut val = Validation::new(Algorithm::HS256);
        val.set_audience(&[self.cfg.audience.clone()]);
        val.set_issuer(&[self.cfg.issuer.clone()]);
        decode::<Claims>(token, &self.cfg.dec(), &val)
            .map(|d| d.claims)
            .map_err(|_| TokenError::Invalid)
    }

    pub fn generate_refresh_token(&self) -> String {
        let mut buf = [0u8; 64];
        rand::thread_rng().fill_bytes(&mut buf);
        b64.encode(buf)
    }

    pub fn hash_refresh_token(&self, refresh: &str) -> String {
        use sha2::{Digest, Sha256};
        let mut h = Sha256::new();
        h.update(refresh.as_bytes());
        b64.encode(h.finalize())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn roundtrip_manager() {
        std::env::set_var("JWT_HS256_SECRET", "test_secret");
        let svc = TokenService::new(TokenConfig::from_env());
        let token = svc.generate_manager_token("u@e.com", Uuid::new_v4(), Some("confirmed"), None).unwrap();
        let claims = svc.validate_token(&token).unwrap();
        assert_eq!(claims.role, "manager");
        assert_eq!(claims.manager_status.as_deref(), Some("confirmed"));
        assert_eq!(claims.student_confirmed, None);
    }

    #[test]
    fn roundtrip_student_confirm_toggle() {
        std::env::set_var("JWT_HS256_SECRET", "test_secret");
        let svc = TokenService::new(TokenConfig::from_env());
        let t1 = svc.generate_student_token("s@e.com", Uuid::new_v4(), false).unwrap();
        let c1 = svc.validate_token(&t1).unwrap();
        assert_eq!(c1.role, "student");
        assert_eq!(c1.student_confirmed, Some(false));

        let t2 = svc.reissue_with_student_confirmed(&t1, true).unwrap();
        let c2 = svc.validate_token(&t2).unwrap();
        assert_eq!(c2.student_confirmed, Some(true));
    }
}