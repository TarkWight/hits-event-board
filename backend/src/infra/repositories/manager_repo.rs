use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use super::types::*;

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "manager_status", rename_all = "lowercase")]
pub enum ManagerStatus { Pending, Confirmed, Rejected }

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole { Student, Manager, Dean }

#[async_trait]
pub trait ManagerRepository {
    async fn request_join(&self, company_id: Uuid, user_id: Uuid) -> RepoResult<()>;
    async fn approve(&self, company_id: Uuid, user_id: Uuid) -> RepoResult<()>;
    async fn role_of(&self, user_id: Uuid) -> RepoResult<(UserRole, Option<(Uuid, ManagerStatus)>)>;
}

#[derive(Clone)]
pub struct PgManagerRepository { pool: Pool<Postgres> }
impl PgManagerRepository { pub fn new(pool: Pool<Postgres>) -> Self { Self { pool } } }

#[async_trait]
impl ManagerRepository for PgManagerRepository {
    async fn request_join(&self, company_id: Uuid, user_id: Uuid) -> RepoResult<()> {
        let res = sqlx::query!(
            r#"
            INSERT INTO managers (user_id, status, company_id)
            VALUES ($1, 'pending', $2)
            ON CONFLICT (user_id) DO UPDATE
                SET company_id = EXCLUDED.company_id, status = 'pending'
            "#,
            user_id, company_id
        )
            .execute(&self.pool)
            .await;

        match res {
            Ok(_) => Ok(()),
            Err(e) => {
                if let Some(c) = super::types::is_unique_violation(&e) {
                    return Err(RepoError::Conflict(format!("unique: {c}")));
                }
                Err(e.into())
            }
        }
    }

    async fn approve(&self, company_id: Uuid, user_id: Uuid) -> RepoResult<()> {
        let res = sqlx::query!(
            r#"UPDATE managers
               SET status = 'confirmed'
             WHERE user_id = $1 AND company_id = $2"#,
            user_id, company_id
        )
            .execute(&self.pool)
            .await?;
        if res.rows_affected() == 0 { return Err(RepoError::NotFound); }
        Ok(())
    }

    async fn role_of(&self, user_id: Uuid) -> RepoResult<(UserRole, Option<(Uuid, ManagerStatus)>)> {
        let rec = sqlx::query!(
            r#"
            SELECT u.role              as "role: UserRole",
                   m.company_id        as "company_id?",
                   m.status            as "status: ManagerStatus?"
              FROM users u
              LEFT JOIN managers m ON m.user_id = u.id
             WHERE u.id = $1
            "#,
            user_id
        )
            .fetch_optional(&self.pool)
            .await?;

        match rec {
            Some(r) => Ok((
                r.role,
                r.company_id.zip(r.status) // Option<(Uuid, ManagerStatus)>
            )),
            None => Err(RepoError::NotFound),
        }
    }
}
