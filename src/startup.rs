use anyhow::anyhow;
use tokio::net::UdpSocket;
use tracing::Level;

use crate::{scheduler::Scheduler, udp_listener::Listener};

pub struct AppBuilder {
    tracing_level: Level,
    database_url: String,
    udp_address: Option<String>,
    udp_buffer_size: Option<usize>,
}

impl AppBuilder {
    pub fn new(database_url: String) -> Self {
        Self {
            tracing_level: Level::ERROR,
            database_url,
            udp_address: None,
            udp_buffer_size: None,
        }
    }

    pub fn with_udp_listener(
        mut self,
        address: impl Into<String>,
        buffer_size: usize,
    ) -> Self {
        self.udp_address = Some(address.into());
        self.udp_buffer_size = Some(buffer_size);
        self
    }

    pub fn with_tracing_level(mut self, level: Level) -> Self {
        self.tracing_level = level;
        self
    }

    pub async fn build(self) -> anyhow::Result<App> {
        tracing_subscriber::fmt()
            .with_max_level(self.tracing_level)
            .init();

        tracing::info!("Initializing DB connection...");

        let pool = sqlx::PgPool::connect_lazy(&self.database_url)?;
        let notification_listener =
            sqlx::postgres::PgListener::connect(&self.database_url).await?;
        if let Err(e) = sqlx::migrate!("./migrations").run(&pool).await {
            tracing::error!("Error running migrations: {e}");
            return Err(anyhow!(e));
        }

        tracing::info!("DB initialization successfull.");

        let udp_listener = if let (Some(address), Some(buffer_size)) =
            (self.udp_address, self.udp_buffer_size)
        {
            let socket = UdpSocket::bind(&address).await?;
            let listener = Listener::new(pool.clone(), socket, buffer_size);
            Some(listener)
        } else {
            None
        };

        let scheduler = Scheduler::new(pool.clone(), notification_listener);

        Ok(App {
            udp_listener,
            scheduler,
        })
    }
}

pub struct App {
    udp_listener: Option<Listener>,
    scheduler: Scheduler,
}

impl App {
    pub async fn run(self) -> anyhow::Result<()> {
        if let Some(listener) = self.udp_listener {
            tokio::spawn(async move {
                if let Err(e) = listener.listen().await {
                    tracing::error!("{e}");
                }
            });
        }
        self.scheduler.run().await
    }
}
