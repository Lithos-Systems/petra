// src/storage/mod.rs
use crate::error::Result;
use async_trait::async_trait;
use std::path::Path;

#[cfg(feature = "advanced-storage")]
pub mod clickhouse;

#[cfg(feature = "advanced-storage")]
pub mod s3;

#[cfg(feature = "advanced-storage")]
pub mod wal;

#[cfg(feature = "compression")]
pub mod compression;

// Base storage trait
#[async_trait]
pub trait StorageBackend: Send + Sync {
    async fn write(&self, data: &[u8], key: &str) -> Result<()>;
    async fn read(&self, key: &str) -> Result<Vec<u8>>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;
    async fn exists(&self, key: &str) -> Result<bool>;
}

// Simple local storage (always available)
pub struct LocalStorage {
    base_path: std::path::PathBuf,
    #[cfg(feature = "compression")]
    compression: compression::CompressionType,
}

impl LocalStorage {
    pub fn new<P: AsRef<Path>>(path: P) -> Result<Self> {
        std::fs::create_dir_all(&path)?;
        Ok(Self {
            base_path: path.as_ref().to_path_buf(),
            #[cfg(feature = "compression")]
            compression: compression::CompressionType::None,
        })
    }

    #[cfg(feature = "compression")]
    pub fn with_compression(mut self, compression: compression::CompressionType) -> Self {
        self.compression = compression;
        self
    }
}

#[async_trait]
impl StorageBackend for LocalStorage {
    async fn write(&self, data: &[u8], key: &str) -> Result<()> {
        let path = self.base_path.join(key);
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        #[cfg(feature = "compression")]
        let data = self.compression.compress(data)?;

        tokio::fs::write(path, data).await?;
        Ok(())
    }

    async fn read(&self, key: &str) -> Result<Vec<u8>> {
        let path = self.base_path.join(key);
        let data = tokio::fs::read(path).await?;

        #[cfg(feature = "compression")]
        let data = self.compression.decompress(&data)?;

        Ok(data)
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let path = self.base_path.join(key);
        tokio::fs::remove_file(path).await?;
        Ok(())
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let mut entries = Vec::new();
        let prefix_path = self.base_path.join(prefix);
        
        let mut dir = tokio::fs::read_dir(prefix_path).await?;
        while let Some(entry) = dir.next_entry().await? {
            if let Some(name) = entry.file_name().to_str() {
                entries.push(name.to_string());
            }
        }
        
        Ok(entries)
    }

    async fn exists(&self, key: &str) -> Result<bool> {
        let path = self.base_path.join(key);
        Ok(path.exists())
    }
}

// Unified storage manager with tiered storage
pub struct StorageManager {
    primary: Box<dyn StorageBackend>,
    #[cfg(feature = "advanced-storage")]
    secondary: Option<Box<dyn StorageBackend>>,
    #[cfg(feature = "advanced-storage")]
    archive: Option<Box<dyn StorageBackend>>,
    #[cfg(feature = "wal")]
    wal: Option<wal::WriteAheadLog>,
}

impl StorageManager {
    pub fn new(primary: Box<dyn StorageBackend>) -> Self {
        Self {
            primary,
            #[cfg(feature = "advanced-storage")]
            secondary: None,
            #[cfg(feature = "advanced-storage")]
            archive: None,
            #[cfg(feature = "wal")]
            wal: None,
        }
    }

    #[cfg(feature = "advanced-storage")]
    pub fn with_secondary(mut self, secondary: Box<dyn StorageBackend>) -> Self {
        self.secondary = Some(secondary);
        self
    }

    #[cfg(feature = "advanced-storage")]
    pub fn with_archive(mut self, archive: Box<dyn StorageBackend>) -> Self {
        self.archive = Some(archive);
        self
    }

    #[cfg(feature = "wal")]
    pub fn with_wal(mut self, wal_path: &Path) -> Result<Self> {
        self.wal = Some(wal::WriteAheadLog::new(wal_path)?);
        Ok(self)
    }

    pub async fn write(&self, data: &[u8], key: &str) -> Result<()> {
        // Write to WAL first if enabled
        #[cfg(feature = "wal")]
        if let Some(wal) = &self.wal {
            wal.append(key, data).await?;
        }

        // Write to primary storage
        self.primary.write(data, key).await?;

        // Replicate to secondary if configured
        #[cfg(feature = "advanced-storage")]
        if let Some(secondary) = &self.secondary {
            // Fire and forget for async replication
            let secondary = secondary.as_ref();
            let data = data.to_vec();
            let key = key.to_string();
            tokio::spawn(async move {
                let _ = secondary.write(&data, &key).await;
            });
        }

        Ok(())
    }

    pub async fn read(&self, key: &str) -> Result<Vec<u8>> {
        // Try primary first
        match self.primary.read(key).await {
            Ok(data) => Ok(data),
            Err(_) => {
                // Try secondary if primary fails
                #[cfg(feature = "advanced-storage")]
                if let Some(secondary) = &self.secondary {
                    return secondary.read(key).await;
                }
                
                #[cfg(not(feature = "advanced-storage"))]
                Err(crate::error::PlcError::Storage("Key not found".into()))
            }
        }
    }

    #[cfg(feature = "advanced-storage")]
    pub async fn archive(&self, key: &str, age_days: u32) -> Result<()> {
        if let Some(archive) = &self.archive {
            // Read from primary
            let data = self.primary.read(key).await?;
            
            // Write to archive
            archive.write(&data, key).await?;
            
            // Delete from primary if older than threshold
            // (implement age checking logic here)
            
            Ok(())
        } else {
            Ok(())
        }
    }

    #[cfg(feature = "wal")]
    pub async fn recover(&mut self) -> Result<()> {
        if let Some(wal) = &self.wal {
            let entries = wal.recover().await?;
            
            for (key, data) in entries {
                self.primary.write(&data, &key).await?;
            }
            
            wal.truncate().await?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_local_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(temp_dir.path()).unwrap();

        // Test write and read
        let data = b"test data";
        storage.write(data, "test.txt").await.unwrap();
        
        let read_data = storage.read("test.txt").await.unwrap();
        assert_eq!(data, &read_data[..]);

        // Test exists
        assert!(storage.exists("test.txt").await.unwrap());
        assert!(!storage.exists("missing.txt").await.unwrap());

        // Test delete
        storage.delete("test.txt").await.unwrap();
        assert!(!storage.exists("test.txt").await.unwrap());
    }

    #[cfg(feature = "compression")]
    #[tokio::test]
    async fn test_compressed_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalStorage::new(temp_dir.path())
            .unwrap()
            .with_compression(compression::CompressionType::Zstd);

        let data = b"test data that should be compressed";
        storage.write(data, "compressed.txt").await.unwrap();
        
        let read_data = storage.read("compressed.txt").await.unwrap();
        assert_eq!(data, &read_data[..]);
    }
}
