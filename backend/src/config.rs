use std::env;

#[derive(Clone, Debug)]
pub struct Config {
    pub host: String,
    pub port: u16,

    pub database_url: String,
    pub telegram_code_ttl: i64,

    pub refresh_token_ttl_days: i64,

    pub debug_expose_jwt: bool,
    pub jwt_secret: String,

    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
}

impl Config {
    pub fn from_env() -> Self {
        let _ = dotenvy::dotenv();

        let host = env::var("APP_HOST")
            .or_else(|_| env::var("HOST"))
            .unwrap_or_else(|_| "127.0.0.1".into());

        let port = env::var("APP_PORT")
            .or_else(|_| env::var("PORT"))
            .ok()
            .and_then(|s| s.parse::<u16>().ok())
            .unwrap_or(8080);

        let database_url = env::var("DATABASE_URL")
            .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/tsu_hits_events".into());

        let telegram_code_ttl = env::var("TELEGRAM_CODE_TTL_MINUTES")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(10);

        let debug_expose_jwt = std::env::var("APP_DEBUG_EXPOSE_JWT")
            .ok()
            .map(|s| matches!(s.as_str(), "1" | "true" | "yes" | "on"))
            .unwrap_or(false);
        let jwt_secret = env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".into());
        let refresh_token_ttl_days = env::var("REFRESH_TOKEN_TTL_DAYS")
            .ok()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(30);

        let google_client_id = env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
        let google_client_secret = env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default();
        let google_redirect_uri = env::var("GOOGLE_REDIRECT_URI").unwrap_or_default();


        Self {
            host,
            port,
            database_url,
            telegram_code_ttl,
            jwt_secret,
            debug_expose_jwt,
            google_client_id,
            google_client_secret,
            google_redirect_uri,
            refresh_token_ttl_days,
        }
    }
}