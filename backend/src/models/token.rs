use serde::Serialize;
use time::OffsetDateTime;

#[derive(Debug, Serialize)]
pub struct TokenDTO {
    pub access_token: String,
    #[serde(with = "time::serde::rfc3339")]
    pub access_token_expiration: OffsetDateTime,
    pub refresh_token: String,
    #[serde(with = "time::serde::rfc3339")]
    pub refresh_token_expiration: OffsetDateTime,
}