use axum::response::IntoResponse;
use axum::response::Response;
use axum::Json;
use http::StatusCode;
use serde::Serialize;

#[cfg_attr(debug_assertions, allow(dead_code))]
#[derive(Debug)]
pub enum AppError {
    BadRequest(String),
    NotFound(String),
    Internal(String),
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    details: Option<String>,
}

impl AppError {
    #[inline]
    pub fn bad<T: Into<String>>(msg: T) -> Self {
        AppError::BadRequest(msg.into())
    }
    #[inline]
    #[cfg_attr(debug_assertions, allow(dead_code))]
    pub fn not_found<T: Into<String>>(msg: T) -> Self {
        AppError::NotFound(msg.into())
    }
    #[inline]
    pub fn internal<T: Into<String>>(msg: T) -> Self {
        AppError::Internal(msg.into())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                Json(ErrorBody {
                    error: "bad_request",
                    details: Some(msg),
                }),
            )
                .into_response(),
            AppError::NotFound(msg) => (
                StatusCode::NOT_FOUND,
                Json(ErrorBody {
                    error: "not_found",
                    details: Some(msg),
                }),
            )
                .into_response(),
            AppError::Internal(msg) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ErrorBody {
                    error: "internal",
                    details: Some(msg),
                }),
            )
                .into_response(),
        }
    }
}

pub type AppResult<T> = Result<T, AppError>;

impl From<axum::extract::rejection::QueryRejection> for AppError {
    fn from(e: axum::extract::rejection::QueryRejection) -> Self {
        tracing::warn!("query rejection: {e}");
        AppError::BadRequest("invalid query parameters".into())
    }
}
