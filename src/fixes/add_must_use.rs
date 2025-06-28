// Helper script to add #[must_use] attributes

use std::fs;
use std::path::Path;
use regex::Regex;

fn add_must_use_to_builders(path: &Path) -> std::io::Result<()> {
    let content = fs::read_to_string(path)?;
    
    // Patterns to match builder methods
    let patterns = vec![
        // Methods that return Self
        r"pub fn (\w+).*-> Self",
        // Methods that return Result<Self>
        r"pub fn (\w+).*-> Result<Self",
        // new() constructors
        r"pub fn new\(.*\) -> ",
        // with_* methods
        r"pub fn with_\w+.*-> ",
    ];
    
    let mut modified = content.clone();
    
    for pattern in patterns {
        let re = Regex::new(pattern).unwrap();
        for cap in re.captures_iter(&content) {
            let method_line = cap.get(0).unwrap().as_str();
            
            // Check if #[must_use] already exists
            if !content.contains(&format!("#[must_use]\n    {}", method_line)) {
                modified = modified.replace(
                    method_line,
                    &format!("#[must_use]\n    {}", method_line)
                );
            }
        }
    }
    
    if modified != content {
        fs::write(path, modified)?;
        println!("✅ Added #[must_use] to {}", path.display());
    }
    
    Ok(())
}
