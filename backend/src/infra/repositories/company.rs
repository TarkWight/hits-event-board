use async_trait::async_trait;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::domain::entities::company_row::CompanyRow;
use crate::domain::entities::company_row::CompanyStatus;
use crate::domain::entities::company::CompanyWithCounts;
use crate::infra::errors::{RepoResult, RepoError, is_unique_violation};

#[async_trait]
pub trait CompanyRepository {
    async fn list_admin(&self, page: i32, limit: i32, q: Option<String>, include_archived: bool)
                        -> RepoResult<Vec<CompanyWithCounts>>;
    async fn set_status(&self, id: Uuid, status: CompanyStatus) -> RepoResult<CompanyWithCounts>;
    async fn list(&self, page: i32, limit: i32, q: Option<String>) -> RepoResult<Vec<CompanyWithCounts>>;
    async fn create(&self, row: CompanyRow) -> RepoResult<CompanyRow>;
    async fn get(&self, id: Uuid) -> RepoResult<CompanyWithCounts>;
    async fn update_name(&self, id: Uuid, name: &str) -> RepoResult<CompanyWithCounts>;
}

#[derive(Clone)]
pub struct PgCompanyRepository { pool: Pool<Postgres> }
impl PgCompanyRepository { pub fn new(pool: Pool<Postgres>) -> Self { Self { pool } } }

#[derive(sqlx::FromRow)]
struct CompanyListRow {
    id: Uuid,
    name: String,
    status: CompanyStatus,
    manager_count: Option<i64>,
    event_count: Option<i64>,
}

impl From<CompanyListRow> for CompanyWithCounts {
    fn from(r: CompanyListRow) -> Self {
        CompanyWithCounts {
            id: r.id,
            name: r.name,
            status: r.status,
            manager_count: r.manager_count,
            event_count: r.event_count,
        }
    }
}

#[async_trait]
impl CompanyRepository for PgCompanyRepository {
    async fn list(&self, page: i32, limit: i32, q: Option<String>) -> RepoResult<Vec<CompanyWithCounts>> {
        let page = page.max(1);
        let limit = limit.max(1);
        let offset_i64 = i64::from((page - 1) * limit);
        let limit_i64  = i64::from(limit);
        let q_like: Option<String> = q.as_ref().map(|s| format!("{}%", s.to_lowercase()));

        let rows = sqlx::query_as!(
            CompanyListRow,
            r#"
            SELECT c.id, c.name, c.status as "status: CompanyStatus",
                   (SELECT COUNT(*)::bigint FROM managers m WHERE m.company_id = c.id) AS "manager_count?",
                   (SELECT COUNT(*)::bigint FROM events   e WHERE e.company_id = c.id) AS "event_count?"
            FROM companies c
            WHERE c.status = 'active'
              AND ($1::text IS NULL OR lower(c.name) LIKE $1)
            ORDER BY lower(c.name)
            LIMIT $2 OFFSET $3
            "#,
            q_like,
            limit_i64,
            offset_i64
        )
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(CompanyWithCounts::from).collect())
    }

    async fn list_admin(&self, page: i32, limit: i32, q: Option<String>, include_archived: bool)
                        -> RepoResult<Vec<CompanyWithCounts>>
    {
        let page = page.max(1);
        let limit = limit.max(1);
        let offset_i64 = i64::from((page - 1) * limit);
        let limit_i64  = i64::from(limit);
        let q_like: Option<String> = q.as_ref().map(|s| format!("{}%", s.to_lowercase()));

        let rows = sqlx::query_as!(
            CompanyListRow,
            r#"
            SELECT c.id, c.name, c.status as "status: CompanyStatus",
                   (SELECT COUNT(*)::bigint FROM managers m WHERE m.company_id = c.id) AS "manager_count?",
                   (SELECT COUNT(*)::bigint FROM events   e WHERE e.company_id = c.id) AS "event_count?"
            FROM companies c
            WHERE ($1::bool OR c.status = 'active')
              AND ($2::text IS NULL OR lower(c.name) LIKE $2)
            ORDER BY lower(c.name)
            LIMIT $3 OFFSET $4
            "#,
            include_archived,
            q_like,
            limit_i64,
            offset_i64
        )
            .fetch_all(&self.pool)
            .await?;

        Ok(rows.into_iter().map(CompanyWithCounts::from).collect())
    }

    async fn create(&self, row: CompanyRow) -> RepoResult<CompanyRow> {
        let res = sqlx::query_as!(
            CompanyRow,
            r#"
            INSERT INTO companies (id, name, status)
            VALUES ($1, $2, 'active')
            RETURNING id, name
            "#,
            row.id, row.name
        )
            .fetch_one(&self.pool)
            .await;

        match res {
            Ok(r) => Ok(r),
            Err(e) => {
                if let Some(c) = is_unique_violation(&e) {
                    return Err(RepoError::Conflict(format!("unique violation: {c}")));
                }
                Err(e.into())
            }
        }
    }

    async fn get(&self, id: Uuid) -> RepoResult<CompanyWithCounts> {
        let r = sqlx::query_as!(
            CompanyListRow,
            r#"
            SELECT c.id, c.name, c.status as "status: CompanyStatus",
                   (SELECT COUNT(*)::bigint FROM managers m WHERE m.company_id = c.id) AS "manager_count?",
                   (SELECT COUNT(*)::bigint FROM events   e WHERE e.company_id = c.id) AS "event_count?"
            FROM companies c
            WHERE c.id = $1
            "#,
            id
        )
            .fetch_optional(&self.pool)
            .await?;

        r.map(CompanyWithCounts::from).ok_or(RepoError::NotFound)
    }

    async fn update_name(&self, id: Uuid, name: &str) -> RepoResult<CompanyWithCounts> {
        let r = sqlx::query_as!(
            CompanyListRow,
            r#"
            UPDATE companies
               SET name = $2, updated_at = now()
             WHERE id = $1
         RETURNING id, name, status as "status: CompanyStatus",
                   (SELECT COUNT(*)::bigint FROM managers m WHERE m.company_id = companies.id) AS "manager_count?",
                   (SELECT COUNT(*)::bigint FROM events   e WHERE e.company_id = companies.id) AS "event_count?"
            "#,
            id, name
        )
            .fetch_optional(&self.pool)
            .await?;

        r.map(CompanyWithCounts::from).ok_or(RepoError::NotFound)
    }

    async fn set_status(&self, id: Uuid, status: CompanyStatus) -> RepoResult<CompanyWithCounts> {
        let r = sqlx::query_as!(
            CompanyListRow,
            r#"
            UPDATE companies
               SET status = $2::company_status, updated_at = now()
             WHERE id = $1
         RETURNING id, name, status as "status: CompanyStatus",
                   (SELECT COUNT(*)::bigint FROM managers m WHERE m.company_id = companies.id) AS "manager_count?",
                   (SELECT COUNT(*)::bigint FROM events   e WHERE e.company_id = companies.id) AS "event_count?"
            "#,
            id, status as _
        )
            .fetch_optional(&self.pool)
            .await?;

        r.map(CompanyWithCounts::from).ok_or(RepoError::NotFound)
    }
}