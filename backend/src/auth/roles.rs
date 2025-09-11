use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum Role {
    Student,
    Manager,
    Dean
}

#[derive(
    sqlx::Type, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize
)]
#[sqlx(type_name = "manager_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ManagerStatus {
    Pending,
    Confirmed,
    Rejected,
}