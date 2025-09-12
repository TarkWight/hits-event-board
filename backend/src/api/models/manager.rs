use serde::Serialize;
use uuid::Uuid;
use crate::auth::roles::ManagerStatus;

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct ManagerOut {
    pub user_id: Uuid,
    pub name: String,
    pub email: String,
    pub status: ManagerStatus,
}