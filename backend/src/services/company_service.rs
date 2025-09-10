use uuid::Uuid;

use crate::api::models::company::CompanyOut;
use crate::api::requests::company::{CreateCompanyIn, UpdateCompanyIn};
use crate::domain::entities::company::CompanyWithCounts;
use crate::domain::entities::company_row::CompanyRow;
use crate::infra::repositories::company::CompanyRepository;
use crate::error::ApiResult;

#[derive(Clone)]
pub struct CompanyService<R: CompanyRepository + Send + Sync + 'static> {
    repo: R,
}

impl<R: CompanyRepository + Send + Sync + 'static> CompanyService<R> {
    pub fn new(repo: R) -> Self { Self { repo } }

    pub async fn list(&self, page: i32, limit: i32, q: Option<String>) -> ApiResult<Vec<CompanyOut>> {
        let rows: Vec<CompanyWithCounts> = self.repo.list(page, limit, q).await?;
        Ok(rows.into_iter().map(CompanyOut::from).collect())
    }

    pub async fn create(&self, payload: CreateCompanyIn, _creator: Uuid) -> ApiResult<CompanyOut> {
        let row: CompanyRow = payload.try_into()?;

        let created = self.repo.create(row).await?;

        let full = self.repo.get(created.id).await?;

        Ok(full.into())
    }

    pub async fn get(&self, id: Uuid) -> ApiResult<CompanyOut> {
        let with_counts = self.repo.get(id).await?;
        Ok(CompanyOut::from(with_counts))
    }

    pub async fn update(&self, id: Uuid, payload: UpdateCompanyIn) -> ApiResult<CompanyOut> {
        if let Some(name) = payload.name {
            let updated = self.repo.update_name(id, &name).await?;
            return Ok(CompanyOut::from(updated));
        }
        let current = self.repo.get(id).await?;
        Ok(CompanyOut::from(current))
    }

    pub async fn list_managers(&self, _company_id: Uuid) -> ApiResult<Vec<Uuid>> {
        Ok(vec![])
    }
    pub async fn invite_manager(&self, _company_id: Uuid, _email: String) -> ApiResult<()> { Ok(()) }
    pub async fn approve_manager(&self, _company_id: Uuid, _user_id: Uuid) -> ApiResult<()> { Ok(()) }
}