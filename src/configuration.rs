use std::time::{SystemTime, UNIX_EPOCH};

use config::Config;
use sqlx::{migrate, Connection, PgPool};

pub fn get_configuration() -> Result<Settings, config::ConfigError> {
    let settings = Config::builder()
        .add_source(config::File::new(
            "configuration.yml",
            config::FileFormat::Yaml,
        ))
        .build()?;

    settings.try_deserialize()
}

#[derive(serde::Deserialize)]
pub struct Settings {
    pub application: ApplicationSettings,
    pub database: DatabaseSettings,
}

#[derive(serde::Deserialize)]
pub struct ApplicationSettings {
    pub udp_port: u16,
    pub udp_buffer_size: usize,
    pub http_port: u16,
    pub http_host: String,
}

#[derive(serde::Deserialize)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: String,
    pub port: u16,
    pub host: String,
    pub database_name: String,
}

impl DatabaseSettings {
    pub fn connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password,
            self.host,
            self.port,
            self.database_name
        )
    }

    pub fn connection_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username, self.password, self.host, self.port
        )
    }

    pub async fn create_test_db(self) -> PgPool {
        let mut connection =
            sqlx::PgConnection::connect(&self.connection_string_without_db())
                .await
                .expect("Failed to connect to the database");

        let db_name = format!(
            "test_{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_micros()
        );

        sqlx::query(format!(r#"CREATE DATABASE "{}";"#, db_name).as_str())
            .execute(&mut connection)
            .await
            .expect("Failed to create test database");

        let new_db_url =
            format!("{}/{}", self.connection_string_without_db(), db_name);

        let pool = PgPool::connect_lazy(&new_db_url)
            .expect("Failed to connect to the test database");

        migrate!("./migrations")
            .run(&pool)
            .await
            .expect("Failed to run migrations");

        pool
    }
}
