mod parser;

pub struct Listener {
    pool: sqlx::PgPool,
    socket: tokio::net::UdpSocket,
    buffer: Vec<u8>,
}

impl Listener {
    pub fn new(
        pool: sqlx::PgPool,
        socket: tokio::net::UdpSocket,
        buf_size: usize,
    ) -> Self {
        Self {
            pool,
            socket,
            buffer: vec![0; buf_size],
        }
    }

    pub async fn listen(mut self) -> anyhow::Result<()> {
        tracing::info!(
            "Listening for UDP messages on {}",
            self.socket.local_addr()?
        );
        loop {
            let (len, addr) = self.socket.recv_from(&mut self.buffer).await?;
            let message = std::str::from_utf8(&self.buffer[..len])?;
            tracing::info!("Received message from {}: {}", addr, message);

            let (_, orders) = match parser::parse_command(message) {
                Ok(o) => o,
                Err(e) => {
                    tracing::error!("{e} while parsing orders");
                    continue;
                }
            };

            let pool = self.pool.clone();
            tokio::spawn(async move {
                for order in orders.into_iter() {
                    match order.insert_to_db(&pool).await {
                        Ok(id) => tracing::info!("Inserted order id: {}", id),
                        Err(e) => tracing::error!("{:?}", e),
                    }
                }
            });
        }
    }
}
