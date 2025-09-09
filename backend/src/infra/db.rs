use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

pub async fn init_pool(database_url: &str) -> anyhow::Result<Pool<Postgres>> {
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;
    Ok(pool)
}
