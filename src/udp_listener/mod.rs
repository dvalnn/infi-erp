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
        loop {
            let (len, addr) = self.socket.recv_from(&mut self.buffer).await?;
            let message = std::str::from_utf8(&self.buffer[..len])?;
            tracing::info!("Received message from {}: {}", addr, message);

            let (_, orders) = match parser::parse_many(message) {
                Ok(o) => o,
                Err(e) => {
                    tracing::error!("{e} while parsing orders");
                    continue;
                }
            };

            let pool = self.pool.clone();

            tokio::spawn(async move {
                let orders =
                    orders.iter().fold(Vec::new(), |mut acc, order| {
                        acc.push(order.insert_to_db(&pool));
                        acc
                    });

                let n_orders = orders.len();

                match futures::future::try_join_all(orders).await {
                    Ok(_) => tracing::info!("Placed {} orders", n_orders),
                    Err(e) => tracing::error!("{e} while placing new orders"),
                };
            });
        }
    }
}
