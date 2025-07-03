use crate::{error::*, value::Value};
// use rocksdb::{DB, Options, WriteBatch};
use serde::{Serialize, Deserialize};
use std::path::Path;
use bincode;
use crc32fast::Hasher;
use tracing::{info, error, warn};

#[derive(Debug, Serialize, Deserialize)]
pub struct WalEntry {
    pub sequence: u64,
    pub timestamp: i64,
    pub operation: WalOperation,
    pub checksum: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum WalOperation {
    SignalUpdate { name: String, value: Value },
    Batch { updates: Vec<(String, Value)> },
    Checkpoint { marker: u64 },
}

pub struct EnhancedWal {
    // db: DB,
    db: (), // RocksDB removed
    sequence: AtomicU64,
    corruption_count: AtomicU64,
}

impl EnhancedWal {
    pub fn new(_path: &Path) -> Result<Self> {
        // RocksDB removed
        let last_seq = 0;

        Ok(Self {
            db: (),
            sequence: AtomicU64::new(last_seq + 1),
            corruption_count: AtomicU64::new(0),
        })
    }

    fn find_last_sequence(_db: &()) -> u64 {
        // RocksDB removed
        0
    }
    
    fn key_to_sequence(key: &[u8]) -> Result<u64> {
        if key.len() == 8 {
            Ok(u64::from_be_bytes([
                key[0], key[1], key[2], key[3],
                key[4], key[5], key[6], key[7]
            ]))
        } else {
            Err(PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid key format"
            )))
        }
    }
    
    pub fn append(&self, operation: WalOperation) -> Result<u64> {
        let seq = self.sequence.fetch_add(1, Ordering::Relaxed);
        
        let entry = WalEntry {
            sequence: seq,
            timestamp: chrono::Utc::now().timestamp_nanos(),
            operation,
            checksum: 0, // Will be calculated below
        };
        
        // Serialize without checksum first
        let mut data = bincode::serialize(&entry)
            .map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Serialization error: {}", e)
            )))?;
        
        // Calculate checksum
        let mut hasher = Hasher::new();
        hasher.update(&data);
        let checksum = hasher.finalize();
        
        // Update the entry with checksum
        let entry_with_checksum = WalEntry { checksum, ..entry };
        
        let data = bincode::serialize(&entry_with_checksum)
            .map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Serialization error: {}", e)
            )))?;
        
        let key = seq.to_be_bytes();
        self.db.put(&key, &data)
            .map_err(|e| PlcError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                format!("WAL write error: {}", e)
            )))?;
        
        Ok(seq)
    }
    
    pub fn read_range_with_checksum(&self, start: u64, end: u64) -> Result<Vec<WalEntry>> {
        let mut entries = Vec::new();
        let mut corrupted = 0;
        
        let start_key = start.to_be_bytes();
        // let iter = self.db.iterator(rocksdb::IteratorMode::From(&start_key, rocksdb::Direction::Forward));
        let iter = [].iter();
        
        // for (key, value) in iter {
            let seq = Self::key_to_sequence(&key)?;
            
            if seq > end {
                break;
            }
            
            match bincode::deserialize::<WalEntry>(&value) {
                Ok(mut entry) => {
                    // Verify checksum
                    let stored_checksum = entry.checksum;
                    entry.checksum = 0;
                    
                    let data = bincode::serialize(&entry).unwrap();
                    let mut hasher = Hasher::new();
                    hasher.update(&data);
                    let calculated_checksum = hasher.finalize();
                    
                    if stored_checksum == calculated_checksum {
                        entry.checksum = stored_checksum;
                        entries.push(entry);
                    } else {
                        corrupted += 1;
                        warn!(
                            "Corrupted WAL entry at sequence {}: checksum mismatch",
                            seq
                        );
                    }
                }
                Err(e) => {
                    corrupted += 1;
                    error!("Failed to deserialize WAL entry at sequence {}: {}", seq, e);
                }
            }
        // }
        
        if corrupted > 0 {
            self.corruption_count.fetch_add(corrupted, Ordering::Relaxed);
            warn!("Found {} corrupted entries during WAL read", corrupted);
        }
        
        Ok(entries)
    }
    
    pub fn checkpoint(&self, keep_entries: u64) -> Result<()> {
        info!("Creating WAL checkpoint, keeping last {} entries", keep_entries);
        
        // Find the cutoff sequence
        let current_seq = self.sequence.load(Ordering::Relaxed);
        if current_seq <= keep_entries {
            return Ok(());
        }
        
        let cutoff = current_seq - keep_entries;
        
        // Delete old entries
        // let mut batch = WriteBatch::default();
        let mut batch = ();
        let mut deleted = 0;
        
        // let iter = self.db.iterator(rocksdb::IteratorMode::Start);
        // for (key, _) in iter {
            let seq = Self::key_to_sequence(&key)?;
            
            if seq >= cutoff {
                break;
            }
            
        //     batch.delete(&key);
        //     deleted += 1;

        //     if deleted % 10000 == 0 {
        //         self.db.write(batch)?;
        //         batch = WriteBatch::default();
        //     }
        // }

        // if deleted > 0 {
        //     self.db.write(batch)?;
        //     info!("Deleted {} old WAL entries", deleted);
        // }

        // // Compact the database
        // self.db.compact_range(None::<&[u8]>, None::<&[u8]>);
        
        Ok(())
    }
    
    pub fn corruption_count(&self) -> u64 {
        self.corruption_count.load(Ordering::Relaxed)
    }
}

use std::sync::atomic::{AtomicU64, Ordering};
