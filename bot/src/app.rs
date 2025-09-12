use std::sync::Arc;
use reqwest::Client;

#[derive(Clone)]
pub struct App {
    pub http: Client,
    pub base_url: String,
    pub ping_url: String,
}

impl App {
    pub fn from_env() -> Self {
        dotenvy::dotenv().ok();
        let base_url = std::env::var("BACKEND_BASE_URL")
            .unwrap_or_else(|_| "http://127.0.0.1:8080".into());
        let ping_url = std::env::var("BACKEND_PING_URL")
            .unwrap_or_else(|_| format!("{base_url}/health"));
        Self {
            http: Client::new(),
            base_url,
            ping_url,
        }
    }

    pub fn shared() -> Arc<Self> {
        Arc::new(Self::from_env())
    }
}