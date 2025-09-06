use sqlx::{Pool, Postgres};

pub type PgPool = Pool<Postgres>;

pub async fn make_pool(database_url: &str) -> Result<PgPool, sqlx::Error> {
    sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .connect(database_url)
        .await
}
