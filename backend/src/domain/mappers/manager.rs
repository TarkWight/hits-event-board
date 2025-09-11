use crate::api::models::manager::ManagerOut;
use crate::domain::entities::manager_row::ManagerRow;

impl From<ManagerRow> for ManagerOut {
    fn from(r: ManagerRow) -> Self {
        Self {
            user_id: r.user_id,
            name: r.name,
            email: r.email,
            status: r.status,
        }
    }
}

pub fn to_manager_out_list(rows: Vec<ManagerRow>) -> Vec<ManagerOut> {
    rows.into_iter().map(ManagerOut::from).collect()
}