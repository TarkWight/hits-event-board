use async_trait::async_trait;
use sqlx::{Pool, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::infra::errors::{RepoError, RepoResult};
use crate::domain::entities::registration_row::RegistrationRow;

#[async_trait]
pub trait RegistrationRepository {
    async fn count_for_event(&self, event_id: Uuid) -> RepoResult<i64>;
    async fn is_registered(&self, event_id: Uuid, student_id: Uuid) -> RepoResult<bool>;

    async fn register(&self, event_id: Uuid, student_id: Uuid, now_utc: OffsetDateTime) -> RepoResult<RegistrationRow>;

    async fn cancel(&self, event_id: Uuid, student_id: Uuid, now_utc: OffsetDateTime) -> RepoResult<()>;

    async fn list_for_event(&self, event_id: Uuid) -> RepoResult<Vec<RegistrationRow>>;
}

#[derive(Clone)]
pub struct PgRegistrationRepository { pool: Pool<Postgres> }
impl PgRegistrationRepository { pub fn new(pool: Pool<Postgres>) -> Self { Self { pool } } }

#[async_trait]
impl RegistrationRepository for PgRegistrationRepository {
    async fn count_for_event(&self, event_id: Uuid) -> RepoResult<i64> {
        let n: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*)::bigint AS "count!"
            FROM registrations
            WHERE event_id = $1 AND status = 'registered'
            "#,
            event_id
        )
            .fetch_one(&self.pool)
            .await?;
        Ok(n)
    }

    async fn is_registered(&self, event_id: Uuid, student_id: Uuid) -> RepoResult<bool> {
        let ex: bool = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM registrations
                WHERE event_id = $1 AND student_id = $2 AND status = 'registered'
            ) AS "exists!"
            "#,
            event_id, student_id
        )
            .fetch_one(&self.pool)
            .await?;
        Ok(ex)
    }

    async fn register(&self, event_id: Uuid, student_id: Uuid, now_utc: OffsetDateTime) -> RepoResult<RegistrationRow> {
        let mut tx: Transaction<'_, Postgres> = self.pool.begin().await?;

        let ev = sqlx::query!(
            r#"
            SELECT capacity, signup_deadline, starts_at, is_published
            FROM events
            WHERE id = $1
            FOR UPDATE
            "#,
            event_id
        )
            .fetch_optional(&mut *tx)
            .await?;

        let ev = match ev {
            Some(e) => e,
            None => { tx.rollback().await.ok(); return Err(RepoError::NotFound); }
        };

        if !ev.is_published {
            tx.rollback().await.ok();
            return Err(RepoError::Precondition("event not published".into()));
        }
        if let Some(dl) = ev.signup_deadline {
            if dl < now_utc {
                tx.rollback().await.ok();
                return Err(RepoError::Precondition("deadline passed".into()));
            }
        }

        if let Some(existing) = sqlx::query_as!(
            RegistrationRow,
            r#"
            SELECT
              r.event_id,
              r.student_id,
              u.name  AS student_name,
              u.email AS student_email,
              r.registered_at,
              r.gcal_event_id
            FROM registrations r
            JOIN users u ON u.id = r.student_id
            WHERE r.event_id = $1 AND r.student_id = $2 AND r.status = 'registered'
            "#,
            event_id, student_id
        )
            .fetch_optional(&mut *tx)
            .await?
        {
            tx.commit().await.ok();
            return Ok(existing);
        }

        if let Some(cap) = ev.capacity {
            let used: i64 = sqlx::query_scalar!(
                r#"
                SELECT COUNT(*)::bigint AS "count!"
                FROM registrations
                WHERE event_id = $1 AND status = 'registered'
                "#,
                event_id
            )
                .fetch_one(&mut *tx)
                .await?;
            if used >= cap as i64 {
                tx.rollback().await.ok();
                return Err(RepoError::Precondition("no seats".into()));
            }
        }

        let row = sqlx::query_as!(
            RegistrationRow,
            r#"
            WITH upsert AS (
              INSERT INTO registrations (event_id, student_id, status, registered_at)
              VALUES ($1, $2, 'registered', $3)
              ON CONFLICT (event_id, student_id) DO UPDATE
                SET status = 'registered',
                    canceled_at = NULL
              RETURNING event_id, student_id, registered_at, gcal_event_id
            )
            SELECT
              upsert.event_id,
              upsert.student_id,
              u.name  AS student_name,
              u.email AS student_email,
              upsert.registered_at,
              upsert.gcal_event_id
            FROM upsert
            JOIN users u ON u.id = upsert.student_id
            "#,
            event_id, student_id, now_utc
        )
            .fetch_one(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(row)
    }

    async fn cancel(&self, event_id: Uuid, student_id: Uuid, now_utc: OffsetDateTime) -> RepoResult<()> {
        let res = sqlx::query!(
            r#"
            UPDATE registrations
            SET status = 'canceled', canceled_at = $3
            WHERE event_id = $1 AND student_id = $2 AND status = 'registered'
            "#,
            event_id, student_id, now_utc
        )
            .execute(&self.pool)
            .await?;

        if res.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn list_for_event(&self, event_id: Uuid) -> RepoResult<Vec<RegistrationRow>> {
        let rows = sqlx::query_as!(
            RegistrationRow,
            r#"
            SELECT
              r.event_id,
              r.student_id,
              u.name  AS student_name,
              u.email AS student_email,
              r.registered_at,
              r.gcal_event_id
            FROM registrations r
            JOIN users u ON u.id = r.student_id
            WHERE r.event_id = $1 AND r.status = 'registered'
            ORDER BY r.registered_at DESC
            "#,
            event_id
        )
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }
}