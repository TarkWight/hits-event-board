use serde::Serialize;
use uuid::Uuid;
use crate::utils::token::TokenDTO;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserOut {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub role: String, // "student" | "manager" | "dean"
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RegisterOut {
    pub user: UserOut,
    pub tokens: TokenDTO,
}