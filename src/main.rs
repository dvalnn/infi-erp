#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    dotenv::dotenv().ok();

    let database_url = match std::env::var("DATABASE_URL") {
        Ok(url) => url,
        Err(e) => return Err(anyhow::anyhow!(e)),
    };

    const UDP_ADDR: &str = "localhost:24680";
    const BUFFER_SIZE: usize = 10024;

    let app = infi_erp::AppBuilder::new(database_url)
        .with_udp_listener(UDP_ADDR, BUFFER_SIZE)
        .with_tracing_level(tracing::Level::DEBUG)
        .build()
        .await?;

    app.run().await?;

    Ok(())
}
