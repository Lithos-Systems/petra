// You might have a struct like this for loading the overall project config.
#[derive(Debug)]
pub struct ProjectConfig {
    pub name: String,
    pub plcs: Vec<String>,  // IDs of PLCs in this project
    // Extend as needed!
}
