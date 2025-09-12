use uuid::Uuid;
use time::OffsetDateTime;

use crate::error::ApiResult;
use crate::infra::errors::RepoError;
use crate::infra::repositories::telegram_repo::TelegramLinkRepository;
use crate::infra::repositories::telegram_code_repo::TelegramCodeRepository;

#[derive(Clone)]
pub struct TelegramService<L, C>
where
    L: TelegramLinkRepository + Send + Sync + 'static,
    C: TelegramCodeRepository + Send + Sync + 'static,
{
    links_repo: L,
    codes_repo: C,
    code_ttl_minutes: i64,
}

impl<L, C> TelegramService<L, C>
where
    L: TelegramLinkRepository + Send + Sync + 'static,
    C: TelegramCodeRepository + Send + Sync + 'static,
{
    pub fn new(links_repo: L, codes_repo: C, code_ttl_minutes: i64) -> Self {
        Self { links_repo, codes_repo, code_ttl_minutes }
    }

    pub async fn create_link_code_for_user(&self, user_id: Uuid) -> ApiResult<String> {
        if !self.links_repo.is_student(user_id).await? {
            return Err(crate::error::ApiError::Forbidden);
        }
        let exists = self.links_repo.exists_for_user(user_id).await?;
        if exists {
            return Err(crate::error::ApiError::Conflict("already linked".into()));
        }

        let code = self
            .codes_repo
            .create_code(user_id, self.code_ttl_minutes)
            .await?;
        Ok(code)
    }

    pub async fn consume_link_code(&self, code: &str, telegram_user_id: i64) -> ApiResult<Uuid> {
        let user_id = self.codes_repo.consume_code(code).await?;
        // доп.проверка, что это студент
        if !self.links_repo.is_student(user_id).await? {
            return Err(crate::error::ApiError::Forbidden);
        }
        self.links_repo.link(user_id, telegram_user_id).await?;
        Ok(user_id)
    }

    pub async fn unlink(&self, user_id: Uuid) -> ApiResult<()> {
        self.links_repo.unlink_by_user(user_id).await?;
        Ok(())
    }
}