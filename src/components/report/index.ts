// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Report components for forensic report generation
 * 
 * The ReportWizard is now split into a modular structure under ./wizard/
 * for better maintainability. The old ReportWizard.tsx is kept for
 * backward compatibility but new development should use the wizard module.
 */

// Main component - now uses the modular wizard structure
export { ReportWizard } from './wizard';

// Types - re-export from types.ts
export type {
  Classification,
  Severity,
  EvidenceType,
  HashAlgorithmType,
  AppendixContentType,
  SignatureRole,
  OutputFormatType,
  ReportMetadata,
  CaseInfo,
  ExaminerInfo,
  HashValue,
  EvidenceItem,
  Finding,
  TimelineEvent,
  ToolInfo,
  CustodyRecord,
  HashRecord,
  Appendix,
  SignatureRecord,
  OutputFormat,
  ForensicReport,
} from './types';

// Constants
export {
  CLASSIFICATIONS,
  SEVERITIES,
  EVIDENCE_TYPES,
  INVESTIGATION_TYPES,
  FINDING_CATEGORIES,
  CUSTODY_ACTIONS,
} from './constants';

// Presets
export type { ReportPreset, ReportPresetSections, ReportPresetConfig } from './constants';
export { REPORT_PRESETS, getPresetById, getDefaultPreset } from './constants';
