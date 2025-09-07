use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

pub async fn init_pool() -> anyhow::Result<Pool<Postgres>> {
    let url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/tsu_hits_events".into());
    let pool = PgPoolOptions::new().max_connections(10).connect(&url).await?;
    Ok(pool)
}
