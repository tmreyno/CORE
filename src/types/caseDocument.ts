// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Case document types for chain of custody, intake forms, etc.
 */

/** Types of case documents */
export type CaseDocumentType =
  | "ChainOfCustody"
  | "EvidenceIntake"
  | "CaseNotes"
  | "EvidenceReceipt"
  | "LabRequest"
  | "ExternalReport"
  | "Other";

/** A discovered case document (COC form, intake form, etc.) */
export type CaseDocument = {
  /** Full path to the document */
  path: string;
  /** Filename */
  filename: string;
  /** Document type */
  document_type: CaseDocumentType;
  /** File size in bytes */
  size: number;
  /** File format (PDF, DOCX, TXT, etc.) */
  format: string;
  /** Case number extracted from filename (if found) */
  case_number?: string | null;
  /** Evidence ID extracted from filename (if found) */
  evidence_id?: string | null;
  /** Last modified timestamp (ISO 8601) */
  modified?: string | null;
};
