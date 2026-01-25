// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # Performance Regression Testing Module
//!
//! Provides automated performance regression detection with baseline comparison,
//! statistical analysis, and trend monitoring.

use chrono::{DateTime, Utc};
use ordered_float::OrderedFloat;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock};
use thiserror::Error;

/// Global regression detector instance
static REGRESSION_DETECTOR: LazyLock<Arc<RwLock<RegressionDetector>>> =
    LazyLock::new(|| Arc::new(RwLock::new(RegressionDetector::new())));

/// Regression testing errors
#[derive(Debug, Error)]
pub enum RegressionError {
    #[error("No baseline found for test: {0}")]
    NoBaseline(String),
    
    #[error("Insufficient data points: need at least {0}, got {1}")]
    InsufficientData(usize, usize),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Invalid threshold: {0}")]
    InvalidThreshold(String),
}

pub type RegressionResult<T> = Result<T, RegressionError>;

/// Performance measurement for a single test run
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceMeasurement {
    /// Test name/identifier
    pub name: String,
    
    /// Duration in milliseconds
    pub duration_ms: f64,
    
    /// Memory used in bytes (optional)
    pub memory_bytes: Option<u64>,
    
    /// CPU samples collected (optional)
    pub cpu_samples: Option<usize>,
    
    /// Timestamp of measurement
    pub timestamp: DateTime<Utc>,
    
    /// Git commit hash (optional)
    pub commit_hash: Option<String>,
    
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Statistical summary of performance measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceStatistics {
    pub count: usize,
    pub mean: f64,
    pub median: f64,
    pub stddev: f64,
    pub min: f64,
    pub max: f64,
    pub p95: f64,  // 95th percentile
    pub p99: f64,  // 99th percentile
}

/// Performance baseline for comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PerformanceBaseline {
    /// Test name
    pub name: String,
    
    /// Statistical summary of baseline runs
    pub statistics: PerformanceStatistics,
    
    /// Individual measurements
    pub measurements: Vec<PerformanceMeasurement>,
    
    /// When baseline was created
    pub created_at: DateTime<Utc>,
    
    /// Git commit hash (optional)
    pub commit_hash: Option<String>,
}

/// Regression detection result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegressionReport {
    /// Test name
    pub test_name: String,
    
    /// Is this a regression?
    pub is_regression: bool,
    
    /// Percent change from baseline (positive = slower)
    pub percent_change: f64,
    
    /// Current measurement
    pub current_duration_ms: f64,
    
    /// Baseline mean duration
    pub baseline_mean_ms: f64,
    
    /// Threshold that was applied
    pub threshold_percent: f64,
    
    /// Statistical confidence (0.0 - 1.0)
    pub confidence: f64,
    
    /// Detailed message
    pub message: String,
    
    /// Timestamp of test
    pub timestamp: DateTime<Utc>,
}

/// Trend analysis result
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrendAnalysis {
    /// Test name
    pub test_name: String,
    
    /// Linear regression slope (ms per day)
    pub slope: f64,
    
    /// Is performance degrading over time?
    pub is_degrading: bool,
    
    /// Percent change over analysis period
    pub total_change_percent: f64,
    
    /// Number of data points analyzed
    pub sample_count: usize,
    
    /// Analysis period in days
    pub period_days: f64,
}

/// Regression threshold configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegressionThresholds {
    /// Default threshold for regressions (percent slower)
    pub default_threshold_percent: f64,
    
    /// Per-test thresholds
    pub test_thresholds: HashMap<String, f64>,
    
    /// Minimum confidence level (0.0 - 1.0)
    pub min_confidence: f64,
}

impl Default for RegressionThresholds {
    fn default() -> Self {
        Self {
            default_threshold_percent: 10.0,  // 10% slower = regression
            test_thresholds: HashMap::new(),
            min_confidence: 0.8,  // 80% confidence required
        }
    }
}

/// Core regression detection engine
pub struct RegressionDetector {
    baselines: HashMap<String, PerformanceBaseline>,
    history: Vec<PerformanceMeasurement>,
    thresholds: RegressionThresholds,
    storage_path: Option<PathBuf>,
}

impl Default for RegressionDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl RegressionDetector {
    /// Create a new regression detector
    pub fn new() -> Self {
        Self {
            baselines: HashMap::new(),
            history: Vec::new(),
            thresholds: RegressionThresholds::default(),
            storage_path: None,
        }
    }
    
    /// Set storage path for baselines
    pub fn set_storage_path(&mut self, path: PathBuf) {
        self.storage_path = Some(path);
    }
    
    /// Record a new baseline from multiple measurements
    pub fn record_baseline(
        &mut self,
        name: impl Into<String>,
        measurements: Vec<PerformanceMeasurement>,
    ) -> RegressionResult<PerformanceBaseline> {
        let name = name.into();
        
        if measurements.is_empty() {
            return Err(RegressionError::InsufficientData(1, 0));
        }
        
        let durations: Vec<f64> = measurements
            .iter()
            .map(|m| m.duration_ms)
            .collect();
        
        let statistics = Self::calculate_statistics(&durations)?;
        
        let baseline = PerformanceBaseline {
            name: name.clone(),
            statistics,
            measurements: measurements.clone(),
            created_at: Utc::now(),
            commit_hash: measurements.first().and_then(|m| m.commit_hash.clone()),
        };
        
        self.baselines.insert(name, baseline.clone());
        
        Ok(baseline)
    }
    
    /// Run a test and compare against baseline
    pub fn run_test(
        &mut self,
        measurement: PerformanceMeasurement,
    ) -> RegressionResult<RegressionReport> {
        let test_name = measurement.name.clone();
        
        // Add to history
        self.history.push(measurement.clone());
        
        // Get baseline
        let baseline = self.baselines
            .get(&test_name)
            .ok_or_else(|| RegressionError::NoBaseline(test_name.clone()))?;
        
        // Calculate percent change
        let baseline_mean = baseline.statistics.mean;
        let current = measurement.duration_ms;
        let percent_change = ((current - baseline_mean) / baseline_mean) * 100.0;
        
        // Get threshold
        let threshold = self.thresholds
            .test_thresholds
            .get(&test_name)
            .copied()
            .unwrap_or(self.thresholds.default_threshold_percent);
        
        // Calculate confidence using coefficient of variation
        let cv = baseline.statistics.stddev / baseline.statistics.mean;
        let confidence = (1.0 - cv.min(1.0)).max(0.0);
        
        let is_regression = percent_change > threshold 
            && confidence >= self.thresholds.min_confidence;
        
        let message = if is_regression {
            format!(
                "⚠️ Regression detected: {:.2}% slower than baseline ({:.2}ms vs {:.2}ms)",
                percent_change, current, baseline_mean
            )
        } else {
            format!(
                "✅ Performance within threshold: {:.2}% change ({:.2}ms vs {:.2}ms)",
                percent_change, current, baseline_mean
            )
        };
        
        Ok(RegressionReport {
            test_name,
            is_regression,
            percent_change,
            current_duration_ms: current,
            baseline_mean_ms: baseline_mean,
            threshold_percent: threshold,
            confidence,
            message,
            timestamp: measurement.timestamp,
        })
    }
    
    /// Detect regressions across all tests with baselines
    pub fn detect_regressions(&self) -> RegressionResult<Vec<RegressionReport>> {
        let mut reports = Vec::new();
        
        // Group history by test name
        let mut by_test: HashMap<String, Vec<&PerformanceMeasurement>> = HashMap::new();
        for measurement in &self.history {
            by_test.entry(measurement.name.clone())
                .or_default()
                .push(measurement);
        }
        
        // Check each test against baseline
        for (test_name, measurements) in by_test {
            if let Some(baseline) = self.baselines.get(&test_name) {
                if let Some(latest) = measurements.last() {
                    let current = latest.duration_ms;
                    let baseline_mean = baseline.statistics.mean;
                    let percent_change = ((current - baseline_mean) / baseline_mean) * 100.0;
                    
                    let threshold = self.thresholds
                        .test_thresholds
                        .get(&test_name)
                        .copied()
                        .unwrap_or(self.thresholds.default_threshold_percent);
                    
                    let cv = baseline.statistics.stddev / baseline.statistics.mean;
                    let confidence = (1.0 - cv.min(1.0)).max(0.0);
                    
                    let is_regression = percent_change > threshold 
                        && confidence >= self.thresholds.min_confidence;
                    
                    let message = if is_regression {
                        format!(
                            "⚠️ Regression: {:.2}% slower ({:.2}ms vs {:.2}ms)",
                            percent_change, current, baseline_mean
                        )
                    } else {
                        format!(
                            "✅ OK: {:.2}% change ({:.2}ms vs {:.2}ms)",
                            percent_change, current, baseline_mean
                        )
                    };
                    
                    reports.push(RegressionReport {
                        test_name,
                        is_regression,
                        percent_change,
                        current_duration_ms: current,
                        baseline_mean_ms: baseline_mean,
                        threshold_percent: threshold,
                        confidence,
                        message,
                        timestamp: latest.timestamp,
                    });
                }
            }
        }
        
        Ok(reports)
    }
    
    /// Analyze performance trends over time
    pub fn analyze_trends(&self, test_name: &str, days: f64) -> RegressionResult<TrendAnalysis> {
        let cutoff = Utc::now() - chrono::Duration::days(days as i64);
        
        let measurements: Vec<&PerformanceMeasurement> = self.history
            .iter()
            .filter(|m| m.name == test_name && m.timestamp >= cutoff)
            .collect();
        
        if measurements.len() < 3 {
            return Err(RegressionError::InsufficientData(3, measurements.len()));
        }
        
        // Simple linear regression: y = mx + b
        let n = measurements.len() as f64;
        let first_time = measurements.first().unwrap().timestamp.timestamp() as f64;
        
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_x2 = 0.0;
        
        for m in &measurements {
            let x = (m.timestamp.timestamp() as f64 - first_time) / 86400.0; // Days since first
            let y = m.duration_ms;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_x2 += x * x;
        }
        
        let slope = (n * sum_xy - sum_x * sum_y) / (n * sum_x2 - sum_x * sum_x);
        
        let first_duration = measurements.first().unwrap().duration_ms;
        let last_duration = measurements.last().unwrap().duration_ms;
        let total_change_percent = ((last_duration - first_duration) / first_duration) * 100.0;
        
        let is_degrading = slope > 0.0 && total_change_percent > 5.0;
        
        Ok(TrendAnalysis {
            test_name: test_name.to_string(),
            slope,
            is_degrading,
            total_change_percent,
            sample_count: measurements.len(),
            period_days: days,
        })
    }
    
    /// Get baseline for a test
    pub fn get_baseline(&self, test_name: &str) -> Option<&PerformanceBaseline> {
        self.baselines.get(test_name)
    }
    
    /// Get all baselines
    pub fn get_baselines(&self) -> Vec<PerformanceBaseline> {
        self.baselines.values().cloned().collect()
    }
    
    /// Delete a baseline
    pub fn delete_baseline(&mut self, test_name: &str) -> bool {
        self.baselines.remove(test_name).is_some()
    }
    
    /// Get measurement history
    pub fn get_history(&self) -> &[PerformanceMeasurement] {
        &self.history
    }
    
    /// Clear history
    pub fn clear_history(&mut self) {
        self.history.clear();
    }
    
    /// Set threshold for a specific test
    pub fn set_threshold(&mut self, test_name: impl Into<String>, threshold_percent: f64) -> RegressionResult<()> {
        if !(0.0..=1000.0).contains(&threshold_percent) {
            return Err(RegressionError::InvalidThreshold(
                format!("Threshold must be between 0 and 1000, got {}", threshold_percent)
            ));
        }
        
        self.thresholds.test_thresholds.insert(test_name.into(), threshold_percent);
        Ok(())
    }
    
    /// Get thresholds
    pub fn get_thresholds(&self) -> &RegressionThresholds {
        &self.thresholds
    }
    
    /// Calculate statistics from a set of durations
    fn calculate_statistics(durations: &[f64]) -> RegressionResult<PerformanceStatistics> {
        if durations.is_empty() {
            return Err(RegressionError::InsufficientData(1, 0));
        }
        
        let count = durations.len();
        let sum: f64 = durations.iter().sum();
        let mean = sum / count as f64;
        
        // Calculate standard deviation
        let variance: f64 = durations
            .iter()
            .map(|&x| (x - mean).powi(2))
            .sum::<f64>() / count as f64;
        let stddev = variance.sqrt();
        
        // Sort for percentiles
        let mut sorted: Vec<OrderedFloat<f64>> = durations
            .iter()
            .map(|&x| OrderedFloat(x))
            .collect();
        sorted.sort_unstable();
        
        let min = sorted.first().unwrap().0;
        let max = sorted.last().unwrap().0;
        
        let median_idx = count / 2;
        let median = if count.is_multiple_of(2) {
            (sorted[median_idx - 1].0 + sorted[median_idx].0) / 2.0
        } else {
            sorted[median_idx].0
        };
        
        let p95_idx = (count as f64 * 0.95) as usize;
        let p95 = sorted[p95_idx.min(count - 1)].0;
        
        let p99_idx = (count as f64 * 0.99) as usize;
        let p99 = sorted[p99_idx.min(count - 1)].0;
        
        Ok(PerformanceStatistics {
            count,
            mean,
            median,
            stddev,
            min,
            max,
            p95,
            p99,
        })
    }
    
    /// Save baselines to disk
    pub fn save_baselines(&self) -> RegressionResult<()> {
        if let Some(path) = &self.storage_path {
            let json = serde_json::to_string_pretty(&self.baselines)?;
            std::fs::write(path, json)?;
        }
        Ok(())
    }
    
    /// Load baselines from disk
    pub fn load_baselines(&mut self) -> RegressionResult<()> {
        if let Some(path) = &self.storage_path {
            if path.exists() {
                let json = std::fs::read_to_string(path)?;
                self.baselines = serde_json::from_str(&json)?;
            }
        }
        Ok(())
    }
}

// Public API functions using global singleton

/// Record a new performance baseline
pub fn record_baseline(
    name: impl Into<String>,
    measurements: Vec<PerformanceMeasurement>,
) -> RegressionResult<PerformanceBaseline> {
    REGRESSION_DETECTOR.write().record_baseline(name, measurements)
}

/// Run a test and compare against baseline
pub fn run_test(measurement: PerformanceMeasurement) -> RegressionResult<RegressionReport> {
    REGRESSION_DETECTOR.write().run_test(measurement)
}

/// Detect regressions across all tests
pub fn detect_regressions() -> RegressionResult<Vec<RegressionReport>> {
    REGRESSION_DETECTOR.read().detect_regressions()
}

/// Analyze performance trends
pub fn analyze_trends(test_name: &str, days: f64) -> RegressionResult<TrendAnalysis> {
    REGRESSION_DETECTOR.read().analyze_trends(test_name, days)
}

/// Get baseline for a test
pub fn get_baseline(test_name: &str) -> Option<PerformanceBaseline> {
    REGRESSION_DETECTOR.read().get_baseline(test_name).cloned()
}

/// Get all baselines
pub fn get_baselines() -> Vec<PerformanceBaseline> {
    REGRESSION_DETECTOR.read().get_baselines()
}

/// Delete a baseline
pub fn delete_baseline(test_name: &str) -> bool {
    REGRESSION_DETECTOR.write().delete_baseline(test_name)
}

/// Get measurement history
pub fn get_history() -> Vec<PerformanceMeasurement> {
    REGRESSION_DETECTOR.read().get_history().to_vec()
}

/// Clear history
pub fn clear_history() {
    REGRESSION_DETECTOR.write().clear_history();
}

/// Set threshold for a test
pub fn set_threshold(test_name: impl Into<String>, threshold_percent: f64) -> RegressionResult<()> {
    REGRESSION_DETECTOR.write().set_threshold(test_name, threshold_percent)
}

/// Get thresholds
pub fn get_thresholds() -> RegressionThresholds {
    REGRESSION_DETECTOR.read().get_thresholds().clone()
}

/// Set storage path for baselines
pub fn set_storage_path(path: PathBuf) {
    REGRESSION_DETECTOR.write().set_storage_path(path);
}

/// Save baselines to disk
pub fn save_baselines() -> RegressionResult<()> {
    REGRESSION_DETECTOR.read().save_baselines()
}

/// Load baselines from disk
pub fn load_baselines() -> RegressionResult<()> {
    REGRESSION_DETECTOR.write().load_baselines()
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_statistics_calculation() {
        let durations = vec![100.0, 110.0, 105.0, 115.0, 120.0];
        let stats = RegressionDetector::calculate_statistics(&durations).unwrap();
        
        assert_eq!(stats.count, 5);
        assert!((stats.mean - 110.0).abs() < 0.1);
        assert!((stats.median - 110.0).abs() < 0.1);
        assert!(stats.stddev > 0.0);
        assert_eq!(stats.min, 100.0);
        assert_eq!(stats.max, 120.0);
    }
    
    #[test]
    fn test_baseline_recording() {
        let mut detector = RegressionDetector::new();
        
        let measurements = vec![
            PerformanceMeasurement {
                name: "test1".to_string(),
                duration_ms: 100.0,
                memory_bytes: None,
                cpu_samples: None,
                timestamp: Utc::now(),
                commit_hash: None,
                metadata: HashMap::new(),
            },
            PerformanceMeasurement {
                name: "test1".to_string(),
                duration_ms: 110.0,
                memory_bytes: None,
                cpu_samples: None,
                timestamp: Utc::now(),
                commit_hash: None,
                metadata: HashMap::new(),
            },
        ];
        
        let baseline = detector.record_baseline("test1", measurements).unwrap();
        assert_eq!(baseline.name, "test1");
        assert_eq!(baseline.statistics.count, 2);
        assert!((baseline.statistics.mean - 105.0).abs() < 0.1);
    }
    
    #[test]
    fn test_regression_detection() {
        let mut detector = RegressionDetector::new();
        
        // Record baseline
        let baseline_measurements = vec![
            PerformanceMeasurement {
                name: "test1".to_string(),
                duration_ms: 100.0,
                memory_bytes: None,
                cpu_samples: None,
                timestamp: Utc::now(),
                commit_hash: None,
                metadata: HashMap::new(),
            },
            PerformanceMeasurement {
                name: "test1".to_string(),
                duration_ms: 110.0,
                memory_bytes: None,
                cpu_samples: None,
                timestamp: Utc::now(),
                commit_hash: None,
                metadata: HashMap::new(),
            },
        ];
        
        detector.record_baseline("test1", baseline_measurements).unwrap();
        
        // Test with no regression (within threshold)
        let good_measurement = PerformanceMeasurement {
            name: "test1".to_string(),
            duration_ms: 108.0,  // +3% from mean of 105
            memory_bytes: None,
            cpu_samples: None,
            timestamp: Utc::now(),
            commit_hash: None,
            metadata: HashMap::new(),
        };
        
        let report = detector.run_test(good_measurement).unwrap();
        assert!(!report.is_regression);
        
        // Test with regression (>10% slower)
        let bad_measurement = PerformanceMeasurement {
            name: "test1".to_string(),
            duration_ms: 120.0,  // +14% from mean of 105
            memory_bytes: None,
            cpu_samples: None,
            timestamp: Utc::now(),
            commit_hash: None,
            metadata: HashMap::new(),
        };
        
        let report = detector.run_test(bad_measurement).unwrap();
        assert!(report.is_regression);
        assert!(report.percent_change > 10.0);
    }
    
    #[test]
    fn test_threshold_configuration() {
        let mut detector = RegressionDetector::new();
        
        // Set custom threshold
        detector.set_threshold("test1", 20.0).unwrap();
        
        assert_eq!(
            detector.thresholds.test_thresholds.get("test1"),
            Some(&20.0)
        );
        
        // Invalid threshold should fail
        assert!(detector.set_threshold("test2", -5.0).is_err());
        assert!(detector.set_threshold("test3", 2000.0).is_err());
    }
    
    #[test]
    fn test_history_management() {
        let mut detector = RegressionDetector::new();
        
        let m1 = PerformanceMeasurement {
            name: "test1".to_string(),
            duration_ms: 100.0,
            memory_bytes: None,
            cpu_samples: None,
            timestamp: Utc::now(),
            commit_hash: None,
            metadata: HashMap::new(),
        };
        
        // Record baseline first
        detector.record_baseline("test1", vec![m1.clone()]).unwrap();
        
        // Add to history via run_test
        detector.run_test(m1.clone()).unwrap();
        
        assert_eq!(detector.get_history().len(), 1);
        
        detector.clear_history();
        assert_eq!(detector.get_history().len(), 0);
    }
}
