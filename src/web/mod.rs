use crate::{PlcError, Result, SignalBus};
use axum::{
    extract::{State, WebSocketUpgrade},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

mod static_files;
use static_files::spa_fallback;

pub mod handlers;
pub mod websocket;

#[derive(Clone)]
pub struct AppState {
    pub signal_bus: Arc<SignalBus>,
    pub config: Arc<RwLock<crate::Config>>,
}

pub async fn create_server(signal_bus: Arc<SignalBus>, config: crate::Config) -> Result<()> {
    let state = AppState {
        signal_bus,
        config: Arc::new(RwLock::new(config)),
    };

    let app = Router::new()
        .route("/health", get(handlers::health))
        .route("/api/signals", get(handlers::get_signals))
        .route("/api/signals/:name", get(handlers::get_signal))
        .route("/api/signals/:name", post(handlers::set_signal))
        .route("/api/config", get(handlers::get_config))
        .route("/api/config", post(handlers::update_config))
        .route("/ws", get(websocket_handler))
        .nest_service("/", ServeDir::new("petra-designer/dist"))
        .fallback(spa_fallback)
        .layer(CorsLayer::permissive())
        .with_state(state);

    let addr = "0.0.0.0:8080".parse().unwrap();
    println!("Web server listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .map_err(|e| PlcError::WebServer(e.to_string()))?;

    Ok(())
}

async fn websocket_handler(ws: WebSocketUpgrade, State(state): State<AppState>) -> impl IntoResponse {
    ws.on_upgrade(move |socket| websocket::handle_socket(socket, state))
}
