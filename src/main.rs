mod db_api;
mod scheduler;
mod udp_listener;

use anyhow::anyhow;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(e) => return Err(anyhow!(e)),
    };

    let pool = sqlx::PgPool::connect_lazy(&database_url)?;

    if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
        tracing::error!("Error running migrations: {e}");
        return Err(anyhow!(e));
    }

    let notification_listener = sqlx::postgres::PgListener::connect(&database_url).await?;

    tracing::info!("DB initialization successfull.");

    let socket = tokio::net::UdpSocket::bind("127.0.0.1:24680").await?;
    tracing::info!("udp_listener on port 24680");

    const BUF_SIZE: usize = 10024;
    let listener = udp_listener::Listener::new(pool.clone(), socket, BUF_SIZE);

    tokio::spawn(async move { listener.listen().await });
    // listener.listen().await?;
    let scheduler = scheduler::Scheduler::new(pool, notification_listener);
    scheduler.run().await?;

    Ok(())
}
