// use std::env;
//
// #[derive(Debug, Clone)]
// pub struct Config {
//     pub port: u16,
//     pub database_url: Option<String>,
// }
//
// impl Config {
//     pub fn from_env() -> Self {
//         let _ = dotenvy::dotenv();
//
//         let port = env::var("PORT")
//             .ok()
//             .and_then(|s| s.parse::<u16>().ok())
//             .unwrap_or(8080);
//
//         let database_url = env::var("DATABASE_URL").ok();
//
//         Self { port, database_url }
//     }
// }
//

#[derive(Clone)]
pub struct Config {
    pub jwt_secret: String,
    pub google_client_id: String,
    pub google_client_secret: String,
    pub google_redirect_uri: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".into()),
            google_client_id: std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default(),
            google_client_secret: std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default(),
            google_redirect_uri: std::env::var("GOOGLE_REDIRECT_URI").unwrap_or_default(),
        }
    }
}
