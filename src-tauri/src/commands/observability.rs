// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # Observability Tauri Commands
//!
//! Exposes metrics, health checks, and tracing configuration to the frontend.

use crate::common::{health, metrics, tracing_setup};
use serde::{Deserialize, Serialize};
use tauri::command;

// =============================================================================
// Metrics Commands
// =============================================================================

/// Get current metrics snapshot
#[command]
pub async fn get_metrics() -> Result<Vec<MetricEntry>, String> {
    let snapshot = metrics::get_metrics_snapshot();
    Ok(snapshot
        .into_iter()
        .map(|(name, value)| MetricEntry { name, value })
        .collect())
}

/// Get specific metric by name
#[command]
pub async fn get_metric(name: String) -> Result<Option<MetricEntry>, String> {
    let snapshot = metrics::get_metrics_snapshot();
    Ok(snapshot
        .into_iter()
        .find(|(n, _)| n == &name)
        .map(|(name, value)| MetricEntry { name, value }))
}

/// Increment a counter metric
#[command]
pub async fn increment_counter(name: String, amount: f64) -> Result<(), String> {
    metrics::increment_counter(&name, amount, &[]);
    Ok(())
}

/// Set a gauge metric value
#[command]
pub async fn set_gauge(name: String, value: f64) -> Result<(), String> {
    metrics::set_gauge(&name, value, &[]);
    Ok(())
}

/// Record a value in a histogram metric
#[command]
pub async fn record_histogram(name: String, value: f64) -> Result<(), String> {
    metrics::record_histogram(&name, value, &[]);
    Ok(())
}

/// Export all metrics as JSON
#[command]
pub async fn export_metrics() -> Result<String, String> {
    let snapshot = metrics::get_metrics_snapshot();
    serde_json::to_string_pretty(&snapshot).map_err(|e| e.to_string())
}

/// Reset all metrics (testing/development only)
#[command]
pub async fn reset_metrics() -> Result<(), String> {
    metrics::reset_metrics();
    Ok(())
}

/// Get system uptime
#[command]
pub async fn get_system_uptime() -> Result<f64, String> {
    Ok(metrics::get_uptime())
}

/// Get metrics count
#[command]
pub async fn get_metrics_count() -> Result<usize, String> {
    Ok(metrics::get_metric_count())
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricEntry {
    pub name: String,
    pub value: metrics::MetricValue,
}

// =============================================================================
// Health Check Commands
// =============================================================================

/// Get system health status
#[command]
pub async fn get_health() -> Result<health::SystemHealth, String> {
    Ok(health::get_system_health())
}

/// Get system health with custom thresholds
#[command]
pub async fn get_health_with_thresholds(
    thresholds: HealthThresholdsInput,
) -> Result<health::SystemHealth, String> {
    let thresholds = health::HealthThresholds {
        cpu_warning: thresholds.cpu_warning,
        cpu_critical: thresholds.cpu_critical,
        memory_warning: thresholds.memory_warning,
        memory_critical: thresholds.memory_critical,
        disk_warning: thresholds.disk_warning,
        disk_critical: thresholds.disk_critical,
        queue_depth_warning: thresholds.queue_depth_warning,
        queue_depth_critical: thresholds.queue_depth_critical,
        error_rate_warning: thresholds.error_rate_warning,
        error_rate_critical: thresholds.error_rate_critical,
    };
    Ok(health::get_system_health_with_thresholds(&thresholds))
}

/// Check if system is healthy
#[command]
pub async fn is_system_healthy() -> Result<bool, String> {
    let health = health::get_system_health();
    Ok(health.status == health::HealthStatus::Healthy)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HealthThresholdsInput {
    pub cpu_warning: f64,
    pub cpu_critical: f64,
    pub memory_warning: f64,
    pub memory_critical: f64,
    pub disk_warning: f64,
    pub disk_critical: f64,
    pub queue_depth_warning: usize,
    pub queue_depth_critical: usize,
    pub error_rate_warning: f64,
    pub error_rate_critical: f64,
}

impl Default for HealthThresholdsInput {
    fn default() -> Self {
        let defaults = health::HealthThresholds::default();
        Self {
            cpu_warning: defaults.cpu_warning,
            cpu_critical: defaults.cpu_critical,
            memory_warning: defaults.memory_warning,
            memory_critical: defaults.memory_critical,
            disk_warning: defaults.disk_warning,
            disk_critical: defaults.disk_critical,
            queue_depth_warning: defaults.queue_depth_warning,
            queue_depth_critical: defaults.queue_depth_critical,
            error_rate_warning: defaults.error_rate_warning,
            error_rate_critical: defaults.error_rate_critical,
        }
    }
}

// =============================================================================
// Tracing Commands
// =============================================================================

/// Initialize tracing with specified log level and directory
#[command]
pub async fn init_tracing(level: String, log_dir: String) -> Result<(), String> {
    tracing_setup::init_tracing(&level, &log_dir).map_err(|e| e.to_string())
}

/// Get default log directory
#[command]
pub async fn get_default_log_dir() -> Result<String, String> {
    tracing_setup::default_log_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| e.to_string())
}

/// Parse log level string
#[command]
pub async fn parse_log_level(level: String) -> Result<tracing_setup::LogLevel, String> {
    level.parse::<tracing_setup::LogLevel>().map_err(|e| e.to_string())
}

// =============================================================================
// Combined Status Command
// =============================================================================

/// Get comprehensive system status (metrics + health)
#[command]
pub async fn get_system_status() -> Result<SystemStatus, String> {
    let health = health::get_system_health();
    let metrics_count = metrics::get_metric_count();
    let uptime = metrics::get_uptime();

    Ok(SystemStatus {
        health,
        metrics_count,
        uptime_seconds: uptime,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemStatus {
    pub health: health::SystemHealth,
    pub metrics_count: usize,
    pub uptime_seconds: f64,
}
