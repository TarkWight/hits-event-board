use sqlx::{Pool, Postgres};
use std::sync::Arc;

use crate::config::Config;
use crate::infra::db;

use crate::infra::repositories::{
    company::PgCompanyRepository,
    event_repo::PgEventRepository,
};
use crate::services::{
    company_service::CompanyService,
    event_service::EventService,
};

use crate::auth::extractor::AuthState;
use crate::infra::security::jwt::{TokenConfig, TokenService};

use crate::services::telegram_service::TelegramService;
use crate::infra::repositories::telegram_repo::PgTelegramLinkRepository;

use crate::infra::repositories::user_repo::PgUserRepository;
use crate::services::auth_service::AuthService;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Postgres>,
    pub config: Arc<Config>,

    pub companies: CompanyService<PgCompanyRepository>,
    pub events:    EventService<PgEventRepository>,

    pub telegram:  TelegramService<PgTelegramLinkRepository>,

    pub auth:      AuthState,

    pub auth_service: AuthService<PgUserRepository>,
}

impl AppState {
    pub async fn init_with(config: Config) -> anyhow::Result<Self> {
        let db = db::init_pool(&config.database_url).await?;

        let companies = CompanyService::new(PgCompanyRepository::new(db.clone()));
        let events    = EventService::new(PgEventRepository::new(db.clone()));

        let token_service = TokenService::new(TokenConfig::from_env());
        let auth = AuthState { token_service };

        let telegram = TelegramService::new(PgTelegramLinkRepository::new(db.clone()));

        let auth_service = AuthService::new(PgUserRepository::new(db.clone()));

        Ok(Self {
            db,
            config: Arc::new(config),
            companies,
            events,
            auth,
            telegram,
            auth_service,
        })
    }

    pub async fn init() -> anyhow::Result<Self> {
        let cfg = Config::from_env();
        Self::init_with(cfg).await
    }
}