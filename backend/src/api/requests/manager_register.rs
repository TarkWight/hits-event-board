// api/requests/manager_register.rs
use serde::Deserialize;
use uuid::Uuid;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ManagerRegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub company_id: Uuid,
    pub telegram_user_id: Option<i64>,
}