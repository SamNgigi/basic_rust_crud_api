mod config;
mod errors;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
};
use config::{AppConfig, create_pool};
use errors::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use utoipa::{OpenApi, ToSchema};
use utoipa_swagger_ui::SwaggerUi;
use uuid::Uuid;

use std::net::SocketAddr;

#[derive(OpenApi)]
#[openapi(
    paths(create_user),
    components(schemas(User, UserPayload)),
    tags((name="users", description="User management endpoints"))
)]
struct ApiDocs;

#[derive(Debug, Deserialize, ToSchema)]
struct UserPayload {
    name: String,
    email: String,
}

#[derive(Serialize, FromRow, ToSchema)]
struct User {
    id: i64,
    uuid: Uuid,
    name: String,
    email: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,tower_http=info,axum=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let config = AppConfig::from_env();
    let pool = create_pool(&config.db)
        .await
        .expect("Failed to Connect to Postgres");
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
        .merge(SwaggerUi::new("/swagger-ui").url("/api-docs/openapi.json", ApiDocs::openapi()))
        .with_state(pool);

    let addr = format!("{}:{}", &config.app_host, &config.app_port);
    let addr: SocketAddr = addr.parse()?;
    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("Failed to bind to {addr}");
    println!("🚀 Server running on port 8000");
    tracing::info!("listening on http://{}", addr);
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

// -----------------------------------------------------------------
// ENDPOINT HANDLERS
// -----------------------------------------------------------------

// Test Endpoint
async fn root() -> &'static str {
    "Welcome to Basic Crud User Management"
}

// CREATE USER
#[utoipa::path(
    post,
    path = "/users",
    request_body = UserPayload,
    responses(
        (status = 201, description = "User created", body = User),
        (status = 400, description = "Invalid payload"),
        (status = 500, description = "Internal error"),
    ),
    tag = "users"
)]
async fn create_user(
    State(pool): State<PgPool>,
    Json(payload): Json<UserPayload>,
) -> AppResult<(StatusCode, Json<User>)> {
    validate_user_payload(&payload)?;
    let user = sqlx::query_as::<_, User>(
        r#"
            INSERT INTO users (name, email) 
            VALUES ($1, $2) 
            RETURNING id, uuid, name, email
        "#,
    )
    .bind(payload.name)
    .bind(payload.email)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::from_sqlx(e, "Failed to create user"))?;

    Ok((StatusCode::CREATED, Json(user)))
}

// GET ALL USERS
async fn list_users(State(pool): State<PgPool>) -> AppResult<Json<Vec<User>>> {
    let users = sqlx::query_as::<_, User>(
        r#"
                SELECT id, uuid, name, email 
                    FROM users
                ORDER BY id
        "#,
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| AppError::from_sqlx(e, "failed to list users"))?;

    Ok(Json(users))
}

// GET USER BY ID
async fn get_user_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
) -> AppResult<(StatusCode, Json<User>)> {
    let user = sqlx::query_as::<_, User>(
        r#"
                SELECT id, uuid, name, email
                    FROM users 
                WHERE id = $1
            "#,
    )
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::from_sqlx(e, format!("Failed to fetch user with id={id}")))?;
    Ok((StatusCode::OK, Json(user)))
}

// UPDATE USER
async fn update_user(
    State(pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(payload): Json<UserPayload>,
) -> AppResult<(StatusCode, Json<User>)> {
    validate_user_payload(&payload)?;

    let user = sqlx::query_as::<_, User>(
        r#"
            UPDATE users SET name = $1, email = $2 
            WHERE id = $3 
            RETURNING id, uuid, name, email
        "#,
    )
    .bind(payload.name)
    .bind(payload.email)
    .bind(id)
    .fetch_one(&pool)
    .await
    .map_err(|e| AppError::from_sqlx(e, format!("Failed to update user with id={id}")))?;

    Ok((StatusCode::OK, Json(user)))
}

// DELETE USER
async fn delete_user(State(pool): State<PgPool>, Path(id): Path<i64>) -> AppResult<StatusCode> {
    let result = sqlx::query(
        r#"
                DELETE FROM users 
                WHERE id = $1
            "#,
    )
    .bind(id)
    .execute(&pool)
    .await
    .map_err(|e| AppError::from_sqlx(e, format!("Failed to delete user with id={id}")))?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok(StatusCode::NO_CONTENT)
}

fn validate_user_payload(payload: &UserPayload) -> AppResult<()> {
    if payload.name.trim().is_empty() {
        return Err(AppError::BadRequest("name cannot be empty".to_string()));
    }

    if payload.email.trim().is_empty() {
        return Err(AppError::BadRequest("email cannot be empty".to_string()));
    }

    if !payload.email.contains('@') {
        return Err(AppError::BadRequest("email must be valid".to_string()));
    }

    Ok(())
}
