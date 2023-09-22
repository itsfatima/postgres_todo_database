use sqlx::postgres::PgPool;
use sqlx::Error;

pub async fn init_pool() -> Result<PgPool, Error> {
    let database_url = "postgres://postgres:03075@localhost:5432/postgres";
    PgPool::connect(&database_url).await
}