// src/bin/petra_dashboard.rs
use eframe::egui::{self, Context, CentralPanel, SidePanel, TopBottomPanel};
use egui_plot::{Line, Plot, PlotPoints, PlotPoint};
use petra::{Config, Engine, SignalBus, Value, Result};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{info, error};
use tracing_subscriber;

const MAX_POINTS: usize = 1000;

#[derive(Clone)]
struct SignalData {
    name: String,
    values: VecDeque<(f64, f64)>, // (time, value)
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
    bus: SignalBus,
    signal_data: Arc<Mutex<Vec<SignalData>>>,
    start_time: Instant,
    running: bool,
    current_values: Arc<Mutex<std::collections::HashMap<String, Value>>>,
}

impl PetraApp {
    fn new(_cc: &eframe::CreationContext<'_>) -> Box<Self> {
        // Initialize tracing
        tracing_subscriber::fmt()
            .with_env_filter("petra_dashboard=info,petra=info")
            .init();

        // Load config and start engine
        let config = Config::from_file("configs/storage-test.yaml")
            .expect("Failed to load config");
        
        let mut engine = Engine::new(config.clone())
            .expect("Failed to create engine");
        
        let bus = engine.bus().clone();
        
        // Set up signal change tracking
        let (tx, mut rx) = mpsc::channel(1000);
        engine.set_signal_change_channel(tx);
        
        // Initialize signal data for plotting
        let signal_data = Arc::new(Mutex::new(vec![
            SignalData::new("test_input".to_string(), egui::Color32::BLUE),
            SignalData::new("test_output".to_string(), egui::Color32::RED),
        ]));
        
        let current_values = Arc::new(Mutex::new(std::collections::HashMap::new()));
        
        // Start engine in background
        let signal_data_clone = signal_data.clone();
        let current_values_clone = current_values.clone();
        let start_time = Instant::now();
        
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
                    let mut current = current_values_clone.lock().unwrap();
                    current.insert(name.clone(), value.clone());
                }
                
                // Update plot data
                if let Ok(mut data) = signal_data_clone.lock() {
                    for signal in data.iter_mut() {
                        if signal.name == name {
                            if let Some(float_val) = value.as_float() {
                                signal.add_point(time, float_val);
                            }
                        }
                    }
                }
            }
        });
        
        Box::new(Self {
            bus,
            signal_data,
            start_time,
            running: true,
            current_values,
        })
    }
}

impl eframe::App for PetraApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        // Request repaint for real-time updates
        ctx.request_repaint_after(Duration::from_millis(50));
        
        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.heading("🔧 Petra PLC Dashboard");
                ui.separator();
                
                if ui.button(if self.running { "⏸ Pause" } else { "▶ Resume" }).clicked() {
                    self.running = !self.running;
                }
                
                ui.separator();
                ui.label(format!("Runtime: {:.1}s", self.start_time.elapsed().as_secs_f64()));
            });
        });
        
        SidePanel::left("control_panel").show(ctx, |ui| {
            ui.heading("📊 Current Values");
            ui.separator();
            
            if let Ok(values) = self.current_values.lock() {
                for (name, value) in values.iter() {
                    ui.horizontal(|ui| {
                        ui.label(format!("{}:", name));
                        ui.label(match value {
                            Value::Bool(b) => format!("{}", b),
                            Value::Int(i) => format!("{}", i),
                            Value::Float(f) => format!("{:.3}", f),
                        });
                    });
                }
            }
            
            ui.separator();
            ui.heading("🔧 Controls");
            
            // Manual signal controls
            ui.horizontal(|ui| {
                if ui.button("Reset Generator").clicked() {
                    let _ = self.bus.set("test_input", Value::Float(0.0));
                }
            });
            
            ui.horizontal(|ui| {
                if ui.button("Toggle Generator").clicked() {
                    let current = self.bus.get_bool("data_generator").unwrap_or(false);
                    let _ = self.bus.set("data_generator", Value::Bool(!current));
                }
            });
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
            ui.horizontal(|ui| {
                ui.label("Data Generator:");
                let enabled = self.bus.get_bool("data_generator").unwrap_or(false);
                ui.colored_label(
                    if enabled { egui::Color32::GREEN } else { egui::Color32::RED },
                    if enabled { "RUNNING" } else { "STOPPED" }
                );
            });
            
            ui.horizontal(|ui| {
                ui.label("Sample Count:");
                let count = self.bus.get_int("sample_count").unwrap_or(0);
                ui.label(format!("{}", count));
            });
        });
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    let _guard = rt.enter();
    
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
