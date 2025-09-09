use thiserror::Error;

#[derive(Debug, Error)]
pub enum RepoError {
    #[error("not found")]
    NotFound,
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("db: {0}")]
    Db(String),
    #[error("precondition failed: {0}")]
    Precondition(String),
}

impl From<sqlx::Error> for RepoError {
    fn from(e: sqlx::Error) -> Self { Self::Db(e.to_string()) }
}

pub type RepoResult<T> = Result<T, RepoError>;

pub fn is_unique_violation(e: &sqlx::Error) -> Option<String> {
    if let sqlx::Error::Database(db) = e {
        if db.code().as_deref() == Some("23505") {
            return db.constraint().map(|s| s.to_string());
        }
    }
    None
}