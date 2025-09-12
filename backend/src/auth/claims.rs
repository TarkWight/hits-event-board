use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub iss: String,
    pub aud: String,
    pub sub: String,
    pub iat: i64,
    pub exp: i64,
    pub jti: String,

    pub user_id: Uuid,
    pub role: String,                  // "student" | "manager" | "dean"
    pub manager_status: Option<String>,// "pending" | "confirmed" | "rejected"
    pub company_id: Option<Uuid>,
}