use sqlx::FromRow;
use uuid::Uuid;
use crate::auth::roles::UserRole;

#[derive(Debug, Clone, FromRow)]
pub struct UserRow {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub role: UserRole,
}