use thiserror::Error;

pub type RepoResult<T> = Result<T, RepoError>;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("not found")]
    NotFound,

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("precondition failed: {0}")]
    Precondition(String),

    #[error(transparent)]
    Db(#[from] sqlx::Error),
}

pub fn is_unique_violation(e: &sqlx::Error) -> Option<String> {
    match e {
        sqlx::Error::Database(db) if db.kind() == sqlx::error::ErrorKind::UniqueViolation => {
            db.constraint().map(|s| s.to_string())
        }
        _ => None,
    }
}