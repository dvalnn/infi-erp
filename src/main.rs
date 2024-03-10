use infi_erp::AppBuilder;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let settings = infi_erp::get_configuration()?;

    let app = AppBuilder::new(settings.database.connection_string())
        .with_udp_listener(
            settings.application.udp_port,
            settings.application.udp_buffer_size,
        )
        .with_tracing_level(tracing::Level::DEBUG)
        .build()
        .await?;

    app.run().await?;

    Ok(())
}
