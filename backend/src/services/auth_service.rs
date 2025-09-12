use crate::infra::repositories::user_repo::UserRepository;
use crate::infra::repositories::telegram_repo::TelegramLinkRepository;
use crate::infra::security::{password, password_policy};
use crate::infra::security::jwt::TokenService;
use crate::error::{ApiError, ApiResult};
use crate::api::models::auth::{UserOut, RegisterOut, LoginOut};
use crate::utils::token::TokenDTO;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::auth::roles::{ManagerStatus, StudentStatus, UserRole};
use crate::domain::entities::user_row::UserRow;

#[derive(Clone)]
pub struct AuthService<R, L>
where
    R: UserRepository + Send + Sync + 'static,
    L: TelegramLinkRepository + Send + Sync + 'static,
{
    repo: R,
    tokens: TokenService,
    tg_links: L,
}

impl<R, L> AuthService<R, L>
where
    R: UserRepository + Send + Sync + 'static,
    L: TelegramLinkRepository + Send + Sync + 'static,
{
    pub fn new(repo: R, tokens: TokenService, tg_links: L) -> Self {
        Self { repo, tokens, tg_links }
    }

    pub async fn register_manager(
        &self,
        req: crate::api::requests::manager_register::ManagerRegisterRequest,
    ) -> ApiResult<RegisterOut> {
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
            .create_manager(&req.name, &req.email, &hash, req.company_id)
            .await?;

        self.maybe_link_telegram(&user, req.telegram_user_id).await?;

        let access = self.tokens
            .generate_manager_token(&user.email, user.id, Some("pending"), req.company_id.into())
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let refresh_plain = self.tokens.generate_refresh_token();
        let refresh_hash  = self.tokens.hash_refresh_token(&refresh_plain);
        let refresh_exp   = OffsetDateTime::now_utc() + Duration::days(30);
        self.repo.set_refresh_token(user.id, &refresh_hash, refresh_exp).await?;

        let access_exp = OffsetDateTime::now_utc()
            + Duration::minutes(self.tokens.lifetime_minutes());

        Ok(RegisterOut {
            user: Self::user_row_to_out(&user),
            tokens: TokenDTO {
                access_token: access,
                access_token_expiration: access_exp,
                refresh_token: refresh_plain,
                refresh_token_expiration: refresh_exp,
            },
        })
    }

    pub async fn register_student(
        &self,
        req: crate::api::requests::student_register::StudentRegisterRequest,
    ) -> ApiResult<RegisterOut> {
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

        self.maybe_link_telegram(&user, req.telegram_user_id).await?;

        let access = self.tokens
            .generate_student_token(&user.email, user.id, "created")
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let refresh_plain = self.tokens.generate_refresh_token();
        let refresh_hash  = self.tokens.hash_refresh_token(&refresh_plain);
        let refresh_exp   = OffsetDateTime::now_utc() + Duration::days(30);
        self.repo.set_refresh_token(user.id, &refresh_hash, refresh_exp).await?;

        let access_exp = OffsetDateTime::now_utc()
            + Duration::minutes(self.tokens.lifetime_minutes());

        Ok(RegisterOut {
            user: Self::user_row_to_out(&user),
            tokens: TokenDTO {
                access_token: access,
                access_token_expiration: access_exp,
                refresh_token: refresh_plain,
                refresh_token_expiration: refresh_exp,
            },
        })
    }

    pub async fn login(
        &self,
        req: crate::api::requests::login::LoginRequest,
    ) -> ApiResult<LoginOut> {
        let user = self.repo.find_by_email(&req.email).await?;

        let ok = password::verify_password(&req.password, &user.password_hash);
        if !ok {
            return Err(ApiError::Unauthorized);
        }

        // self.maybe_link_telegram(&user, req.telegram_user_id).await?;

        let (access, access_exp, refresh_plain, refresh_exp) =
            self.issue_full_token_set(&user).await?;

        Ok(LoginOut {
            user: Self::user_row_to_out(&user),
            tokens: TokenDTO {
                access_token: access,
                access_token_expiration: access_exp,
                refresh_token: refresh_plain,
                refresh_token_expiration: refresh_exp,
            },
        })
    }

    pub async fn logout(&self, user_id: Uuid) -> ApiResult<()> {
        let refresh_hash = "-";
        let refresh_exp  = OffsetDateTime::now_utc() - Duration::seconds(1);
        self.repo.set_refresh_token(user_id, refresh_hash, refresh_exp).await?;
        Ok(())
    }

    pub async fn refresh_by_token(
        &self,
        body: crate::api::requests::refresh::RefreshRequest,
    ) -> ApiResult<TokenDTO> {
        let now = OffsetDateTime::now_utc();
        let provided_plain = body.refresh_token.trim();
        if provided_plain.is_empty() {
            return Err(ApiError::Unauthorized);
        }

        let provided_hash = self.tokens.hash_refresh_token(provided_plain);

        let user = self
            .repo
            .find_by_refresh_token(&provided_hash, now)
            .await
            .map_err(|_| ApiError::Unauthorized)?;

        let (access, access_exp, refresh_plain, refresh_exp) =
            self.issue_full_token_set(&user).await?;

        Ok(TokenDTO {
            access_token: access,
            access_token_expiration: access_exp,
            refresh_token: refresh_plain,
            refresh_token_expiration: refresh_exp,
        })
    }

    pub async fn refresh(&self, user_id: Uuid) -> ApiResult<TokenDTO> {
        let user = self.repo.find_by_id(user_id).await?;
        let (access, access_exp, refresh_plain, refresh_exp) =
            self.issue_full_token_set(&user).await?;
        Ok(TokenDTO {
            access_token: access,
            access_token_expiration: access_exp,
            refresh_token: refresh_plain,
            refresh_token_expiration: refresh_exp,
        })
    }

    async fn maybe_link_telegram(
        &self,
        user: &UserRow,
        telegram_user_id: Option<i64>,
    ) -> ApiResult<()> {
        let Some(tg) = telegram_user_id else { return Ok(()); };

        self.tg_links.link(user.id, tg).await?;

        if matches!(user.role, UserRole::Student) {
            if let Some(st) = self.repo.student_status(user.id).await? {
                if st == StudentStatus::Created {
                    self.repo.set_student_status(user.id, StudentStatus::Linked).await?;
                }
            } else {
                self.repo.set_student_status(user.id, StudentStatus::Linked).await?;
            }
        }

        Ok(())
    }

    async fn issue_full_token_set(
        &self,
        user: &UserRow,
    ) -> ApiResult<(String, OffsetDateTime, String, OffsetDateTime)> {
        let (student_status, manager_status, company_id) =
            self.resolve_statuses_and_company(user).await?;

        let access = self.tokens
            .generate_token(
                &user.email,
                user.id,
                Self::role_str(user.role),
                student_status.clone(),
                manager_status.clone(),
                company_id,
            )
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let access_exp = OffsetDateTime::now_utc()
            + Duration::minutes(self.tokens.lifetime_minutes());

        let refresh_plain = self.tokens.generate_refresh_token();
        let refresh_hash  = self.tokens.hash_refresh_token(&refresh_plain);
        let refresh_exp   = OffsetDateTime::now_utc() + Duration::days(30);
        self.repo.set_refresh_token(user.id, &refresh_hash, refresh_exp).await?;

        Ok((access, access_exp, refresh_plain, refresh_exp))
    }

    async fn resolve_statuses_and_company(
        &self,
        user: &UserRow,
    ) -> ApiResult<(Option<String>, Option<String>, Option<Uuid>)> {
        match user.role {
            UserRole::Student => {
                let st = self.repo.student_status(user.id).await?;
                let s = match st {
                    Some(StudentStatus::Created)   => Some("created".to_string()),
                    Some(StudentStatus::Linked)    => Some("linked".to_string()),
                    Some(StudentStatus::Confirmed) => Some("confirmed".to_string()),
                    Some(StudentStatus::Rejected)  => Some("rejected".to_string()),
                    None                           => Some("created".to_string()),
                };
                Ok((s, None, None))
            }
            UserRole::Manager => {
                if let Some((st, cid)) = self.repo.manager_info(user.id).await? {
                    let s = match st {
                        ManagerStatus::Pending   => Some("pending".to_string()),
                        ManagerStatus::Confirmed => Some("confirmed".to_string()),
                        ManagerStatus::Rejected  => Some("rejected".to_string()),
                    };
                    Ok((None, s, Some(cid)))
                } else {
                    Ok((None, Some("pending".to_string()), None))
                }
            }
            UserRole::Dean => Ok((None, None, None)),
        }
    }

    fn user_row_to_out(user: &UserRow) -> UserOut {
        UserOut {
            id: user.id,
            name: user.name.clone(),
            email: user.email.clone(),
            role: match user.role {
                UserRole::Student => "student".into(),
                UserRole::Manager => "manager".into(),
                UserRole::Dean    => "dean".into(),
            },
        }
    }

    fn role_str(role: UserRole) -> &'static str {
        match role {
            UserRole::Student => "student",
            UserRole::Manager => "manager",
            UserRole::Dean    => "dean",
        }
    }
}