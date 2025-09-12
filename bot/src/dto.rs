use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Tokens {
    pub access_token: String,
    pub access_token_expiration: String,
    pub refresh_token: String,
    pub refresh_token_expiration: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct UserOut {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct LoginOut {
    pub user: UserOut,
    pub tokens: Tokens,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct RegisterOut {
    pub user: UserOut,
    pub tokens: Tokens,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct Company {
    pub id: Uuid,
    pub name: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct MeOut {
    pub user_id: Uuid,
    pub role: String,                    // "student" | "manager" | "dean"
    pub manager_status: Option<String>,  // "pending" | "confirmed" | "rejected"
    pub company_id: Option<Uuid>,
    pub student_status: Option<String>,  // "created" | "linked" | "confirmed" | "rejected"
    pub email: String,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct EventShort {
    pub id: Uuid,
    pub title: String,
    pub starts_at: String,             // ISO
    pub is_published: bool,
    pub capacity: Option<i32>,
    pub registered_count: Option<i64>,
    // is_registered
}

#[derive(Debug, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub struct RegistrationEntry {
    pub student_id: Uuid,
    pub student_name: String,
    pub student_email: String,
    pub registered_at: String, // ISO
}