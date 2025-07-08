use std::sync::Arc;
use axum::extract::ws::Message;
use serde_json::Value;

use crate::{error::{Result, PlcError}, signal::SignalBus, web::AppState};

pub async fn handle_websocket_message(
    msg: Message,
    signal_bus: Arc<SignalBus>,
    state: Arc<AppState>,
) -> Result<()> {
    let text = msg.to_str()?;
    let json: Value = serde_json::from_str(text)?;

    match json["type"].as_str() {
        Some("subscribe_signal") => handle_subscribe_signal(json, state).await?,
        Some("unsubscribe_signal") => handle_unsubscribe_signal(json, state).await?,
        Some("set_signal") => handle_set_signal(json, signal_bus).await?,
        Some("batch_set") => handle_batch_set(json, signal_bus).await?,
        Some("get_metadata") => handle_get_metadata(json, signal_bus).await?,
        Some("subscribe_group") => handle_subscribe_group(json, state).await?,
        Some("get_history") => handle_get_history(json, state).await?,
        Some("subscribe_alarms") => handle_subscribe_alarms(json, state).await?,
        Some("acknowledge_alarm") => handle_acknowledge_alarm(json, state).await?,
        Some("get_system_status") => handle_get_system_status(state).await?,
        Some("ping") => handle_ping(json).await?,
        _ => return Err(PlcError::WebSocket("Unknown message type".into())),
    }

    Ok(())
}
