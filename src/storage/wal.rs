// src/storage/wal.rs - Fixed with proper i64 handling for Value::Int
use crate::{error::*, value::Value};
// use rocksdb::{DB, Options, WriteBatch};
use bytes::{Buf, BufMut, BytesMut};
use parking_lot::Mutex;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tracing::{debug, info};

pub struct WriteAheadLog {
    /// Simple in-memory log of serialized entries keyed by sequence number
    db: Arc<Mutex<Vec<(u64, Vec<u8>)>>>,
    sequence: Arc<Mutex<u64>>,
}

#[derive(Debug, Clone)]
pub struct WalEntry {
    pub sequence: u64,
    pub timestamp: i64,
    pub signal: String,
    pub value: Value,
}

#[derive(Debug, Clone)]
pub struct WalConfig {
    pub wal_dir: PathBuf,
    pub max_size_mb: u64,
    pub sync_interval_ms: u64,
    pub retention_hours: u64,
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            wal_dir: PathBuf::from("./data/wal"),
            max_size_mb: 100,
            sync_interval_ms: 1000,
            retention_hours: 48,
        }
    }
}

impl WriteAheadLog {
    pub fn recover_sequence(&self) -> u64 {
        *self.sequence.lock()
    }

    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        std::fs::create_dir_all(&path)?;

        let db: Vec<(u64, Vec<u8>)> = Vec::new();
        let sequence = 0;

        info!("WAL initialized with sequence {}", sequence);

        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            sequence: Arc::new(Mutex::new(sequence)),
        })
    }

    fn recover_sequence_from_db(db: &[(u64, Vec<u8>)]) -> u64 {
        db.last().map(|(s, _)| s + 1).unwrap_or(0)
    }

    pub fn append(&self, signal: &str, value: Value, timestamp: i64) -> Result<u64> {
        let seq = {
            let mut seq_guard = self.sequence.lock();
            let seq = *seq_guard;
            *seq_guard += 1;
            seq
        };

        let entry = WalEntry {
            sequence: seq,
            timestamp,
            signal: signal.to_string(),
            value,
        };

        let value_bytes = self.serialize_entry(&entry)?;

        self.db.lock().push((seq, value_bytes));

        Ok(seq)
    }

    pub fn read_range(&self, start_seq: u64, end_seq: u64) -> Result<Vec<WalEntry>> {
        let db = self.db.lock();
        let mut entries = Vec::new();

        for (seq, bytes) in db.iter() {
            if *seq < start_seq {
                continue;
            }
            if *seq > end_seq {
                break;
            }
            let entry = self.deserialize_entry(bytes)?;
            entries.push(entry);
        }

        Ok(entries)
    }

    pub fn checkpoint(&self, up_to_seq: u64) -> Result<()> {
        let mut db = self.db.lock();
        db.retain(|(seq, _)| *seq > up_to_seq);

        debug!("WAL checkpointed up to sequence {}", up_to_seq);
        Ok(())
    }

    fn serialize_entry(&self, entry: &WalEntry) -> Result<Vec<u8>> {
        let mut buf = BytesMut::with_capacity(256);

        // Simple binary format: timestamp(8) + signal_len(2) + signal + value_type(1) + value
        buf.put_i64(entry.timestamp);
        buf.put_u16(entry.signal.len() as u16);
        buf.put_slice(entry.signal.as_bytes());

        match &entry.value {
            Value::Bool(b) => {
                buf.put_u8(0);
                buf.put_u8(*b as u8);
            }
            Value::Integer(i) => {
                buf.put_u8(1);
                buf.put_i64(*i); // Fixed: use i64 instead of i32
            }
            Value::Float(f) => {
                buf.put_u8(2);
                buf.put_f64(*f);
            }
        }

        Ok(buf.freeze().to_vec())
    }

    fn deserialize_entry(&self, data: &[u8]) -> Result<WalEntry> {
        let mut buf = bytes::Bytes::copy_from_slice(data);

        if buf.remaining() < 8 {
            return Err(PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "WAL entry truncated",
            )));
        }

        let timestamp = buf.get_i64();

        if buf.remaining() < 2 {
            return Err(PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "WAL entry truncated",
            )));
        }

        let sig_len = buf.get_u16() as usize;

        if buf.remaining() < sig_len + 1 {
            return Err(PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::UnexpectedEof,
                "WAL entry truncated",
            )));
        }

        let mut sig_bytes = vec![0u8; sig_len];
        buf.copy_to_slice(&mut sig_bytes);
        let signal = String::from_utf8(sig_bytes).map_err(|e| {
            PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Invalid UTF-8 in WAL entry: {}", e),
            ))
        })?;

        let value_type = buf.get_u8();

        let value = match value_type {
            0 => {
                if buf.remaining() < 1 {
                    return Err(PlcError::Io(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "WAL entry truncated",
                    )));
                }
                Value::Bool(buf.get_u8() != 0)
            }
            1 => {
                if buf.remaining() < 8 {
                    // Fixed: check for 8 bytes instead of 4
                    return Err(PlcError::Io(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "WAL entry truncated",
                    )));
                }
                Value::Integer(buf.get_i64()) // Fixed: use get_i64 instead of get_i32
            }
            2 => {
                if buf.remaining() < 8 {
                    return Err(PlcError::Io(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "WAL entry truncated",
                    )));
                }
                Value::Float(buf.get_f64())
            }
            _ => {
                return Err(PlcError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "Unknown value type in WAL entry",
                )));
            }
        };

        Ok(WalEntry {
            sequence: 0,
            timestamp,
            signal,
            value,
        })
    }

    pub fn recover(&self) -> Result<Vec<(String, Vec<u8>)>> {
        let db = self.db.lock();
        let mut entries = Vec::new();

        for (seq, bytes) in db.iter() {
            entries.push((seq.to_string(), bytes.clone()));
        }

        Ok(entries)
    }

    pub fn truncate(&self) -> Result<()> {
        self.checkpoint(u64::MAX)?;
        *self.sequence.lock() = 0;
        Ok(())
    }
}

/// Type alias used by the rest of the crate
pub type Wal = WriteAheadLog;
