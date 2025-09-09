use crate::infra::repositories::user_repo::{UserRepository, UserRow, UserRole};
use crate::infra::security::{password, password_policy};
use crate::infra::security::jwt::TokenService;
use crate::error::{ApiError, ApiResult};
use crate::api::models::auth::{UserOut, RegisterOut};

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

        // создаём студента
        let user: UserRow = self.repo
            .create_student(&req.name, &req.email, &hash)
            .await?;

        // генерим access + refresh
        let access = self.tokens
            .generate_token(&user.email, user.id, "student", None, None)
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let refresh_plain = self.tokens.generate_refresh_token();
        // если хочешь сохранять refresh в БД — добавь метод в репо и сохрани hash + exp
        let refresh_hash = self.tokens.hash_refresh_token(&refresh_plain);
        let exp = time::OffsetDateTime::now_utc() + time::Duration::days(30);
        // TODO: repo.save_refresh(user.id, refresh_hash, exp).await?;

        let out = RegisterOut {
            user: UserOut {
                id: user.id,
                name: user.name,
                email: user.email,
                role: "student".into(),
            },
            tokens: crate::utils::token::TokenDTO {
                access_token: access,
                access_token_expiration: time::OffsetDateTime::now_utc()
                    + time::Duration::minutes(self.tokens.clone().cfg.lifetime_minutes),
                refresh_token: refresh_plain,
                refresh_token_expiration: exp,
            },
        };
        Ok(out)
    }
}