use anyhow::Result;
use axum::{
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Json},
};
use stripe::{Webhook, Event, EventObject, EventType};
use tracing::{error, info, warn};

use crate::{AppState, error::ApiError, email::send_certificate_email};

pub async fn handle_stripe_webhook(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: String,
) -> Result<impl IntoResponse, ApiError> {
    // Get Stripe signature from headers
    let signature = headers
        .get("stripe-signature")
        .and_then(|v| v.to_str().ok())
        .ok_or(ApiError::BadRequest("Missing Stripe signature".to_string()))?;

    // Verify webhook signature
    let event = Webhook::construct_event(&body, signature, &state.stripe_webhook_secret)
        .map_err(|e| {
            error!("Failed to verify Stripe webhook: {}", e);
            ApiError::BadRequest("Invalid webhook signature".to_string())
        })?;

    info!("Received Stripe event: {} ({})", event.type_, event.id);

    // Handle different event types
    match &event.type_ {
        EventType::CheckoutSessionCompleted => {
            handle_checkout_completed(&state, event).await?;
        }
        EventType::CustomerSubscriptionDeleted => {
            handle_subscription_deleted(&state, event).await?;
        }
        EventType::CustomerSubscriptionUpdated => {
            handle_subscription_updated(&state, event).await?;
        }
        _ => {
            info!("Unhandled event type: {}", event.type_);
        }
    }

    Ok(Json(serde_json::json!({ "status": "ok" })))
}

async fn handle_checkout_completed(state: &AppState, event: Event) -> Result<()> {
    let session = match &event.data.object {
        EventObject::CheckoutSession(session) => session,
        _ => {
            error!("Expected CheckoutSession object");
            return Ok(());
        }
    };

    // Extract required fields
    let customer_id = session
        .customer
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No customer ID in session"))?
        .id();

    let subscription_id = session
        .subscription
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!("No subscription ID in session"))?
        .id();
    
    let customer_email = session.customer_email.as_ref()
        .or(session.customer_details.as_ref().and_then(|d| d.email.as_ref()))
        .ok_or_else(|| anyhow::anyhow!("No customer email in session"))?;

    info!(
        "Processing new subscription: customer={}, subscription={}, email={}",
        customer_id,
        subscription_id,
        customer_email
    );

    // Check if certificate already exists for this subscription
    if let Some(existing) = state
        .db
        .get_certificate_by_stripe_subscription(subscription_id.as_str())
        .await?
    {
        info!(
            "Certificate already exists for subscription {}: {}",
            subscription_id,
            existing.id
        );
        return Ok(());
    }

    // Issue new certificate
    let (bundle, cert_record) = state.ca.issue_client_certificate(
        customer_id.as_str(),
        subscription_id.as_str(),
        customer_email,
        365, // 1 year validity
    ).await?;

    // Save to database
    state.db.save_certificate(&cert_record).await?;

    info!(
        "Issued certificate {} for customer {} ({})",
        cert_record.id,
        customer_id,
        customer_email
    );

    // Send email with certificate
    if let Err(e) = send_certificate_email(customer_email, &bundle, &cert_record).await {
        error!("Failed to send certificate email: {}", e);
        // Don't fail the webhook - we can retry email later
    }

    Ok(())
}

async fn handle_subscription_deleted(state: &AppState, event: Event) -> Result<()> {
    let subscription = match &event.data.object {
        EventObject::Subscription(sub) => sub,
        _ => {
            error!("Expected Subscription object");
            return Ok(());
        }
    };

    let subscription_id = subscription.id.as_str();
    info!("Processing subscription deletion: {}", subscription_id);

    // Find and revoke all certificates for this subscription
    if let Some(cert) = state
        .db
        .get_certificate_by_stripe_subscription(subscription_id)
        .await?
    {
        info!("Revoking certificate {} for deleted subscription", cert.id);
        state.db.revoke_certificate(cert.id).await?;
        state.ca.add_to_revocation_list(cert.id).await?;
    }

    Ok(())
}

async fn handle_subscription_updated(state: &AppState, event: Event) -> Result<()> {
    let subscription = match &event.data.object {
        EventObject::Subscription(sub) => sub,
        _ => {
            error!("Expected Subscription object");
            return Ok(());
        }
    };

    // Check if subscription is still active
    match subscription.status.as_str() {
        "active" | "trialing" => {
            info!("Subscription {} is still active", subscription.id);
        }
        "canceled" | "unpaid" | "past_due" => {
            warn!("Subscription {} status changed to {}", subscription.id, subscription.status);
            // Optionally revoke certificate for unpaid subscriptions
        }
        _ => {
            info!("Subscription {} status: {}", subscription.id, subscription.status);
        }
    }

    Ok(())
}
