// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Report components for forensic report generation
 */

// Main component
export { ReportWizard } from './ReportWizard';

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

// Templates
export type { ReportTemplateType, ReportTemplate, ReportTemplateSections } from './templates';
export { REPORT_TEMPLATES, getTemplateById, getDefaultTemplate } from './templates';
