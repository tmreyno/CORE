// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Report Types - Type definitions for forensic report generation
 * 
 * These types match the Rust backend structures for serialization compatibility.
 */

// =============================================================================
// ENUMS / UNION TYPES
// =============================================================================

/** Document classification levels */
export type Classification = 
  | "Public"
  | "Internal"
  | "Confidential"
  | "Restricted"
  | "LawEnforcementSensitive";

/** Finding severity levels */
export type Severity = "Critical" | "High" | "Medium" | "Low" | "Informational";

/** Evidence type categories - must match Rust EvidenceType enum */
export type EvidenceType = 
  | "HardDrive" 
  | "SSD" 
  | "UsbDrive" 
  | "ExternalDrive" 
  | "MemoryCard" 
  | "MobilePhone" 
  | "Tablet" 
  | "Computer" 
  | "Laptop" 
  | "OpticalDisc" 
  | "CloudStorage" 
  | "NetworkCapture" 
  | "ForensicImage" 
  | "Other";

/** Hash algorithm types - must match Rust HashAlgorithmType enum */
export type HashAlgorithmType = 
  | "MD5" 
  | "SHA1" 
  | "SHA256" 
  | "SHA512" 
  | "Blake2b" 
  | "Blake3" 
  | "XXH3" 
  | "XXH64";

/** Appendix content types */
export type AppendixContentType = 
  | "Markdown" 
  | "Text" 
  | "FileListing" 
  | "HashTable" 
  | "FileReference";

/** Signature role types */
export type SignatureRole = "examiner" | "supervisor" | "reviewer";

/** Output format types */
export type OutputFormatType = "Pdf" | "Docx" | "Html" | "Markdown" | "Typst";

// =============================================================================
// CORE INTERFACES
// =============================================================================

/** Report metadata */
export interface ReportMetadata {
  title: string;
  report_number: string;
  version: string;
  classification: Classification;
  generated_at: string;
  generated_by: string;
}

/** Case information */
export interface CaseInfo {
  case_number: string;
  case_name?: string;
  agency?: string;
  requestor?: string;
  request_date?: string;
  exam_start_date?: string;
  exam_end_date?: string;
  investigation_type?: string;
  description?: string;
}

/** Examiner information */
export interface ExaminerInfo {
  name: string;
  title?: string;
  organization?: string;
  email?: string;
  phone?: string;
  certifications: string[];
  badge_number?: string;
}

/** Hash value record */
export interface HashValue {
  item: string;
  algorithm: HashAlgorithmType;
  value: string;
  computed_at?: string;
  verified?: boolean;
}

/** Evidence item */
export interface EvidenceItem {
  evidence_id: string;
  description: string;
  evidence_type: EvidenceType;
  make?: string;
  model?: string;
  serial_number?: string;
  capacity?: string;
  acquisition_date?: string;
  acquisition_method?: string;
  acquisition_tool?: string;
  acquisition_hashes: HashValue[];
  verification_hashes: HashValue[];
  notes?: string;
}

/** Finding/artifact record */
export interface Finding {
  id: string;
  title: string;
  severity: Severity;
  category: string;
  description: string;
  artifact_paths: string[];
  timestamps: string[];
  evidence_refs: string[];
  analysis: string;
  conclusions?: string;
}

/** Timeline event */
export interface TimelineEvent {
  timestamp: string;
  event_type: string;
  description: string;
  source: string;
  evidence_ref?: string;
  artifact_path?: string;
}

/** Tool information */
export interface ToolInfo {
  name: string;
  version: string;
  vendor?: string;
  purpose?: string;
}

/** Chain of custody record */
export interface CustodyRecord {
  timestamp: string;
  action: string;
  handler: string;
  location?: string;
  notes?: string;
}

/** Hash record for report */
export interface HashRecord {
  algorithm: string;
  value: string;
  verified?: boolean;
  timestamp?: string;
  item_reference?: string;
}

/** Report appendix */
export interface Appendix {
  appendix_id: string;
  title: string;
  content_type: AppendixContentType;
  content: string;
}

/** Signature/approval record */
export interface SignatureRecord {
  role: SignatureRole;
  name: string;
  signature?: string;
  signed_date?: string;
  notes?: string;
  certified?: boolean;
}

/** Output format configuration */
export interface OutputFormat {
  format: OutputFormatType;
  name: string;
  description: string;
  extension: string;
  supported: boolean;
}

// =============================================================================
// MAIN REPORT INTERFACE
// =============================================================================

/** Complete forensic report structure */
export interface ForensicReport {
  metadata: ReportMetadata;
  case_info: CaseInfo;
  examiner: ExaminerInfo;
  executive_summary?: string;
  scope?: string;
  methodology?: string;
  evidence_items: EvidenceItem[];
  chain_of_custody: CustodyRecord[];
  findings: Finding[];
  timeline: TimelineEvent[];
  hash_records: HashRecord[];
  tools: ToolInfo[];
  conclusions?: string;
  appendices: Appendix[];
  signatures?: SignatureRecord[];
  notes?: string;
}
