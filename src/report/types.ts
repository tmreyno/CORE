// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// =============================================================================
// QUICK EXPORT REPORT - Simplified JSON schema for quick exports
// =============================================================================
// This defines a lightweight JSON structure for quick evidence export/summaries.
// For the full forensic report with wizard support (findings, timeline, custody,
// signatures, AI assistance), see: src/components/report/types.ts
//
// Canonical ForensicReport type: src/components/report/types.ts → ForensicReport
// This file defines: QuickExportReport (formerly ForensicReport)
// =============================================================================

/**
 * Quick export report structure — lightweight summary of evidence and hashes.
 * 
 * For the full wizard-generated forensic report, use ForensicReport from
 * `src/components/report/types.ts`.
 */
export interface QuickExportReport {
  /** Report schema version for future compatibility */
  schemaVersion: "1.0";
  /** Report metadata */
  meta: ReportMeta;
  /** Case information */
  case: CaseInfo;
  /** Evidence items (containers analyzed) */
  evidence: EvidenceItem[];
  /** Hash verification results */
  hashes: HashRecord[];
  /** Analysis session info */
  session?: SessionInfo;
}

/**
 * @deprecated Use QuickExportReport instead. The canonical ForensicReport is
 * defined in `src/components/report/types.ts`.
 */
export type ForensicReport = QuickExportReport;

/**
 * Report metadata
 */
export interface ReportMeta {
  /** When this report was generated */
  generatedAt: string; // ISO 8601
  /** Application that generated the report */
  generatedBy: string;
  /** Application version */
  appVersion: string;
  /** Report title (optional) */
  title?: string;
  /** Report notes/comments */
  notes?: string;
}

/**
 * Case information (from container metadata or user input)
 */
export interface CaseInfo {
  /** Case number/identifier */
  caseNumber?: string;
  /** Evidence number/identifier */
  evidenceNumber?: string;
  /** Examiner name */
  examiner?: string;
  /** Department/organization */
  department?: string;
  /** Location of examination */
  location?: string;
  /** Additional case notes */
  notes?: string;
}

/**
 * Single evidence item (forensic container)
 */
export interface EvidenceItem {
  /** Unique identifier for this item */
  id: string;
  /** Original filename */
  filename: string;
  /** Full path to file */
  path: string;
  /** Container type (AD1, E01, L01, Raw, UFED, Archive) */
  containerType: string;
  /** File size in bytes */
  size: number;
  /** Number of segments (for split containers) */
  segmentCount?: number;
  /** File creation timestamp */
  created?: string;
  /** File modification timestamp */
  modified?: string;
  /** Container-specific metadata */
  metadata: ContainerMetadata;
  /** Device information (if available) */
  device?: DeviceInfo;
  /** Extraction/acquisition information */
  extraction?: ExtractionInfo;
}

/**
 * Container-specific metadata (normalized across types)
 */
export interface ContainerMetadata {
  /** Container format/version string */
  format: string;
  /** Format version number */
  formatVersion?: string;
  /** Total logical size (uncompressed) */
  totalSize?: number;
  /** Compression type/level */
  compression?: string;
  /** Item/file count inside container */
  itemCount?: number;
  /** Entry count (for archives) */
  entryCount?: number;
  /** Chunk/sector information */
  chunkInfo?: {
    chunkSize?: number;
    chunkCount?: number;
    bytesPerSector?: number;
    sectorsPerChunk?: number;
  };
  /** Encryption status */
  encryption?: {
    encrypted: boolean;
    method?: string;
    headersEncrypted?: boolean;
  };
  /** Source description */
  sourceDescription?: string;
  /** Additional notes from container */
  notes?: string;
}

/**
 * Device information (for mobile/device extractions)
 */
export interface DeviceInfo {
  /** Device vendor/manufacturer */
  vendor?: string;
  /** Device model */
  model?: string;
  /** Full device name */
  fullName?: string;
  /** Serial number */
  serialNumber?: string;
  /** IMEI (primary) */
  imei?: string;
  /** IMEI (secondary/dual SIM) */
  imei2?: string;
  /** ICCID (SIM card) */
  iccid?: string;
  /** Operating system */
  os?: string;
  /** OS version */
  osVersion?: string;
}

/**
 * Extraction/acquisition information
 */
export interface ExtractionInfo {
  /** Acquisition tool name */
  tool?: string;
  /** Tool version */
  toolVersion?: string;
  /** Extraction type (logical, physical, filesystem, etc.) */
  extractionType?: string;
  /** Connection method */
  connectionType?: string;
  /** Extraction start time */
  startTime?: string;
  /** Extraction end time */
  endTime?: string;
  /** Machine/workstation name */
  machineName?: string;
  /** Unique extraction GUID */
  guid?: string;
  /** Unit/dongle ID */
  unitId?: string;
}

/**
 * Hash verification record
 */
export interface HashRecord {
  /** Reference to evidence item ID */
  evidenceId: string;
  /** Filename that was hashed */
  filename: string;
  /** Hash algorithm used */
  algorithm: "MD5" | "SHA1" | "SHA256" | "SHA512" | "XXH64" | "XXH3" | string;
  /** Computed hash value (uppercase hex) */
  computedHash: string;
  /** Stored/expected hash value (if available) */
  storedHash?: string;
  /** Verification result */
  verified: boolean | null;
  /** Source of stored hash (container, companion, computed) */
  source: "container" | "companion" | "computed" | "user";
  /** When hash was computed */
  computedAt: string;
  /** Duration in seconds */
  durationSecs?: number;
  /** Hash timestamp from source (e.g., extraction date) */
  sourceTimestamp?: string;
}

/**
 * Analysis session information
 */
export interface SessionInfo {
  /** Session start time */
  startedAt: string;
  /** Session end time (if completed) */
  endedAt?: string;
  /** Working directory */
  workingDirectory: string;
  /** Total files discovered */
  filesDiscovered: number;
  /** Files processed */
  filesProcessed: number;
  /** Errors encountered */
  errors?: SessionError[];
}

/**
 * Session error record
 */
export interface SessionError {
  /** Timestamp of error */
  timestamp: string;
  /** Error message */
  message: string;
  /** Related file path (if applicable) */
  filePath?: string;
  /** Error type/category */
  errorType?: string;
}

// =============================================================================
// EXPORT FORMAT OPTIONS
// =============================================================================

export type ExportFormat = "json" | "markdown" | "html" | "pdf" | "csv";

export interface ExportOptions {
  /** Output format */
  format: ExportFormat;
  /** Include hash history */
  includeHashHistory?: boolean;
  /** Include file tree */
  includeFileTree?: boolean;
  /** Include session info */
  includeSession?: boolean;
  /** Pretty print JSON */
  prettyPrint?: boolean;
  /** Custom title */
  title?: string;
}
