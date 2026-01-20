// =============================================================================
// CORE-FFX - Forensic File Explorer
// Case Document Types - Chain of custody and intake forms
// =============================================================================

/**
 * Case Document Types
 * 
 * Types for case documents such as chain of custody forms, intake forms,
 * lab requests, and other case-related documentation.
 */

// ============================================================================
// Document Type Enum
// ============================================================================

/** Types of case documents */
export type CaseDocumentType =
  | "ChainOfCustody"
  | "EvidenceIntake"
  | "CaseNotes"
  | "EvidenceReceipt"
  | "LabRequest"
  | "ExternalReport"
  | "Other";

// ============================================================================
// Case Document
// ============================================================================

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
