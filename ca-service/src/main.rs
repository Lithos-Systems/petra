use anyhow::Result;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use std::{net::SocketAddr, sync::Arc};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::{info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod ca;
mod db;
mod email;
mod error;
mod stripe_handler;

use ca::CertificateAuthority;
use db::Database;
use error::ApiError;
use stripe_handler::handle_stripe_webhook;

#[derive(Clone)]
struct AppState {
    ca: Arc<CertificateAuthority>,
    db: Arc<Database>,
    stripe_webhook_secret: String,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "petra_ca_service=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Initialize database
    let database_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    let db = Arc::new(Database::new(&database_url).await?);
    db.run_migrations().await?;

    // Initialize CA
    let ca_root_path = std::env::var("CA_ROOT_PATH")
        .unwrap_or_else(|_| "./ca/root".to_string());
    let ca = Arc::new(CertificateAuthority::load_or_create(&ca_root_path).await?);

    // Get Stripe webhook secret
    let stripe_webhook_secret = std::env::var("STRIPE_WEBHOOK_SECRET")
        .expect("STRIPE_WEBHOOK_SECRET must be set");

    let app_state = AppState {
        ca,
        db,
        stripe_webhook_secret,
    };

    // Build router
    let app = Router::new()
        .route("/health", get(health_check))
        .route("/api/webhook/stripe", post(handle_stripe_webhook))
        .route("/api/certificates/:id", get(get_certificate))
        .route("/api/certificates/:id/bundle", get(download_certificate_bundle))
        .route("/api/certificates/:id/revoke", post(revoke_certificate))
        .route("/api/certificates/crl", get(get_crl))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(app_state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 8443));
    info!("CA service listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app)
        .await
        .unwrap();

    Ok(())
}

async fn health_check() -> impl IntoResponse {
    Json(serde_json::json!({
        "status": "healthy",
        "service": "petra-ca"
    }))
}

async fn get_certificate(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let cert = state.db.get_certificate(id).await?;
    Ok(Json(serde_json::to_value(cert)?))
}

async fn download_certificate_bundle(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    let cert = state.db.get_certificate(id).await?;
    
    if cert.revoked {
        return Err(ApiError::CertificateRevoked);
    }

    let bundle = state.ca.get_certificate_bundle(&cert).await?;
    
    let headers = HeaderMap::new();
    Ok((
        StatusCode::OK,
        headers,
        Json(serde_json::json!({
            "certificate": bundle.certificate,
            "private_key": bundle.private_key,
            "ca_certificate": bundle.ca_certificate,
            "mqtt_config": {
                "broker_host": std::env::var("MQTT_BROKER_HOST").unwrap_or_else(|_| "mqtt.petra.systems".to_string()),
                "broker_port": 8883,
                "client_id": cert.common_name.clone(),
            }
        }))
    ))
}

async fn revoke_certificate(
    State(state): State<AppState>,
    Path(id): Path<uuid::Uuid>,
) -> Result<impl IntoResponse, ApiError> {
    state.db.revoke_certificate(id).await?;
    state.ca.add_to_revocation_list(id).await?;
    
    Ok(Json(serde_json::json!({
        "status": "revoked",
        "certificate_id": id
    })))
}

async fn get_crl(
    State(state): State<AppState>,
) -> Result<impl IntoResponse, ApiError> {
    let crl = state.ca.generate_crl().await?;
    
    let mut headers = HeaderMap::new();
    headers.insert("Content-Type", "application/pkix-crl".parse().unwrap());
    
    Ok((StatusCode::OK, headers, crl))
}
