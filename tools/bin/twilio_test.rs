// src/bin/twilio_test.rs
use petra::{SignalBus, TwilioConfig, TwilioConnector, TwilioAction, TwilioActionType, Value};
use clap::{Parser, Subcommand};
use tracing_subscriber;

#[derive(Parser)]
#[command(name = "twilio_test")]
#[command(about = "Test Twilio integration")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Send a test SMS
    Sms {
        /// To phone number (E.164 format)
        #[arg(short, long)]
        to: String,
        
        /// Message content
        #[arg(short, long)]
        message: String,
        
        /// From phone number (optional, uses env var if not provided)
        #[arg(short, long)]
        from: Option<String>,
    },
    
    /// Make a test call
    Call {
        /// To phone number (E.164 format)
        #[arg(short, long)]
        to: String,
        
        /// Message to speak or TwiML
        #[arg(short, long)]
        message: String,
        
        /// From phone number (optional, uses env var if not provided)
        #[arg(short, long)]
        from: Option<String>,
    },
    
    /// Test with a signal trigger
    Signal {
        /// Config file path
        #[arg(short, long)]
        config: String,
        
        /// Signal name to trigger
        #[arg(short, long)]
        signal: String,
        
        /// Value to set (true/false for bool, number for int/float)
        #[arg(short, long)]
        value: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("twilio_test=debug,petra=debug")
        .init();
    
    let cli = Cli::parse();
    
    match &cli.command {
        Commands::Sms { to, message, from } => {
            test_sms(to.clone(), message.clone(), from.clone()).await?;
        }
        Commands::Call { to, message, from } => {
            test_call(to.clone(), message.clone(), from.clone()).await?;
        }
        Commands::Signal { config, signal, value } => {
            test_signal(config.clone(), signal.clone(), value.clone()).await?;
        }
    }
    
    Ok(())
}

async fn test_sms(to: String, message: String, from: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing SMS to {}", to);
    
    let bus = SignalBus::new();
    bus.set("test_trigger", Value::Bool(false))?;
    bus.set("test_result", Value::Bool(false))?;
    
    let mut config = TwilioConfig::default();
    config.from_number = from.unwrap_or_default();
    config.actions.push(TwilioAction {
        name: "test_sms".to_string(),
        trigger_signal: "test_trigger".to_string(),
        action_type: TwilioActionType::Sms,
        to_number: to,
        content: message,
        trigger_value: Some(Value::Bool(true)),
        cooldown_seconds: 0,
        result_signal: Some("test_result".to_string()),
        from_number: None,
    });
    
    let connector = TwilioConnector::new(config, bus.clone())?;
    
    // Start connector
    let handle = tokio::spawn(async move {
        connector.run().await
    });
    
    // Give it time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Trigger the action
    bus.set("test_trigger", Value::Bool(true))?;
    
    // Wait for result
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    let result = bus.get_bool("test_result")?;
    if result {
        println!("✓ SMS sent successfully!");
    } else {
        println!("✗ SMS send failed!");
    }
    
    handle.abort();
    Ok(())
}

async fn test_call(to: String, message: String, from: Option<String>) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing call to {}", to);
    
    let bus = SignalBus::new();
    bus.set("test_trigger", Value::Bool(false))?;
    bus.set("test_result", Value::Bool(false))?;
    
    let mut config = TwilioConfig::default();
    config.from_number = from.unwrap_or_default();
    config.actions.push(TwilioAction {
        name: "test_call".to_string(),
        trigger_signal: "test_trigger".to_string(),
        action_type: TwilioActionType::Call,
        to_number: to,
        content: message,
        trigger_value: Some(Value::Bool(true)),
        cooldown_seconds: 0,
        result_signal: Some("test_result".to_string()),
        from_number: None,
    });
    
    let connector = TwilioConnector::new(config, bus.clone())?;
    
    // Start connector
    let handle = tokio::spawn(async move {
        connector.run().await
    });
    
    // Give it time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    
    // Trigger the action
    bus.set("test_trigger", Value::Bool(true))?;
    
    // Wait for result
    tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
    
    let result = bus.get_bool("test_result")?;
    if result {
        println!("✓ Call initiated successfully!");
    } else {
        println!("✗ Call initiation failed!");
    }
    
    handle.abort();
    Ok(())
}

async fn test_signal(config_path: String, signal: String, value_str: String) -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing with config file: {}", config_path);
    
    let config_str = std::fs::read_to_string(&config_path)?;
    let config: petra::Config = serde_yaml::from_str(&config_str)?;
    
    if config.twilio.is_none() {
        return Err("No Twilio configuration in file".into());
    }
    
    let bus = SignalBus::new();
    
    // Parse and set the signal value
    let value = if value_str == "true" {
        Value::Bool(true)
    } else if value_str == "false" {
        Value::Bool(false)
    } else if let Ok(i) = value_str.parse::<i32>() {
        Value::Integer(i as i64)
    } else if let Ok(f) = value_str.parse::<f64>() {
        Value::Float(f)
    } else {
        return Err(format!("Cannot parse value: {}", value_str).into());
    };
    
    // Initialize signals
    for sig in &config.signals {
        let initial = match sig.signal_type.as_str() {
            "bool" => Value::Bool(false),
            "int" => Value::Integer(0),
            "float" => Value::Float(0.0),
            _ => continue,
        };
        bus.set(&sig.name, initial)?;
    }
    
    let twilio_config = config.twilio.unwrap();
    let connector = TwilioConnector::new(twilio_config, bus.clone())?;
    
    // Start connector
    let handle = tokio::spawn(async move {
        connector.run().await
    });
    
    // Give it time to start
    tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    
    println!("Setting {} = {}", signal, value);
    bus.set(&signal, value)?;
    
    // Wait for any actions to complete
    tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
    
    println!("Test completed. Check your phone for results.");
    
    handle.abort();
    Ok(())
}
