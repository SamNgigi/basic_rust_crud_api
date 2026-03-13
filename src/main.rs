#[allow(unused_imports)]
use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use serde::{Deserialize, Serialize};

#[allow(unused_imports)]
use sqlx::{
    FromRow, PgPool,
    postgres::{PgConnectOptions, PgPoolOptions},
};
use uuid::Uuid;

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

#[allow(dead_code)]
#[derive(Deserialize)]
struct UserPayload {
    name: String,
    email: String,
}

#[allow(dead_code)]
#[derive(Serialize)]
struct User {
    uuid: Uuid,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() {
    let config = AppConfig::from_env();
    let pool = create_pool(&config.db)
        .await
        .expect("❌ Failed to Connect to Postgres");
    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Migration Failed!");

    let app = Router::new().route("/", get(root)).with_state(pool);

    let addr = format!("{}:{}", &config.app_host, &config.app_port);
    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    println!("🚀 Server running on port 8000");
    axum::serve(listener, app).await.unwrap();
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
    fn from_env() -> Self {
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

async fn create_pool(cfg: &DatabaseConfig) -> Result<PgPool, sqlx::Error> {
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

async fn root() -> &'static str {
    "Welcome to Basic Crud User Management"
}
