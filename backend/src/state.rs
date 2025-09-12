use sqlx::{Pool, Postgres};
use std::sync::Arc;

use crate::config::Config;
use crate::infra::db;

use crate::infra::repositories::{
    company::PgCompanyRepository,
    event_repo::PgEventRepository,
    user_repo::PgUserRepository,
    telegram_repo::PgTelegramLinkRepository,
    telegram_code_repo::PgTelegramCodeRepository,
    manager_repo::PgManagerRepository,
};

use crate::services::{
    company_service::CompanyService,
    event_service::EventService,
    auth_service::AuthService,
    telegram_service::TelegramService,
    manager_service::ManagerService,
    user_service::UsersService,
};

use crate::auth::extractor::AuthState;
use crate::infra::security::jwt::{TokenConfig, TokenService};
use crate::services::registration_service::RegistrationService;

#[derive(Clone)]
pub struct AppState {
    pub db: Pool<Postgres>,
    pub config: Arc<Config>,

    pub companies: CompanyService<PgCompanyRepository>,
    pub events:    EventService<PgEventRepository>,
    pub managers:  ManagerService<PgManagerRepository>,
    pub users:     UsersService<PgUserRepository>,

    pub telegram:  TelegramService<PgTelegramLinkRepository, PgTelegramCodeRepository>,

    pub auth:         AuthState,
    pub auth_service: AuthService<PgUserRepository, PgTelegramLinkRepository>,
}

impl AppState {
    pub async fn init_with(config: Config) -> anyhow::Result<Self> {
        let db = db::init_pool(&config.database_url).await?;

        let companies_repo = PgCompanyRepository::new(db.clone());
        let events_repo    = PgEventRepository::new(db.clone());
        let users_repo     = PgUserRepository::new(db.clone());
        let tg_links       = PgTelegramLinkRepository::new(db.clone());
        let tg_codes       = PgTelegramCodeRepository::new(db.clone());
        let managers_repo  = PgManagerRepository::new(db.clone());

        // let registration = RegistrationService::new(registration_repo);
        let companies = CompanyService::new(companies_repo);
        let events    = EventService::new(events_repo);
        let managers  = ManagerService::new(managers_repo);
        let users     = UsersService::new(users_repo.clone());

        let token_service = TokenService::new(TokenConfig::from_env());
        let auth          = AuthState { token_service: token_service.clone() };

        let telegram = TelegramService::new(tg_links.clone(), tg_codes, config.telegram_code_ttl);

        let auth_service = AuthService::new(users_repo, token_service, tg_links);

        Ok(Self {
            db,
            config: Arc::new(config),
            companies,
            managers,
            events,
            users,
            telegram,
            auth,
            auth_service,
        })
    }

    pub async fn init() -> anyhow::Result<Self> {
        let cfg = Config::from_env();
        Self::init_with(cfg).await
    }
}