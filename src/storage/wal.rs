// src/storage/wal.rs - Fixed with proper i64 handling for Value::Int
use crate::{error::*, value::Value};
// use rocksdb::{DB, Options, WriteBatch};
use std::path::Path;
use std::sync::Arc;
use parking_lot::Mutex;
use bytes::{BytesMut, BufMut, Buf};
use tracing::{info, debug};

pub struct WriteAheadLog {
    // db: Arc<Mutex<DB>>,
    db: Arc<Mutex<()>>, // RocksDB removed
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
//         opts.set_compression_type(rocksdb::DBCompressionType::Lz4);
        
        // let db = DB::open(&opts, path)
        //     .map_err(|e| PlcError::Io(std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         format!("Failed to open WAL: {}", e)
        //     )))?;
        let db = (); // placeholder
        
        // Recover sequence number
        // let sequence = Self::recover_sequence_from_db(&db);
        let sequence = 0;
        
        info!("WAL initialized with sequence {}", sequence);
        
        Ok(Self {
            // db: Arc::new(Mutex::new(db)),
            db: Arc::new(Mutex::new(())),
            sequence: Arc::new(Mutex::new(sequence)),
        })
    }
    
    fn recover_sequence_from_db(_db: &()) -> u64 {
        // RocksDB removed
        0
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
        
        let key = seq.to_be_bytes();
        let value_bytes = self.serialize_entry(&entry)?;
        
        self.db.lock()
            .put(&key, value_bytes)
            .map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("WAL write failed: {}", e)
            )))?;
        
        Ok(seq)
    }
    
    pub fn read_range(&self, start_seq: u64, end_seq: u64) -> Result<Vec<WalEntry>> {
        let db = self.db.lock();
        let mut entries = Vec::new();
        
        let start_key = start_seq.to_be_bytes();
        let end_key = end_seq.to_be_bytes();
        
//         let iter = db.iterator(rocksdb::IteratorMode::From(&start_key, rocksdb::Direction::Forward));
        
        // for item in iter {
            let (key, value) = item.map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("WAL read failed: {}", e)
            )))?;
            
            if key.as_ref() > end_key.as_slice() {
                break;
            }
            
            let entry = self.deserialize_entry(&value)?;
            entries.push(entry);
        // }
        
        Ok(entries)
    }
    
    pub fn checkpoint(&self, up_to_seq: u64) -> Result<()> {
        let db = self.db.lock();
        // let mut batch = WriteBatch::default();
        let mut batch = (); // placeholder

        let end_key = up_to_seq.to_be_bytes();
        // let iter = db.iterator(rocksdb::IteratorMode::Start);

        // for item in iter {
        //     let (key, _) = item.map_err(|e| PlcError::Io(std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         format!("WAL checkpoint failed: {}", e)
        //     )))?;

        //     if key.as_ref() > end_key.as_slice() {
        //         break;
        //     }

        //     batch.delete(&key);
        // }

        // db.write(batch)
        //     .map_err(|e| PlcError::Io(std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         format!("WAL checkpoint write failed: {}", e)
        //     )))?;

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
            Value::Int(i) => {
                buf.put_u8(1);
                buf.put_i64(*i);  // Fixed: use i64 instead of i32
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
                if buf.remaining() < 8 {  // Fixed: check for 8 bytes instead of 4
                    return Err(PlcError::Io(std::io::Error::new(
                        std::io::ErrorKind::UnexpectedEof,
                        "WAL entry truncated",
                    )));
                }
                Value::Int(buf.get_i64())  // Fixed: use get_i64 instead of get_i32
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

        // let iter = db.iterator(rocksdb::IteratorMode::Start);

        // for item in iter {
        //     let (key, value) = item.map_err(|e| PlcError::Io(std::io::Error::new(
        //         std::io::ErrorKind::Other,
        //         format!("WAL read failed: {}", e),
        //     )))?;
        //
        //     let key_str = match String::from_utf8(key.to_vec()) {
        //         Ok(s) => s,
        //         Err(_) => {
        //             let seq = u64::from_be_bytes(
        //                 key.as_ref()[..8]
        //                     .try_into()
        //                     .unwrap_or([0u8; 8]),
        //             );
        //             seq.to_string()
        //         }
        //     };
        //     entries.push((key_str, value.to_vec()));
        // }

        Ok(entries)
    }

    pub fn truncate(&self) -> Result<()> {
        self.checkpoint(u64::MAX)?;
        *self.sequence.lock() = 0;
        Ok(())
    }
}
