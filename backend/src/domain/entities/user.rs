use time::OffsetDateTime;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCommon {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub password_hash: String,
    pub refresh_token_hash: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub refresh_token_expiration: Option<OffsetDateTime>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Student,
    Manager,
    Dean,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "manager_status", rename_all = "lowercase")]
pub enum ManagerStatus {
    Pending,
    Confirmed,
    Rejected,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Student { pub common: UserCommon }

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manager {
    pub common: UserCommon,
    pub status: ManagerStatus,
    pub company_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dean { pub common: UserCommon }

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum User {
    Student(Student),
    Manager(Manager),
    Dean(Dean),
}

#[derive(Debug, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum UserIn {
    Student { name: String, email: String, password: String },
    Manager { name: String, email: String, password: String, company_id: Uuid },
    Dean    { name: String, email: String, password: String },
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct UserUpdateCommon {
    pub name: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub refresh_token_hash: Option<String>,
    #[serde(with = "time::serde::rfc3339::option")]
    pub refresh_token_expiration: Option<OffsetDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "role", rename_all = "lowercase")]
pub enum UserUpdate {
    Student { common: UserUpdateCommon },
    Manager { common: UserUpdateCommon, status: Option<ManagerStatus>, company_id: Option<Uuid> },
    Dean    { common: UserUpdateCommon },
}
