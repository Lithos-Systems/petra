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
