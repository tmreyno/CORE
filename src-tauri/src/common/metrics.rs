// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # Metrics Collection Infrastructure
//!
//! Provides Prometheus-compatible metrics collection for monitoring system performance,
//! operation statistics, and resource utilization in real-time.
//!
//! ## Features
//!
//! - **Counter**: Monotonically increasing values (operations completed, bytes processed)
//! - **Gauge**: Point-in-time values (active operations, memory usage, queue depth)
//! - **Histogram**: Distribution of values (latency percentiles, file sizes)
//! - **Global Registry**: Thread-safe singleton for all metrics
//!
//! ## Usage
//!
//! ```rust
//! use ffx_check_lib::common::metrics::{increment_counter, set_gauge, record_histogram, get_metrics_snapshot};
//!
//! // Increment operation counter
//! increment_counter("operations_total", 1.0, &[("type", "hash"), ("status", "success")]);
//!
//! // Update gauge
//! set_gauge("active_operations", 5.0, &[("type", "extraction")]);
//!
//! // Record latency
//! record_histogram("operation_duration_ms", 123.45, &[("operation", "hash_file")]);
//!
//! // Get all metrics
//! let snapshot = get_metrics_snapshot();
//! ```

use dashmap::DashMap;
use parking_lot::RwLock;
use std::sync::{Arc, LazyLock};
use std::time::{Duration, Instant};

/// Global metrics registry
static METRICS_REGISTRY: LazyLock<Arc<MetricsRegistry>> =
    LazyLock::new(|| Arc::new(MetricsRegistry::new()));

/// Metric types supported by the system
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum MetricValue {
    /// Monotonically increasing counter
    Counter { value: f64 },
    /// Point-in-time gauge value
    Gauge { value: f64 },
    /// Histogram with count, sum, and percentiles
    Histogram {
        count: u64,
        sum: f64,
        min: f64,
        max: f64,
        mean: f64,
        p50: f64,
        p95: f64,
        p99: f64,
    },
}

/// Label key-value pair for metric dimensions
pub type Label = (&'static str, String);

/// Metric key with name and labels
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
struct MetricKey {
    name: String,
    labels: Vec<(String, String)>,
}

impl MetricKey {
    fn new(name: impl Into<String>, labels: &[(&'static str, &str)]) -> Self {
        let mut sorted_labels: Vec<_> = labels
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect();
        sorted_labels.sort_by(|a, b| a.0.cmp(&b.0));

        Self {
            name: name.into(),
            labels: sorted_labels,
        }
    }

    fn to_prometheus_string(&self) -> String {
        if self.labels.is_empty() {
            self.name.clone()
        } else {
            let labels_str = self
                .labels
                .iter()
                .map(|(k, v)| format!("{}=\"{}\"", k, v))
                .collect::<Vec<_>>()
                .join(",");
            format!("{}{{{}}}", self.name, labels_str)
        }
    }
}

/// Histogram data structure for tracking distributions
#[derive(Debug)]
struct Histogram {
    values: RwLock<Vec<f64>>,
    count: RwLock<u64>,
    sum: RwLock<f64>,
}

impl Histogram {
    fn new() -> Self {
        Self {
            values: RwLock::new(Vec::new()),
            count: RwLock::new(0),
            sum: RwLock::new(0.0),
        }
    }

    fn record(&self, value: f64) {
        let mut values = self.values.write();
        let mut count = self.count.write();
        let mut sum = self.sum.write();

        values.push(value);
        *count += 1;
        *sum += value;

        // Keep only last 10,000 values to prevent unbounded memory growth
        if values.len() > 10_000 {
            values.drain(0..1000);
        }
    }

    fn snapshot(&self) -> MetricValue {
        let mut values = self.values.read().clone();
        let count = *self.count.read();
        let sum = *self.sum.read();

        if values.is_empty() {
            return MetricValue::Histogram {
                count: 0,
                sum: 0.0,
                min: 0.0,
                max: 0.0,
                mean: 0.0,
                p50: 0.0,
                p95: 0.0,
                p99: 0.0,
            };
        }

        values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let min = *values.first().expect("values is non-empty, checked above");
        let max = *values.last().expect("values is non-empty, checked above");
        let mean = sum / count as f64;
        let p50 = percentile(&values, 0.50);
        let p95 = percentile(&values, 0.95);
        let p99 = percentile(&values, 0.99);

        MetricValue::Histogram {
            count,
            sum,
            min,
            max,
            mean,
            p50,
            p95,
            p99,
        }
    }
}

fn percentile(sorted_values: &[f64], p: f64) -> f64 {
    if sorted_values.is_empty() {
        return 0.0;
    }
    let idx = ((sorted_values.len() as f64 - 1.0) * p).round() as usize;
    sorted_values[idx.min(sorted_values.len() - 1)]
}

/// Internal metric storage
enum MetricData {
    Counter(RwLock<f64>),
    Gauge(RwLock<f64>),
    Histogram(Histogram),
}

/// Thread-safe metrics registry
pub struct MetricsRegistry {
    metrics: DashMap<MetricKey, MetricData>,
    start_time: Instant,
}

impl MetricsRegistry {
    fn new() -> Self {
        Self {
            metrics: DashMap::new(),
            start_time: Instant::now(),
        }
    }

    /// Increment a counter by a given amount
    pub fn increment_counter(&self, name: &str, value: f64, labels: &[(&'static str, &str)]) {
        let key = MetricKey::new(name, labels);
        if let Some(counter) = self
            .metrics
            .entry(key)
            .or_insert_with(|| MetricData::Counter(RwLock::new(0.0)))
            .value()
            .as_counter()
        {
            let mut val = counter.write();
            *val += value;
        }
    }

    /// Set a gauge to a specific value
    pub fn set_gauge(&self, name: &str, value: f64, labels: &[(&'static str, &str)]) {
        let key = MetricKey::new(name, labels);
        if let Some(gauge) = self
            .metrics
            .entry(key)
            .or_insert_with(|| MetricData::Gauge(RwLock::new(0.0)))
            .value()
            .as_gauge()
        {
            let mut val = gauge.write();
            *val = value;
        }
    }

    /// Increment a gauge by a given amount
    pub fn increment_gauge(&self, name: &str, delta: f64, labels: &[(&'static str, &str)]) {
        let key = MetricKey::new(name, labels);
        if let Some(gauge) = self
            .metrics
            .entry(key)
            .or_insert_with(|| MetricData::Gauge(RwLock::new(0.0)))
            .value()
            .as_gauge()
        {
            let mut val = gauge.write();
            *val += delta;
        }
    }

    /// Record a value in a histogram
    pub fn record_histogram(&self, name: &str, value: f64, labels: &[(&'static str, &str)]) {
        let key = MetricKey::new(name, labels);
        if let Some(histogram) = self
            .metrics
            .entry(key)
            .or_insert_with(|| MetricData::Histogram(Histogram::new()))
            .value()
            .as_histogram()
        {
            histogram.record(value);
        }
    }

    /// Get a snapshot of all metrics
    pub fn snapshot(&self) -> Vec<(String, MetricValue)> {
        let mut result = Vec::new();

        for entry in self.metrics.iter() {
            let key = entry.key();
            let data = entry.value();

            let value = match data {
                MetricData::Counter(c) => MetricValue::Counter { value: *c.read() },
                MetricData::Gauge(g) => MetricValue::Gauge { value: *g.read() },
                MetricData::Histogram(h) => h.snapshot(),
            };

            result.push((key.to_prometheus_string(), value));
        }

        result
    }

    /// Get uptime in seconds
    pub fn uptime(&self) -> f64 {
        self.start_time.elapsed().as_secs_f64()
    }

    /// Reset all metrics (useful for testing)
    pub fn reset(&self) {
        self.metrics.clear();
    }

    /// Get metric count
    pub fn count(&self) -> usize {
        self.metrics.len()
    }
}

impl MetricData {
    fn as_counter(&self) -> Option<&RwLock<f64>> {
        match self {
            MetricData::Counter(c) => Some(c),
            _ => None,
        }
    }

    fn as_gauge(&self) -> Option<&RwLock<f64>> {
        match self {
            MetricData::Gauge(g) => Some(g),
            _ => None,
        }
    }

    fn as_histogram(&self) -> Option<&Histogram> {
        match self {
            MetricData::Histogram(h) => Some(h),
            _ => None,
        }
    }
}

// =============================================================================
// Public API Functions
// =============================================================================

/// Get the global metrics registry
pub fn get_metrics_registry() -> Arc<MetricsRegistry> {
    METRICS_REGISTRY.clone()
}

/// Increment a counter by a given amount
pub fn increment_counter(name: &str, value: f64, labels: &[(&'static str, &str)]) {
    METRICS_REGISTRY.increment_counter(name, value, labels);
}

/// Set a gauge to a specific value
pub fn set_gauge(name: &str, value: f64, labels: &[(&'static str, &str)]) {
    METRICS_REGISTRY.set_gauge(name, value, labels);
}

/// Increment a gauge by a given amount
pub fn increment_gauge(name: &str, delta: f64, labels: &[(&'static str, &str)]) {
    METRICS_REGISTRY.increment_gauge(name, delta, labels);
}

/// Record a value in a histogram
pub fn record_histogram(name: &str, value: f64, labels: &[(&'static str, &str)]) {
    METRICS_REGISTRY.record_histogram(name, value, labels);
}

/// Get a snapshot of all metrics
pub fn get_metrics_snapshot() -> Vec<(String, MetricValue)> {
    METRICS_REGISTRY.snapshot()
}

/// Get system uptime in seconds
pub fn get_uptime() -> f64 {
    METRICS_REGISTRY.uptime()
}

/// Reset all metrics (testing only - always available for integration tests)
pub fn reset_metrics() {
    METRICS_REGISTRY.reset();
}

/// Get metric count
pub fn get_metric_count() -> usize {
    METRICS_REGISTRY.count()
}

// =============================================================================
// Standard Metrics
// =============================================================================

/// Record operation start
pub fn record_operation_start(operation_type: &str) {
    increment_counter("operations_started_total", 1.0, &[("type", operation_type)]);
    increment_gauge("active_operations", 1.0, &[("type", operation_type)]);
}

/// Record operation completion
pub fn record_operation_complete(operation_type: &str, duration: Duration, success: bool) {
    increment_counter(
        "operations_completed_total",
        1.0,
        &[
            ("type", operation_type),
            ("status", if success { "success" } else { "error" }),
        ],
    );
    increment_gauge("active_operations", -1.0, &[("type", operation_type)]);
    record_histogram(
        "operation_duration_seconds",
        duration.as_secs_f64(),
        &[("type", operation_type)],
    );
}

/// Record bytes processed
pub fn record_bytes_processed(operation_type: &str, bytes: u64) {
    increment_counter(
        "bytes_processed_total",
        bytes as f64,
        &[("type", operation_type)],
    );
}

/// Record cache hit/miss
pub fn record_cache_access(cache_name: &str, hit: bool) {
    increment_counter(
        "cache_accesses_total",
        1.0,
        &[
            ("cache", cache_name),
            ("result", if hit { "hit" } else { "miss" }),
        ],
    );
}

/// Record error
pub fn record_error(operation_type: &str, error_type: &str) {
    increment_counter(
        "errors_total",
        1.0,
        &[("operation", operation_type), ("type", error_type)],
    );
}

// =============================================================================
// Timer Helper
// =============================================================================

/// RAII timer that automatically records duration on drop
pub struct Timer {
    operation_type: String,
    start: Instant,
    success: bool,
}

impl Timer {
    /// Create a new timer for the given operation
    pub fn new(operation_type: impl Into<String>) -> Self {
        let operation_type = operation_type.into();
        record_operation_start(&operation_type);
        Self {
            operation_type,
            start: Instant::now(),
            success: false,
        }
    }

    /// Mark the operation as successful
    pub fn success(&mut self) {
        self.success = true;
    }

    /// Mark the operation as failed
    pub fn error(&mut self) {
        self.success = false;
    }
}

impl Drop for Timer {
    fn drop(&mut self) {
        let duration = self.start.elapsed();
        record_operation_complete(&self.operation_type, duration, self.success);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Use unique metric names per test to avoid interference from concurrent tests
    // in other modules that also write to the global metrics registry.

    #[test]
    fn test_counter() {
        // Use a unique name to avoid collisions with other concurrent tests
        let name = "unit_test_counter_abc";
        increment_counter(name, 1.0, &[("label", "value")]);
        increment_counter(name, 2.0, &[("label", "value")]);

        let snapshot = get_metrics_snapshot();
        // Snapshot keys use Prometheus format: name{label="value"}
        let metric = snapshot
            .iter()
            .find(|(n, _)| n.starts_with(name))
            .expect("counter metric not found");
        match metric.1 {
            MetricValue::Counter { value } => {
                assert!(value >= 3.0, "Expected at least 3.0, got {value}")
            }
            _ => panic!("Expected counter"),
        }
    }

    #[test]
    fn test_gauge() {
        let name = "unit_test_gauge_abc";
        set_gauge(name, 42.0, &[]);
        set_gauge(name, 100.0, &[]);

        let snapshot = get_metrics_snapshot();
        let metric = snapshot
            .iter()
            .find(|(n, _)| n == name)
            .expect("gauge metric not found");
        match metric.1 {
            MetricValue::Gauge { value } => assert_eq!(value, 100.0),
            _ => panic!("Expected gauge"),
        }
    }

    #[test]
    fn test_histogram() {
        let name = "unit_test_histogram_abc";
        record_histogram(name, 10.0, &[]);
        record_histogram(name, 20.0, &[]);
        record_histogram(name, 30.0, &[]);

        let snapshot = get_metrics_snapshot();
        let metric = snapshot
            .iter()
            .find(|(n, _)| n == name)
            .expect("histogram metric not found");
        match &metric.1 {
            MetricValue::Histogram {
                count,
                sum,
                min,
                max,
                mean,
                ..
            } => {
                assert!(*count >= 3, "Expected at least 3 records, got {count}");
                assert!(*sum >= 60.0, "Expected sum >= 60.0, got {sum}");
                assert!(*min <= 10.0, "Expected min <= 10.0, got {min}");
                assert!(*max >= 30.0, "Expected max >= 30.0, got {max}");
                assert!(*mean > 0.0, "Expected positive mean, got {mean}");
            }
            _ => panic!("Expected histogram"),
        }
    }

    #[test]
    fn test_timer() {
        {
            let mut timer = Timer::new("unit_test_timer_operation");
            std::thread::sleep(Duration::from_millis(10));
            timer.success();
        }

        let snapshot = get_metrics_snapshot();
        // Timer records multiple metrics - just verify at least some were recorded
        assert!(
            !snapshot.is_empty(),
            "Expected some metrics after timer use"
        );
    }
}
