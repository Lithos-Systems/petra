use crate::error::*;
use clickhouse::Client;
use std::sync::Arc;
use parking_lot::Mutex;
use std::collections::VecDeque;
use std::time::Duration;
use tokio::time::interval;
use tracing::{info, warn, error};

pub struct ClickHousePool {
    connections: Vec<Client>,
    available: Arc<Mutex<VecDeque<usize>>>,
    health_check_interval: Duration,
    min_connections: usize,
    max_connections: usize,
    connection_timeout: Duration,
}

pub struct PooledConnection {
    index: usize,
    client: Client,
    pool: Arc<Mutex<VecDeque<usize>>>,
}

impl PooledConnection {
    fn new(index: usize, client: &Client, pool: Arc<Mutex<VecDeque<usize>>>) -> Self {
        Self {
            index,
            client: client.clone(),
            pool,
        }
    }
    
    pub fn client(&self) -> &Client {
        &self.client
    }
}

impl Drop for PooledConnection {
    fn drop(&mut self) {
        // Return connection to pool
        if let Ok(mut available) = self.pool.lock() {
            available.push_back(self.index);
        }
    }
}

impl ClickHousePool {
    pub async fn new(
        url: &str,
        database: &str,
        username: Option<&str>,
        password: Option<&str>,
        min_connections: usize,
        max_connections: usize,
    ) -> Result<Self> {
        let mut connections = Vec::with_capacity(max_connections);
        let mut available = VecDeque::with_capacity(max_connections);
        
        // Create initial connections
        for i in 0..min_connections {
            let client = Self::create_client(url, database, username, password)?;
            
            // Test the connection
            if let Err(e) = Self::test_connection(&client).await {
                error!("Failed to create connection {}: {}", i, e);
                continue;
            }
            
            connections.push(client);
            available.push_back(i);
        }
        
        if connections.is_empty() {
            return Err(PlcError::Config("Failed to create any ClickHouse connections".into()));
        }
        
        info!("Created ClickHouse pool with {} connections", connections.len());
        
        let pool = Self {
            connections,
            available: Arc::new(Mutex::new(available)),
            health_check_interval: Duration::from_secs(30),
            min_connections,
            max_connections,
            connection_timeout: Duration::from_secs(5),
        };
        
        // Start health check task
        let pool_clone = pool.clone();
        tokio::spawn(async move {
            pool_clone.health_check_loop().await;
        });
        
        Ok(pool)
    }
    
    fn create_client(
        url: &str,
        database: &str,
        username: Option<&str>,
        password: Option<&str>,
    ) -> Result<Client> {
        let mut client = Client::default()
            .with_url(url)
            .with_database(database)
            .with_option("connect_timeout", "5")
            .with_option("send_timeout", "30")
            .with_option("receive_timeout", "30")
            .with_option("tcp_keepalive_idle", "60")
            .with_option("tcp_keepalive_interval", "5")
            .with_option("tcp_keepalive_cnt", "3");
        
        if let Some(user) = username {
            client = client.with_user(user);
        }
        
        if let Some(pass) = password {
            client = client.with_password(pass);
        }
        
        Ok(client)
    }
    
    async fn test_connection(client: &Client) -> Result<()> {
        let query = client.query("SELECT 1");
        
        tokio::time::timeout(Duration::from_secs(5), query.fetch_one::<u8>())
            .await
            .map_err(|_| PlcError::Config("Connection test timeout".into()))?
            .map_err(|e| PlcError::Config(format!("Connection test failed: {}", e)))?;
        
        Ok(())
    }
    
    pub async fn get_connection(&self) -> Result<PooledConnection> {
        let mut retries = 3;
        let mut last_error = None;
        
        while retries > 0 {
            // Try to get an available connection
            if let Some(index) = self.try_get_available() {
                let conn = PooledConnection::new(
                    index,
                    &self.connections[index],
                    self.available.clone()
                );
                return Ok(conn);
            }
            
            // If we haven't reached max connections, try to create a new one
            if self.connections.len() < self.max_connections {
                match self.create_new_connection().await {
                    Ok(index) => {
                        let conn = PooledConnection::new(
                            index,
                            &self.connections[index],
                            self.available.clone()
                        );
                        return Ok(conn);
                    }
                    Err(e) => {
                        last_error = Some(e);
                    }
                }
            }
            
            retries -= 1;
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        Err(last_error.unwrap_or_else(|| PlcError::Config("No connections available".into())))
    }
    
    fn try_get_available(&self) -> Option<usize> {
        if let Ok(mut available) = self.available.try_lock() {
            available.pop_front()
        } else {
            None
        }
    }
    
    async fn create_new_connection(&self) -> Result<usize> {
        warn!("Creating new ClickHouse connection");
        
        // This is a simplified version - in production you'd need proper synchronization
        let index = self.connections.len();
        
        // Create client based on first connection's config
        let client = self.connections[0].clone();
        
        Self::test_connection(&client).await?;
        
        // This would need proper mutex handling in production
        // connections.push(client);
        
        Ok(index)
    }
    
    async fn health_check_loop(&self) {
        let mut interval = interval(self.health_check_interval);
        
        loop {
            interval.tick().await;
            self.check_all_connections().await;
        }
    }
    
    async fn check_all_connections(&self) {
        let mut failed = Vec::new();
        
        for (i, client) in self.connections.iter().enumerate() {
            if let Err(e) = Self::test_connection(client).await {
                warn!("Connection {} health check failed: {}", i, e);
                failed.push(i);
            }
        }
        
        // Mark failed connections as unavailable
        if !failed.is_empty() {
            if let Ok(mut available) = self.available.lock() {
                available.retain(|&idx| !failed.contains(&idx));
            }
            
            // Try to reconnect failed connections
            for idx in failed {
                tokio::spawn(async move {
                    // Implement reconnection logic
                });
            }
        }
    }
    
    pub fn stats(&self) -> PoolStats {
        let available_count = self.available.lock().len();
        
        PoolStats {
            total_connections: self.connections.len(),
            available_connections: available_count,
            active_connections: self.connections.len() - available_count,
        }
    }
}

impl Clone for ClickHousePool {
    fn clone(&self) -> Self {
        Self {
            connections: self.connections.clone(),
            available: self.available.clone(),
            health_check_interval: self.health_check_interval,
            min_connections: self.min_connections,
            max_connections: self.max_connections,
            connection_timeout: self.connection_timeout,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_connections: usize,
    pub available_connections: usize,
    pub active_connections: usize,
}
