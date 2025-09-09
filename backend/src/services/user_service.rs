use uuid::Uuid;
use crate::infra::repositories::user_repo::UserRepository;
use crate::error::ApiResult;

#[derive(Clone)]
pub struct UserService<R: UserRepository + Send + Sync + 'static> { repo: R }

impl<R: UserRepository + Send + Sync + 'static> UserService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }
    pub async fn approve(&self, user_id: Uuid, approver_id: Uuid) -> ApiResult<()> {
        let x = self.repo.approve_user(user_id, approver_id).await?;
        Ok(())
    }
}
