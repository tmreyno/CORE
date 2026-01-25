// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { createSignal, onCleanup } from "solid-js";

// Type Definitions
export type MetricValue =
  | { type: "counter"; value: number }
  | { type: "gauge"; value: number }
  | { type: "histogram"; count: number; sum: number; min: number; max: number; mean: number; p50: number; p95: number; p99: number };

export interface MetricEntry {
  name: string;
  value: MetricValue;
}

export type HealthStatus = "Healthy" | "Degraded" | "Unhealthy";

export interface ComponentHealth {
  name: string;
  status: HealthStatus;
  message: string | null;
  lastCheck: number;
}

export interface SystemHealth {
  status: HealthStatus;
  cpuPercent: number;
  memoryPercent: number;
  diskPercent: number;
  components: ComponentHealth[];
  timestamp: number;
}

export interface HealthThresholdsInput {
  cpuWarning: number;
  cpuCritical: number;
  memoryWarning: number;
  memoryCritical: number;
  diskWarning: number;
  diskCritical: number;
  queueDepthWarning: number;
  queueDepthCritical: number;
  errorRateWarning: number;
  errorRateCritical: number;
}

export interface SystemStatus {
  health: SystemHealth;
  metricsCount: number;
  uptimeSeconds: number;
}

export type LogLevel = "trace" | "debug" | "info" | "warn" | "error";

// Hook Implementation
export function useObservability() {
  const [metrics, setMetrics] = createSignal<MetricEntry[]>([]);
  const [health, setHealth] = createSignal<SystemHealth | null>(null);
  const [isMonitoring, setIsMonitoring] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  let healthCheckInterval: number | undefined;

  const getMetrics = async (): Promise<MetricEntry[]> => {
    try {
      const result = await invoke<MetricEntry[]>("get_metrics");
      setMetrics(result);
      setError(null);
      return result;
    } catch (e) {
      const errorMsg = `Failed to get metrics: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  const getMetric = async (name: string): Promise<MetricEntry | null> => {
    try {
      const result = await invoke<MetricEntry | null>("get_metric", { name });
      setError(null);
      return result;
    } catch (e) {
      const errorMsg = `Failed to get metric '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  const incrementCounter = async (name: string, amount: number = 1): Promise<void> => {
    try {
      await invoke("increment_counter", { name, amount });
      setError(null);
      await getMetrics();
    } catch (e) {
      const errorMsg = `Failed to increment counter '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  const setGauge = async (name: string, value: number): Promise<void> => {
    try {
      await invoke("set_gauge", { name, value });
      setError(null);
      await getMetrics();
    } catch (e) {
      const errorMsg = `Failed to set gauge '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  const recordHistogram = async (name: string, value: number): Promise<void> => {
    try {
      await invoke("record_histogram", { name, value });
      setError(null);
      await getMetrics();
    } catch (e) {
      const errorMsg = `Failed to record histogram '${name}': ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  const exportMetrics = async (): Promise<string> => {
    try {
      const result = await invoke<string>("export_metrics");
      setError(null);
      return result;
    } catch (e) {
      const errorMsg = `Failed to export metrics: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  const resetMetrics = async (): Promise<void> => {
    try {
      await invoke("reset_metrics");
      setMetrics([]);
      setError(null);
    } catch (e) {
      const errorMsg = `Failed to reset metrics: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  const getSystemUptime = async (): Promise<number> => {
    try {
      const result = await invoke<number>("get_system_uptime");
      setError(null);
      return result;
    } catch (e) {
      const errorMsg = `Failed to get uptime: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  const getMetricsCount = async (): Promise<number> => {
    try {
      const result = await invoke<number>("get_metrics_count");
      setError(null);
      return result;
    } catch (e) {
      const errorMsg = `Failed to get metrics count: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  const getHealth = async (): Promise<SystemHealth> => {
    try {
      const result = await invoke<SystemHealth>("get_health");
      setHealth(result);
      setError(null);
      return result;
    } catch (e) {
      const errorMsg = `Failed to get health status: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  const getHealthWithThresholds = async (thresholds: HealthThresholdsInput): Promise<SystemHealth> => {
    try {
      const result = await invoke<SystemHealth>("get_health_with_thresholds", { thresholds });
      setHealth(result);
      setError(null);
      return result;
    } catch (e) {
      const errorMsg = `Failed to get health with thresholds: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  const isSystemHealthy = async (): Promise<boolean> => {
    try {
      const result = await invoke<boolean>("is_system_healthy");
      setError(null);
      return result;
    } catch (e) {
      const errorMsg = `Failed to check system health: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  const getSystemStatus = async (): Promise<SystemStatus> => {
    try {
      const result = await invoke<SystemStatus>("get_system_status");
      setHealth(result.health);
      setError(null);
      return result;
    } catch (e) {
      const errorMsg = `Failed to get system status: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  const startHealthMonitoring = (intervalMs: number = 5000): void => {
    if (isMonitoring()) return;
    setIsMonitoring(true);
    getHealth().catch(() => {});
    healthCheckInterval = window.setInterval(() => {
      getHealth().catch(() => {});
    }, intervalMs);
  };

  const stopHealthMonitoring = (): void => {
    if (healthCheckInterval) {
      clearInterval(healthCheckInterval);
      healthCheckInterval = undefined;
    }
    setIsMonitoring(false);
  };

  const initTracing = async (level: string, logDir: string): Promise<void> => {
    try {
      await invoke("init_tracing", { level, logDir });
      setError(null);
    } catch (e) {
      const errorMsg = `Failed to initialize tracing: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  const getDefaultLogDir = async (): Promise<string> => {
    try {
      const result = await invoke<string>("get_default_log_dir");
      setError(null);
      return result;
    } catch (e) {
      const errorMsg = `Failed to get default log dir: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  const parseLogLevel = async (level: string): Promise<LogLevel> => {
    try {
      const result = await invoke<LogLevel>("parse_log_level", { level });
      setError(null);
      return result;
    } catch (e) {
      const errorMsg = `Failed to parse log level: ${e}`;
      setError(errorMsg);
      throw new Error(errorMsg);
    }
  };

  onCleanup(() => {
    stopHealthMonitoring();
  });

  return {
    metrics,
    health,
    isMonitoring,
    error,
    getMetrics,
    getMetric,
    incrementCounter,
    setGauge,
    recordHistogram,
    exportMetrics,
    resetMetrics,
    getSystemUptime,
    getMetricsCount,
    getHealth,
    getHealthWithThresholds,
    isSystemHealthy,
    getSystemStatus,
    startHealthMonitoring,
    stopHealthMonitoring,
    initTracing,
    getDefaultLogDir,
    parseLogLevel,
  };
}
