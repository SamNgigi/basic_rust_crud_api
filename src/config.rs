use sqlx::{
    PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};

use std::env;

#[derive(Clone, Debug)]
pub struct AppConfig {
    pub app_host: String,
    pub app_port: u16,
    pub db: DatabaseConfig,
}

#[derive(Clone, Debug)]
pub struct DatabaseConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
    pub user: String,
    pub password: String,
    pub max_connections: u32,
}

impl AppConfig {
    pub fn from_env() -> Self {
        Self {
            app_host: env::var("APP_HOST").unwrap_or_else(|_| "0.0.0.0".to_string()),
            app_port: env::var("APP_PORT")
                .unwrap_or_else(|_| "8000".to_string())
                .parse()
                .expect("APP_PORT must be valid u16"),
            db: DatabaseConfig::from_env(),
        }
    }
}

impl DatabaseConfig {
    pub fn from_env() -> Self {
        Self {
            host: env::var("DB_HOST").unwrap_or_else(|_| "postgres".to_string()),
            port: env::var("DB_PORT")
                .unwrap_or_else(|_| "5432".to_string())
                .parse()
                .expect("DB_PORT must be a valid u16"),
            name: env::var("DB_NAME").expect("DB_NAME is required"),
            user: env::var("DB_USER").expect("DB_USER is required"),
            password: env::var("DB_PASSWORD").expect("DB_PASSWORD is required"),
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .expect("DB_MAX_CONNECTIONS must be valid u32"),
        }
    }
}

pub async fn create_pool(cfg: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
    let options = PgConnectOptions::new()
        .host(&cfg.host)
        .port(cfg.port)
        .database(&cfg.name)
        .username(&cfg.user)
        .password(&cfg.password);

    PgPoolOptions::new()
        .max_connections(cfg.max_connections)
        .connect_with(options)
        .await
}
