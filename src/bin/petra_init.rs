// src/bin/petra_init.rs
#[derive(clap::Parser)]
struct InitCommand {
    #[clap(subcommand)]
    template: Template,
}

enum Template {
    Basic,
    Industrial { plc_count: u8 },
    EdgeGateway,
    BuildingAutomation,
}

// Generates complete config with comments
petra init industrial --plc-count 3 > config.yaml
