use async_trait::async_trait;
use sqlx::{Pool, Postgres, Transaction};
use time::OffsetDateTime;
use uuid::Uuid;

use super::types::*;

#[async_trait]
pub trait RegistrationRepository {
    async fn count_for_event(&self, event_id: Uuid) -> RepoResult<i64>;
    async fn is_registered(&self, event_id: Uuid, student_id: Uuid) -> RepoResult<bool>;
    async fn register(&self, event_id: Uuid, student_id: Uuid, now_utc: OffsetDateTime) -> RepoResult<()>;
    async fn cancel(&self, event_id: Uuid, student_id: Uuid, now_utc: OffsetDateTime) -> RepoResult<()>;
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

    async fn register(&self, event_id: Uuid, student_id: Uuid, now_utc: OffsetDateTime) -> RepoResult<()> {
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

        let already: bool = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
                SELECT 1 FROM registrations
                WHERE event_id = $1 AND student_id = $2 AND status = 'registered'
            ) AS "exists!"
            "#,
            event_id, student_id
        )
            .fetch_one(&mut *tx)
            .await?;
        if already {
            tx.commit().await.ok();
            return Ok(());
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

        sqlx::query!(
            r#"
            INSERT INTO registrations (event_id, student_id, status)
            VALUES ($1, $2, 'registered')
            ON CONFLICT (event_id, student_id) DO UPDATE
              SET status = 'registered',
                  canceled_at = NULL
            "#,
            event_id, student_id
        )
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;
        Ok(())
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
}
