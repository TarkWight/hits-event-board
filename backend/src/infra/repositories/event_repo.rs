use async_trait::async_trait;
use uuid::Uuid;
use time::OffsetDateTime;
use sqlx::{Pool, Postgres};
use crate::api::models::Event;
use super::types::*;

#[derive(Clone)]
pub struct EventCreate {
    pub company_id: Uuid,
    pub manager_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: OffsetDateTime,
    pub end_time: OffsetDateTime,
    pub registration_deadline: OffsetDateTime,
    pub capacity: Option<i32>,
    pub is_published: bool,
}

#[derive(Clone, Default)]
pub struct EventUpdate {
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub start_time: Option<OffsetDateTime>,
    pub end_time: Option<OffsetDateTime>,
    pub registration_deadline: Option<OffsetDateTime>,
    pub capacity: Option<i32>,
    pub is_published: Option<bool>,
}

#[async_trait]
pub trait EventRepository {
    async fn list(&self, page: i32, limit: i32, from: Option<OffsetDateTime>, to: Option<OffsetDateTime>, company: Option<Uuid>, published: Option<bool>, q: Option<String>) -> RepoResult<Vec<Event>>;
    async fn create(&self, payload: EventCreate) -> RepoResult<Event>;
    async fn get(&self, id: Uuid) -> RepoResult<Event>;
    async fn update(&self, id: Uuid, payload: EventUpdate) -> RepoResult<Event>;
    async fn delete(&self, id: Uuid) -> RepoResult<()>;
    async fn publish(&self, id: Uuid, on: bool) -> RepoResult<()>;
}

#[derive(Clone)]
pub struct PgEventRepository { pool: Pool<Postgres> }
impl PgEventRepository { pub fn new(pool: std::sync::Arc<Pool<Postgres>>) -> Self { Self { pool: (*pool).clone() } } }

#[async_trait]
impl EventRepository for PgEventRepository {
    async fn list(&self, _page: i32, _limit: i32, _from: Option<OffsetDateTime>, _to: Option<OffsetDateTime>, _company: Option<Uuid>, _published: Option<bool>, _q: Option<String>) -> RepoResult<Vec<Event>> { Ok(vec![]) }
    async fn create(&self, _payload: EventCreate) -> RepoResult<Event> { Err(RepoError::Db("not implemented".into())) }
    async fn get(&self, _id: Uuid) -> RepoResult<Event> { Err(RepoError::NotFound) }
    async fn update(&self, _id: Uuid, _payload: EventUpdate) -> RepoResult<Event> { Err(RepoError::Db("not implemented".into())) }
    async fn delete(&self, _id: Uuid) -> RepoResult<()> { Ok(()) }
    async fn publish(&self, _id: Uuid, _on: bool) -> RepoResult<()> { Ok(()) }
}
