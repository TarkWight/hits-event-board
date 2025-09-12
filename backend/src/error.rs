use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use crate::infra::errors::RepoError;
use crate::domain::entities::company::CompanyValidationError;

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Not implemented")]
    NotImplemented,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden")]
    Forbidden,

    #[error("Not found")]
    NotFound,

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unprocessable entity: {0}")]
    Unprocessable(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Precondition failed: {0}")]
    PreconditionFailed(String),

    #[error("Internal server error: {0}")]
    Internal(String),
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
        use ApiError::*;
        let (code, msg, http) = match &self {
            NotImplemented               => ("NOT_IMPLEMENTED",      "Not implemented",               StatusCode::NOT_IMPLEMENTED),
            Unauthorized                 => ("UNAUTHORIZED",         "Unauthorized",                  StatusCode::UNAUTHORIZED),
            Forbidden                    => ("FORBIDDEN",            "Forbidden",                     StatusCode::FORBIDDEN),
            NotFound                     => ("NOT_FOUND",            "Not found",                     StatusCode::NOT_FOUND),
            BadRequest(m)                => ("BAD_REQUEST",          m.as_str(),                      StatusCode::BAD_REQUEST),
            Unprocessable(m)             => ("UNPROCESSABLE_ENTITY", m.as_str(),                      StatusCode::UNPROCESSABLE_ENTITY),
            Conflict(m)                  => ("CONFLICT",             m.as_str(),                      StatusCode::CONFLICT),
            PreconditionFailed(m)        => ("PRECONDITION_FAILED",  m.as_str(),                      StatusCode::PRECONDITION_FAILED),
            Internal(m)                  => ("INTERNAL",             m.as_str(),                      StatusCode::INTERNAL_SERVER_ERROR),
        };
        let body = ErrorBody { error: ErrorContent { code, message: msg } };
        (http, Json(body)).into_response()
    }
}

impl From<CompanyValidationError> for ApiError {
    fn from(e: CompanyValidationError) -> Self {
        ApiError::Unprocessable(e.to_string())
    }
}

impl From<RepoError> for ApiError {
    fn from(e: RepoError) -> Self {
        match e {
            RepoError::NotFound => ApiError::NotFound,
            RepoError::Conflict(msg) => ApiError::Conflict(msg),
            RepoError::Precondition(m) => ApiError::PreconditionFailed(m),
            RepoError::Db(err) => ApiError::Internal(err.to_string()),
        }
    }
}