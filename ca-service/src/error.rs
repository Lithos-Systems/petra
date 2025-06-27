use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ApiError {
    #[error("Internal server error")]
    Internal(#[from] anyhow::Error),
    
    #[error("Database error")]
    Database(#[from] sqlx::Error),
    
    #[error("Bad request: {0}")]
    BadRequest(String),
    
    #[error("Not found")]
    NotFound,
    
    #[error("Certificate has been revoked")]
    CertificateRevoked,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            ApiError::BadRequest(msg) => (StatusCode::BAD_REQUEST, msg),
            ApiError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            ApiError::CertificateRevoked => (StatusCode::FORBIDDEN, "Certificate has been revoked".to_string()),
            ApiError::Database(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string()),
            ApiError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string()),
        };

        let body = Json(serde_json::json!({
            "error": error_message,
        }));

        (status, body).into_response()
    }
}
