// infra/repositories/telegram_repo.rs
use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::infra::errors::{RepoError, RepoResult};

#[async_trait]
pub trait TelegramLinkRepository {
    async fn link(&self, user_id: Uuid, telegram_user_id: i64) -> RepoResult<()>;
    async fn unlink_by_user(&self, user_id: Uuid) -> RepoResult<()>;
    async fn get_user_by_telegram(&self, telegram_user_id: i64) -> RepoResult<Uuid>;
    async fn exists_for_user(&self, user_id: Uuid) -> RepoResult<bool>;
    async fn is_student(&self, user_id: Uuid) -> RepoResult<bool>;
}

#[derive(Clone)]
pub struct PgTelegramLinkRepository { pool: Pool<Postgres> }
impl PgTelegramLinkRepository { pub fn new(pool: Pool<Postgres>) -> Self { Self { pool } } }

#[async_trait]
impl TelegramLinkRepository for PgTelegramLinkRepository {
    async fn link(&self, user_id: Uuid, telegram_user_id: i64) -> RepoResult<()> {
        sqlx::query!(
            r#"
            INSERT INTO telegram_links (user_id, telegram_user_id, created_at)
            VALUES ($1, $2, now())
            ON CONFLICT (user_id) DO UPDATE
              SET telegram_user_id = EXCLUDED.telegram_user_id
            "#,
            user_id, telegram_user_id
        )
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    async fn unlink_by_user(&self, user_id: Uuid) -> RepoResult<()> {
        let res = sqlx::query!(
            r#"DELETE FROM telegram_links WHERE user_id = $1"#,
            user_id
        )
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn get_user_by_telegram(&self, telegram_user_id: i64) -> RepoResult<Uuid> {
        let row = sqlx::query!(
            r#"SELECT user_id FROM telegram_links WHERE telegram_user_id = $1"#,
            telegram_user_id
        )
            .fetch_optional(&self.pool)
            .await?;
        row.map(|r| r.user_id).ok_or(RepoError::NotFound)
    }

    async fn exists_for_user(&self, user_id: Uuid) -> RepoResult<bool> {
        let exists: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM telegram_links WHERE user_id = $1) AS "exists!""#,
            user_id
        )
            .fetch_one(&self.pool)
            .await?;
        Ok(exists)
    }

    async fn is_student(&self, user_id: Uuid) -> RepoResult<bool> {
        let exists: bool = sqlx::query_scalar!(
            r#"SELECT EXISTS(SELECT 1 FROM students WHERE user_id = $1) AS "exists!""#,
            user_id
        )
            .fetch_one(&self.pool)
            .await?;
        Ok(exists)
    }
}