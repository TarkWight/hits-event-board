use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum UserRole {
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

#[derive(
    sqlx::Type, Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize
)]
#[sqlx(type_name = "student_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum StudentStatus {
    Created,
    Linked,
    Confirmed,
    Rejected,
}
