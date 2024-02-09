use anyhow::anyhow;
use sqlx::error::BoxDynError;

#[tokio::main]
async fn main() -> Result<(), BoxDynError> {
    let url = "postgres://admin:admin@localhost:5432/infi-postgres";
    //NOTE: connection pool instead of single connection for better performance
    let pool = sqlx::postgres::PgPool::connect(url).await?;

    sqlx::migrate!("./migrations").run(&pool).await?;

    let res = sqlx::query!("SELECT 1 + 1 as sum").fetch_one(&pool).await?;

    println!("{res:?}");
    println!("sum: {}", res.sum.ok_or(anyhow!("sum not here"))?);

    Ok(())
}
