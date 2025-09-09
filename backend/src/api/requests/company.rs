use serde::Deserialize;
use uuid::Uuid;

use crate::domain::entities::company::{Company, CompanyPatch};

#[derive(Debug, Deserialize)]
pub struct CreateCompanyIn {
    pub name: String,
}

impl From<CreateCompanyIn> for Company {
    fn from(v: CreateCompanyIn) -> Self {
        Company::new(Uuid::new_v4(), v.name).expect("validated")
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateCompanyIn {
    pub name: Option<String>,
}

impl From<UpdateCompanyIn> for CompanyPatch {
    fn from(v: UpdateCompanyIn) -> Self {
        CompanyPatch { name: v.name }
    }
}