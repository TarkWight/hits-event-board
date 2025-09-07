use async_trait::async_trait;
use uuid::Uuid;
use sqlx::{Pool, Postgres};
use crate::api::models::Registration;
use super::types::*;

#[async_trait]
pub trait RegistrationRepository {
    async fn list_for_event(&self, event_id: Uuid) -> RepoResult<Vec<Registration>>;
    async fn register(&self, event_id: Uuid, student_id: Uuid, idempotency_key: Option<String>) -> RepoResult<Registration>;
    async fn cancel(&self, event_id: Uuid, student_id: Uuid) -> RepoResult<()>;
}

#[derive(Clone)]
pub struct PgRegistrationRepository { pool: Pool<Postgres> }
impl PgRegistrationRepository { pub fn new(pool: std::sync::Arc<Pool<Postgres>>) -> Self { Self { pool: (*pool).clone() } } }

#[async_trait]
impl RegistrationRepository for PgRegistrationRepository {
    async fn list_for_event(&self, _event_id: Uuid) -> RepoResult<Vec<Registration>> { Ok(vec![]) }
    async fn register(&self, _event_id: Uuid, _student_id: Uuid, _idempotency_key: Option<String>) -> RepoResult<Registration> { Err(RepoError::Db("not implemented".into())) }
    async fn cancel(&self, _event_id: Uuid, _student_id: Uuid) -> RepoResult<()> { Ok(()) }
}
