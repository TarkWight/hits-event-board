use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

// -------------------------------------------------
// DB row (агностично к TPT/TPH): одна строка = компания
// -------------------------------------------------
#[derive(Debug, Clone, FromRow)]
pub struct CompanyRow {
    pub id: Uuid,
    pub name: String,
}

impl CompanyRow {
    pub fn validate(&self) -> Result<(), CompanyValidationError> {
        if self.name.trim().is_empty() {
            return Err(CompanyValidationError::EmptyName);
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CompanyValidationError {
    #[error("company name must not be empty")]
    EmptyName,
}

// --------------------------------
// API DTOs (внешние контракты)
// --------------------------------
#[derive(Debug, Serialize)]
pub struct CompanyOut {
    pub id: Uuid,
    pub name: String,
    // Часто на список менеджеров/ивентов делают отдельные endpoints.
    // Здесь возвращаем агрегаты, чтобы не раздувать ответ.
    pub manager_count: Option<i64>,
    pub event_count: Option<i64>,
}

impl From<(CompanyRow, Option<i64>, Option<i64>)> for CompanyOut {
    fn from((row, manager_count, event_count): (CompanyRow, Option<i64>, Option<i64>)) -> Self {
        Self {
            id: row.id,
            name: row.name,
            manager_count,
            event_count,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct CreateCompanyIn {
    pub name: String,
}

impl TryFrom<CreateCompanyIn> for CompanyRow {
    type Error = CompanyValidationError;
    fn try_from(value: CreateCompanyIn) -> Result<Self, Self::Error> {
        let row = CompanyRow {
            id: Uuid::new_v4(),
            name: value.name,
        };
        row.validate()?;
        Ok(row)
    }
}

#[derive(Debug, Deserialize)]
pub struct UpdateCompanyIn {
    pub name: Option<String>,
}

impl CompanyRow {
    pub fn apply_update(&mut self, upd: UpdateCompanyIn) -> Result<(), CompanyValidationError> {
        if let Some(name) = upd.name {
            self.name = name;
        }
        self.validate()
    }
}

// ---------------------------------------------
// Связи (для справки, не обязательны в этом файле)
// ---------------------------------------------
// managers.company_id -> companies.id
// events.company_id   -> companies.id
// Для загрузки связанных сущностей используй отдельные запросы и собирай ответы
// вроде `CompanyOut` + counts, либо делай эндпоинты:
//   GET /companies/{id}/managers
//   GET /companies/{id}/events
