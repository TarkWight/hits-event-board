use uuid::Uuid;
use crate::infra::repositories::company_repo::{CompanyRepository, CompanyRow};
use crate::api::requests::{CreateCompanyIn, UpdateCompanyIn};
use crate::api::models::CompanyOut;
use crate::error::ApiResult;

#[derive(Clone)]
pub struct CompanyService<R: CompanyRepository + Send + Sync + 'static> { repo: R }

impl<R: CompanyRepository + Send + Sync + 'static> CompanyService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }
    pub async fn list(&self, page: i32, limit: i32, q: Option<String>) -> ApiResult<Vec<CompanyOut>> { Ok(self.repo.list(page, limit, q).await?) }
    pub async fn create(&self, payload: CreateCompanyIn, creator: Uuid) -> ApiResult<CompanyOut> {
        let row: CompanyRow = payload.try_into().map_err(|e: String| crate::error::ApiError::Unprocessable(e))?;
        Ok(self.repo.create(row, creator).await?)
    }
    pub async fn get(&self, id: Uuid) -> ApiResult<CompanyOut> { Ok(self.repo.get(id).await?) }
    pub async fn update(&self, id: Uuid, payload: UpdateCompanyIn) -> ApiResult<CompanyOut> { Ok(self.repo.update(id, payload).await?) }
}
