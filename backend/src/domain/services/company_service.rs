use uuid::Uuid;
use crate::infra::repositories::company_repo::{CompanyRepository, CompanyCreate, CompanyUpdate};
use crate::api::models::Company;
use crate::error::ApiResult;

#[derive(Clone)]
pub struct CompanyService<R: CompanyRepository + Send + Sync + 'static> { repo: R }

impl<R: CompanyRepository + Send + Sync + 'static> CompanyService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn list(&self, page: i32, limit: i32, q: Option<String>) -> ApiResult<Vec<Company>> {
        self.repo.list(page, limit, q).await
    }
    pub async fn create(&self, payload: CompanyCreate, creator: Uuid) -> ApiResult<Company> {
        self.repo.create(payload, creator).await
    }
    pub async fn get(&self, id: Uuid) -> ApiResult<Company> { self.repo.get(id).await }
    pub async fn update(&self, id: Uuid, payload: CompanyUpdate) -> ApiResult<Company> { self.repo.update(id, payload).await }
}
