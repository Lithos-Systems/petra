use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;
use crate::{PlcError, Value};
use super::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WsMessage {
    #[serde(rename = "subscribe_signal")]
    SubscribeSignal { signal: String },

    #[serde(rename = "unsubscribe_signal")]
    UnsubscribeSignal { signal: String },

    #[serde(rename = "set_signal")]
    SetSignal { signal: String, value: Value },

    #[serde(rename = "ping")]
    Ping { timestamp: Option<u64> },

    #[serde(rename = "signal_update")]
    SignalUpdate {
        signal: String,
        value: Value,
        timestamp: u64,
        quality: Option<String>,
    },

    #[serde(rename = "error")]
    Error { error: String },

    #[serde(rename = "pong")]
    Pong { timestamp: u64 },
}

pub async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel(100);

    let tx_clone = tx.clone();
    let state_clone = state.clone();
    tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                if let Message::Text(text) = msg {
                    handle_message(text, &state_clone, &tx_clone).await;
                }
            }
        }
    });

    while let Some(msg) = rx.recv().await {
        if sender.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}

async fn handle_message(text: String, state: &AppState, tx: &mpsc::Sender<String>) {
    let msg: Result<WsMessage, _> = serde_json::from_str(&text);

    match msg {
        Ok(WsMessage::SubscribeSignal { signal }) => {
            println!("Subscribe to signal: {}", signal);
        }
        Ok(WsMessage::UnsubscribeSignal { signal }) => {
            println!("Unsubscribe from signal: {}", signal);
        }
        Ok(WsMessage::SetSignal { signal, value }) => {
            if let Err(e) = state.signal_bus.set(&signal, value) {
                let error_msg = WsMessage::Error { error: e.to_string() };
                let _ = tx.send(serde_json::to_string(&error_msg).unwrap()).await;
            }
        }
        Ok(WsMessage::Ping { timestamp }) => {
            let pong = WsMessage::Pong {
                timestamp: timestamp.unwrap_or_else(|| {
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_millis() as u64
                }),
            };
            let _ = tx.send(serde_json::to_string(&pong).unwrap()).await;
        }
        Err(e) => {
            let error_msg = WsMessage::Error { error: format!("Invalid message: {}", e) };
            let _ = tx.send(serde_json::to_string(&error_msg).unwrap()).await;
        }
        _ => {}
    }
}
