// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Tauri commands for regression testing

use crate::common::regression::{
    self, PerformanceBaseline, PerformanceMeasurement, RegressionReport,
    RegressionThresholds, TrendAnalysis,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Summary of regression testing status
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegressionSummary {
    pub baseline_count: usize,
    pub history_count: usize,
    pub tests_with_baselines: Vec<String>,
    pub recent_regressions: Vec<String>,
    pub default_threshold_percent: f64,
    pub recommendation: String,
}

/// Record a new performance baseline
#[tauri::command]
pub async fn regression_record_baseline(
    name: String,
    measurements: Vec<PerformanceMeasurement>,
) -> Result<PerformanceBaseline, String> {
    regression::record_baseline(name, measurements)
        .map_err(|e| e.to_string())
}

/// Run a test and compare against baseline
#[tauri::command]
pub async fn regression_run_test(
    measurement: PerformanceMeasurement,
) -> Result<RegressionReport, String> {
    regression::run_test(measurement)
        .map_err(|e| e.to_string())
}

/// Compare current results against all baselines
#[tauri::command]
pub async fn regression_compare_results() -> Result<Vec<RegressionReport>, String> {
    regression::detect_regressions()
        .map_err(|e| e.to_string())
}

/// Get all recorded baselines
#[tauri::command]
pub async fn regression_get_baselines() -> Result<Vec<PerformanceBaseline>, String> {
    Ok(regression::get_baselines())
}

/// Get a specific baseline
#[tauri::command]
pub async fn regression_get_baseline(test_name: String) -> Result<Option<PerformanceBaseline>, String> {
    Ok(regression::get_baseline(&test_name))
}

/// Delete a baseline
#[tauri::command]
pub async fn regression_delete_baseline(test_name: String) -> Result<bool, String> {
    Ok(regression::delete_baseline(&test_name))
}

/// Detect regressions across all tests
#[tauri::command]
pub async fn regression_detect_regressions() -> Result<Vec<RegressionReport>, String> {
    regression::detect_regressions()
        .map_err(|e| e.to_string())
}

/// Get measurement history
#[tauri::command]
pub async fn regression_get_history() -> Result<Vec<PerformanceMeasurement>, String> {
    Ok(regression::get_history())
}

/// Export regression report as JSON
#[tauri::command]
pub async fn regression_export_report() -> Result<String, String> {
    let reports = regression::detect_regressions()
        .map_err(|e| e.to_string())?;
    
    let export = HashMap::from([
        ("timestamp", chrono::Utc::now().to_rfc3339()),
        ("baselines", format!("{}", regression::get_baselines().len())),
        ("history_count", format!("{}", regression::get_history().len())),
    ]);
    
    serde_json::to_string_pretty(&serde_json::json!({
        "metadata": export,
        "reports": reports,
    }))
    .map_err(|e| e.to_string())
}

/// Clear measurement history
#[tauri::command]
pub async fn regression_clear_history() -> Result<(), String> {
    regression::clear_history();
    Ok(())
}

/// Get regression testing summary
#[tauri::command]
pub async fn regression_get_summary() -> Result<RegressionSummary, String> {
    let baselines = regression::get_baselines();
    let history = regression::get_history();
    let thresholds = regression::get_thresholds();
    
    let tests_with_baselines: Vec<String> = baselines
        .iter()
        .map(|b| b.name.clone())
        .collect();
    
    let recent_reports = regression::detect_regressions()
        .map_err(|e| e.to_string())?;
    
    let recent_regressions: Vec<String> = recent_reports
        .iter()
        .filter(|r| r.is_regression)
        .map(|r| r.test_name.clone())
        .collect();
    
    let recommendation = if baselines.is_empty() {
        "No baselines recorded. Run 'regression_record_baseline' to create performance baselines.".to_string()
    } else if recent_regressions.is_empty() {
        format!("✅ All {} tests performing within thresholds.", baselines.len())
    } else {
        format!("⚠️ {} regression(s) detected out of {} tests.", recent_regressions.len(), baselines.len())
    };
    
    Ok(RegressionSummary {
        baseline_count: baselines.len(),
        history_count: history.len(),
        tests_with_baselines,
        recent_regressions,
        default_threshold_percent: thresholds.default_threshold_percent,
        recommendation,
    })
}

/// Analyze performance trends
#[tauri::command]
pub async fn regression_analyze_trends(
    test_name: String,
    days: f64,
) -> Result<TrendAnalysis, String> {
    regression::analyze_trends(&test_name, days)
        .map_err(|e| e.to_string())
}

/// Set regression threshold for a test
#[tauri::command]
pub async fn regression_set_threshold(
    test_name: String,
    threshold_percent: f64,
) -> Result<(), String> {
    regression::set_threshold(test_name, threshold_percent)
        .map_err(|e| e.to_string())
}

/// Get current thresholds
#[tauri::command]
pub async fn regression_get_thresholds() -> Result<RegressionThresholds, String> {
    Ok(regression::get_thresholds())
}

/// Save baselines to disk
#[tauri::command]
pub async fn regression_save_baselines(path: String) -> Result<(), String> {
    regression::set_storage_path(path.into());
    regression::save_baselines()
        .map_err(|e| e.to_string())
}

/// Load baselines from disk
#[tauri::command]
pub async fn regression_load_baselines(path: String) -> Result<(), String> {
    regression::set_storage_path(path.into());
    regression::load_baselines()
        .map_err(|e| e.to_string())
}
