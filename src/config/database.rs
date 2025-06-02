use anyhow::Result;
use sqlx::{MySqlPool, mysql::MySqlPoolOptions};

pub async fn create_connection_pool(database_url: &str) -> Result<MySqlPool> {
    let pool = MySqlPoolOptions::new()
        .max_connections(10)
        .connect(database_url)
        .await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    Ok(pool)
}
