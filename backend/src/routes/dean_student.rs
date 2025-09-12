// src/api/dean_students.rs
use axum::{
    extract::{Query, Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    auth::extractor::AuthUser,
    auth::roles::{UserRole, StudentStatus},
    error::{ApiError, ApiResult},
    state::AppState,
};

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/v1/dean/students", get(list_pending_students))
        .route("/api/v1/dean/students/:id/approve", post(approve_student))
        .route("/api/v1/dean/students/:id/reject",  post(reject_student))
        .with_state(state)
}

#[derive(Deserialize)]
struct ListQ {
    status: Option<String>,
    page:   Option<i32>,
    limit:  Option<i32>,
    q:      Option<String>, 
}

#[derive(serde::Serialize)]
#[serde(rename_all = "snake_case")]
struct StudentAdminOut {
    id: Uuid,
    name: String,
    email: String,
    status: String, // "created" | "linked" | "confirmed" | "rejected"
}

async fn list_pending_students(
    State(st): State<AppState>,
    user: AuthUser,
    Query(q): Query<ListQ>,
) -> ApiResult<Json<Vec<StudentAdminOut>>> {
    // только декан
    if user.role != UserRole::Dean {
        return Err(ApiError::Forbidden);
    }

    // разберём статус(ы)
    let statuses: Vec<StudentStatus> = match q.status.as_deref() {
        Some("created") => vec![StudentStatus::Created],
        Some("linked")  => vec![StudentStatus::Linked],
        Some("confirmed") => vec![StudentStatus::Confirmed],
        Some("rejected")  => vec![StudentStatus::Rejected],
        Some(_) => return Err(ApiError::Unprocessable("unknown status".into())),
        None => vec![StudentStatus::Created, StudentStatus::Linked], // по умолчанию «ожидающие»
    };

    let page  = q.page.unwrap_or(1);
    let limit = q.limit.unwrap_or(50);
    let search = q.q.unwrap_or_default();

    let rows = st.users.list_students_by_status(&statuses, page, limit, &search).await?;
    let out = rows.into_iter().map(|(u, st)| StudentAdminOut {
        id: u.id,
        name: u.name,
        email: u.email,
        status: match st {
            Some(StudentStatus::Created)   => "created".into(),
            Some(StudentStatus::Linked)    => "linked".into(),
            Some(StudentStatus::Confirmed) => "confirmed".into(),
            Some(StudentStatus::Rejected)  => "rejected".into(),
            None => "created".into(),
        }
    }).collect();

    Ok(Json(out))
}

async fn approve_student(
    State(st): State<AppState>,
    user: AuthUser,
    Path(student_user_id): Path<Uuid>,
) -> ApiResult<()> {
    if user.role != UserRole::Dean {
        return Err(ApiError::Forbidden);
    }
    st.users.set_student_status(student_user_id, StudentStatus::Confirmed).await?;
    Ok(())
}

async fn reject_student(
    State(st): State<AppState>,
    user: AuthUser,
    Path(student_user_id): Path<Uuid>,
) -> ApiResult<()> {
    if user.role != UserRole::Dean {
        return Err(ApiError::Forbidden);
    }
    st.users.set_student_status(student_user_id, StudentStatus::Rejected).await?;
    Ok(())
}