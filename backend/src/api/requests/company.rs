use serde::Deserialize;
use crate::domain::entities::company::Company;

#[derive(Debug, Deserialize)]
pub struct CreateCompanyIn {
    pub name: String,
}

impl From<CreateCompanyIn> for Company {
    fn from(value: CreateCompanyIn) -> Self {
        Company::new(uuid::Uuid::new_v4(), value.name).unwrap()
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateCompanyIn {
    pub name: Option<String>,
}