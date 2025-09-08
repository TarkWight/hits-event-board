use crate::infra::repositories::company::CompanyRepository;
use crate::domain::entities::company_row::CompanyRow;
use crate::api::requests::company::{CreateCompanyIn, UpdateCompanyIn};
use crate::api::models::company::CompanyOut;
use crate::error::ApiResult;
use uuid::Uuid;

#[derive(Clone)]
pub struct CompanyService<R: CompanyRepository + Send + Sync + 'static> {
    repo: R,
}

impl<R: CompanyRepository + Send + Sync + 'static> CompanyService<R> {
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    // ---- List ----
    pub async fn list(&self, page: i32, limit: i32, q: Option<String>) -> ApiResult<Vec<CompanyOut>> {
        let rows = self.repo.list(page, limit, q).await?;
        Ok(rows.into_iter().map(CompanyOut::from).collect())
    }

    // ---- Create ----
    pub async fn create(&self, payload: CreateCompanyIn, _creator: Uuid) -> ApiResult<CompanyOut> {
        // Type mismatch resolving `<CompanyRow as TryFrom<CreateCompanyIn>>::Error == String`
        let row: CompanyRow = payload.try_into() // The trait bound `CompanyRow: TryFrom<CreateCompanyIn>` is not satisfied
            .map_err(|e: String| crate::error::ApiError::Unprocessable(e))?;
        let created = self.repo.create(row).await?;
        Ok(created.into())
    }

    // ---- Get by ID ----
    pub async fn get(&self, id: Uuid) -> ApiResult<CompanyOut> {
        let row = self.repo.get(id).await?;
        Ok(row.into())
    }

    // ---- Update ----
    pub async fn update(&self, id: Uuid, payload: UpdateCompanyIn) -> ApiResult<CompanyOut> {
        // Получаем текущую запись
        let row = self.repo.get(id).await?;
        // Определяем новое имя
        let name = payload.name.unwrap_or(row.name.clone());
        // Обновляем в репозитории
        let updated = self.repo.update_name(id, &name).await?;
        Ok(updated.into())
    }

    // ---- Заглушки для менеджеров (будет реализовано позже) ----
    pub async fn list_managers(&self, _company_id: Uuid) -> ApiResult<Vec<uuid::Uuid>> {
        // здесь можно будет возвращать список manager_id
        Ok(vec![])
    }

    pub async fn invite_manager(&self, _company_id: Uuid, _email: String) -> ApiResult<()> {
        // логика приглашения менеджера
        Ok(())
    }

    pub async fn approve_manager(&self, _company_id: Uuid, _user_id: Uuid) -> ApiResult<()> {
        // логика подтверждения менеджера
        Ok(())
    }
}