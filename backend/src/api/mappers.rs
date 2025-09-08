// Mappers between API In/DB Row/Out
use uuid::Uuid;
use crate::api::requests::{CreateCompanyIn, CreateEventIn};
use crate::domain::entities::company::CompanyRow;
use crate::domain::entities::event::EventRow;
use crate::api::models::{CompanyOut, EventOut};

impl TryFrom<CreateCompanyIn> for CompanyRow {
    type Error = String;
    fn try_from(value: CreateCompanyIn) -> Result<Self, Self::Error> {
        let row = CompanyRow { id: Uuid::new_v4(), name: value.name };
        row.validate()?;
        Ok(row)
    }
}

impl TryFrom<CreateEventIn> for EventRow {
    type Error = String;
    fn try_from(value: CreateEventIn) -> Result<Self, Self::Error> {
        let row = EventRow {
            id: Uuid::new_v4(),
            company_id: value.company_id,
            manager_id: value.manager_id,
            title: value.title,
            description: value.short_desc,
            location: value.location,
            starts_at: value.starts_at,
            ends_at: value.ends_at,
            signup_deadline: value.signup_deadline,
            capacity: value.capacity,
            is_published: value.is_published,
        };
        row.validate()?;
        Ok(row)
    }
}

impl From<(CompanyRow, Option<i64>, Option<i64>)> for CompanyOut {
    fn from((row, manager_count, event_count): (CompanyRow, Option<i64>, Option<i64>)) -> Self {
        CompanyOut { id: row.id, name: row.name, manager_count, event_count }
    }
}

impl From<(EventRow, Option<i64>)> for EventOut {
    fn from((row, count): (EventRow, Option<i64>)) -> Self {
        EventOut {
            id: row.id,
            company_id: row.company_id,
            manager_id: row.manager_id,
            title: row.title,
            short_desc: row.description,
            starts_at: row.starts_at,
            ends_at: row.ends_at,
            signup_deadline: row.signup_deadline,
            location: row.location,
            registered_count: count,
        }
    }
}
