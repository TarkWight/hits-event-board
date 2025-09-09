use async_trait::async_trait;
use sqlx::{Pool, Postgres, FromRow};
use uuid::Uuid;
use crate::infra::errors::{RepoError, RepoResult};

#[derive(sqlx::Type, Debug, Clone, Copy, PartialEq, Eq)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Student,
    Manager,
    Dean,
}

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
}

#[async_trait]
pub trait UserRepository {
    async fn create(
        &self,
        id: Uuid,
        name: &str,
        email: &str,
        password_hash: &str,
        role: UserRole,
    ) -> RepoResult<UserRow>;

    async fn find_by_email(&self, email: &str) -> RepoResult<UserRow>;

    async fn approve_user(&self, user_id: Uuid, approver_id: Uuid) -> RepoResult<()>;

    async fn create_student(
        &self,
        name: &str,
        email: &str,
        password_hash: &str,
    ) -> RepoResult<UserRow>;
}

#[derive(Clone)]
pub struct PgUserRepository {
    pool: Pool<Postgres>,
}

impl PgUserRepository {
    pub fn new(pool: Pool<Postgres>) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl UserRepository for PgUserRepository {
    async fn create(
        &self,
        id: Uuid,
        name: &str,
        email: &str,
        password_hash: &str,
        role: UserRole,
    ) -> RepoResult<UserRow> {
        let res = sqlx::query_as!(
            UserRow,
            r#"
            INSERT INTO users (id, name, email, password_hash, role)
            VALUES ($1, $2, $3, $4, $5)
            RETURNING
                id,
                name,
                email::text as "email!",
                password_hash,
                role as "role: UserRole"
            "#,
            id,
            name,
            email,
            password_hash,
            role as _
        )
            .fetch_one(&self.pool)
            .await;

        res.map_err(RepoError::Db)
    }

    async fn find_by_email(&self, email: &str) -> RepoResult<UserRow> {
        let u = sqlx::query_as!(
            UserRow,
            r#"
            SELECT
                id,
                name,
                email::text as "email!",
                password_hash,
                role as "role: UserRole"
            FROM users
            WHERE email = $1
            "#,
            email
        )
            .fetch_optional(&self.pool)
            .await?;

        u.ok_or(RepoError::NotFound)
    }

    async fn approve_user(&self, user_id: Uuid, _approver_id: Uuid) -> RepoResult<()> {
        let res = sqlx::query!(
            r#"
            UPDATE managers
            SET status = 'confirmed'
            WHERE user_id = $1
              AND status  = 'pending'
            "#,
            user_id
        )
            .execute(&self.pool)
            .await?;

        if res.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn create_student(
        &self,
        name: &str,
        email: &str,
        password_hash: &str,
    ) -> RepoResult<UserRow> {
        let mut tx = self.pool.begin().await?;

        let user = sqlx::query_as!(
            UserRow,
            r#"
            INSERT INTO users (id, name, email, password_hash, role)
            VALUES ($1, $2, $3, $4, 'student')
            RETURNING
                id,
                name,
                email::text as "email!",
                password_hash,
                role as "role: UserRole"
            "#,
            Uuid::new_v4(),
            name,
            email,
            password_hash
        )
            .fetch_one(&mut *tx)
            .await
            .map_err(RepoError::Db)?;

        sqlx::query!(
            r#"
            INSERT INTO students (user_id)
            VALUES ($1)
            ON CONFLICT (user_id) DO NOTHING
            "#,
            user.id
        )
            .execute(&mut *tx)
            .await
            .map_err(RepoError::Db)?;

        tx.commit().await?;
        Ok(user)
    }
}