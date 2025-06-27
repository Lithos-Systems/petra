use regex::Regex;

pub fn validate_signal_name(name: &str) -> Result<()> {
    let pattern = Regex::new(r"^[a-zA-Z_][a-zA-Z0-9_]*$")?;
    if !pattern.is_match(name) {
        return Err(PlcError::Config("Invalid signal name format".into()));
    }
    if name.len() > 64 {
        return Err(PlcError::Config("Signal name too long".into()));
    }
    Ok(())
}

pub fn sanitize_mqtt_topic(topic: &str) -> Result<String> {
    // Remove null bytes and control characters
    let clean = topic.chars()
        .filter(|&c| c >= ' ' && c != '\0')
        .collect();
    Ok(clean)
}
