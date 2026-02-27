use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("Not found")]
    NotFound,

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unprocessable entity: {0}")]
    UnprocessableEntity(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "NOT_FOUND", self.to_string()),
            AppError::Conflict(msg) => (StatusCode::CONFLICT, "CONFLICT", msg.clone()),
            AppError::BadRequest(msg) => (StatusCode::BAD_REQUEST, "BAD_REQUEST", msg.clone()),
            AppError::UnprocessableEntity(msg) => (
                StatusCode::UNPROCESSABLE_ENTITY,
                "UNPROCESSABLE_ENTITY",
                msg.clone(),
            ),
            AppError::Database(sqlx::Error::RowNotFound) => {
                (StatusCode::NOT_FOUND, "NOT_FOUND", "Not found".to_string())
            }
            AppError::Database(sqlx::Error::Database(db_err))
                if db_err.code().as_deref() == Some("23505") =>
            {
                (
                    StatusCode::CONFLICT,
                    "CONFLICT",
                    "Resource already exists".to_string(),
                )
            }
            AppError::Database(e) => {
                tracing::error!("Database error: {}", e);
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "DATABASE_ERROR",
                    "An internal database error occurred".to_string(),
                )
            }
        };

        let body = json!({
            "error": {
                "code": code,
                "message": message,
                "details": null
            }
        });

        (status, Json(body)).into_response()
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
