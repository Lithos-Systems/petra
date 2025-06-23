// src/bin/petra_dashboard.rs
use eframe::egui::{self, Context, CentralPanel, SidePanel, TopBottomPanel};
use egui_plot::{Line, Plot, PlotPoints};
use petra::{Config, Engine, SignalBus, Value, Result};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{info, error, warn};

const MAX_POINTS: usize = 1000;

#[derive(Clone)]
struct SignalData {
    name: String,
    values: VecDeque<(f64, f64)>,
    color: egui::Color32,
}

impl SignalData {
    fn new(name: String, color: egui::Color32) -> Self {
        Self {
            name,
            values: VecDeque::with_capacity(MAX_POINTS),
            color,
        }
    }
    
    fn add_point(&mut self, time: f64, value: f64) {
        if self.values.len() >= MAX_POINTS {
            self.values.pop_front();
        }
        self.values.push_back((time, value));
    }
    
    fn to_plot_points(&self) -> PlotPoints {
        PlotPoints::from_iter(self.values.iter().map(|(t, v)| [*t, *v]))
    }
}

struct PetraApp {
    bus: Option<SignalBus>,
    signal_data: Arc<Mutex<Vec<SignalData>>>,
    start_time: Instant,
    running: bool,
    current_values: Arc<Mutex<std::collections::HashMap<String, Value>>>,
    error_message: Option<String>,
}

impl PetraApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Box<Self> {
        let signal_data = Arc::new(Mutex::new(vec![
            SignalData::new("test_input".to_string(), egui::Color32::BLUE),
            SignalData::new("test_output".to_string(), egui::Color32::RED),
            SignalData::new("sample_count".to_string(), egui::Color32::GREEN),
        ]));
        
        let current_values = Arc::new(Mutex::new(std::collections::HashMap::new()));
        let start_time = Instant::now();
        
        // Try to load config and start engine
        let (bus, error_message) = match Self::init_engine(signal_data.clone(), current_values.clone(), start_time) {
            Ok(bus) => (Some(bus), None),
            Err(e) => {
                error!("Failed to initialize engine: {}", e);
                (None, Some(format!("Failed to initialize: {}", e)))
            }
        };
        
        Box::new(Self {
            bus,
            signal_data,
            start_time,
            running: true,
            current_values,
            error_message,
        })
    }
    
    fn init_engine(
        signal_data: Arc<Mutex<Vec<SignalData>>>,
        current_values: Arc<Mutex<std::collections::HashMap<String, Value>>>,
        start_time: Instant
    ) -> Result<SignalBus> {
        let config = Config::from_file("configs/storage-test.yaml")?;
        let mut engine = Engine::new(config)?;
        let bus = engine.bus().clone();
        
        let (tx, mut rx) = mpsc::channel(1000);
        engine.set_signal_change_channel(tx);
        
        // Start engine in background
        tokio::spawn(async move {
            if let Err(e) = engine.run().await {
                error!("Engine error: {}", e);
            }
        });
        
        // Start signal monitoring task
        tokio::spawn(async move {
            while let Some((name, value)) = rx.recv().await {
                let time = start_time.elapsed().as_secs_f64();
                
                // Update current values
                {
                    let mut current = current_values.lock().unwrap();
                    current.insert(name.clone(), value.clone());
                }
                
                // Update plot data
                if let Ok(mut data) = signal_data.lock() {
                    for signal in data.iter_mut() {
                        if signal.name == name {
                            if let Some(float_val) = value.as_float() {
                                signal.add_point(time, float_val);
                            } else if let Some(int_val) = value.as_int() {
                                signal.add_point(time, int_val as f64);
                            } else if let Some(bool_val) = value.as_bool() {
                                signal.add_point(time, if bool_val { 1.0 } else { 0.0 });
                            }
                        }
                    }
                }
            }
        });
        
        Ok(bus)
    }
}

impl eframe::App for PetraApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(100));
        
        if let Some(error) = &self.error_message {
            CentralPanel::default().show(ctx, |ui| {
                ui.colored_label(egui::Color32::RED, "Error:");
                ui.label(error);
                ui.separator();
                ui.label("Make sure configs/storage-test.yaml exists and is valid.");
                if ui.button("Retry").clicked() {
                    // Try to reinitialize
                    match Self::init_engine(self.signal_data.clone(), self.current_values.clone(), self.start_time) {
                        Ok(bus) => {
                            self.bus = Some(bus);
                            self.error_message = None;
                        }
                        Err(e) => {
                            self.error_message = Some(format!("Still failed: {}", e));
                        }
                    }
                }
            });
            return;
        }
        
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🔧 Petra PLC Dashboard");
                ui.separator();
                
                if ui.button(if self.running { "⏸ Pause" } else { "▶ Resume" }).clicked() {
                    self.running = !self.running;
                }
                
                ui.separator();
                ui.label(format!("Runtime: {:.1}s", self.start_time.elapsed().as_secs_f64()));
                
                ui.separator();
                if let Ok(values) = self.current_values.lock() {
                    ui.label(format!("Signals: {}", values.len()));
                }
            });
        });
        
        SidePanel::left("control_panel").min_width(200.0).show(ctx, |ui| {
            ui.heading("📊 Current Values");
            ui.separator();
            
            if let Ok(values) = self.current_values.lock() {
                for (name, value) in values.iter() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", name));
                        match value {
                            Value::Bool(b) => {
                                ui.colored_label(
                                    if *b { egui::Color32::GREEN } else { egui::Color32::RED },
                                    format!("{}", b)
                                );
                            }
                            Value::Int(i) => {
                                ui.label(format!("{}", i));
                            }
                            Value::Float(f) => {
                                ui.label(format!("{:.3}", f));
                            }
                        }
                    });
                }
            }
            
            ui.separator();
            ui.heading("🔧 Controls");
            
            if let Some(bus) = &self.bus {
                if ui.button("Reset Generator").clicked() {
                    if let Err(e) = bus.set("test_input", Value::Float(0.0)) {
                        warn!("Failed to reset generator: {}", e);
                    }
                }
                
                if ui.button("Toggle Generator").clicked() {
                    let current = bus.get_bool("data_generator").unwrap_or(false);
                    if let Err(e) = bus.set("data_generator", Value::Bool(!current)) {
                        warn!("Failed to toggle generator: {}", e);
                    }
                }
                
                ui.separator();
                
                if ui.button("📁 Open Data Folder").clicked() {
                    if let Err(e) = std::process::Command::new("explorer")
                        .arg("data\\storage_test")
                        .spawn()
                    {
                        // Try different commands for different OS
                        let _ = std::process::Command::new("open")
                            .arg("data/storage_test")
                            .spawn()
                            .or_else(|_| std::process::Command::new("xdg-open")
                                .arg("data/storage_test")
                                .spawn()
                            );
                    }
                }
            }
        });
        
        CentralPanel::default().show(ctx, |ui| {
            ui.heading("📈 Real-time Signal Plot");
            
            if let Ok(data) = self.signal_data.lock() {
                Plot::new("signal_plot")
                    .height(400.0)
                    .legend(egui_plot::Legend::default())
                    .show(ui, |plot_ui| {
                        for signal in data.iter() {
                            if !signal.values.is_empty() {
                                let line = Line::new(signal.to_plot_points())
                                    .color(signal.color)
                                    .name(&signal.name);
                                plot_ui.line(line);
                            }
                        }
                    });
            }
            
            ui.separator();
            
            // Block status display
            ui.heading("🧱 Block Status");
            if let Some(bus) = &self.bus {
                ui.horizontal(|ui| {
                    ui.label("Data Generator:");
                    let enabled = bus.get_bool("data_generator").unwrap_or(false);
                    ui.colored_label(
                        if enabled { egui::Color32::GREEN } else { egui::Color32::RED },
                        if enabled { "RUNNING" } else { "STOPPED" }
                    );
                });
                
                ui.horizontal(|ui| {
                    ui.label("Sample Count:");
                    let count = bus.get_int("sample_count").unwrap_or(0);
                    ui.label(format!("{}", count));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Test Input:");
                    let input = bus.get_float("test_input").unwrap_or(0.0);
                    ui.label(format!("{:.3}", input));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Test Output:");
                    let output = bus.get_float("test_output").unwrap_or(0.0);
                    ui.label(format!("{:.3}", output));
                });
            }
        });
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter("petra_dashboard=info,petra=info")
        .init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Petra PLC Dashboard"),
        ..Default::default()
    };
    
    eframe::run_native(
        "Petra Dashboard",
        options,
        Box::new(|cc| Ok(PetraApp::new(cc))),
    )?;
    
    Ok(())
}
