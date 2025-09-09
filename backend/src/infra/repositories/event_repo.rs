use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::domain::entities::event_row::EventRow;
use crate::domain::mappers::event::EventWithCount;
use crate::infra::errors::{RepoError, RepoResult};

#[derive(Debug, Default, Clone)]
pub struct EventListFilter {
    pub company_id: Option<Uuid>,
    pub manager_id: Option<Uuid>,
    pub published: Option<bool>,
    pub q: Option<String>,
    pub from: Option<OffsetDateTime>,
    pub to: Option<OffsetDateTime>,
}

#[async_trait]
pub trait EventRepository {
    async fn list(&self, page: i32, limit: i32, f: EventListFilter) -> RepoResult<Vec<EventWithCount>>;
    async fn create(&self, row: EventRow) -> RepoResult<EventRow>;
    async fn get(&self, id: Uuid) -> RepoResult<EventWithCount>;
    async fn update_all(&self, row: EventRow) -> RepoResult<EventWithCount>;
    async fn delete(&self, id: Uuid) -> RepoResult<()>;
    async fn set_published(&self, id: Uuid, flag: bool) -> RepoResult<EventWithCount>;
    async fn set_deadline(&self, id: Uuid, deadline: Option<OffsetDateTime>) -> RepoResult<EventWithCount>;
    async fn list_registrations(&self, event_id: Uuid) -> RepoResult<Vec<(Uuid, OffsetDateTime)>>;
    async fn count_registrations(&self, event_id: Uuid) -> RepoResult<i64>;
    async fn register(&self, event_id: Uuid, student_id: Uuid, now_utc: OffsetDateTime) -> RepoResult<()>;
    async fn cancel_registration(&self, event_id: Uuid, student_id: Uuid, now_utc: OffsetDateTime) -> RepoResult<()>;
}

#[derive(Clone)]
pub struct PgEventRepository { pool: Pool<Postgres> }
impl PgEventRepository { pub fn new(pool: Pool<Postgres>) -> Self { Self { pool } } }

#[derive(sqlx::FromRow)]
struct EventListRow {
    id: Uuid,
    company_id: Uuid,
    manager_id: Uuid,
    title: String,
    description: Option<String>,
    location: Option<String>,
    starts_at: OffsetDateTime,
    ends_at: Option<OffsetDateTime>,
    signup_deadline: Option<OffsetDateTime>,
    capacity: Option<i32>,
    is_published: bool,
    registered_count: Option<i64>,
}

impl From<EventListRow> for EventWithCount {
    fn from(r: EventListRow) -> Self {
        Self {
            id: r.id, company_id: r.company_id, manager_id: r.manager_id,
            title: r.title, description: r.description, location: r.location,
            starts_at: r.starts_at, ends_at: r.ends_at, signup_deadline: r.signup_deadline,
            capacity: r.capacity, is_published: r.is_published, registered_count: r.registered_count,
        }
    }
}

#[async_trait]
impl EventRepository for PgEventRepository {
    async fn list(&self, page: i32, limit: i32, f: EventListFilter) -> RepoResult<Vec<EventWithCount>> {
        let page = page.max(1);
        let limit = limit.max(1);
        let offset_i64 = i64::from((page - 1) * limit);
        let limit_i64  = i64::from(limit);

        let q_like: Option<String> = f.q.as_ref().map(|s| format!("%{}%", s));

        let rows = sqlx::query_as!(
            EventListRow,
            r#"
            SELECT e.id, e.company_id, e.manager_id, e.title, e.description, e.location,
                   e.starts_at, e.ends_at, e.signup_deadline, e.capacity, e.is_published,
                   (SELECT COUNT(*)::bigint FROM registrations er WHERE er.event_id = e.id) AS "registered_count?"
            FROM events e
            WHERE ($1::uuid IS NULL OR e.company_id   = $1)
              AND ($2::uuid IS NULL OR e.manager_id   = $2)
              AND ($3::bool IS NULL OR e.is_published = $3)
              AND ($4::text IS NULL OR e.title ILIKE $4)
              AND ($5::timestamptz IS NULL OR e.starts_at >= $5)
              AND ($6::timestamptz IS NULL OR e.starts_at <= $6)
            ORDER BY e.starts_at DESC
            LIMIT $7 OFFSET $8
            "#,
            f.company_id, f.manager_id, f.published, q_like, f.from, f.to,
            limit_i64, offset_i64
        )
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(EventWithCount::from).collect())
    }

    async fn create(&self, row: EventRow) -> RepoResult<EventRow> {
        let r = sqlx::query_as!(
            EventRow,
            r#"
            INSERT INTO events
                (id, company_id, manager_id, title, description, location,
                 starts_at, ends_at, signup_deadline, capacity, is_published)
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11)
            RETURNING id, company_id, manager_id, title, description, location,
                      starts_at, ends_at, signup_deadline, capacity, is_published
            "#,
            row.id, row.company_id, row.manager_id, row.title, row.description, row.location,
            row.starts_at, row.ends_at, row.signup_deadline, row.capacity, row.is_published
        )
            .fetch_one(&self.pool)
            .await?;

        Ok(r)
    }

    async fn get(&self, id: Uuid) -> RepoResult<EventWithCount> {
        let r = sqlx::query_as!(
            EventListRow,
            r#"
            SELECT e.id, e.company_id, e.manager_id, e.title, e.description, e.location,
                   e.starts_at, e.ends_at, e.signup_deadline, e.capacity, e.is_published,
                   (SELECT COUNT(*)::bigint FROM registrations er WHERE er.event_id = e.id) AS "registered_count?"
            FROM events e
            WHERE e.id = $1
            "#,
            id
        )
            .fetch_optional(&self.pool)
            .await?;

        r.map(EventWithCount::from).ok_or(RepoError::NotFound)
    }

    async fn update_all(&self, row: EventRow) -> RepoResult<EventWithCount> {
        let r = sqlx::query_as!(
            EventListRow,
            r#"
            UPDATE events SET
                title=$2, description=$3, location=$4,
                starts_at=$5, ends_at=$6, signup_deadline=$7,
                capacity=$8, is_published=$9, updated_at=now()
            WHERE id=$1
            RETURNING id, company_id, manager_id, title, description, location,
                      starts_at, ends_at, signup_deadline, capacity, is_published,
                      (SELECT COUNT(*)::bigint FROM registrations er WHERE er.event_id = events.id) AS "registered_count?"
            "#,
            row.id, row.title, row.description, row.location,
            row.starts_at, row.ends_at, row.signup_deadline,
            row.capacity, row.is_published
        )
            .fetch_optional(&self.pool)
            .await?;

        r.map(EventWithCount::from).ok_or(RepoError::NotFound)
    }

    async fn delete(&self, id: Uuid) -> RepoResult<()> {
        let res = sqlx::query!("DELETE FROM events WHERE id = $1", id)
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 { return Err(RepoError::NotFound); }
        Ok(())
    }
    async fn set_published(&self, id: Uuid, flag: bool) -> RepoResult<EventWithCount> {
        let r = sqlx::query_as!(
            EventListRow,
            r#"
            UPDATE events
               SET is_published = $2, updated_at = now()
             WHERE id = $1
            RETURNING id, company_id, manager_id, title, description, location,
                      starts_at, ends_at, signup_deadline, capacity, is_published,
                      (SELECT COUNT(*)::bigint FROM registrations er WHERE er.event_id = events.id) AS "registered_count?"
            "#,
            id, flag
        )
            .fetch_optional(&self.pool).await?;
        r.map(EventWithCount::from).ok_or(RepoError::NotFound)
    }

    async fn set_deadline(&self, id: Uuid, deadline: Option<OffsetDateTime>) -> RepoResult<EventWithCount> {
        let r = sqlx::query_as!(
            EventListRow,
            r#"
            UPDATE events
               SET signup_deadline = $2, updated_at = now()
             WHERE id = $1
            RETURNING id, company_id, manager_id, title, description, location,
                      starts_at, ends_at, signup_deadline, capacity, is_published,
                      (SELECT COUNT(*)::bigint FROM registrations er WHERE er.event_id = events.id) AS "registered_count?"
            "#,
            id, deadline
        )
            .fetch_optional(&self.pool).await?;
        r.map(EventWithCount::from).ok_or(RepoError::NotFound)
    }

    async fn list_registrations(&self, event_id: Uuid) -> RepoResult<Vec<(Uuid, OffsetDateTime)>> {
        let rows = sqlx::query!(
            r#"SELECT student_id, registered_at FROM registrations WHERE event_id = $1 ORDER BY registered_at DESC"#,
            event_id
        )
            .fetch_all(&self.pool).await?;
        Ok(rows.into_iter().map(|r| (r.student_id, r.registered_at)).collect())
    }

    async fn count_registrations(&self, event_id: Uuid) -> RepoResult<i64> {
        let n = sqlx::query_scalar!(
            r#"SELECT COUNT(*)::bigint FROM registrations WHERE event_id = $1"#,
            event_id
        )
            .fetch_one(&self.pool)
            .await?
            .unwrap_or(0);
        Ok(n)
    }

    async fn register(&self, event_id: Uuid, student_id: Uuid, now_utc: OffsetDateTime) -> RepoResult<()> {
        // простая upsert-семантика: если уже есть — ничего не делаем, иначе вставка
        let _ = sqlx::query!(
            r#"
            INSERT INTO registrations (event_id, student_id, registered_at)
            VALUES ($1, $2, $3)
            ON CONFLICT (event_id, student_id) DO NOTHING
            "#,
            event_id, student_id, now_utc
        )
            .execute(&self.pool).await?;
        Ok(())
    }

    async fn cancel_registration(&self, event_id: Uuid, student_id: Uuid, _now_utc: OffsetDateTime) -> RepoResult<()> {
        let res = sqlx::query!(
            r#"DELETE FROM registrations WHERE event_id = $1 AND student_id = $2"#,
            event_id, student_id
        )
            .execute(&self.pool).await?;
        if res.rows_affected() == 0 { return Err(RepoError::NotFound); }
        Ok(())
    }
}