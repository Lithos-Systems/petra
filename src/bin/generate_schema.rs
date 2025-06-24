#[cfg(feature = "json-schema")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    petra::config_schema::generate_schema()?;
    Ok(())
}

#[cfg(not(feature = "json-schema"))]
fn main() {
    eprintln!("This binary requires the 'json-schema' feature to be enabled");
    eprintln!("Run with: cargo run --bin generate_schema --features json-schema");
    std::process::exit(1);
}
