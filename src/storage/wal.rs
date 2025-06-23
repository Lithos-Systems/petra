use crate::{error::*, value::Value};
use rocksdb::{DB, Options, WriteBatch};
use std::path::Path;
use std::sync::Arc;
use parking_lot::Mutex;
use bytes::{Bytes, BytesMut, BufMut};
use tracing::{info, error, debug};

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
        let sequence = Self::recover_sequence(&db);
        
        info!("WAL initialized with sequence {}", sequence);
        
        Ok(Self {
            db: Arc::new(Mutex::new(db)),
            sequence: Arc::new(Mutex::new(sequence)),
        })
    }
    
    fn recover_sequence(db: &DB) -> u64 {
        let iter = db.iterator(rocksdb::IteratorMode::End);
        if let Some(Ok((key, _))) = iter.into_iter().next() {
            u64::from_be_bytes(key[..8].try_into().unwrap_or([0; 8])) + 1
        } else {
            0
        }
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
        
        let iter = db.iterator(rocksdb::IteratorMode::From(&start_key, rocksdb::Direction::Forward));
        
        for item in iter {
            let (key, value) = item.map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("WAL read failed: {}", e)
            )))?;
            
            if key.as_ref() > &end_key {
                break;
            }
            
            let entry = self.deserialize_entry(&value)?;
            entries.push(entry);
        }
        
        Ok(entries)
    }
    
    pub fn checkpoint(&self, up_to_seq: u64) -> Result<()> {
        let db = self.db.lock();
        let mut batch = WriteBatch::default();
        
        let end_key = up_to_seq.to_be_bytes();
        let iter = db.iterator(rocksdb::IteratorMode::Start);
        
        for item in iter {
            let (key, _) = item.map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("WAL checkpoint failed: {}", e)
            )))?;
            
            if key.as_ref() > &end_key {
                break;
            }
            
            batch.delete(&key);
        }
        
        db.write(batch)
            .map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("WAL checkpoint write failed: {}", e)
            )))?;
        
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
                buf.put_i32(*i);
            }
            Value::Float(f) => {
                buf.put_u8(2);
                buf.put_f64(*f);
            }
        }
        
        Ok(buf.freeze().to_vec())
    }
    
    fn deserialize_entry(&self, data: &[u8]) -> Result<WalEntry> {
        // Implement deserialization
        unimplemented!("Deserialize WAL entry")
    }
}
