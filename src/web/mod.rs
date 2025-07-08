#[cfg(feature = "health")]
pub mod health;
pub mod websocket;

#[cfg(feature = "web")]
use axum::{routing::get, Router, serve};
#[cfg(feature = "web")]
use crate::error::Result;

#[cfg(feature = "web")]
pub async fn start_server() -> Result<()> {
    let app = Router::new();
    #[cfg(feature = "health")]
    let app = app.route("/health", get(|| async { "OK" }));

    use std::net::SocketAddr;
    let addr: SocketAddr = "0.0.0.0:8080".parse().unwrap();
    let listener = tokio::net::TcpListener::bind(addr).await?;
    serve(listener, app.into_make_service()).await?;
    Ok(())
}
