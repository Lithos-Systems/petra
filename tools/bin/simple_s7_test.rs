// src/bin/simple_s7_test.rs
use rust_snap7::{S7Client, InternalParam, InternalParamValue};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <ip_address> [rack] [slot]", args[0]);
        std::process::exit(1);
    }
    
    let ip = &args[1];
    let rack = args.get(2).and_then(|s| s.parse().ok()).unwrap_or(0);
    let slot = args.get(3).and_then(|s| s.parse().ok()).unwrap_or(2);
    
    println!("Testing connection to {}:{}:{}", ip, rack, slot);
    
    let client = S7Client::create();
    
    // Set remote port
    client.set_param(InternalParam::RemotePort, InternalParamValue::U16(102))?;
    
    match client.connect_to(ip, rack, slot) {
        Ok(_) => {
            println!("✓ Connected successfully!");
            
            // Try to get PLC status
            let mut status = 0i32;
            if let Ok(_) = client.get_plc_status(&mut status) {
                let status_text = match status {
                    0x04 => "STOP",
                    0x08 => "RUN",
                    _ => "UNKNOWN",
                };
                println!("  PLC Status: {}", status_text);
            }
            
            // Try a simple read from DB1
            let mut buffer = [0u8; 2];
            match client.db_read(1, 0, 2, &mut buffer) {
                Ok(_) => {
                    let value = u16::from_be_bytes(buffer);
                    println!("  DB1.W0 = {}", value);
                },
                Err(e) => println!("  Read failed: {:?}", e),
            }
            
            client.disconnect()?;
            println!("✓ Disconnected");
        },
        Err(e) => {
            println!("✗ Connection failed: {:?}", e);
            println!("Check:");
            println!("  - IP address is correct");
            println!("  - PLC is reachable (ping {})", ip);
            println!("  - Rack/slot numbers are correct");
            println!("  - PLC allows GET/PUT communication");
            std::process::exit(1);
        }
    }
    
    Ok(())
}
