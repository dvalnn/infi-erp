use infi_erp::AppBuilder;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let settings = infi_erp::get_configuration()?;

    let app = AppBuilder::new(settings.database.connection_string())
        .with_udp_listener(
            settings.application.udp_port,
            settings.application.udp_buffer_size,
        )
        .with_web_server(
            settings.application.http_host.as_str(),
            settings.application.http_port,
        )
        .with_tracing_level(tracing::Level::INFO)
        .build()
        .await?;

    if let Err(e) = app.run().await {
        tracing::error!("{:?}", e)
    }

    Ok(())
}
