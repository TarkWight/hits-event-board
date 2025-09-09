use uuid::Uuid;
use crate::error::ApiResult;
use crate::infra::repositories::telegram_repo::TelegramLinkRepository;

#[derive(Clone)]
pub struct TelegramService<R: TelegramLinkRepository + Send + Sync + 'static> {
    repo: R,
}

impl<R: TelegramLinkRepository + Send + Sync + 'static> TelegramService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn link_student(&self, user_id: Uuid, telegram_user_id: i64) -> ApiResult<()> {
        self.repo.link(user_id, telegram_user_id).await?;
        Ok(())
    }

    pub async fn unlink_student(&self, user_id: Uuid) -> ApiResult<()> {
        self.repo.unlink_by_user(user_id).await?;
        Ok(())
    }

    pub async fn is_student_confirmed(&self, user_id: Uuid) -> ApiResult<bool> {
        Ok(self.repo.exists_for_user(user_id).await?)
    }

    pub async fn find_user_by_telegram(&self, telegram_user_id: i64) -> ApiResult<Uuid> {
        Ok(self.repo.get_user_by_telegram(telegram_user_id).await?)
    }
}