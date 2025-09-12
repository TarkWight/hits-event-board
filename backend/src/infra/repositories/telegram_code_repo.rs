use async_trait::async_trait;
use rand::{thread_rng, Rng};
use sqlx::{Pool, Postgres, Row};
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::infra::errors::{RepoError, RepoResult};

#[async_trait]
pub trait TelegramCodeRepository {
    async fn create_code(&self, user_id: Uuid, ttl_minutes: i64) -> RepoResult<String>;
    async fn consume_code(&self, code: &str) -> RepoResult<Uuid>;
}

#[derive(Clone)]
pub struct PgTelegramCodeRepository {
    pool: Pool<Postgres>,
}
impl PgTelegramCodeRepository {
    pub fn new(pool: Pool<Postgres>) -> Self { Self { pool } }
}

fn random_code_6() -> String {
    let n: u32 = thread_rng().gen_range(0..1_000_000);
    format!("{n:06}")
}

#[async_trait]
impl TelegramCodeRepository for PgTelegramCodeRepository {
    async fn create_code(&self, user_id: Uuid, ttl_minutes: i64) -> RepoResult<String> {
        let expires_at = OffsetDateTime::now_utc() + Duration::minutes(ttl_minutes);

        const MAX_TRIES: usize = 5;
        for _ in 0..MAX_TRIES {
            let code = random_code_6();

            let res = sqlx::query(
                r#"
                INSERT INTO telegram_link_codes (code, user_id, expires_at)
                VALUES ($1, $2, $3)
                "#,
            )
                .bind(&code)
                .bind(user_id)
                .bind(expires_at)
                .execute(&self.pool)
                .await;

            match res {
                Ok(_) => return Ok(code),
                Err(e) => {
                    if let sqlx::Error::Database(db) = &e {
                        if db.kind() == sqlx::error::ErrorKind::UniqueViolation {
                            continue;
                        }
                    }
                    return Err(RepoError::Db(e));
                }
            }
        }
        Err(RepoError::Conflict("cannot allocate unique code, try again".into()))
    }

    async fn consume_code(&self, code: &str) -> RepoResult<Uuid> {
        let row = sqlx::query(
            r#"
            UPDATE telegram_link_codes
               SET used_at = now()
             WHERE code = $1
               AND used_at IS NULL
               AND expires_at > now()
         RETURNING user_id
            "#,
        )
            .bind(code)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepoError::Db)?;

        match row {
            Some(r) => Ok(r.get::<Uuid, _>("user_id")),
            None    => Err(RepoError::NotFound),
        }
    }
}