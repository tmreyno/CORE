// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useRegressionTesting Hook
 * 
 * SolidJS hook for Phase 16: Automated Performance Regression Testing
 * Provides statistical regression detection with baseline management
 * 
 * @example
 * ```tsx
 * const regression = useRegressionTesting();
 * 
 * // Record baseline
 * await regression.recordBaseline("hash_sha256", [100.5, 102.1, 99.8, 101.3]);
 * 
 * // Run test and detect regressions
 * const result = await regression.runTest("hash_sha256", 150.0); // 150ms (regression!)
 * if (result.is_regression) {
 *   log.warn(`Regression detected: ${result.percent_change}% slower`);
 * }
 * 
 * // Analyze trends
 * const trends = await regression.analyzeTrends("hash_sha256", 30);
 * if (trends.is_degrading) {
 *   log.warn("Performance degrading over time");
 * }
 * ```
 */

import { invoke } from "@tauri-apps/api/core";
import { createSignal } from "solid-js";
import { logger } from "../utils/logger";
const log = logger.scope("RegressionTesting");

// ============================================================================
// Type Definitions
// ============================================================================

export interface PerformanceBaseline {
  name: string;
  statistics: PerformanceStatistics;
  measurement_count: number;
  created_at: number;
  commit_hash?: string;
}

export interface PerformanceStatistics {
  count: number;
  mean: number;
  median: number;
  stddev: number;
  min: number;
  max: number;
  p95: number;
  p99: number;
}

export interface RegressionReport {
  test_name: string;
  is_regression: boolean;
  percent_change: number;
  current_duration_ms: number;
  baseline_mean_ms: number;
  threshold_percent: number;
  confidence: number;
  message: string;
  timestamp: number;
}

export interface TrendAnalysis {
  test_name: string;
  slope: number;
  is_degrading: boolean;
  total_change_percent: number;
  sample_count: number;
  period_days: number;
}

export interface TestHistory {
  test_name: string;
  measurements: PerformanceMeasurement[];
  baseline?: PerformanceBaseline;
}

export interface PerformanceMeasurement {
  name: string;
  duration_ms: number;
  memory_bytes?: number;
  cpu_samples?: number;
  timestamp: number;
  commit_hash?: string;
  metadata?: { [key: string]: string };
}

export interface RegressionSummary {
  total_tests: number;
  total_baselines: number;
  regressions_detected: number;
  tests_improved: number;
  tests_stable: number;
  timestamp: number;
}

export interface ThresholdConfig {
  test_name: string;
  threshold_percent: number;
}

// ============================================================================
// Hook Implementation
// ============================================================================

export function useRegressionTesting() {
  const [baselines, setBaselines] = createSignal<PerformanceBaseline[]>([]);
  const [summary, setSummary] = createSignal<RegressionSummary | null>(null);
  const [isLoading, setIsLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  // ============================================================================
  // Baseline Management
  // ============================================================================

  /**
   * Record a performance baseline
   * @param name Test name/identifier
   * @param durations Array of duration measurements in milliseconds
   * @param commitHash Optional git commit hash for versioning
   */
  const recordBaseline = async (
    name: string,
    durations: number[],
    commitHash?: string
  ): Promise<PerformanceBaseline> => {
    try {
      setIsLoading(true);
      setError(null);
      
      const baseline = await invoke<PerformanceBaseline>("regression_record_baseline", {
        name,
        durations,
        commitHash,
      });
      
      await refreshBaselines();
      return baseline;
    } catch (e) {
      const errorMsg = `Failed to record baseline for '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Get all baselines
   */
  const getBaselines = async (): Promise<PerformanceBaseline[]> => {
    try {
      const result = await invoke<PerformanceBaseline[]>("regression_get_baselines");
      setBaselines(result);
      return result;
    } catch (e) {
      const errorMsg = `Failed to get baselines: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Get a specific baseline
   */
  const getBaseline = async (name: string): Promise<PerformanceBaseline> => {
    try {
      return await invoke<PerformanceBaseline>("regression_get_baseline", { name });
    } catch (e) {
      const errorMsg = `Failed to get baseline '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Delete a baseline
   */
  const deleteBaseline = async (name: string): Promise<void> => {
    try {
      setIsLoading(true);
      await invoke("regression_delete_baseline", { name });
      await refreshBaselines();
    } catch (e) {
      const errorMsg = `Failed to delete baseline '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Refresh baselines list (internal helper)
   */
  const refreshBaselines = async (): Promise<void> => {
    try {
      await getBaselines();
    } catch (e) {
      log.error("Failed to refresh baselines:", e);
    }
  };

  // ============================================================================
  // Testing & Detection
  // ============================================================================

  /**
   * Run a test and compare against baseline
   * @param name Test name
   * @param durationMs Current measurement in milliseconds
   * @param thresholdPercent Optional custom threshold (default: 10%)
   */
  const runTest = async (
    name: string,
    durationMs: number,
    thresholdPercent?: number
  ): Promise<RegressionReport> => {
    try {
      setIsLoading(true);
      setError(null);
      
      return await invoke<RegressionReport>("regression_run_test", {
        name,
        durationMs,
        thresholdPercent,
      });
    } catch (e) {
      const errorMsg = `Failed to run test '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Compare results against baseline
   */
  const compareResults = async (
    name: string,
    durationMs: number,
    thresholdPercent: number = 10.0
  ): Promise<RegressionReport> => {
    try {
      setIsLoading(true);
      return await invoke<RegressionReport>("regression_compare_results", {
        name,
        durationMs,
        thresholdPercent,
      });
    } catch (e) {
      const errorMsg = `Failed to compare results for '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Detect regressions across all tests
   */
  const detectRegressions = async (): Promise<RegressionReport[]> => {
    try {
      setIsLoading(true);
      return await invoke<RegressionReport[]>("regression_detect_regressions");
    } catch (e) {
      const errorMsg = `Failed to detect regressions: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  // ============================================================================
  // History & Analysis
  // ============================================================================

  /**
   * Get test history
   */
  const getHistory = async (name: string): Promise<TestHistory> => {
    try {
      return await invoke<TestHistory>("regression_get_history", { name });
    } catch (e) {
      const errorMsg = `Failed to get history for '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Analyze performance trends over time
   * @param name Test name
   * @param days Number of days to analyze
   */
  const analyzeTrends = async (name: string, days: number): Promise<TrendAnalysis> => {
    try {
      setIsLoading(true);
      return await invoke<TrendAnalysis>("regression_analyze_trends", { name, days });
    } catch (e) {
      const errorMsg = `Failed to analyze trends for '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Get regression testing summary
   */
  const getSummary = async (): Promise<RegressionSummary> => {
    try {
      const result = await invoke<RegressionSummary>("regression_get_summary");
      setSummary(result);
      return result;
    } catch (e) {
      const errorMsg = `Failed to get summary: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Clear history for a specific test
   */
  const clearHistory = async (name: string): Promise<void> => {
    try {
      setIsLoading(true);
      await invoke("regression_clear_history", { name });
    } catch (e) {
      const errorMsg = `Failed to clear history for '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  // ============================================================================
  // Configuration
  // ============================================================================

  /**
   * Set regression threshold for a test
   * @param name Test name
   * @param thresholdPercent Threshold percentage (e.g., 10.0 for 10%)
   */
  const setThreshold = async (name: string, thresholdPercent: number): Promise<void> => {
    try {
      await invoke("regression_set_threshold", { name, thresholdPercent });
    } catch (e) {
      const errorMsg = `Failed to set threshold for '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Get all threshold configurations
   */
  const getThresholds = async (): Promise<ThresholdConfig[]> => {
    try {
      return await invoke<ThresholdConfig[]>("regression_get_thresholds");
    } catch (e) {
      const errorMsg = `Failed to get thresholds: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  // ============================================================================
  // Export & Import
  // ============================================================================

  /**
   * Export regression report to JSON
   */
  const exportReport = async (path: string): Promise<void> => {
    try {
      setIsLoading(true);
      await invoke("regression_export_report", { path });
    } catch (e) {
      const errorMsg = `Failed to export report: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Save baselines to file
   */
  const saveBaselines = async (path: string): Promise<void> => {
    try {
      setIsLoading(true);
      await invoke("regression_save_baselines", { path });
    } catch (e) {
      const errorMsg = `Failed to save baselines: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  /**
   * Load baselines from file
   */
  const loadBaselines = async (path: string): Promise<void> => {
    try {
      setIsLoading(true);
      await invoke("regression_load_baselines", { path });
      await refreshBaselines();
    } catch (e) {
      const errorMsg = `Failed to load baselines: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    } finally {
      setIsLoading(false);
    }
  };

  // ============================================================================
  // Convenience Methods
  // ============================================================================

  /**
   * Benchmark an async function and compare against baseline
   */
  const benchmarkAsync = async <T>(
    name: string,
    fn: () => Promise<T>,
    thresholdPercent?: number
  ): Promise<{ result: T; report: RegressionReport }> => {
    const startTime = performance.now();
    
    try {
      const result = await fn();
      const durationMs = performance.now() - startTime;
      const report = await runTest(name, durationMs, thresholdPercent);
      
      return { result, report };
    } catch (error) {
      throw error;
    }
  };

  /**
   * Quick regression check - run test and return if regression detected
   */
  const quickCheck = async (name: string, durationMs: number): Promise<boolean> => {
    try {
      const report = await runTest(name, durationMs);
      return report.is_regression;
    } catch (e) {
      log.error(`Quick check failed for '${name}':`, e);
      return false;
    }
  };

  /**
   * Initialize hook - load baselines and summary
   */
  const initialize = async (): Promise<void> => {
    await refreshBaselines();
    await getSummary().catch(() => {});
  };

  // Initialize on mount
  initialize();

  // ============================================================================
  // Return API
  // ============================================================================

  return {
    // State
    baselines,
    summary,
    isLoading,
    error,

    // Baseline management
    recordBaseline,
    getBaselines,
    getBaseline,
    deleteBaseline,
    refreshBaselines,

    // Testing & detection
    runTest,
    compareResults,
    detectRegressions,

    // History & analysis
    getHistory,
    analyzeTrends,
    getSummary,
    clearHistory,

    // Configuration
    setThreshold,
    getThresholds,

    // Export & import
    exportReport,
    saveBaselines,
    loadBaselines,

    // Convenience
    benchmarkAsync,
    quickCheck,
  };
}
