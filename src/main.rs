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

#[derive(Deserialize)]
struct UserPayload {
    name: String,
    email: String,
}

#[derive(Serialize, FromRow)]
struct User {
    id: i64,
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

    let app = Router::new()
        .route("/", get(root))
        .route("/users", post(create_user).get(list_users))
        .route(
            "/users/{id}",
            get(get_user_by_id).put(update_user).delete(delete_user),
        )
        .with_state(pool);

    let addr = format!("{}:{}", &config.app_host, &config.app_port);
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to {addr}");
    println!("🚀 Server running on port 8000");
    axum::serve(listener, app).await.unwrap();
}

// -----------------------------------------------------------------
// ENDPOINT HANDLERS
// -----------------------------------------------------------------

// Test Endpoint
async fn root() -> &'static str {
    "Welcome to Basic Crud User Management"
}

// CREATE USER
async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<UserPayload>,
) -> Result<(StatusCode, Json<User>), StatusCode> {
    sqlx::query_as::<_, User>("INSERT INTO users (name, email) VALUES ($1, $2) RETURNING *")
        .bind(payload.name)
        .bind(payload.email)
        .fetch_one(&pool)
        .await
        .map(|u| (StatusCode::CREATED, Json(u)))
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// GET ALL USERS
async fn list_users(State(pool): State<PgPool>) -> Result<Json<Vec<User>>, StatusCode> {
    sqlx::query_as::<_, User>("SELECT * FROM users")
        .fetch_all(&pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// GET USER BY ID
async fn get_user_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<Json<User>, StatusCode> {
    sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await
        .map(Json)
        .map_err(|_| StatusCode::NOT_FOUND)
}

// UPDATE USER
async fn update_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(payload): Json<UserPayload>,
) -> Result<Json<User>, StatusCode> {
    sqlx::query_as::<_, User>(
        "UPDATE users SET name = $1, email = $2 WHERE id = $3 RETURNING id, name, email",
    )
    .bind(payload.name)
    .bind(payload.email)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map(Json)
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

// DELETE USER
async fn delete_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        Err(StatusCode::NOT_FOUND)
    } else {
        Ok(StatusCode::NO_CONTENT)
    }
}
