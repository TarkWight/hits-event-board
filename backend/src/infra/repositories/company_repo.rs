use async_trait::async_trait;
use uuid::Uuid;
use sqlx::{Pool, Postgres};
use crate::api::models::Company;
use super::types::*;

#[derive(Clone)]
pub struct CompanyCreate { pub name: String, pub description: Option<String>, pub website: Option<String> }
#[derive(Clone)]
pub struct CompanyUpdate { pub name: Option<String>, pub description: Option<String>, pub website: Option<String> }

#[async_trait]
pub trait CompanyRepository {
    async fn list(&self, page: i32, limit: i32, q: Option<String>) -> RepoResult<Vec<Company>>;
    async fn create(&self, payload: CompanyCreate, creator: Uuid) -> RepoResult<Company>;
    async fn get(&self, id: Uuid) -> RepoResult<Company>;
    async fn update(&self, id: Uuid, payload: CompanyUpdate) -> RepoResult<Company>;
}

#[derive(Clone)]
pub struct PgCompanyRepository { pool: Pool<Postgres> }
impl PgCompanyRepository { pub fn new(pool: std::sync::Arc<Pool<Postgres>>) -> Self { Self { pool: (*pool).clone() } } }

#[async_trait]
impl CompanyRepository for PgCompanyRepository {
    async fn list(&self, _page: i32, _limit: i32, _q: Option<String>) -> RepoResult<Vec<Company>> { Ok(vec![]) }
    async fn create(&self, _payload: CompanyCreate, _creator: Uuid) -> RepoResult<Company> { Err(RepoError::Db("not implemented".into())) }
    async fn get(&self, _id: Uuid) -> RepoResult<Company> { Err(RepoError::NotFound) }
    async fn update(&self, _id: Uuid, _payload: CompanyUpdate) -> RepoResult<Company> { Err(RepoError::Db("not implemented".into())) }
}
