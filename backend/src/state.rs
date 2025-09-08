use std::sync::Arc;
use sqlx::{Pool, Postgres};
use crate::infra::db;
use crate::config::Config;
use crate::domain::services::{company_service::CompanyService, event_service::EventService};
use crate::infra::repositories::{company_repo::PgCompanyRepository, event_repo::PgEventRepository};

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<Pool<Postgres>>,
    pub config: Arc<Config>,

    pub companies: Arc<CompanyService<PgCompanyRepository>>,
    pub events: Arc<EventService<PgEventRepository>>,
}

impl AppState {
    pub async fn init_with(config: Config) -> anyhow::Result<Self> {
        let db = db::init_pool(&config.database_url).await?;
        let db = Arc::new(db);
        let config = Arc::new(config);

        let companies = Arc::new(CompanyService::new(PgCompanyRepository::new(db.clone())));
        let events = Arc::new(EventService::new(PgEventRepository::new(db.clone())));

        Ok(Self { db, config, companies, events })
    }

    // сахар: собрать конфиг из env и инициализироваться
    pub async fn init() -> anyhow::Result<Self> {
        let cfg = Config::from_env();
        Self::init_with(cfg).await
    }
}
