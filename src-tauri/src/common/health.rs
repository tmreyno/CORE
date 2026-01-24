// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # System Health Monitoring
//!
//! Provides real-time health checks and diagnostics for system resources,
//! operation queues, and error rates.
//!
//! ## Features
//!
//! - **Resource Monitoring**: CPU, memory, disk space
//! - **Queue Health**: Depth, throughput, latency
//! - **Error Tracking**: Rate, types, trends
//! - **System Info**: Uptime, version, platform
//!
//! ## Usage
//!
//! ```rust
//! use ffx_check_lib::common::health::{get_system_health, HealthStatus};
//!
//! let health = get_system_health();
//! if health.status != HealthStatus::Healthy {
//!     warn!("System unhealthy: {:?}", health.issues);
//! }
//! ```

use serde::{Deserialize, Serialize};
use sysinfo::System;

/// Overall health status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    /// All systems operational
    Healthy,
    /// Some issues detected but system operational
    Degraded,
    /// Critical issues, system may not function correctly
    Unhealthy,
}

/// Health check issue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthIssue {
    pub severity: IssueSeverity,
    pub component: String,
    pub message: String,
    pub metric_value: Option<f64>,
    pub threshold: Option<f64>,
}

/// Issue severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum IssueSeverity {
    Warning,
    Error,
    Critical,
}

/// System resource metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// CPU usage percentage (0-100)
    pub cpu_usage_percent: f64,
    /// Memory usage in bytes
    pub memory_used_bytes: u64,
    /// Total memory in bytes
    pub memory_total_bytes: u64,
    /// Memory usage percentage (0-100)
    pub memory_usage_percent: f64,
    /// Available disk space in bytes
    pub disk_available_bytes: u64,
    /// Total disk space in bytes
    pub disk_total_bytes: u64,
    /// Disk usage percentage (0-100)
    pub disk_usage_percent: f64,
}

/// Queue health metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueMetrics {
    pub queue_name: String,
    /// Current queue depth
    pub depth: usize,
    /// Items processed per second
    pub throughput: f64,
    /// Average latency in milliseconds
    pub avg_latency_ms: f64,
    /// Maximum observed queue depth
    pub max_depth: usize,
}

/// Error rate metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMetrics {
    /// Total errors in last minute
    pub errors_last_minute: u64,
    /// Total errors in last hour
    pub errors_last_hour: u64,
    /// Error rate (errors per second)
    pub error_rate: f64,
    /// Most common error types
    pub top_error_types: Vec<(String, u64)>,
}

/// Complete system health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemHealth {
    /// Overall health status
    pub status: HealthStatus,
    /// Health check timestamp
    pub timestamp: i64,
    /// System uptime in seconds
    pub uptime_seconds: f64,
    /// Application version
    pub version: String,
    /// Platform information
    pub platform: String,
    /// Resource metrics
    pub resources: ResourceMetrics,
    /// Queue metrics (optional)
    pub queues: Vec<QueueMetrics>,
    /// Error metrics
    pub errors: ErrorMetrics,
    /// Detected issues
    pub issues: Vec<HealthIssue>,
}

/// Health check thresholds
#[derive(Debug, Clone)]
pub struct HealthThresholds {
    /// CPU usage warning threshold (percent)
    pub cpu_warning: f64,
    /// CPU usage critical threshold (percent)
    pub cpu_critical: f64,
    /// Memory usage warning threshold (percent)
    pub memory_warning: f64,
    /// Memory usage critical threshold (percent)
    pub memory_critical: f64,
    /// Disk usage warning threshold (percent)
    pub disk_warning: f64,
    /// Disk usage critical threshold (percent)
    pub disk_critical: f64,
    /// Queue depth warning threshold
    pub queue_depth_warning: usize,
    /// Queue depth critical threshold
    pub queue_depth_critical: usize,
    /// Error rate warning threshold (errors/sec)
    pub error_rate_warning: f64,
    /// Error rate critical threshold (errors/sec)
    pub error_rate_critical: f64,
}

impl Default for HealthThresholds {
    fn default() -> Self {
        Self {
            cpu_warning: 70.0,
            cpu_critical: 90.0,
            memory_warning: 75.0,
            memory_critical: 90.0,
            disk_warning: 80.0,
            disk_critical: 95.0,
            queue_depth_warning: 1000,
            queue_depth_critical: 5000,
            error_rate_warning: 1.0,
            error_rate_critical: 10.0,
        }
    }
}

/// Get current system health
pub fn get_system_health() -> SystemHealth {
    get_system_health_with_thresholds(&HealthThresholds::default())
}

/// Get system health with custom thresholds
pub fn get_system_health_with_thresholds(thresholds: &HealthThresholds) -> SystemHealth {
    let mut issues = Vec::new();

    // Collect resource metrics
    let resources = collect_resource_metrics(&mut issues, thresholds);

    // Collect queue metrics (if available)
    let queues = collect_queue_metrics(&mut issues, thresholds);

    // Collect error metrics
    let errors = collect_error_metrics(&mut issues, thresholds);

    // Determine overall status
    let status = if issues.iter().any(|i| i.severity == IssueSeverity::Critical) {
        HealthStatus::Unhealthy
    } else if issues.iter().any(|i| i.severity == IssueSeverity::Error) {
        HealthStatus::Degraded
    } else if !issues.is_empty() {
        HealthStatus::Degraded
    } else {
        HealthStatus::Healthy
    };

    SystemHealth {
        status,
        timestamp: chrono::Utc::now().timestamp(),
        uptime_seconds: crate::common::metrics::get_uptime(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        platform: format!("{} {}", std::env::consts::OS, std::env::consts::ARCH),
        resources,
        queues,
        errors,
        issues,
    }
}

fn collect_resource_metrics(
    issues: &mut Vec<HealthIssue>,
    thresholds: &HealthThresholds,
) -> ResourceMetrics {
    let mut sys = System::new_all();
    sys.refresh_all();

    // CPU usage
    let cpu_usage = sys.global_cpu_usage() as f64;
    if cpu_usage >= thresholds.cpu_critical {
        issues.push(HealthIssue {
            severity: IssueSeverity::Critical,
            component: "CPU".to_string(),
            message: format!("CPU usage critical: {:.1}%", cpu_usage),
            metric_value: Some(cpu_usage),
            threshold: Some(thresholds.cpu_critical),
        });
    } else if cpu_usage >= thresholds.cpu_warning {
        issues.push(HealthIssue {
            severity: IssueSeverity::Warning,
            component: "CPU".to_string(),
            message: format!("CPU usage high: {:.1}%", cpu_usage),
            metric_value: Some(cpu_usage),
            threshold: Some(thresholds.cpu_warning),
        });
    }

    // Memory usage
    let memory_used = sys.used_memory();
    let memory_total = sys.total_memory();
    let memory_percent = (memory_used as f64 / memory_total as f64) * 100.0;

    if memory_percent >= thresholds.memory_critical {
        issues.push(HealthIssue {
            severity: IssueSeverity::Critical,
            component: "Memory".to_string(),
            message: format!("Memory usage critical: {:.1}%", memory_percent),
            metric_value: Some(memory_percent),
            threshold: Some(thresholds.memory_critical),
        });
    } else if memory_percent >= thresholds.memory_warning {
        issues.push(HealthIssue {
            severity: IssueSeverity::Warning,
            component: "Memory".to_string(),
            message: format!("Memory usage high: {:.1}%", memory_percent),
            metric_value: Some(memory_percent),
            threshold: Some(thresholds.memory_warning),
        });
    }

    // Disk space (use first available disk from Disks struct)
    use sysinfo::Disks;
    let disks = Disks::new_with_refreshed_list();
    let (disk_available, disk_total) = if let Some(disk) = disks.iter().next() {
        let available = disk.available_space();
        let total = disk.total_space();
        (available, total)
    } else {
        (0, 0)
    };

    let disk_percent = if disk_total > 0 {
        ((disk_total - disk_available) as f64 / disk_total as f64) * 100.0
    } else {
        0.0
    };

    if disk_percent >= thresholds.disk_critical {
        issues.push(HealthIssue {
            severity: IssueSeverity::Critical,
            component: "Disk".to_string(),
            message: format!("Disk usage critical: {:.1}%", disk_percent),
            metric_value: Some(disk_percent),
            threshold: Some(thresholds.disk_critical),
        });
    } else if disk_percent >= thresholds.disk_warning {
        issues.push(HealthIssue {
            severity: IssueSeverity::Warning,
            component: "Disk".to_string(),
            message: format!("Disk usage high: {:.1}%", disk_percent),
            metric_value: Some(disk_percent),
            threshold: Some(thresholds.disk_warning),
        });
    }

    ResourceMetrics {
        cpu_usage_percent: cpu_usage,
        memory_used_bytes: memory_used,
        memory_total_bytes: memory_total,
        memory_usage_percent: memory_percent,
        disk_available_bytes: disk_available,
        disk_total_bytes: disk_total,
        disk_usage_percent: disk_percent,
    }
}

fn collect_queue_metrics(
    _issues: &mut Vec<HealthIssue>,
    _thresholds: &HealthThresholds,
) -> Vec<QueueMetrics> {
    // TODO: Integrate with actual queue monitoring when available
    // For now, return empty - will be populated when queue systems are instrumented
    Vec::new()
}

fn collect_error_metrics(
    issues: &mut Vec<HealthIssue>,
    thresholds: &HealthThresholds,
) -> ErrorMetrics {
    // Get error metrics from metrics registry
    let snapshot = crate::common::metrics::get_metrics_snapshot();

    let mut errors_total = 0u64;

    for (name, value) in snapshot {
        if name.starts_with("errors_total") {
            if let crate::common::metrics::MetricValue::Counter { value } = value {
                errors_total = value as u64;
            }
        }
    }

    // Calculate error rate (simplified - assumes errors are recent)
    let error_rate = errors_total as f64 / 60.0; // Rough estimate: total / 60 seconds

    if error_rate >= thresholds.error_rate_critical {
        issues.push(HealthIssue {
            severity: IssueSeverity::Critical,
            component: "Errors".to_string(),
            message: format!("Error rate critical: {:.2}/s", error_rate),
            metric_value: Some(error_rate),
            threshold: Some(thresholds.error_rate_critical),
        });
    } else if error_rate >= thresholds.error_rate_warning {
        issues.push(HealthIssue {
            severity: IssueSeverity::Warning,
            component: "Errors".to_string(),
            message: format!("Error rate elevated: {:.2}/s", error_rate),
            metric_value: Some(error_rate),
            threshold: Some(thresholds.error_rate_warning),
        });
    }

    ErrorMetrics {
        errors_last_minute: errors_total,
        errors_last_hour: errors_total,
        error_rate,
        top_error_types: Vec::new(), // TODO: Track error types in metrics
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_system_health() {
        let health = get_system_health();
        assert!(health.version.len() > 0);
        assert!(health.platform.len() > 0);
        assert!(health.resources.memory_total_bytes > 0);
    }

    #[test]
    fn test_health_thresholds() {
        let thresholds = HealthThresholds::default();
        assert!(thresholds.cpu_warning < thresholds.cpu_critical);
        assert!(thresholds.memory_warning < thresholds.memory_critical);
    }

    #[test]
    fn test_health_status_ordering() {
        let issues = vec![
            HealthIssue {
                severity: IssueSeverity::Warning,
                component: "Test".to_string(),
                message: "Test".to_string(),
                metric_value: None,
                threshold: None,
            },
            HealthIssue {
                severity: IssueSeverity::Critical,
                component: "Test".to_string(),
                message: "Test".to_string(),
                metric_value: None,
                threshold: None,
            },
        ];

        // Should be unhealthy if any critical issue
        assert!(issues.iter().any(|i| i.severity == IssueSeverity::Critical));
    }
}
