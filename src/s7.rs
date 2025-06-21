// src/s7.rs
use crate::{error::*, value::Value, signal::SignalBus};
use s7::{client::Client, tcp, transport};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::time::{interval, Duration};
use tracing::{info, warn, error, debug};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S7Config {
    /// IP address of the S7 PLC
    pub ip: String,
    /// Rack number (usually 0)
    #[serde(default)]
    pub rack: u16,
    /// Slot number (usually 2 for S7-300/400, 1 for S7-1200/1500)
    #[serde(default = "default_slot")]
    pub slot: u16,
    /// Connection type (PG, OP, Basic)
    #[serde(default = "default_connection_type")]
    pub connection_type: String,
    /// Poll interval in milliseconds
    #[serde(default = "default_poll_interval")]
    pub poll_interval_ms: u64,
    /// Timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u32,
    /// Memory mappings
    pub mappings: Vec<S7Mapping>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct S7Mapping {
    /// Signal name in Petra
    pub signal: String,
    /// Memory area (DB, I, Q, M, etc.)
    pub area: S7Area,
    /// DB number (for DB area)
    #[serde(default)]
    pub db_number: u16,
    /// Start address
    pub address: u32,
    /// Data type
    pub data_type: S7DataType,
    /// Bit offset (for bool types)
    #[serde(default)]
    pub bit: u8,
    /// Direction (read, write, read_write)
    #[serde(default = "default_direction")]
    pub direction: Direction,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(rename_all = "UPPERCASE")]
pub enum S7Area {
    DB,  // Data Block
    I,   // Inputs
    Q,   // Outputs
    M,   // Flags/Markers
    C,   // Counters
    T,   // Timers
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum S7DataType {
    Bool,
    Byte,
    Word,
    DWord,
    Int,
    DInt,
    Real,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Direction {
    Read,
    Write,
    ReadWrite,
}

fn default_slot() -> u16 { 2 }
fn default_connection_type() -> String { "PG".to_string() }
fn default_poll_interval() -> u64 { 100 }
fn default_timeout() -> u32 { 5000 }
fn default_direction() -> Direction { Direction::ReadWrite }



impl S7DataType {
    fn size(&self) -> usize {
        match self {
            S7DataType::Bool | S7DataType::Byte => 1,
            S7DataType::Word | S7DataType::Int => 2,
            S7DataType::DWord | S7DataType::DInt | S7DataType::Real => 4,
        }
    }
}

pub struct S7Connector {
    config: S7Config,
    client: Arc<Mutex<Option<Client<tcp::Transport>>>>,
    bus: SignalBus,
    mappings: Vec<S7Mapping>,
    running: Arc<Mutex<bool>>,
}

impl S7Connector {
    pub fn new(config: S7Config, bus: SignalBus) -> Result<Self> {
        Ok(Self {
            mappings: config.mappings.clone(),
            config,
            client: Arc::new(Mutex::new(None)),
            bus,
            running: Arc::new(Mutex::new(false)),
        })
    }

    pub async fn connect(&self) -> Result<()> {
        use std::net::IpAddr;

        let addr: IpAddr = self
            .config
            .ip
            .parse()
            .map_err(|e| PlcError::Config(format!("Invalid IP: {}", e)))?;

        let conn_type = match self.config.connection_type.as_str() {
            "PG" => transport::Connection::PG,
            "OP" => transport::Connection::OP,
            "Basic" => transport::Connection::Basic,
            _ => {
                return Err(PlcError::Config(format!(
                    "Invalid connection type: {}",
                    self.config.connection_type
                )))
            }
        };

        let mut opts = tcp::Options::new(addr, self.config.rack, self.config.slot, conn_type);
        let timeout = Duration::from_millis(self.config.timeout_ms as u64);
        opts.read_timeout = timeout;
        opts.write_timeout = timeout;

        let transport = tcp::Transport::connect(opts)?;
        let client = Client::new(transport)?;

        *self.client.lock().await = Some(client);
        info!(
            "Connected to S7 PLC at {}:{}:{}",
            self.config.ip, self.config.rack, self.config.slot
        );
        Ok(())
    }

    pub async fn run(&self) -> Result<()> {
        *self.running.lock().await = true;
        
        let mut ticker = interval(Duration::from_millis(self.config.poll_interval_ms));
        ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        
        while *self.running.lock().await {
            ticker.tick().await;
            
            // Read cycle
            for mapping in &self.mappings {
                if matches!(mapping.direction, Direction::Read | Direction::ReadWrite) {
                    if let Err(e) = self.read_mapping(mapping).await {
                        warn!("Failed to read {}: {}", mapping.signal, e);
                    }
                }
            }
            
            // Write cycle
            for mapping in &self.mappings {
                if matches!(mapping.direction, Direction::Write | Direction::ReadWrite) {
                    if let Err(e) = self.write_mapping(mapping).await {
                        warn!("Failed to write {}: {}", mapping.signal, e);
                    }
                }
            }
        }
        
        Ok(())
    }

    pub async fn stop(&self) {
        *self.running.lock().await = false;
        self.client.lock().await.take();
        info!("S7 connector stopped");
    }

    async fn read_mapping(&self, mapping: &S7Mapping) -> Result<()> {
        let mut guard = self.client.lock().await;
        let client = guard.as_mut().ok_or_else(|| PlcError::Config("Not connected".into()))?;
        let size = mapping.data_type.size();

        // Read data from PLC
        let mut buffer = vec![0u8; size];

        match mapping.area {
            S7Area::DB => client.ag_read(
                mapping.db_number as i32,
                mapping.address as i32,
                size as i32,
                &mut buffer,
            )?,
            S7Area::I => client.eb_read(mapping.address as i32, size as i32, &mut buffer)?,
            S7Area::Q => client.ab_read(mapping.address as i32, size as i32, &mut buffer)?,
            S7Area::M => client.mb_read(mapping.address as i32, size as i32, &mut buffer)?,
            _ => return Err(PlcError::Config("Unsupported area".into())),
        }
        
        // Convert to Petra value
        let value = match mapping.data_type {
            S7DataType::Bool => {
                let byte_val = buffer[0];
                let bit_val = (byte_val >> mapping.bit) & 1;
                Value::Bool(bit_val != 0)
            }
            S7DataType::Byte => {
                Value::Int(buffer[0] as i32)
            }
            S7DataType::Word => {
                let val = u16::from_be_bytes([buffer[0], buffer[1]]);
                Value::Int(val as i32)
            }
            S7DataType::Int => {
                let val = i16::from_be_bytes([buffer[0], buffer[1]]);
                Value::Int(val as i32)
            }
            S7DataType::DWord => {
                let val = u32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                Value::Int(val as i32)
            }
            S7DataType::DInt => {
                let val = i32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                Value::Int(val)
            }
            S7DataType::Real => {
                let val = f32::from_be_bytes([buffer[0], buffer[1], buffer[2], buffer[3]]);
                Value::Float(val as f64)
            }
        };
        
        // Update signal bus
        self.bus.set(&mapping.signal, value.clone())?;
        debug!("Read {} = {} from S7", mapping.signal, value);
        
        Ok(())
    }

    async fn write_mapping(&self, mapping: &S7Mapping) -> Result<()> {
        // Get value from signal bus
        let value = match self.bus.get(&mapping.signal) {
            Ok(v) => v,
            Err(_) => return Ok(()), // Signal doesn't exist yet
        };
        
        let mut guard = self.client.lock().await;
        let client = guard.as_mut().ok_or_else(|| PlcError::Config("Not connected".into()))?;
        
        // Convert Petra value to bytes
        let mut buffer = match (&mapping.data_type, &value) {
            (S7DataType::Bool, _) => {
                // For bool, we need to read-modify-write
                let mut existing = vec![0u8; 1];

                match mapping.area {
                    S7Area::DB => client.ag_read(
                        mapping.db_number as i32,
                        mapping.address as i32,
                        1,
                        &mut existing,
                    )?,
                    S7Area::I => client.eb_read(mapping.address as i32, 1, &mut existing)?,
                    S7Area::Q => client.ab_read(mapping.address as i32, 1, &mut existing)?,
                    S7Area::M => client.mb_read(mapping.address as i32, 1, &mut existing)?,
                    _ => return Err(PlcError::Config("Unsupported area".into())),
                }
                
                let bit_val = value.as_bool().unwrap_or(false);
                if bit_val {
                    existing[0] |= 1 << mapping.bit;
                } else {
                    existing[0] &= !(1 << mapping.bit);
                }
                existing
            }
            (S7DataType::Byte, _) => {
                vec![value.as_int().unwrap_or(0) as u8]
            }
            (S7DataType::Word, _) => {
                let val = value.as_int().unwrap_or(0) as u16;
                val.to_be_bytes().to_vec()
            }
            (S7DataType::Int, _) => {
                let val = value.as_int().unwrap_or(0) as i16;
                val.to_be_bytes().to_vec()
            }
            (S7DataType::DWord, _) => {
                let val = value.as_int().unwrap_or(0) as u32;
                val.to_be_bytes().to_vec()
            }
            (S7DataType::DInt, _) => {
                let val = value.as_int().unwrap_or(0);
                val.to_be_bytes().to_vec()
            }
            (S7DataType::Real, _) => {
                let val = value.as_float().unwrap_or(0.0) as f32;
                val.to_be_bytes().to_vec()
            }
        };
        
        // Write to PLC
        match mapping.area {
            S7Area::DB => client.ag_write(
                mapping.db_number as i32,
                mapping.address as i32,
                buffer.len() as i32,
                &mut buffer.clone(),
            )?,
            S7Area::I => client.eb_write(mapping.address as i32, buffer.len() as i32, &mut buffer.clone())?,
            S7Area::Q => client.ab_write(mapping.address as i32, buffer.len() as i32, &mut buffer.clone())?,
            S7Area::M => client.mb_write(mapping.address as i32, buffer.len() as i32, &mut buffer.clone())?,
            _ => return Err(PlcError::Config("Unsupported area".into())),
        }
        
        debug!("Wrote {} = {} to S7", mapping.signal, value);
        Ok(())
    }
}

// Helper function to create optimized read requests
pub fn optimize_mappings(mappings: &[S7Mapping]) -> Vec<ReadRequest> {
    // Group mappings by area and DB number
    let mut grouped: HashMap<(S7Area, u16), Vec<&S7Mapping>> = HashMap::new();
    
    for mapping in mappings {
        if matches!(mapping.direction, Direction::Read | Direction::ReadWrite) {
            let key = (mapping.area.clone(), mapping.db_number);
            grouped.entry(key).or_insert_with(Vec::new).push(mapping);
        }
    }
    
    // Create optimized read requests
    let mut requests = Vec::new();
    
    for ((area, db_number), mappings) in grouped {
        // Sort by address
        let mut sorted = mappings;
        sorted.sort_by_key(|m| m.address);
        
        // Find contiguous ranges
        let mut current_start = sorted[0].address;
        let mut current_end = current_start + sorted[0].data_type.size() as u32;
        let mut current_mappings = vec![sorted[0]];
        
        for mapping in &sorted[1..] {
            if mapping.address <= current_end + 4 { // Allow small gaps
                current_end = mapping.address + mapping.data_type.size() as u32;
                current_mappings.push(mapping);
            } else {
                // Start new request
                requests.push(ReadRequest {
                    area: area.clone(),
                    db_number,
                    start_address: current_start,
                    length: (current_end - current_start) as usize,
                    mappings: current_mappings.into_iter().cloned().collect(),
                });
                
                current_start = mapping.address;
                current_end = current_start + mapping.data_type.size() as u32;
                current_mappings = vec![mapping];
            }
        }
        
        // Add final request
        requests.push(ReadRequest {
            area,
            db_number,
            start_address: current_start,
            length: (current_end - current_start) as usize,
            mappings: current_mappings.into_iter().cloned().collect(),
        });
    }
    
    requests
}

#[derive(Debug)]
struct ReadRequest {
    area: S7Area,
    db_number: u16,
    start_address: u32,
    length: usize,
    mappings: Vec<S7Mapping>,
}

// Add S7 error handling
impl From<s7::error::Error> for PlcError {
    fn from(err: s7::error::Error) -> Self {
        PlcError::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("S7 error: {:?}", err),
        ))
    }
}
