use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct ManagerRegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub id_company: Uuid,
}