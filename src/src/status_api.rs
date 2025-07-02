// src/status_api.rs - Real-time status monitoring API
use crate::{
    engine::Engine,
    value::Value,
    error::*,
};
use warp::{Filter, Reply, Rejection};
use serde::{Serialize, Deserialize};
use std::sync::{Arc, RwLock};
use std::collections::HashMap;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use futures::{Stream, StreamExt};
use tracing::{info, debug, error};

#[cfg(feature = "enhanced-monitoring")]
use chrono::{DateTime, Utc};

/// Status API server
pub struct StatusServer {
    engine: Arc<RwLock<Engine>>,
    port: u16,
    update_interval_ms: u64,
}

/// Engine status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngineStatus {
    pub running: bool,
    pub scan_count: u64,
    pub error_count: u64,
    pub uptime_seconds: u64,
    pub scan_time_target_ms: u64,
    pub scan_time_actual_ms: f64,
    pub jitter_ms: f64,
    pub signal_count: usize,
    pub block_count: usize,
    pub last_error: Option<String>,
    #[cfg(feature = "enhanced-monitoring")]
    pub performance_metrics: Option<PerformanceMetrics>,
}

#[cfg(feature = "enhanced-monitoring")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub avg_scan_time_us: f64,
    pub max_scan_time_us: f64,
    pub min_scan_time_us: f64,
    pub scan_time_percentiles: HashMap<String, f64>,
}

/// Signal status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalStatus {
    pub name: String,
    pub value: ValueDto,
    pub signal_type: String,
    #[cfg(feature = "enhanced")]
    pub metadata: Option<SignalMetadata>,
}

/// Block status response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockStatus {
    pub name: String,
    pub block_type: String,
    pub enabled: bool,
    #[cfg(feature = "enhanced-monitoring")]
    pub last_execution_ms: Option<f64>,
    pub error_count: u64,
    pub inputs: HashMap<String, ValueDto>,
    pub outputs: HashMap<String, ValueDto>,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Value DTO for JSON serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ValueDto {
    Bool(bool),
    Int(i32),
    Float(f64),
    String(String),
}

impl From<&Value> for ValueDto {
    fn from(value: &Value) -> Self {
        match value {
            Value::Bool(b) => ValueDto::Bool(*b),
            Value::Int(i) => ValueDto::Int(*i),
            Value::Float(f) => ValueDto::Float(*f),
            _ => ValueDto::String(format!("{:?}", value)),
        }
    }
}

#[cfg(feature = "enhanced")]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalMetadata {
    pub last_updated: DateTime<Utc>,
    pub update_count: u64,
    pub min_value: Option<ValueDto>,
    pub max_value: Option<ValueDto>,
}

/// Real-time update event
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum UpdateEvent {
    SignalChanged {
        signal: String,
        old_value: ValueDto,
        new_value: ValueDto,
        timestamp: DateTime<Utc>,
    },
    BlockExecuted {
        block: String,
        duration_ms: f64,
        timestamp: DateTime<Utc>,
    },
    ScanCompleted {
        scan_number: u64,
        duration_ms: f64,
        timestamp: DateTime<Utc>,
    },
    Error {
        message: String,
        source: String,
        timestamp: DateTime<Utc>,
    },
    ConfigReloaded {
        added_signals: Vec<String>,
        removed_signals: Vec<String>,
        added_blocks: Vec<String>,
        removed_blocks: Vec<String>,
        timestamp: DateTime<Utc>,
    },
}

impl StatusServer {
    pub fn new(engine: Arc<RwLock<Engine>>, port: u16) -> Self {
        Self {
            engine,
            port,
            update_interval_ms: 100,
        }
    }
    
    /// Start the status server
    pub async fn start(self) -> Result<()> {
        let routes = self.routes();
        
        info!("Starting status API server on port {}", self.port);
        
        warp::serve(routes)
            .run(([0, 0, 0, 0], self.port))
            .await;
            
        Ok(())
    }
    
    /// Build all routes
    pub fn routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);
        
        self.get_status()
            .or(self.get_signals())
            .or(self.get_blocks())
            .or(self.get_signal())
            .or(self.get_block())
            .or(self.update_signal())
            .or(self.ws_realtime())
            .or(self.sse_events())
            .or(self.reload_config())
            .or(self.partial_update())
            .with(cors)
            .with(warp::log("status_api"))
    }
    
    // GET /api/status
    fn get_status(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let engine = self.engine.clone();
        
        warp::path!("api" / "status")
            .and(warp::get())
            .map(move || {
                let engine = engine.read().unwrap();
                let stats = engine.get_stats();
                
                let status = EngineStatus {
                    running: stats.running,
                    scan_count: stats.scan_count,
                    error_count: stats.error_count,
                    uptime_seconds: stats.uptime_secs,
                    scan_time_target_ms: engine.config.scan_time_ms,
                    scan_time_actual_ms: 0.0, // Would need to track this
                    jitter_ms: 0.0, // Would need to calculate
                    signal_count: stats.signal_count,
                    block_count: stats.block_count,
                    last_error: None,
                    #[cfg(feature = "enhanced-monitoring")]
                    performance_metrics: None, // Would populate from stats
                };
                
                warp::reply::json(&status)
            })
    }
    
    // GET /api/signals
    fn get_signals(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let engine = self.engine.clone();
        
        warp::path!("api" / "signals")
            .and(warp::get())
            .map(move || {
                let engine = engine.read().unwrap();
                let snapshot = engine.bus.snapshot();
                
                let signals: Vec<SignalStatus> = engine.config.signals.iter()
                    .map(|sig_config| {
                        let value = snapshot.get(&sig_config.name)
                            .map(|v| ValueDto::from(v))
                            .unwrap_or(ValueDto::String("unknown".to_string()));
                        
                        SignalStatus {
                            name: sig_config.name.clone(),
                            value,
                            signal_type: sig_config.signal_type.clone(),
                            #[cfg(feature = "enhanced")]
                            metadata: None, // Would get from signal bus
                        }
                    })
                    .collect();
                
                warp::reply::json(&signals)
            })
    }
    
    // GET /api/signals/:name
    fn get_signal(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let engine = self.engine.clone();
        
        warp::path!("api" / "signals" / String)
            .and(warp::get())
            .map(move |name: String| {
                let engine = engine.read().unwrap();
                
                if let Some(value) = engine.bus.get(&name) {
                    let signal = SignalStatus {
                        name: name.clone(),
                        value: ValueDto::from(&value),
                        signal_type: engine.config.signals.iter()
                            .find(|s| s.name == name)
                            .map(|s| s.signal_type.clone())
                            .unwrap_or_else(|| "unknown".to_string()),
                        #[cfg(feature = "enhanced")]
                        metadata: None,
                    };
                    
                    warp::reply::with_status(
                        warp::reply::json(&signal),
                        warp::http::StatusCode::OK,
                    )
                } else {
                    warp::reply::with_status(
                        warp::reply::json(&serde_json::json!({
                            "error": format!("Signal '{}' not found", name)
                        })),
                        warp::http::StatusCode::NOT_FOUND,
                    )
                }
            })
    }
    
    // PUT /api/signals/:name
    fn update_signal(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        #[derive(Deserialize)]
        struct UpdateRequest {
            value: serde_json::Value,
        }
        
        let engine = self.engine.clone();
        
        warp::path!("api" / "signals" / String)
            .and(warp::put())
            .and(warp::body::json())
            .map(move |name: String, req: UpdateRequest| {
                let mut engine = engine.write().unwrap();
                
                // Convert JSON value to Value
                let value = match req.value {
                    serde_json::Value::Bool(b) => Value::Bool(b),
                    serde_json::Value::Number(n) => {
                        if let Some(i) = n.as_i64() {
                            Value::Int(i as i32)
                        } else if let Some(f) = n.as_f64() {
                            Value::Float(f)
                        } else {
                            return warp::reply::with_status(
                                warp::reply::json(&serde_json::json!({
                                    "error": "Invalid number format"
                                })),
                                warp::http::StatusCode::BAD_REQUEST,
                            );
                        }
                    }
                    _ => {
                        return warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({
                                "error": "Unsupported value type"
                            })),
                            warp::http::StatusCode::BAD_REQUEST,
                        );
                    }
                };
                
                match engine.bus.set(&name, value) {
                    Ok(_) => {
                        warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({
                                "success": true,
                                "signal": name,
                            })),
                            warp::http::StatusCode::OK,
                        )
                    }
                    Err(e) => {
                        warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({
                                "error": e.to_string()
                            })),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        )
                    }
                }
            })
    }
    
    // GET /api/blocks
    fn get_blocks(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let engine = self.engine.clone();
        
        warp::path!("api" / "blocks")
            .and(warp::get())
            .map(move || {
                let engine = engine.read().unwrap();
                
                let blocks: Vec<BlockStatus> = engine.blocks.iter()
                    .zip(engine.config.blocks.iter())
                    .map(|(block, config)| {
                        // Get current input values
                        let inputs: HashMap<String, ValueDto> = config.inputs.iter()
                            .map(|(key, signal_name)| {
                                let value = engine.bus.get(signal_name)
                                    .map(|v| ValueDto::from(&v))
                                    .unwrap_or(ValueDto::String("unknown".to_string()));
                                (key.clone(), value)
                            })
                            .collect();
                        
                        // Get current output values
                        let outputs: HashMap<String, ValueDto> = config.outputs.iter()
                            .map(|(key, signal_name)| {
                                let value = engine.bus.get(signal_name)
                                    .map(|v| ValueDto::from(&v))
                                    .unwrap_or(ValueDto::String("unknown".to_string()));
                                (key.clone(), value)
                            })
                            .collect();
                        
                        // Convert parameters to JSON
                        let parameters: HashMap<String, serde_json::Value> = config.params.iter()
                            .map(|(k, v)| {
                                let json_value = serde_yaml_to_json(v);
                                (k.clone(), json_value)
                            })
                            .collect();
                        
                        BlockStatus {
                            name: block.name().to_string(),
                            block_type: block.block_type().to_string(),
                            enabled: true,
                            #[cfg(feature = "enhanced-monitoring")]
                            last_execution_ms: block.last_execution_time()
                                .map(|d| d.as_secs_f64() * 1000.0),
                            error_count: 0,
                            inputs,
                            outputs,
                            parameters,
                        }
                    })
                    .collect();
                
                warp::reply::json(&blocks)
            })
    }
    
    // GET /api/blocks/:name
    fn get_block(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let engine = self.engine.clone();
        
        warp::path!("api" / "blocks" / String)
            .and(warp::get())
            .map(move |name: String| {
                let engine = engine.read().unwrap();
                
                // Find the block and its config
                let block_data = engine.blocks.iter()
                    .zip(engine.config.blocks.iter())
                    .find(|(block, config)| block.name() == name || config.name == name);
                
                if let Some((block, config)) = block_data {
                    // Similar to get_blocks but for single block
                    let status = BlockStatus {
                        name: block.name().to_string(),
                        block_type: block.block_type().to_string(),
                        enabled: true,
                        #[cfg(feature = "enhanced-monitoring")]
                        last_execution_ms: block.last_execution_time()
                            .map(|d| d.as_secs_f64() * 1000.0),
                        error_count: 0,
                        inputs: HashMap::new(), // Would populate
                        outputs: HashMap::new(), // Would populate
                        parameters: HashMap::new(), // Would populate
                    };
                    
                    warp::reply::with_status(
                        warp::reply::json(&status),
                        warp::http::StatusCode::OK,
                    )
                } else {
                    warp::reply::with_status(
                        warp::reply::json(&serde_json::json!({
                            "error": format!("Block '{}' not found", name)
                        })),
                        warp::http::StatusCode::NOT_FOUND,
                    )
                }
            })
    }
    
    // WebSocket endpoint for real-time updates
    fn ws_realtime(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let engine = self.engine.clone();
        let update_interval = self.update_interval_ms;
        
        warp::path!("ws" / "realtime")
            .and(warp::ws())
            .map(move |ws: warp::ws::Ws| {
                let engine = engine.clone();
                ws.on_upgrade(move |websocket| {
                    handle_websocket(websocket, engine, update_interval)
                })
            })
    }
    
    // Server-Sent Events endpoint
    fn sse_events(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let engine = self.engine.clone();
        let update_interval = self.update_interval_ms;
        
        warp::path!("api" / "events")
            .and(warp::get())
            .map(move || {
                let engine = engine.clone();
                let stream = create_event_stream(engine, update_interval);
                warp::sse::reply(stream)
            })
    }
    
    // POST /api/reload
    fn reload_config(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let engine = self.engine.clone();
        
        warp::path!("api" / "reload")
            .and(warp::post())
            .map(move || {
                // This would trigger config reload through ConfigManager
                warp::reply::json(&serde_json::json!({
                    "message": "Reload triggered"
                }))
            })
    }
    
    // POST /api/partial-update
    fn partial_update(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let engine = self.engine.clone();
        
        warp::path!("api" / "partial-update")
            .and(warp::post())
            .and(warp::body::json())
            .map(move |partial: crate::config_manager::PartialConfig| {
                let mut engine = engine.write().unwrap();
                
                match engine.apply_partial_update(partial) {
                    Ok(report) => {
                        warp::reply::with_status(
                            warp::reply::json(&report),
                            warp::http::StatusCode::OK,
                        )
                    }
                    Err(e) => {
                        warp::reply::with_status(
                            warp::reply::json(&serde_json::json!({
                                "error": e.to_string()
                            })),
                            warp::http::StatusCode::INTERNAL_SERVER_ERROR,
                        )
                    }
                }
            })
    }
}

// Helper function to convert serde_yaml::Value to serde_json::Value
fn serde_yaml_to_json(yaml_value: &serde_yaml::Value) -> serde_json::Value {
    match yaml_value {
        serde_yaml::Value::Null => serde_json::Value::Null,
        serde_yaml::Value::Bool(b) => serde_json::Value::Bool(*b),
        serde_yaml::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                serde_json::Value::Number(serde_json::Number::from(i))
            } else if let Some(f) = n.as_f64() {
                serde_json::Number::from_f64(f)
                    .map(serde_json::Value::Number)
                    .unwrap_or(serde_json::Value::Null)
            } else {
                serde_json::Value::Null
            }
        }
        serde_yaml::Value::String(s) => serde_json::Value::String(s.clone()),
        serde_yaml::Value::Sequence(seq) => {
            serde_json::Value::Array(seq.iter().map(serde_yaml_to_json).collect())
        }
        serde_yaml::Value::Mapping(map) => {
            let obj: serde_json::Map<String, serde_json::Value> = map.iter()
                .filter_map(|(k, v)| {
                    k.as_str().map(|key| (key.to_string(), serde_yaml_to_json(v)))
                })
                .collect();
            serde_json::Value::Object(obj)
        }
        _ => serde_json::Value::Null,
    }
}

// WebSocket handler
async fn handle_websocket(
    websocket: warp::ws::WebSocket,
    engine: Arc<RwLock<Engine>>,
    update_interval_ms: u64,
) {
    use futures::{SinkExt, StreamExt};
    use tokio::time::{interval, Duration};
    
    let (mut ws_tx, mut ws_rx) = websocket.split();
    
    // Spawn task to send periodic updates
    let engine_clone = engine.clone();
    let mut interval = interval(Duration::from_millis(update_interval_ms));
    
    tokio::spawn(async move {
        while interval.tick().await.is_ok() {
            let engine = engine_clone.read().unwrap();
            let stats = engine.get_stats();
            
            let event = UpdateEvent::ScanCompleted {
                scan_number: stats.scan_count,
                duration_ms: 0.0, // Would track actual duration
                timestamp: Utc::now(),
            };
            
            let msg = warp::ws::Message::text(
                serde_json::to_string(&event).unwrap()
            );
            
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });
    
    // Handle incoming messages
    while let Some(result) = ws_rx.next().await {
        match result {
            Ok(msg) => {
                if msg.is_text() {
                    // Handle commands from client
                    debug!("Received WebSocket message: {:?}", msg.to_str());
                }
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }
}

// Create Server-Sent Events stream
fn create_event_stream(
    engine: Arc<RwLock<Engine>>,
    update_interval_ms: u64,
) -> impl Stream<Item = Result<warp::sse::Event, warp::Error>> + Send + 'static {
    use tokio::time::{interval, Duration};
    use tokio_stream::wrappers::IntervalStream;
    
    let interval = interval(Duration::from_millis(update_interval_ms));
    let stream = IntervalStream::new(interval);
    
    stream.map(move |_| {
        let engine = engine.read().unwrap();
        let stats = engine.get_stats();
        
        let event = UpdateEvent::ScanCompleted {
            scan_number: stats.scan_count,
            duration_ms: 0.0, // Would need actual tracking
            timestamp: Utc::now(),
        };
        
        Ok(warp::sse::Event::default()
            .event("update")
            .data(serde_json::to_string(&event).unwrap()))
    })
}
