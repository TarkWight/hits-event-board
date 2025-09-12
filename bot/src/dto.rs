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