use async_trait::async_trait;
use sqlx::{FromRow, Pool, Postgres};
use time::OffsetDateTime;
use uuid::Uuid;

use super::types::*;

#[derive(Debug, Clone, FromRow)]
pub struct EventRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub manager_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub starts_at: OffsetDateTime,
    pub ends_at: Option<OffsetDateTime>,
    pub signup_deadline: Option<OffsetDateTime>,
    pub capacity: Option<i32>,
    pub is_published: bool,
}

#[derive(Debug, FromRow)]
struct EventListRow {
    pub id: Uuid,
    pub company_id: Uuid,
    pub manager_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub starts_at: OffsetDateTime,
    pub ends_at: Option<OffsetDateTime>,
    pub signup_deadline: Option<OffsetDateTime>,
    pub capacity: Option<i32>,
    pub is_published: bool,
    pub registered_count: Option<i64>,
}

#[derive(Debug, Clone)]
pub struct EventOut {
    pub id: Uuid,
    pub company_id: Uuid,
    pub manager_id: Uuid,
    pub title: String,
    pub short_desc: Option<String>,
    pub starts_at: OffsetDateTime,
    pub ends_at: Option<OffsetDateTime>,
    pub signup_deadline: Option<OffsetDateTime>,
    pub location: Option<String>,
    pub capacity: Option<i32>,
    pub is_published: bool,
    pub registered_count: Option<i64>,
}
impl From<EventListRow> for EventOut {
    fn from(r: EventListRow) -> Self {
        Self {
            id: r.id,
            company_id: r.company_id,
            manager_id: r.manager_id,
            title: r.title,
            short_desc: r.description,
            starts_at: r.starts_at,
            ends_at: r.ends_at,
            signup_deadline: r.signup_deadline,
            location: r.location,
            capacity: r.capacity,
            is_published: r.is_published,
            registered_count: r.registered_count,
        }
    }
}

#[derive(Debug, Default)]
pub struct EventListFilter {
    pub company: Option<Uuid>,
    pub published: Option<bool>,
    pub q: Option<String>, // по title
}

#[async_trait]
pub trait EventRepository {
    async fn list(&self, page: i32, limit: i32, f: EventListFilter) -> RepoResult<Vec<EventOut>>;
    async fn create(&self, row: EventRow) -> RepoResult<EventOut>;
    async fn get(&self, id: Uuid) -> RepoResult<EventOut>;
    async fn update_partial(&self, id: Uuid, upd: EventRowPatch) -> RepoResult<EventOut>;
    async fn publish(&self, id: Uuid, on: bool) -> RepoResult<()>;
    async fn delete(&self, id: Uuid) -> RepoResult<()>;
}

#[derive(Debug, Default)]
pub struct EventRowPatch {
    pub title: Option<String>,
    pub description: Option<String>,
    pub location: Option<String>,
    pub starts_at: Option<OffsetDateTime>,
    pub ends_at: Option<OffsetDateTime>,
    pub signup_deadline: Option<OffsetDateTime>,
    pub capacity: Option<i32>,
    pub is_published: Option<bool>,
}

#[derive(Clone)]
pub struct PgEventRepository { pool: Pool<Postgres> }
impl PgEventRepository { pub fn new(pool: Pool<Postgres>) -> Self { Self { pool } } }

#[async_trait]
impl EventRepository for PgEventRepository {
    async fn list(&self, page: i32, limit: i32, f: EventListFilter) -> RepoResult<Vec<EventOut>> {
        let page_i64 = page.max(1) as i64;
        let limit_i64 = limit.max(1) as i64;
        let offset_i64 = (page_i64 - 1) * limit_i64;

        let q_like = f.q.as_ref().map(|s| format!("%{}%", s));

        let rows = sqlx::query_as!(
            EventListRow,
            r#"
            SELECT e.id, e.company_id, e.manager_id, e.title, e.description, e.location,
                   e.starts_at, e.ends_at, e.signup_deadline, e.capacity, e.is_published,
                   (SELECT COUNT(*)::bigint FROM registrations r
                     WHERE r.event_id = e.id AND r.status = 'registered') AS "registered_count?"
            FROM events e
            WHERE ($1::uuid IS NULL OR e.company_id = $1)
              AND ($2::bool IS NULL OR e.is_published = $2)
              AND ($3::text IS NULL OR e.title ILIKE $3)
            ORDER BY e.starts_at DESC
            LIMIT $4 OFFSET $5
            "#,
            f.company,
            f.published,
            q_like,
            limit_i64,
            offset_i64
        )
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(EventOut::from).collect())
    }

    async fn create(&self, row: EventRow) -> RepoResult<EventOut> {
        // БД-триггер уже проверит confirmed-менеджера; тут просто insert
        let r = sqlx::query_as!(
            EventListRow,
            r#"
            INSERT INTO events
              (id, company_id, manager_id, title, description, location,
               starts_at, ends_at, signup_deadline, capacity, is_published)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            RETURNING id, company_id, manager_id, title, description, location,
                      starts_at, ends_at, signup_deadline, capacity, is_published,
                      0::bigint AS "registered_count?"
            "#,
            row.id, row.company_id, row.manager_id, row.title, row.description, row.location,
            row.starts_at, row.ends_at, row.signup_deadline, row.capacity, row.is_published
        )
            .fetch_one(&self.pool)
            .await?;

        Ok(r.into())
    }

    async fn get(&self, id: Uuid) -> RepoResult<EventOut> {
        let r = sqlx::query_as!(
            EventListRow,
            r#"
            SELECT e.id, e.company_id, e.manager_id, e.title, e.description, e.location,
                   e.starts_at, e.ends_at, e.signup_deadline, e.capacity, e.is_published,
                   (SELECT COUNT(*)::bigint FROM registrations r
                     WHERE r.event_id = e.id AND r.status = 'registered') AS "registered_count?"
              FROM events e WHERE e.id = $1
            "#,
            id
        )
            .fetch_optional(&self.pool)
            .await?;

        r.map(|x| x.into()).ok_or(RepoError::NotFound)
    }

    async fn update_partial(&self, id: Uuid, p: EventRowPatch) -> RepoResult<EventOut> {
        let r = sqlx::query_as!(
            EventListRow,
            r#"
            UPDATE events SET
                title           = COALESCE($2, title),
                description     = COALESCE($3, description),
                location        = COALESCE($4, location),
                starts_at       = COALESCE($5, starts_at),
                ends_at         = COALESCE($6, ends_at),
                signup_deadline = COALESCE($7, signup_deadline),
                capacity        = COALESCE($8, capacity),
                is_published    = COALESCE($9, is_published),
                updated_at      = now()
            WHERE id = $1
            RETURNING id, company_id, manager_id, title, description, location,
                      starts_at, ends_at, signup_deadline, capacity, is_published,
                      (SELECT COUNT(*)::bigint FROM registrations r
                        WHERE r.event_id = events.id AND r.status = 'registered') AS "registered_count?"
            "#,
            id,
            p.title, p.description, p.location,
            p.starts_at, p.ends_at, p.signup_deadline, p.capacity, p.is_published
        )
            .fetch_optional(&self.pool)
            .await?;

        r.map(|x| x.into()).ok_or(RepoError::NotFound)
    }

    async fn publish(&self, id: Uuid, on: bool) -> RepoResult<()> {
        let res = sqlx::query!(
            "UPDATE events SET is_published = $2, updated_at = now() WHERE id = $1",
            id, on
        )
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 { return Err(RepoError::NotFound); }
        Ok(())
    }

    async fn delete(&self, id: Uuid) -> RepoResult<()> {
        let res = sqlx::query!("DELETE FROM events WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 { return Err(RepoError::NotFound); }
        Ok(())
    }
}
