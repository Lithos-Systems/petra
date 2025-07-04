// Feature detection helpers for benchmarks

#[cfg(feature = "extended-types")]
pub const HAS_EXTENDED_TYPES: bool = true;
#[cfg(not(feature = "extended-types"))]
pub const HAS_EXTENDED_TYPES: bool = false;

#[cfg(feature = "enhanced-monitoring")]
pub const HAS_MONITORING: bool = true;
#[cfg(not(feature = "enhanced-monitoring"))]
pub const HAS_MONITORING: bool = false;

pub fn print_enabled_features() {
    println!("Benchmark features:");
    println!("  Extended types: {}", HAS_EXTENDED_TYPES);
    println!("  Enhanced monitoring: {}", HAS_MONITORING);
}
