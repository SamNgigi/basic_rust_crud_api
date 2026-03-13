use anyhow::anyhow;
use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use sqlx::{Error as SqlxError, error::DatabaseError, postgres::PgDatabaseError};
use thiserror::Error;
use tracing::error;

#[derive(Debug, Serialize)]
pub struct APIErrorBody {
    pub code: &'static str,
    pub message: String,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("resource not found")]
    NotFound,

    #[error("resource already exists")]
    Conflict,

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

pub type AppResult<T> = Result<T, AppError>;

impl AppError {
    pub fn from_sqlx(err: SqlxError, context: impl Into<String>) -> Self {
        let context = context.into();

        match err {
            SqlxError::RowNotFound => AppError::NotFound,
            SqlxError::Database(db_err) => {
                if let Some(pg_err) = db_err.try_downcast_ref::<PgDatabaseError>()
                    && pg_err.is_unique_violation()
                {
                    return AppError::Conflict;
                }
                AppError::Internal(anyhow!(db_err).context(context))
            }
            other => AppError::Internal(anyhow!(other).context(context)),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        error!(error = ?self, "request failed");

        let (status, body) = match &self {
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                APIErrorBody {
                    code: "not_found",
                    message: self.to_string(),
                },
            ),

            AppError::Conflict => (
                StatusCode::CONFLICT,
                APIErrorBody {
                    code: "conflict",
                    message: self.to_string(),
                },
            ),

            AppError::BadRequest(msg) => (
                StatusCode::BAD_REQUEST,
                APIErrorBody {
                    code: "bad_request",
                    message: msg.clone(),
                },
            ),

            AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                APIErrorBody {
                    code: "internal_error",
                    message: "internal server error".to_string(),
                },
            ),
        };

        (status, Json(body)).into_response()
    }
}
