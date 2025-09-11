use crate::infra::repositories::user_repo::{UserRepository, UserRow};
use crate::infra::security::{password, password_policy};
use crate::infra::security::jwt::TokenService;
use crate::error::{ApiError, ApiResult};
use crate::api::models::auth::{UserOut, RegisterOut};
use crate::utils::token::TokenDTO;
use time::{Duration, OffsetDateTime};

#[derive(Clone)]
pub struct AuthService<R: UserRepository + Send + Sync + 'static> {
    repo: R,
    tokens: TokenService,
}

impl<R: UserRepository + Send + Sync + 'static> AuthService<R> {
    pub fn new(repo: R, tokens: TokenService) -> Self {
        Self { repo, tokens }
    }

    pub async fn register_student(&self, req: crate::api::requests::student_register::StudentRegisterRequest)
                                  -> ApiResult<RegisterOut>
    {
        if req.name.trim().is_empty() {
            return Err(ApiError::Unprocessable("name must not be empty".into()));
        }
        if !req.email.contains('@') {
            return Err(ApiError::Unprocessable("email is invalid".into()));
        }
        password_policy::validate(&req.password)
            .map_err(|m| ApiError::Unprocessable(m.into()))?;

        let hash = password::hash_password(&req.password)
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let user: UserRow = self.repo
            .create_student(&req.name, &req.email, &hash)
            .await?;

        let access = self.tokens
            .generate_token(&user.email, user.id, "student", Some(false), None, None)
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let refresh_plain = self.tokens.generate_refresh_token();
        let refresh_hash  = self.tokens.hash_refresh_token(&refresh_plain);
        let refresh_exp   = OffsetDateTime::now_utc() + Duration::days(30);
        // TODO: repo.save_refresh(user.id, &refresh_hash, refresh_exp).await?;

        let access_exp = OffsetDateTime::now_utc() + Duration::minutes(self.tokens.lifetime_minutes());

        let out = RegisterOut {
            user: UserOut {
                id: user.id,
                name: user.name,
                email: user.email,
                role: "student".into(),
            },
            tokens: TokenDTO {
                access_token: access,
                access_token_expiration: access_exp,
                refresh_token: refresh_plain,
                refresh_token_expiration: refresh_exp,
            },
        };

        Ok(out)
    }
}