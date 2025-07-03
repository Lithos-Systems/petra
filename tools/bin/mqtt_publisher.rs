use tokio::time::{sleep, Duration};
use rumqttc::{AsyncClient, MqttOptions, QoS};
use rand::Rng;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        println!("Usage: {} <topic> [initial_value]", args[0]);
        println!("Example: {} sensors/pressure/value 100", args[0]);
        std::process::exit(1);
    }
    
    let topic = &args[1];
    let initial_value: f64 = args.get(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(100.0);
    
    // MQTT setup
    let mut mqttoptions = MqttOptions::new("test-publisher", "localhost", 1883);
    mqttoptions.set_keep_alive(Duration::from_secs(5));
    
    let (client, mut eventloop) = AsyncClient::new(mqttoptions, 10);
    
    // Spawn event loop handler
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(_) => {}
                Err(e) => {
                    eprintln!("MQTT error: {:?}", e);
                    sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });
    
    println!("Publishing to topic: {}", topic);
    println!("Initial value: {}", initial_value);
    println!("Press Ctrl+C to stop");
    
    let mut value = initial_value;
    let mut rng = rand::thread_rng();
    
    loop {
        // Simulate pressure variations
        let change = rng.gen_range(-5.0..5.0);
        value = (value + change).max(0.0).min(150.0);
        
        // Publish value
        client.publish(topic, QoS::AtLeastOnce, false, value.to_string()).await?;
        println!("Published: {} = {:.1}", topic, value);
        
        sleep(Duration::from_secs(1)).await;
    }
}
