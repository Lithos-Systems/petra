// tests/integration/security_tests.rs
#[tokio::test]
async fn test_authentication_required() {
    let config = test_config_with_auth();
    let engine = Engine::new(config).unwrap();
    
    // Test unauthenticated access is denied
    // Test invalid credentials are rejected
    // Test session expiration
}

#[tokio::test] 
async fn test_rate_limiting() {
    // Test MQTT message rate limits
    // Test API call rate limits
    // Test signal update rate limits
}

// tests/security/fuzz_tests.rs
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    if let Ok(config_str) = std::str::from_utf8(data) {
        let _ = Config::from_yaml(config_str);
    }
});
