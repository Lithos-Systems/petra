// src/web/websocket.rs

use axum::extract::ws::{Message, WebSocket};
use futures::{sink::SinkExt, stream::StreamExt};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tokio::time::{interval, Duration};
use crate::Value;
use super::AppState;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ClientMessage {
    #[serde(rename = "subscribe_signal")]
    SubscribeSignal { signal: String },
    
    #[serde(rename = "subscribe_signals")]
    SubscribeSignals { signals: Vec<String> },

    #[serde(rename = "unsubscribe_signal")]
    UnsubscribeSignal { signal: String },

    #[serde(rename = "set_signal")]
    SetSignal { signal: String, value: Value },

    #[serde(rename = "ping")]
    Ping { timestamp: Option<u64> },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ServerMessage {
    #[serde(rename = "signal_update")]
    SignalUpdate {
        #[serde(rename = "data")]
        data: SignalUpdateData,
    },

    #[serde(rename = "error")]
    Error { error: String },

    #[serde(rename = "pong")]
    Pong { timestamp: u64 },
    
    #[serde(rename = "connected")]
    Connected { version: String },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SignalUpdateData {
    signal: String,
    value: Value,
    timestamp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    quality: Option<String>,
}

struct ClientState {
    subscriptions: Arc<RwLock<HashSet<String>>>,
    tx: mpsc::Sender<String>,
}

pub async fn handle_socket(socket: WebSocket, state: AppState) {
    let (mut sender, mut receiver) = socket.split();
    let (tx, mut rx) = mpsc::channel(100);
    
    // Send connected message
    let connected_msg = ServerMessage::Connected {
        version: crate::VERSION.to_string(),
    };
    let _ = tx.send(serde_json::to_string(&connected_msg).unwrap()).await;

    // Client state
    let client_state = ClientState {
        subscriptions: Arc::new(RwLock::new(HashSet::new())),
        tx: tx.clone(),
    };

    // Spawn task to handle incoming messages
    let state_clone = state.clone();
    let client_state_clone = client_state.subscriptions.clone();
    let tx_clone = tx.clone();
    
    tokio::spawn(async move {
        while let Some(msg) = receiver.next().await {
            if let Ok(msg) = msg {
                match msg {
                    Message::Text(text) => {
                        handle_client_message(text, &state_clone, &client_state_clone, &tx_clone).await;
                    }
                    Message::Close(_) => break,
                    _ => {}
                }
            }
        }
    });

    // Spawn task to send signal updates
    let state_clone = state.clone();
    let subscriptions = client_state.subscriptions.clone();
    let tx_clone = tx.clone();
    
    tokio::spawn(async move {
        let mut ticker = interval(Duration::from_millis(100)); // 10Hz update rate
        let mut last_values: std::collections::HashMap<String, Value> = std::collections::HashMap::new();
        
        loop {
            ticker.tick().await;
            
            let subs = subscriptions.read().await;
            if subs.is_empty() {
                continue;
            }
            
            for signal_name in subs.iter() {
                if let Some(current_value) = state_clone.signal_bus.get(signal_name) {
                    let should_send = match last_values.get(signal_name) {
                        Some(last_value) => !values_equal(last_value, &current_value),
                        None => true,
                    };
                    
                    if should_send {
                        last_values.insert(signal_name.clone(), current_value.clone());
                        
                        let update = ServerMessage::SignalUpdate {
                            data: SignalUpdateData {
                                signal: signal_name.clone(),
                                value: current_value,
                                timestamp: get_timestamp(),
                                quality: Some("good".to_string()),
                            },
                        };
                        
                        if tx_clone.send(serde_json::to_string(&update).unwrap()).await.is_err() {
                            return; // Client disconnected
                        }
                    }
                }
            }
        }
    });

    // Send messages to client
    while let Some(msg) = rx.recv().await {
        if sender.send(Message::Text(msg)).await.is_err() {
            break;
        }
    }
}

async fn handle_client_message(
    text: String,
    state: &AppState,
    subscriptions: &Arc<RwLock<HashSet<String>>>,
    tx: &mpsc::Sender<String>,
) {
    let msg: Result<ClientMessage, _> = serde_json::from_str(&text);

    match msg {
        Ok(ClientMessage::SubscribeSignal { signal }) => {
            println!("WebSocket: Subscribe to signal: {}", signal);
            subscriptions.write().await.insert(signal.clone());
            
            // Send initial value
            if let Some(value) = state.signal_bus.get(&signal) {
                let update = ServerMessage::SignalUpdate {
                    data: SignalUpdateData {
                        signal,
                        value,
                        timestamp: get_timestamp(),
                        quality: Some("good".to_string()),
                    },
                };
                let _ = tx.send(serde_json::to_string(&update).unwrap()).await;
            }
        }
        
        Ok(ClientMessage::SubscribeSignals { signals }) => {
            println!("WebSocket: Subscribe to {} signals", signals.len());
            let mut subs = subscriptions.write().await;
            for signal in signals {
                subs.insert(signal.clone());
                
                // Send initial value
                if let Some(value) = state.signal_bus.get(&signal) {
                    let update = ServerMessage::SignalUpdate {
                        data: SignalUpdateData {
                            signal,
                            value,
                            timestamp: get_timestamp(),
                            quality: Some("good".to_string()),
                        },
                    };
                    let _ = tx.send(serde_json::to_string(&update).unwrap()).await;
                }
            }
        }
        
        Ok(ClientMessage::UnsubscribeSignal { signal }) => {
            println!("WebSocket: Unsubscribe from signal: {}", signal);
            subscriptions.write().await.remove(&signal);
        }
        
        Ok(ClientMessage::SetSignal { signal, value }) => {
            println!("WebSocket: Set signal {} = {:?}", signal, value);
            match state.signal_bus.set(&signal, value.clone()) {
                Ok(_) => {
                    // Immediately send update to confirm
                    let update = ServerMessage::SignalUpdate {
                        data: SignalUpdateData {
                            signal,
                            value,
                            timestamp: get_timestamp(),
                            quality: Some("good".to_string()),
                        },
                    };
                    let _ = tx.send(serde_json::to_string(&update).unwrap()).await;
                }
                Err(e) => {
                    let error_msg = ServerMessage::Error { 
                        error: format!("Failed to set signal: {}", e) 
                    };
                    let _ = tx.send(serde_json::to_string(&error_msg).unwrap()).await;
                }
            }
        }
        
        Ok(ClientMessage::Ping { timestamp }) => {
            let pong = ServerMessage::Pong {
                timestamp: timestamp.unwrap_or_else(get_timestamp),
            };
            let _ = tx.send(serde_json::to_string(&pong).unwrap()).await;
        }
        
        Err(e) => {
            let error_msg = ServerMessage::Error { 
                error: format!("Invalid message: {}", e) 
            };
            let _ = tx.send(serde_json::to_string(&error_msg).unwrap()).await;
        }
    }
}

fn get_timestamp() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as u64
}

fn values_equal(a: &Value, b: &Value) -> bool {
    match (a, b) {
        (Value::Bool(x), Value::Bool(y)) => x == y,
        (Value::Integer(x), Value::Integer(y)) => x == y,
        (Value::Float(x), Value::Float(y)) => (x - y).abs() < f64::EPSILON,
        _ => false,
    }
}
