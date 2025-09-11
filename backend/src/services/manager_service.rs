use uuid::Uuid;
use crate::error::ApiResult;
use crate::api::models::manager::ManagerOut;
use crate::domain::entities::manager::ManagerStatus;
use crate::domain::entities::manager_row::ManagerRow;
use crate::infra::repositories::manager_repo::ManagerRepository;
use crate::domain::mappers::manager::to_manager_out_list;

#[derive(Clone)]
pub struct ManagerService<R: ManagerRepository + Send + Sync + 'static> { repo: R }

impl<R: ManagerRepository + Send + Sync + 'static> ManagerService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn list_for_company(&self, company_id: Uuid) -> ApiResult<Vec<ManagerOut>> {
        let rows: Vec<ManagerRow> = self.repo.list_for_company(company_id).await?;
        Ok(to_manager_out_list(rows))
    }

    pub async fn request_join(&self, company_id: Uuid, user_id: Uuid) -> ApiResult<()> {
        self.repo.request_join(company_id, user_id).await?;
        Ok(())
    }

    pub async fn set_status(&self, company_id: Uuid, user_id: Uuid, status: ManagerStatus) -> ApiResult<()> {
        self.repo.set_status(company_id, user_id, status).await?;
        Ok(())
    }
}