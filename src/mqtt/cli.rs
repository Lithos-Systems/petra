//! MQTT CLI utilities

use crate::Result;

/// Test MQTT connection
pub async fn test_connection(broker: &str, topic: &str, count: u32) -> Result<()> {
    #[cfg(feature = "mqtt")]
    {
        println!("Testing MQTT connection to {} on topic {}", broker, topic);
        println!("Would send {} test messages", count);
        Ok(())
    }
    #[cfg(not(feature = "mqtt"))]
    {
        Err(PlcError::Config("MQTT support requires 'mqtt' feature".to_string()))
    }
}
