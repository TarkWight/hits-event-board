use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use time::OffsetDateTime;
use uuid::Uuid;
use crate::auth::roles::{ManagerStatus, UserRole, StudentStatus};
use crate::infra::errors::{RepoError, RepoResult};
use crate::domain::entities::user_row::UserRow;

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
    async fn find_by_refresh_token(&self, refresh_hash: &str, now: OffsetDateTime)
        -> RepoResult<UserRow>;
    async fn approve_user(&self, user_id: Uuid, approver_id: Uuid) -> RepoResult<()>;
    async fn create_student(&self, name: &str, email: &str, password_hash: &str)
        -> RepoResult<UserRow>;
    async fn create_manager(&self, name: &str, email: &str, password_hash: &str, company_id: Uuid)
        -> RepoResult<UserRow>;
    async fn set_refresh_token(&self, user_id: Uuid, refresh_hash: &str, expires_at: OffsetDateTime)
        -> RepoResult<()>;
    async fn find_by_id(&self, id: Uuid) -> RepoResult<UserRow>;
    async fn student_status(&self, user_id: Uuid) -> RepoResult<Option<StudentStatus>>;
    async fn manager_info(&self, user_id: Uuid) -> RepoResult<Option<(ManagerStatus, Uuid)>>;
    async fn list_students_by_status(&self, statuses: &[StudentStatus], page: i32, limit: i32, search: &str)
        -> RepoResult<Vec<(UserRow, Option<StudentStatus>)>>;

    async fn set_student_status(&self, user_id: Uuid, status: StudentStatus) -> RepoResult<()>;
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

    async fn find_by_refresh_token(
        &self,
        refresh_hash: &str,
        now: OffsetDateTime,
    ) -> RepoResult<UserRow> {
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
            WHERE refresh_token_hash = $1
              AND refresh_token_expiration > $2
            "#,
            refresh_hash,
            now
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

    async fn create_manager(
        &self,
        name: &str,
        email: &str,
        password_hash: &str,
        company_id: Uuid,
    ) -> RepoResult<UserRow> {
        let mut tx = self.pool.begin().await?;

        let user = sqlx::query_as!(
            UserRow,
            r#"
            INSERT INTO users (id, name, email, password_hash, role)
            VALUES ($1, $2, $3, $4, 'manager')
            RETURNING
                id,
                name,
                email::text as "email!",
                password_hash,
                role as "role: crate::auth::roles::UserRole"
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
            INSERT INTO managers (user_id, company_id, status)
            VALUES ($1, $2, 'pending')
            "#,
            user.id,
            company_id
        )
            .execute(&mut *tx)
            .await
            .map_err(RepoError::Db)?;

        tx.commit().await?;
        Ok(user)
    }

    async fn set_refresh_token(
        &self,
        user_id: Uuid,
        refresh_hash: &str,
        expires_at: OffsetDateTime,
    ) -> RepoResult<()> {
        let res = sqlx::query!(
            r#"
            UPDATE users
               SET refresh_token_hash = $2,
                   refresh_token_expiration = $3,
                   updated_at = now()
             WHERE id = $1
            "#,
            user_id,
            refresh_hash,
            expires_at
        )
            .execute(&self.pool)
            .await?;

        if res.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }

    async fn find_by_id(&self, id: Uuid) -> RepoResult<UserRow> {
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
            WHERE id = $1
            "#,
            id
        )
            .fetch_optional(&self.pool)
            .await?;

        u.ok_or(RepoError::NotFound)
    }

    async fn student_status(&self, user_id: Uuid) -> RepoResult<Option<StudentStatus>> {

        let row = sqlx::query!(
            r#"
            SELECT status as "status: StudentStatus"
            FROM students
            WHERE user_id = $1
            "#,
            user_id
        )
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| r.status))
    }

    async fn manager_info(&self, user_id: Uuid) -> RepoResult<Option<(ManagerStatus, Uuid)>> {
        let row = sqlx::query!(
            r#"
            SELECT
              status as "status: crate::auth::roles::ManagerStatus",
              company_id
            FROM managers
            WHERE user_id = $1
            LIMIT 1
            "#,
            user_id
        )
            .fetch_optional(&self.pool)
            .await?;

        Ok(row.map(|r| (r.status, r.company_id)))
    }

    async fn list_students_by_status(&self, statuses: &[StudentStatus], page: i32, limit: i32, search: &str)
        -> RepoResult<Vec<(UserRow, Option<StudentStatus>)>> {

        let page = page.max(1);
        let limit = limit.max(1);

        let offset_i64: i64 = ((page - 1) * limit) as i64;
        let limit_i64:  i64 = limit as i64;

        let status_texts: Vec<String> = statuses
            .iter()
            .map(|s| match s {
                StudentStatus::Created   => "created",
                StudentStatus::Linked    => "linked",
                StudentStatus::Confirmed => "confirmed",
                StudentStatus::Rejected  => "rejected",
            }.to_string())
            .collect();

        let pattern = format!("%{}%", search);

        let rows = sqlx::query!(
        r#"
        SELECT
          u.id,
          u.name,
          u.email::text as "email!",
          u.password_hash,
          u.role as "role: crate::auth::roles::UserRole",
          s.status as "status: crate::auth::roles::StudentStatus"
        FROM users u
        JOIN students s ON s.user_id = u.id
        WHERE u.role = 'student'
          AND ( $1 = '' OR u.name ILIKE $2 OR u.email ILIKE $2 )
          AND ( s.status::text = ANY($3::text[]) )
        ORDER BY u.created_at DESC NULLS LAST, u.id
        OFFSET $4 LIMIT $5
        "#,
        // $1..$5
        search,
        pattern,
        &status_texts[..],
        offset_i64,
        limit_i64,
    )
            .fetch_all(&self.pool)
            .await
            .map_err(RepoError::Db)?;

        let list = rows
            .into_iter()
            .map(|r| {
                let u = UserRow {
                    id: r.id,
                    name: r.name,
                    email: r.email,
                    password_hash: r.password_hash,
                    role: r.role,
                };
                (u, Some(r.status))
            })
            .collect::<Vec<(UserRow, Option<StudentStatus>)>>();

        Ok(list)
    }

    async fn set_student_status(
        &self,
        user_id: Uuid,
        status: StudentStatus,
    ) -> RepoResult<()> {
        let status_text = match status {
            StudentStatus::Created   => "created",
            StudentStatus::Linked    => "linked",
            StudentStatus::Confirmed => "confirmed",
            StudentStatus::Rejected  => "rejected",
        };

        let res = sqlx::query!(
        r#"
        UPDATE students
           SET status = ($2)::text::student_status
         WHERE user_id = $1
        "#,
        user_id,
        status_text,
    )
            .execute(&self.pool)
            .await
            .map_err(RepoError::Db)?;

        if res.rows_affected() == 0 {
            return Err(RepoError::NotFound);
        }
        Ok(())
    }
}
