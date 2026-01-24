// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useObservability Hook
 * 
 * SolidJS hook for Phase 13: Advanced Observability & Telemetry
 * Provides access to metrics, health monitoring, and tracing functionality
 * 
 * @example
 * ```tsx
 * const obs = useObservability();
 * 
 * // Increment counter
 * await obs.incrementCounter("files_processed");
 * 
 * // Set gauge
 * await obs.setGauge("memory_usage_mb", 1024);
 * 
 * // Record histogram
 * await obs.recordHistogram("operation_duration_ms", 150);
 * 
 * // Get health status
 * const health = await obs.getHealthStatus();
 * console.log(`System Health: ${health.status}`);
 * 
 * // Start tracing
 * const traceId = await obs.startTracing("container_load", { path: "/evidence/file.ad1" });
 * // ... do work ...
 * await obs.endTracing(traceId);
 * ```
 */

import { invoke } from "@tauri-apps/api/core";
import { createSignal, createEffect, onCleanup } from "solid-js";

// ============================================================================
// Type Definitions
// ============================================================================

export interface MetricValue {
  name: string;
  value: number;
  timestamp: number;
}

export interface CounterMetric extends MetricValue {
  count: number;
}

export interface GaugeMetric extends MetricValue {
  current: number;
}

export interface HistogramMetric extends MetricValue {
  samples: number[];
  mean: number;
  median: number;
  p95: number;
  p99: number;
  min: number;
  max: number;
}

export interface AllMetrics {
  counters: { [key: string]: CounterMetric };
  gauges: { [key: string]: GaugeMetric };
  histograms: { [key: string]: HistogramMetric };
}

export interface SystemHealth {
  status: "healthy" | "degraded" | "unhealthy";
  cpu_percent: number;
  memory_used_mb: number;
  memory_available_mb: number;
  disk_free_gb: number;
  timestamp: number;
}

export interface HealthHistory {
  entries: SystemHealth[];
  duration_seconds: number;
}

export interface DetailedHealth extends SystemHealth {
  process_memory_mb: number;
  thread_count: number;
  active_operations: number;
  error_count_1h: number;
  warnings: string[];
}

export interface TraceEvent {
  trace_id: string;
  operation: string;
  timestamp: number;
  duration_ms?: number;
  metadata?: { [key: string]: string };
  status: "started" | "completed" | "failed";
}

export interface ExportedMetrics {
  metrics: AllMetrics;
  health: SystemHealth;
  traces: TraceEvent[];
  timestamp: number;
  version: string;
}

// ============================================================================
// Hook Implementation
// ============================================================================

export function useObservability() {
  const [metrics, setMetrics] = createSignal<AllMetrics | null>(null);
  const [health, setHealth] = createSignal<SystemHealth | null>(null);
  const [isMonitoring, setIsMonitoring] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  let healthCheckInterval: number | undefined;

  // ============================================================================
  // Metrics Operations
  // ============================================================================

  /**
   * Get all metrics (counters, gauges, histograms)
   */
  const getAllMetrics = async (): Promise<AllMetrics> => {
    try {
      const result = await invoke<AllMetrics>("metrics_get_all");
      setMetrics(result);
      return result;
    } catch (e) {
      const errorMsg = `Failed to get metrics: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Get a specific counter value
   */
  const getCounter = async (name: string): Promise<number> => {
    try {
      return await invoke<number>("metrics_get_counter", { name });
    } catch (e) {
      const errorMsg = `Failed to get counter '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Increment a counter by 1 or specified amount
   */
  const incrementCounter = async (name: string, amount: number = 1): Promise<void> => {
    try {
      await invoke("metrics_increment_counter", { name, amount });
      // Refresh metrics after update
      await getAllMetrics();
    } catch (e) {
      const errorMsg = `Failed to increment counter '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Set a gauge value
   */
  const setGauge = async (name: string, value: number): Promise<void> => {
    try {
      await invoke("metrics_set_gauge", { name, value });
      await getAllMetrics();
    } catch (e) {
      const errorMsg = `Failed to set gauge '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Record a histogram sample
   */
  const recordHistogram = async (name: string, value: number): Promise<void> => {
    try {
      await invoke("metrics_record_histogram", { name, value });
      await getAllMetrics();
    } catch (e) {
      const errorMsg = `Failed to record histogram '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Export all metrics to JSON string
   */
  const exportMetrics = async (): Promise<string> => {
    try {
      return await invoke<string>("metrics_export");
    } catch (e) {
      const errorMsg = `Failed to export metrics: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Reset all metrics
   */
  const resetMetrics = async (): Promise<void> => {
    try {
      await invoke("metrics_reset");
      setMetrics(null);
    } catch (e) {
      const errorMsg = `Failed to reset metrics: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  // ============================================================================
  // Health Monitoring
  // ============================================================================

  /**
   * Get current system health status
   */
  const getHealthStatus = async (): Promise<SystemHealth> => {
    try {
      const result = await invoke<SystemHealth>("health_get_status");
      setHealth(result);
      return result;
    } catch (e) {
      const errorMsg = `Failed to get health status: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Get health history over specified duration (seconds)
   */
  const getHealthHistory = async (durationSeconds: number): Promise<HealthHistory> => {
    try {
      return await invoke<HealthHistory>("health_get_history", {
        durationSeconds,
      });
    } catch (e) {
      const errorMsg = `Failed to get health history: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Get detailed health information
   */
  const getDetailedHealth = async (): Promise<DetailedHealth> => {
    try {
      return await invoke<DetailedHealth>("health_get_detailed");
    } catch (e) {
      const errorMsg = `Failed to get detailed health: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Start automatic health monitoring (polls every interval)
   */
  const startHealthMonitoring = (intervalMs: number = 5000) => {
    if (isMonitoring()) return;

    setIsMonitoring(true);
    
    // Initial health check
    getHealthStatus();

    // Set up interval
    healthCheckInterval = window.setInterval(() => {
      getHealthStatus();
    }, intervalMs);
  };

  /**
   * Stop automatic health monitoring
   */
  const stopHealthMonitoring = () => {
    if (healthCheckInterval) {
      clearInterval(healthCheckInterval);
      healthCheckInterval = undefined;
    }
    setIsMonitoring(false);
  };

  // ============================================================================
  // Distributed Tracing
  // ============================================================================

  /**
   * Start a trace for an operation
   * Returns trace ID for correlation
   */
  const startTracing = async (
    operation: string,
    metadata?: { [key: string]: string }
  ): Promise<string> => {
    try {
      return await invoke<string>("tracing_start", {
        operation,
        metadata: metadata || {},
      });
    } catch (e) {
      const errorMsg = `Failed to start tracing for '${operation}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * End a trace
   */
  const endTracing = async (
    traceId: string,
    status: "completed" | "failed" = "completed",
    metadata?: { [key: string]: string }
  ): Promise<void> => {
    try {
      await invoke("tracing_end", {
        traceId,
        status,
        metadata: metadata || {},
      });
    } catch (e) {
      const errorMsg = `Failed to end tracing '${traceId}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  /**
   * Get all trace events
   */
  const getTraceEvents = async (): Promise<TraceEvent[]> => {
    try {
      return await invoke<TraceEvent[]>("tracing_get_events");
    } catch (e) {
      const errorMsg = `Failed to get trace events: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  // ============================================================================
  // Lifecycle Management
  // ============================================================================

  // Clean up on unmount
  onCleanup(() => {
    stopHealthMonitoring();
  });

  // ============================================================================
  // Return API
  // ============================================================================

  return {
    // State
    metrics,
    health,
    isMonitoring,
    error,

    // Metrics operations
    getAllMetrics,
    getCounter,
    incrementCounter,
    setGauge,
    recordHistogram,
    exportMetrics,
    resetMetrics,

    // Health monitoring
    getHealthStatus,
    getHealthHistory,
    getDetailedHealth,
    startHealthMonitoring,
    stopHealthMonitoring,

    // Tracing
    startTracing,
    endTracing,
    getTraceEvents,
  };
}

export default useObservability;
