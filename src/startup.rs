use actix_web::{web::Data, HttpServer};
use anyhow::anyhow;
use sqlx::PgPool;
use tokio::net::UdpSocket;
use tracing::Level;

use crate::{routes, scheduler::Scheduler, udp_listener::Listener};

pub struct AppBuilder {
    tracing_level: Level,
    database_url: String,
    udp_addr: Option<String>,
    udp_buffer_size: Option<usize>,
    http_addr: Option<String>,
}

impl AppBuilder {
    pub fn new(database_url: String) -> Self {
        Self {
            tracing_level: Level::ERROR,
            database_url,
            udp_addr: None,
            udp_buffer_size: None,
            http_addr: None,
        }
    }

    pub fn with_udp_listener(mut self, port: u16, buffer_size: usize) -> Self {
        self.udp_addr = Some(format!("127.0.0.1:{}", port));
        self.udp_buffer_size = Some(buffer_size);
        self
    }

    pub fn with_web_server(mut self, http_host: &str, http_port: u16) -> Self {
        self.http_addr = Some(format!("{}:{}", http_host, http_port));
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
            (self.udp_addr, self.udp_buffer_size)
        {
            let socket = UdpSocket::bind(&address).await?;
            let listener = Listener::new(pool.clone(), socket, buffer_size);
            Some(listener)
        } else {
            None
        };

        let scheduler = Scheduler::new(pool.clone(), notification_listener);

        Ok(App {
            web_addr: self.http_addr,
            pool,
            udp_listener,
            scheduler,
        })
    }
}

pub struct App {
    udp_listener: Option<Listener>,
    web_addr: Option<String>,
    pool: PgPool,
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

        tokio::spawn(async move { self.scheduler.run().await });

        if let Some(addr) = self.web_addr {
            let server = match HttpServer::new(move || {
                actix_web::App::new()
                    .wrap(actix_web::middleware::Logger::default())
                    .service(routes::check_health)
                    .service(routes::get_date)
                    .service(routes::post_date)
                    .service(routes::get_production)
                    .service(routes::post_transformation_completion)
                    .service(routes::post_warehouse_action)
                    .service(routes::get_expected_shipments)
                    .service(routes::post_material_arrival)
                    .service(routes::get_deliveries)
                    .app_data(Data::new(self.pool.clone()))
            })
            .bind(addr.clone())
            {
                Ok(s) => {
                    tracing::info!("actix-web listening on: {addr}");
                    s
                }
                Err(e) => {
                    tracing::error!("{e}");
                    anyhow::bail!("Error binding to address: {addr}")
                }
            };

            if let Err(e) = server.run().await {
                tracing::error!("{e}");
                anyhow::bail!("Error running web server: {e}");
            }
        }

        Ok(())
    }
}
