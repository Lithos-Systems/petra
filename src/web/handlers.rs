use axum::{extract::{Path, State}, Json};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::{Value, PlcError};
use super::AppState;

#[derive(Serialize)]
pub struct HealthResponse {
    status: String,
    version: String,
    uptime: u64,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: crate::VERSION.to_string(),
        uptime: 0,
    })
}

pub async fn get_signals(State(state): State<AppState>) -> Result<Json<HashMap<String, Value>>, PlcError> {
    let signals = state.signal_bus.get_all_signals()?;
    Ok(Json(signals))
}

pub async fn get_signal(Path(name): Path<String>, State(state): State<AppState>) -> Result<Json<Value>, PlcError> {
    let value = state.signal_bus.get(&name).ok_or_else(|| PlcError::SignalNotFound(name.clone()))?;
    Ok(Json(value))
}

#[derive(Deserialize)]
pub struct SetSignalRequest {
    value: Value,
}

pub async fn set_signal(Path(name): Path<String>, State(state): State<AppState>, Json(req): Json<SetSignalRequest>) -> Result<(), PlcError> {
    state.signal_bus.set(&name, req.value)?;
    Ok(())
}

pub async fn get_config(State(state): State<AppState>) -> Result<Json<crate::Config>, PlcError> {
    let config = state.config.read().await;
    Ok(Json(config.clone()))
}

pub async fn update_config(State(state): State<AppState>, Json(new_config): Json<crate::Config>) -> Result<(), PlcError> {
    let mut config = state.config.write().await;
    *config = new_config;
    Ok(())
}
