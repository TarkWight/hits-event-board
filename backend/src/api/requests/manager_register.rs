use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ManagerRegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub company_id: Uuid,
}