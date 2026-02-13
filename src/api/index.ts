// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * API Module Index
 * 
 * Re-exports typed Tauri command wrappers and API utilities.
 */

export {
  commands,
  containerCommands,
  hashCommands,
  databaseCommands,
  recoveryCommands,
  notificationCommands,
  indexCacheCommands,
  extractionCommands,
  dedupCommands,
  observabilityCommands,
  profilerCommands,
  regressionCommands,
  systemCommands,
  type OperationType,
  type RecoveryState,
  type NotificationType,
  type HashAlgorithm,
  type RecoveryOperation,
  type RecoveryStats,
  type FileRecord,
  type HashRecord,
  type VerificationRecord,
  type ExtractionItem,
  type HealthStatus,
} from './commands';

// File export API
export {
  exportFiles,
  formatBytes,
  formatDuration,
  calculateSpeed,
  type CopyProgress,
  type CopyResult,
  type ExportMetadata,
} from './fileExport';
