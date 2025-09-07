use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("db: {0}")]
    Db(String),
}

impl From<sqlx::Error> for RepoError {
    fn from(e: sqlx::Error) -> Self { Self::Db(e.to_string()) }
}

pub type RepoResult<T> = Result<T, RepoError>;
