use axum::{response::{IntoResponse, Html}, http::StatusCode};

pub async fn spa_fallback() -> impl IntoResponse {
    match tokio::fs::read_to_string("petra-designer/dist/index.html").await {
        Ok(content) => Html(content).into_response(),
        Err(_) => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}
