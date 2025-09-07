use async_trait::async_trait;
use uuid::Uuid;
use sqlx::{Pool, Postgres};
use crate::api::models::User;
use super::types::*;

#[async_trait]
pub trait UserRepository {
    async fn find_by_email(&self, email: &str) -> RepoResult<Option<User>>;
    async fn approve_user(&self, user_id: Uuid, approver_id: Uuid) -> RepoResult<()>;
}

#[derive(Clone)]
pub struct PgUserRepository { pool: Pool<Postgres> }
impl PgUserRepository { pub fn new(pool: std::sync::Arc<Pool<Postgres>>) -> Self { Self { pool: (*pool).clone() } } }

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn find_by_email(&self, _email: &str) -> RepoResult<Option<User>> { Ok(None) }
    async fn approve_user(&self, _user_id: Uuid, _approver_id: Uuid) -> RepoResult<()> { Ok(()) }
}
