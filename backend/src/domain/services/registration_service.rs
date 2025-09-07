use uuid::Uuid;
use crate::infra::repositories::registration_repo::RegistrationRepository;
use crate::error::ApiResult;
use crate::api::models::Registration;

#[derive(Clone)]
pub struct RegistrationService<R: RegistrationRepository + Send + Sync + 'static> { repo: R }

impl<R: RegistrationRepository + Send + Sync + 'static> RegistrationService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }
    pub async fn list_for_event(&self, event_id: Uuid) -> ApiResult<Vec<Registration>> { self.repo.list_for_event(event_id).await }
    pub async fn register(&self, event_id: Uuid, student_id: Uuid, idempotency_key: Option<String>) -> ApiResult<Registration> {
        self.repo.register(event_id, student_id, idempotency_key).await
    }
    pub async fn cancel(&self, event_id: Uuid, student_id: Uuid) -> ApiResult<()> { self.repo.cancel(event_id, student_id).await }
}
