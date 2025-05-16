pub struct Rack {
    pub id: &'static str,
    pub modules: Vec<String>,
    // Each module can have its own config.
}
