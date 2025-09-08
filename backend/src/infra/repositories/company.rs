use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use crate::domain::entities::company_row::CompanyRow;
use crate::domain::types::{RepoResult, RepoError};

#[async_trait]
pub trait CompanyRepository {
    async fn list(&self, page: i32, limit: i32, q: Option<String>) -> RepoResult<Vec<CompanyRow>>;
    async fn create(&self, row: CompanyRow) -> RepoResult<CompanyRow>;
    async fn get(&self, id: Uuid) -> RepoResult<CompanyRow>;
    async fn update_name(&self, id: Uuid, name: &str) -> RepoResult<CompanyRow>;
}

#[derive(Clone)]
pub struct PgCompanyRepository { pool: Pool<Postgres> }

impl PgCompanyRepository { pub fn new(pool: Pool<Postgres>) -> Self { Self { pool } } }

#[async_trait]
impl CompanyRepository for PgCompanyRepository {
    async fn list(&self, page: i32, limit: i32, q: Option<String>) -> RepoResult<Vec<CompanyRow>> { todo!() }
    async fn create(&self, row: CompanyRow) -> RepoResult<CompanyRow> { todo!() }
    async fn get(&self, id: Uuid) -> RepoResult<CompanyRow> { todo!() }
    async fn update_name(&self, id: Uuid, name: &str) -> RepoResult<CompanyRow> { todo!() }
}