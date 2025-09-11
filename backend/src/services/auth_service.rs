use crate::infra::repositories::user_repo::UserRepository;
use crate::infra::security::{password, password_policy};
use crate::infra::security::jwt::TokenService;
use crate::error::{ApiError, ApiResult};
use crate::api::models::auth::{UserOut, RegisterOut};
use crate::utils::token::TokenDTO;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;
use crate::auth::roles::{ManagerStatus, UserRole, StudentStatus};
use crate::domain::entities::user_row::UserRow;

#[derive(Clone)]
pub struct AuthService<R: UserRepository + Send + Sync + 'static> {
    repo: R,
    tokens: TokenService,
}

impl<R: UserRepository + Send + Sync + 'static> AuthService<R> {
    pub fn new(repo: R, tokens: TokenService) -> Self {
        Self { repo, tokens }
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

        let access = self.tokens
            .generate_student_token(&user.email, user.id, "created")
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let refresh_plain = self.tokens.generate_refresh_token();
        let _refresh_hash  = self.tokens.hash_refresh_token(&refresh_plain);
        let refresh_exp    = OffsetDateTime::now_utc() + Duration::days(30);

        let access_exp = OffsetDateTime::now_utc()
            + Duration::minutes(self.tokens.lifetime_minutes());

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

    pub async fn refresh(&self, user_id: Uuid) -> ApiResult<TokenDTO> {
        let user = self.repo.find_by_id(user_id).await?;

        let (student_status_str, manager_status_str, company_id_opt): (Option<&str>, Option<&str>, Option<Uuid>) =
            match user.role {
                UserRole::Student => {
                    let st = self.repo.student_status(user.id).await?;
                    let s = match st {
                        Some(StudentStatus::Created)   => Some("created"),
                        Some(StudentStatus::Linked)    => Some("linked"),
                        Some(StudentStatus::Confirmed) => Some("confirmed"),
                        Some(StudentStatus::Rejected)  => Some("rejected"),
                        None => Some("created"),
                    };
                    (s, None, None)
                }
                UserRole::Manager => {
                    if let Some((st, cid)) = self.repo.manager_info(user.id).await? {
                        let s = match st {
                            ManagerStatus::Pending   => Some("pending"),
                            ManagerStatus::Confirmed => Some("confirmed"),
                            ManagerStatus::Rejected  => Some("rejected"),
                        };
                        (None, s, Some(cid))
                    } else {
                        (None, Some("pending"), None)
                    }
                }
                UserRole::Dean => (None, None, None),
            };

        let student_status_owned = student_status_str.map(str::to_string);
        let manager_status_owned = manager_status_str.map(str::to_string);

        let access = self.tokens
            .generate_token(
                &user.email,
                user.id,
                match user.role {
                    UserRole::Student => "student",
                    UserRole::Manager => "manager",
                    UserRole::Dean    => "dean",
                },
                student_status_owned,
                manager_status_owned,
                company_id_opt,
            )
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let access_exp = OffsetDateTime::now_utc() + Duration::minutes(self.tokens.lifetime_minutes());

        // Смена refresh-токена при рефреше access-а.
        let refresh_plain = self.tokens.generate_refresh_token();
        let refresh_exp   = OffsetDateTime::now_utc() + Duration::days(30);
        // let refresh_hash  = self.tokens.hash_refresh_token(&refresh_plain);
        // self.repo.set_refresh_token(user.id, &refresh_hash, refresh_exp).await?;

        Ok(TokenDTO {
            access_token: access,
            access_token_expiration: access_exp,
            refresh_token: refresh_plain,
            refresh_token_expiration: refresh_exp,
        })
    }
}