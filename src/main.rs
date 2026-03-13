mod config;

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

use config::{AppConfig, create_pool};

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

async fn root() -> &'static str {
    "Welcome to Basic Crud User Management"
}
