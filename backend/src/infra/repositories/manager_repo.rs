use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use crate::auth::roles::ManagerStatus;
use crate::infra::errors::{RepoError, RepoResult};
use crate::domain::entities::manager_row::ManagerRow;

#[async_trait]
pub trait ManagerRepository {
    async fn list_for_company(&self, company_id: Uuid) -> RepoResult<Vec<ManagerRow>>;
    async fn set_status(&self, company_id: Uuid, user_id: Uuid, status: ManagerStatus) -> RepoResult<()>;
    async fn request_join(&self, company_id: Uuid, user_id: Uuid) -> RepoResult<()>;
}

#[derive(Clone)]
pub struct PgManagerRepository { pool: Pool<Postgres> }
impl PgManagerRepository { pub fn new(pool: Pool<Postgres>) -> Self { Self { pool } } }

#[async_trait]
impl ManagerRepository for PgManagerRepository {
    async fn list_for_company(&self, company_id: Uuid) -> RepoResult<Vec<ManagerRow>> {
        let rows = sqlx::query_as!(
            ManagerRow,
            r#"
            SELECT
                m.user_id,
                m.company_id,
                u.name,
                u.email::text as "email!",
                m.status       as "status: ManagerStatus"
            FROM managers m
            JOIN users u ON u.id = m.user_id
            WHERE m.company_id = $1
            ORDER BY lower(u.name)
            "#,
            company_id
        )
            .fetch_all(&self.pool)
            .await?;
        Ok(rows)
    }
    async fn set_status(&self, company_id: Uuid, user_id: Uuid, status: ManagerStatus) -> RepoResult<()> {
        let res = sqlx::query!(
            r#"
            UPDATE managers
               SET status = $3
             WHERE company_id = $1
               AND user_id    = $2
            "#,
            company_id,
            user_id,
            status as _
        )
            .execute(&self.pool)
            .await?;

        if res.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }
    async fn request_join(&self, company_id: Uuid, user_id: Uuid) -> RepoResult<()> {
        let res = sqlx::query!(
        r#"
        INSERT INTO managers (company_id, user_id, status)
        VALUES ($1, $2, 'pending')
        ON CONFLICT (company_id, user_id) DO NOTHING
        "#,
        company_id,
        user_id
    )
            .execute(&self.pool)
            .await?;

        if res.rows_affected() == 0 {
            return Err(RepoError::Conflict("manager already requested join".into()));
        }

        Ok(())
    }

}