// Add src/opcua.rs
#[cfg(feature = "opcua-support")]
pub mod opcua {
    use opcua::server::{ServerBuilder, Session};
    use opcua::types::*;
    
    pub struct OpcUaConfig {
        pub endpoint: String,
        pub port: u16,
        pub security_policies: Vec<SecurityPolicy>,
        pub certificate_path: PathBuf,
        pub private_key_path: PathBuf,
    }
    
    pub struct OpcUaServer {
        config: OpcUaConfig,
        bus: SignalBus,
        server: Option<Server>,
    }
    
    impl OpcUaServer {
        pub async fn start(&mut self) -> Result<()> {
            let server = ServerBuilder::new()
                .application_name("Petra PLC")
                .application_uri("urn:petra:plc")
                .endpoint("opc.tcp://0.0.0.0:4840/", SecurityPolicy::None)
                .endpoint("opc.tcp://0.0.0.0:4840/", SecurityPolicy::Basic256Sha256)
                .trust_client_certs()
                .create_sample_keypair(true)
                .server()
                .unwrap();
                
            // Expose signals as OPC-UA variables
            self.register_signals(&server).await?;
            
            server.run().await;
            Ok(())
        }
    }
}
