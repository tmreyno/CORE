// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Utility Functions and Classes
 * @module utils
 */

// =============================================================================
// Error Utilities
// =============================================================================
export { getErrorMessage } from './errorUtils';

// =============================================================================
// Logger Utility
// =============================================================================
export { logger, type LogLevel } from './logger';

// =============================================================================
// Platform Detection
// =============================================================================
export {
  platform,
  isMac,
  isWindows,
  isLinux,
  isIOS,
  isAndroid,
  isMobile,
  isDesktop,
  isTauri,
  isPrimaryModifier,
  formatShortcut,
} from './platform';

// =============================================================================
// Performance Monitoring
// =============================================================================
export {
  useRenderTracker,
  trackEffect,
  trackMemo,
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
  type RenderMetrics,
  type MemoryMetrics,
  type PerformanceEntry,
  type PerformanceSummary,
} from './performance';

// =============================================================================
// Accessibility Utilities
// =============================================================================
export {
  announce,
  initAnnouncer,
  getFocusableElements,
  type AriaLive,
} from './accessibility';

// =============================================================================
// Error Telemetry
// =============================================================================
export {
  logError,
  logInfo,
  initGlobalErrorHandlers,
  removeGlobalErrorHandlers,
} from './telemetry';

// =============================================================================
// Processed Data Utilities
// =============================================================================
export * from './processed';

// =============================================================================
// Metadata Display Utilities
// =============================================================================
export {
  formatDate,
  formatTimestamp,
  formatNumber,
  formatCount,
  formatOffset,
  formatDecimalOffset,
  truncateHash,
  formatAlgorithm,
  getVerificationStatus,
  getVerificationIcon,
  getVerificationClass,
  createHashDisplayMemo,
  createDateDisplayMemo,
  createTimestampDisplayMemo,
  filterEmptyFields,
  groupFieldsByCategory,
  sortFieldsByLabel,
  getSourceIndicator,
  getSourceDescription,
  SOURCE_INDICATORS,
  type DisplayField,
  type VerificationStatus,
  type HashInfo,
} from './metadata';

