// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

export {
  useRenderTracker,
  trackEffect,
  trackMemo,
  trackAction,
  getMemoryMetrics,
  useMemoryMonitor,
  getPerformanceEntries,
  getPerformanceEntriesByType,
  getAllRenderMetrics,
  getRenderMetrics,
  clearPerformanceData,
  getPerformanceSummary,
  useFPSMonitor,
  formatDuration,
  getPerformanceGrade,
  setPerformanceMonitoringEnabled,
  isPerformanceMonitoringEnabled,
  type MemoryMetrics,
  type PerformanceEntry,
  type PerformanceSummary,
  type RenderMetrics,
} from "@core-suite/logging/performance";