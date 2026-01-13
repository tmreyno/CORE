// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// =============================================================================
// REPORT MODULE - JSON-based forensic report generation
// =============================================================================

// Type exports
export type {
  ForensicReport,
  ReportMeta,
  CaseInfo,
  EvidenceItem,
  HashRecord,
  ContainerMetadata,
  DeviceInfo,
  ExtractionInfo,
  SessionInfo,
  ExportOptions,
  ExportFormat,
} from "./types";

// Generator exports
export {
  generateReport,
  exportAsJson,
  exportAsMarkdown,
  type ReportInput,
} from "./generator";

// API exports (Tauri commands)
export {
  containerToInput,
  containersToInputs,
  extractEvidenceFromContainers,
  createEvidenceFromContainer,
  generateEvidenceFromFiles,
  getReportTemplate,
  isAiAvailable,
  exportReportJson,
  importReportJson,
  type ContainerInfoInput,
  type StoredHashInput,
  type EvidenceItem as BackendEvidenceItem,
  type HashRecord as BackendHashRecord,
  type ImageInfo,
} from "./api";
