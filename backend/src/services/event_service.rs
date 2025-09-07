use crate::db::PgPool;
use crate::error::{AppError, AppResult};
use crate::models::{CreateEventIn, EventOut};
use crate::models::{EventRow, UpdateEventIn};
use sqlx::Acquire;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Clone)]
pub struct EventService {
    pool: PgPool,
}

impl EventService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn list_events(&self) -> AppResult<Vec<EventOut>> { // Multiple associated items are never used
        let rows = sqlx::query!(
            r#"
            SELECT id, company_id, manager_id, title, description AS short_desc,
                   starts_at, ends_at, signup_deadline, location,
                   NULL::bigint AS registered_count
            FROM events
            ORDER BY starts_at DESC
            "#
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("db error: {e}");
                AppError::internal("database error")
            })?;

        Ok(rows
            .into_iter()
            .map(|r| EventOut {
                id: r.id,
                company_id: r.company_id,
                manager_id: r.manager_id,
                title: r.title,
                short_desc: r.short_desc,
                starts_at: r.starts_at,
                ends_at: r.ends_at,
                signup_deadline: r.signup_deadline,
                location: r.location,
                registered_count: None,
            })
            .collect())
    }

    pub async fn get_event(&self, id: Uuid) -> AppResult<EventOut> {
        let row = sqlx::query!(
            r#"
            SELECT e.id, e.company_id, e.manager_id, e.title,
                   e.description AS short_desc, e.starts_at, e.ends_at,
                   e.signup_deadline, e.location,
                   (SELECT COUNT(*)::bigint FROM event_registrations er WHERE er.event_id = e.id) AS registered_count
            FROM events e
            WHERE e.id = $1
            "#,
            id
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("db error: {e}");
                AppError::internal("database error")
            })?
            .ok_or_else(|| AppError::bad("event not found"))?;

        Ok(EventOut {
            id: row.id,
            company_id: row.company_id,
            manager_id: row.manager_id,
            title: row.title,
            short_desc: row.short_desc,
            starts_at: row.starts_at,
            ends_at: row.ends_at,
            signup_deadline: row.signup_deadline,
            location: row.location,
            registered_count: row.registered_count,
        })
    }

    pub async fn create_event(&self, input: CreateEventIn) -> AppResult<EventOut> {
        Self::validate_event_input(&input)?;

        let mut conn = self.pool.acquire().await.map_err(|e| {
            tracing::error!("db error: {e}");
            AppError::internal("database error")
        })?;
        let mut tx = conn.begin().await.map_err(|e| {
            tracing::error!("db error: {e}");
            AppError::internal("database error")
        })?;

        let id = Uuid::new_v4();
        sqlx::query!(
            r#"
            INSERT INTO events (
              id, company_id, manager_id, title, description, location,
              starts_at, ends_at, signup_deadline
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            "#,
            id,
            input.company_id,
            input.manager_id,
            input.title,
            input.short_desc,
            input.location,
            input.starts_at,
            input.ends_at,
            input.signup_deadline
        )
            .execute(&mut *tx)
            .await
            .map_err(|e| {
                tracing::error!("db insert error: {e}");
                AppError::internal("database error")
            })?;

        tx.commit().await.map_err(|e| {
            tracing::error!("db tx commit error: {e}");
            AppError::internal("database error")
        })?;

        Ok(EventOut {
            id,
            company_id: input.company_id,
            manager_id: input.manager_id,
            title: input.title,
            short_desc: input.short_desc,
            starts_at: input.starts_at,
            ends_at: input.ends_at,
            signup_deadline: input.signup_deadline,
            location: input.location,
            registered_count: None,
        })
    }

    fn validate_event_input(input: &CreateEventIn) -> AppResult<()> {
        if input.title.trim().is_empty() {
            return Err(AppError::bad("title must not be empty"));
        }
        if input.location.as_deref().unwrap_or("").trim().is_empty() {
            return Err(AppError::bad("location must not be empty"));
        }
        if let Some(end) = input.ends_at {
            if end < input.starts_at {
                return Err(AppError::bad("ends_at must be after starts_at"));
            }
        }
        if let Some(dl) = input.signup_deadline {
            if dl > input.starts_at {
                return Err(AppError::bad(
                    "signup_deadline must be before or equal to starts_at",
                ));
            }
        }
        Ok(())
    }

    pub async fn register_student_to_event(
        &self,
        event_id: Uuid,
        student_id: Uuid,
    ) -> AppResult<()> {
        // Проверим дедлайн
        let row = sqlx::query!(
            r#"
            SELECT signup_deadline, starts_at
            FROM events
            WHERE id = $1
            "#,
            event_id
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("db error: {e}");
                AppError::internal("database error")
            })?
            .ok_or_else(|| AppError::bad("event not found"))?;

        let now = OffsetDateTime::now_utc();
        if let Some(deadline) = row.signup_deadline {
            if now > deadline {
                return Err(AppError::bad("registration deadline passed"));
            }
        } else if now > row.starts_at {
            return Err(AppError::bad("event already started"));
        }

        // Проверим, что user — студент (FK уже защищает, но вернём понятную ошибку)
        let is_student = sqlx::query_scalar!(
            r#"
            SELECT EXISTS(
               SELECT 1 FROM students s WHERE s.user_id = $1
            )
            "#,
            student_id
        )
            .fetch_one(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("db error: {e}");
                AppError::internal("database error")
            })?
            .unwrap_or(false);

        if !is_student {
            return Err(AppError::bad("student not found"));
        }

        // Вставка (idempotent)
        sqlx::query!(
            r#"
            INSERT INTO event_registrations (event_id, student_id)
            VALUES ($1, $2)
            ON CONFLICT (event_id, student_id) DO NOTHING
            "#,
            event_id,
            student_id
        )
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("db insert error: {e}");
                AppError::internal("database error")
            })?;

        Ok(())
    }

    pub async fn get_events_by_company(&self, company_id: Uuid) -> AppResult<Vec<EventRow>> {
        let rows = sqlx::query_as!(
            EventRow,
            r#"
            SELECT id, company_id, manager_id, title, description, location,
                   starts_at, ends_at, signup_deadline
            FROM events
            WHERE company_id = $1
            ORDER BY starts_at DESC
            "#,
            company_id
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("db error: {e}");
                AppError::internal("database error")
            })?;
        Ok(rows)
    }

    pub async fn get_events_by_student(&self, student_id: Uuid) -> AppResult<Vec<EventRow>> {
        let rows = sqlx::query_as!(
            EventRow,
            r#"
            SELECT e.id, e.company_id, e.manager_id, e.title, e.description, e.location,
                   e.starts_at, e.ends_at, e.signup_deadline
            FROM events e
            JOIN event_registrations er ON er.event_id = e.id
            WHERE er.student_id = $1
            ORDER BY e.starts_at DESC
            "#,
            student_id
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("db error: {e}");
                AppError::internal("database error")
            })?;
        Ok(rows)
    }

    pub async fn get_students_by_event(&self, event_id: Uuid) -> AppResult<Vec<StudentOut>> {
        let rows = sqlx::query!(
            r#"
            SELECT u.id, u.name, u.email
            FROM event_registrations er
            JOIN students s ON s.user_id = er.student_id
            JOIN users u    ON u.id      = s.user_id
            WHERE er.event_id = $1
            ORDER BY u.name
            "#,
            event_id
        )
            .fetch_all(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("db error: {e}");
                AppError::internal("database error")
            })?;

        Ok(rows
            .into_iter()
            .map(|r| StudentOut {
                id: r.id,
                name: r.name,
                email: r.email,
            })
            .collect())
    }

    pub async fn create_event_row(&self, create: CreateEventIn) -> AppResult<EventRow> {
        // Валидация на уровне Rust
        Self::validate_event_input(&create)?;

        let row = EventRow::try_from(create).map_err(|e| AppError::bad(&e.to_string()))?;

        sqlx::query!(
            r#"
            INSERT INTO events (
              id, company_id, manager_id, title, description, location,
              starts_at, ends_at, signup_deadline
            ) VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9)
            "#,
            row.id,
            row.company_id,
            row.manager_id,
            row.title,
            row.description,
            row.location,
            row.starts_at,
            row.ends_at,
            row.signup_deadline
        )
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("db insert error: {e}");
                AppError::internal("database error")
            })?;

        Ok(row)
    }

    pub async fn get_event_dto(&self, event_id: Uuid) -> AppResult<EventDto> {
        let row = sqlx::query!(
            r#"
            SELECT id, company_id, manager_id, title, description, location,
                   starts_at, ends_at, signup_deadline
            FROM events
            WHERE id = $1
            "#,
            event_id
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("db error: {e}");
                AppError::internal("database error")
            })?
            .ok_or_else(|| AppError::bad("event not found"))?;

        Ok(EventDto {
            title: row.title,
            short_description: row.description,
            company_id: row.company_id,
            manager_id: row.manager_id,
            location: row.location,
            start_datetime: row.starts_at,
            end_datetime: row.ends_at,
            registration_deadline: row.signup_deadline,
        })
    }

    pub async fn get_event_by_id(&self, event_id: Uuid) -> AppResult<EventRow> {
        let row = sqlx::query_as!(
            EventRow,
            r#"
            SELECT id, company_id, manager_id, title, description, location,
                   starts_at, ends_at, signup_deadline
            FROM events
            WHERE id = $1
            "#,
            event_id
        )
            .fetch_optional(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("db error: {e}");
                AppError::internal("database error")
            })?
            .ok_or_else(|| AppError::bad("event not found"))?;
        Ok(row)
    }

    pub async fn update_event(&self, id: Uuid, patch: UpdateEventIn) -> AppResult<EventRow> {
        // грузим текущее состояние
        let mut current = self.get_event_by_id(id).await?;
        // применяем патч и валидируем
        current
            .apply_update(patch)
            .map_err(|e| AppError::bad(&e.to_string()))?;

        // сохраняем
        sqlx::query!(
            r#"
            UPDATE events SET
              title = $2,
              description = $3,
              location = $4,
              starts_at = $5,
              ends_at = $6,
              signup_deadline = $7
            WHERE id = $1
            "#,
            current.id,
            current.title,
            current.description,
            current.location,
            current.starts_at,
            current.ends_at,
            current.signup_deadline
        )
            .execute(&self.pool)
            .await
            .map_err(|e| {
                tracing::error!("db update error: {e}");
                AppError::internal("database error")
            })?;

        Ok(current)
    }
}

#[derive(Debug, serde::Serialize)]
pub struct StudentOut {
    pub id: Uuid,
    pub name: String,
    pub email: String,
}

#[derive(Debug, serde::Serialize)]
pub struct EventDto {
    pub title: String,
    pub short_description: Option<String>,
    pub company_id: Uuid,
    pub manager_id: Uuid,
    pub location: Option<String>,
    pub start_datetime: OffsetDateTime,
    pub end_datetime: Option<OffsetDateTime>,
    pub registration_deadline: Option<OffsetDateTime>,
}
