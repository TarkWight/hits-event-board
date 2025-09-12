use uuid::Uuid;
use time::OffsetDateTime;

use crate::error::ApiResult;
use crate::api::models::registration::RegistrationOut;
use crate::infra::repositories::registration_repo::RegistrationRepository;
use crate::domain::entities::registration_row::RegistrationRow;
// use crate::infra::errors::RepoError;

#[derive(Clone)]
pub struct RegistrationService<R: RegistrationRepository + Send + Sync + 'static> {
    repo: R
}

impl<R: RegistrationRepository + Send + Sync + 'static> RegistrationService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn list_for_event(&self, event_id: Uuid) -> ApiResult<Vec<RegistrationOut>> {
        let rows: Vec<RegistrationRow> = self.repo.list_for_event(event_id).await?;
        Ok(rows.into_iter().map(RegistrationOut::from).collect())
    }

    pub async fn register(&self, event_id: Uuid, student_id: Uuid) -> ApiResult<RegistrationOut> {
        let now = OffsetDateTime::now_utc();
        let row = self.repo.register(event_id, student_id, now).await?;
        Ok(row.into())
    }

    pub async fn cancel(&self, event_id: Uuid, student_id: Uuid) -> ApiResult<()> {
        self.repo.cancel(event_id, student_id, OffsetDateTime::now_utc()).await?;
        Ok(())
    }
}

// Row -> DTO
impl From<RegistrationRow> for RegistrationOut {
    fn from(r: RegistrationRow) -> Self {
        Self {
            student_id: r.student_id,
            registered_at: r.registered_at,
            student_name: r.student_name,
            student_email: r.student_email,
        }
    }
}