use sqlx::FromRow;
use uuid::Uuid;
use crate::auth::roles::StudentStatus;

#[derive(Debug, Clone, FromRow)]
pub struct StudentRow {
    pub user_id: Uuid,
    pub status: StudentStatus,
}