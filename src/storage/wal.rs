// src/storage/wal.rs - Fixed with proper i64 handling for Value::Int
use crate::{error::*, value::Value};
use rocksdb::{DB, Options, WriteBatch};
use std::path::Path;
use std::sync::Arc;
use parking_lot::Mutex;
use bytes::{BytesMut, BufMut, Buf};
use tracing::{info, debug};

pub struct WriteAheadLog {
    db: Arc<Mutex<DB>>,
    sequence: Arc<Mutex<u64>>,
}

#[derive(Debug, Clone)]
pub struct WalEntry {
    pub sequence: u64,
    pub timestamp: i64,
    pub signal: String,
    pub value: Value,
}

impl WriteAheadLog {
    pub fn recover_sequence(&self) -> u64 {
        *self.sequence.lock()
    }
    
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut opts = Options::default();
        opts.create_if_missing(true);
        opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        
        let db = DB::open(&opts, path)
            .map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("Failed to open WAL: {}", e)
            )))?;
        
        // Recover sequence number
        let sequence = Self::recover_sequence_from_db(&db);
        
        info!("WAL initialized with sequence {}", sequence);
        
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            sequence: Arc::new(Mutex::
