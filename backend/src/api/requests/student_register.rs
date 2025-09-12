use serde::Deserialize;

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct StudentRegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
    pub telegram_user_id: Option<i64>,
}