use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct StudentRegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}
