use uuid::Uuid;
use sqlx::FromRow;
use crate::auth::roles::ManagerStatus;
use super::manager::{Manager, ManagerValidationError};

#[derive(Debug, Clone, FromRow)]
pub struct ManagerRow {
    pub user_id: Uuid,
    pub company_id: Uuid,
    pub name: String,
    pub email: String,
    pub status: ManagerStatus,
}

impl TryFrom<ManagerRow> for Manager {
    type Error = ManagerValidationError;
    fn try_from(row: ManagerRow) -> Result<Self, Self::Error> {
        let m = Manager {
            user_id: row.user_id,
            company_id: row.company_id,
            name: row.name,
            email: row.email,
            status: row.status,
        };
        m.validate()?;
        Ok(m)
    }
}

impl From<Manager> for ManagerRow {
    fn from(m: Manager) -> Self {
        Self {
            user_id: m.user_id,
            company_id: m.company_id,
            name: m.name,
            email: m.email,
            status: m.status,
        }
    }
}