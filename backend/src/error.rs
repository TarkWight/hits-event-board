use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde::Serialize;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Not implemented")]
    NotImplemented,
    #[error("Unauthorized")]
    Unauthorized,
    #[error("Forbidden")]
    Forbidden,
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Unprocessable entity: {0}")]
    Unprocessable(String),
    #[error("Conflict: {0}")]
    Conflict(String),
    #[error("Internal server error")]
    Internal,
}

#[derive(Serialize)]
struct ErrorBody<'a> {
    error: ErrorContent<'a>,
}

#[derive(Serialize)]
struct ErrorContent<'a> {
    code: &'a str,
    message: &'a str,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (code, msg, http) = match &self {
            ApiError::NotImplemented => ("NOT_IMPLEMENTED", "Not implemented", StatusCode::NOT_IMPLEMENTED),
            ApiError::Unauthorized => ("UNAUTHORIZED", "Unauthorized", StatusCode::UNAUTHORIZED),
            ApiError::Forbidden => ("FORBIDDEN", "Forbidden", StatusCode::FORBIDDEN),
            ApiError::BadRequest(m) => ("BAD_REQUEST", m.as_str(), StatusCode::BAD_REQUEST),
            ApiError::Unprocessable(m) => ("UNPROCESSABLE", m.as_str(), StatusCode::UNPROCESSABLE_ENTITY),
            ApiError::Conflict(m) => ("CONFLICT", m.as_str(), StatusCode::CONFLICT),
            ApiError::Internal => ("INTERNAL", "Internal server error", StatusCode::INTERNAL_SERVER_ERROR),
        };
        let body = ErrorBody { error: ErrorContent { code, message: msg } };
        (http, Json(body)).into_response()
    }
}
