//! Performance metrics collection
//!
//! This module provides system metrics collection and reporting.

use prometheus::{Counter, Gauge, Histogram, Registry};

#[derive(Clone)]
pub struct EngineMetrics {
    pub scan_duration: Histogram,
    pub block_executions: Counter,
    pub signal_updates: Counter,
    pub active_signals: Gauge,
    pub errors: Counter,
}

impl EngineMetrics {
    pub fn new(registry: &Registry) -> Result<Self, prometheus::Error> {
        let scan_duration = Histogram::with_opts(
            prometheus::HistogramOpts::new(
                "petra_scan_duration_seconds",
                "Scan cycle duration in seconds",
            )
            .buckets(vec![0.001, 0.005, 0.01, 0.05, 0.1, 0.5, 1.0]),
        )?;
        registry.register(Box::new(scan_duration.clone()))?;

        let block_executions = Counter::with_opts(
            prometheus::CounterOpts::new(
                "petra_block_executions_total",
                "Total number of block executions",
            ),
        )?;
        registry.register(Box::new(block_executions.clone()))?;

        let signal_updates = Counter::with_opts(
            prometheus::CounterOpts::new(
                "petra_signal_updates_total",
                "Total number of signal updates",
            ),
        )?;
        registry.register(Box::new(signal_updates.clone()))?;

        let active_signals = Gauge::with_opts(
            prometheus::GaugeOpts::new(
                "petra_active_signals",
                "Number of active signals",
            ),
        )?;
        registry.register(Box::new(active_signals.clone()))?;

        let errors = Counter::with_opts(
            prometheus::CounterOpts::new("petra_errors_total", "Total number of errors"),
        )?;
        registry.register(Box::new(errors.clone()))?;

        Ok(Self {
            scan_duration,
            block_executions,
            signal_updates,
            active_signals,
            errors,
        })
    }

    pub fn record_scan_duration(&self, duration: f64) {
        self.scan_duration.observe(duration);
    }

    pub fn increment_block_executions(&self) {
        self.block_executions.inc();
    }

    pub fn increment_signal_updates(&self) {
        self.signal_updates.inc();
    }

    pub fn set_active_signals(&self, count: f64) {
        self.active_signals.set(count);
    }

    pub fn increment_errors(&self) {
        self.errors.inc();
    }
}

pub fn create_metrics_registry() -> Registry {
    Registry::new()
}
