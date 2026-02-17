// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ViewerMetadata - Unified metadata types emitted by specialized viewers
 *
 * Each viewer type can emit structured metadata to the right panel.
 * Uses a discriminated union pattern so the panel can render
 * type-specific tabs/sections.
 */

// =============================================================================
// File Info (always present when a file is being viewed)
// =============================================================================

export interface FileInfoMetadata {
  name: string;
  path: string;
  size: number;
  /** Extension without dot */
  extension?: string;
  /** Container the file lives in (AD1, E01, ZIP, etc.) */
  containerPath?: string;
  containerType?: string;
  /** Whether this is a disk file vs container entry */
  isDiskFile?: boolean;
  /** Whether this entry is from a VFS (E01/Raw) */
  isVfsEntry?: boolean;
  /** Whether this entry is from an archive */
  isArchiveEntry?: boolean;
}

// =============================================================================
// EXIF Metadata (from images)
// =============================================================================

export interface ExifMetadataSection {
  kind: "exif";
  // Camera
  make?: string;
  model?: string;
  software?: string;
  lensModel?: string;
  // Capture
  exposureTime?: string;
  fNumber?: string;
  iso?: number;
  focalLength?: string;
  flash?: string;
  // Timestamps
  dateTimeOriginal?: string;
  dateTimeDigitized?: string;
  dateTime?: string;
  gpsTimestamp?: string;
  // GPS
  gps?: {
    latitude: number;
    longitude: number;
    altitude?: number;
    latitudeRef: string;
    longitudeRef: string;
  };
  // Image
  width?: number;
  height?: number;
  orientation?: number;
  colorSpace?: string;
  // Forensic
  imageUniqueId?: string;
  ownerName?: string;
  serialNumber?: string;
  // Raw tags count
  rawTagCount?: number;
}

// =============================================================================
// Registry Hive Metadata (from Windows registry hive files)
// =============================================================================

export interface RegistryMetadataSection {
  kind: "registry";
  hiveName: string;
  hiveType: string;
  rootKeyName: string;
  totalKeys: number;
  totalValues: number;
  /** True when key/value counts were capped due to hive size limits */
  capped?: boolean;
  lastModified?: string;
  /** Currently selected key path */
  selectedKeyPath?: string;
  selectedKeyInfo?: {
    subkeyCount: number;
    valueCount: number;
    lastModified?: string;
    className?: string;
  };
}

// =============================================================================
// Database Metadata (from SQLite databases)
// =============================================================================

export interface DatabaseMetadataSection {
  kind: "database";
  path: string;
  pageSize: number;
  pageCount: number;
  sizeBytes: number;
  tableCount: number;
  tables: Array<{
    name: string;
    rowCount: number;
    columnCount: number;
    isSystem: boolean;
  }>;
  /** Currently selected table */
  selectedTable?: string;
}

// =============================================================================
// Binary Executable Metadata (PE/ELF/Mach-O)
// =============================================================================

export interface BinaryMetadataSection {
  kind: "binary";
  format: string;
  architecture?: string;
  entryPoint?: string;
  sectionCount?: number;
  importCount?: number;
  exportCount?: number;
  isStripped?: boolean;
  isDynamic?: boolean;
  compiler?: string;
  // PE-specific
  subsystem?: string;
  characteristics?: string[];
  // Timestamps
  compiledDate?: string;
}

// =============================================================================
// Email Metadata (EML/MBOX)
// =============================================================================

export interface EmailMetadataSection {
  kind: "email";
  subject?: string;
  from?: string;
  to?: string[];
  cc?: string[];
  date?: string;
  messageId?: string;
  inReplyTo?: string;
  contentType?: string;
  attachmentCount?: number;
  /** For MBOX files */
  messageCount?: number;
  /** Currently selected message index (MBOX) */
  selectedMessageIndex?: number;
}

// =============================================================================
// Plist Metadata (Apple property lists)
// =============================================================================

export interface PlistMetadataSection {
  kind: "plist";
  format: string;
  entryCount: number;
  rootType: string;
  /** Notable keys found (e.g., CFBundleIdentifier) */
  notableKeys?: Array<{ key: string; value: string }>;
}

// =============================================================================
// Document Metadata (PDF, DOCX, etc.)
// =============================================================================

export interface DocumentMetadataSection {
  kind: "document";
  format: string;
  title?: string;
  author?: string;
  creator?: string;
  producer?: string;
  creationDate?: string;
  modificationDate?: string;
  pageCount?: number;
  wordCount?: number;
  encrypted?: boolean;
  keywords?: string[];
}

// =============================================================================
// Spreadsheet Metadata
// =============================================================================

export interface SpreadsheetMetadataSection {
  kind: "spreadsheet";
  format: string;
  sheetCount: number;
  sheets: Array<{ name: string; rowCount: number; columnCount: number }>;
  selectedSheet?: string;
}

// =============================================================================
// Unified ViewerMetadata
// =============================================================================

/** Discriminated union of all viewer-specific metadata sections */
export type ViewerMetadataSection =
  | ExifMetadataSection
  | RegistryMetadataSection
  | DatabaseMetadataSection
  | BinaryMetadataSection
  | EmailMetadataSection
  | PlistMetadataSection
  | DocumentMetadataSection
  | SpreadsheetMetadataSection;

/**
 * ViewerMetadata - Complete metadata emitted by ContainerEntryViewer
 *
 * Contains the always-present file info plus zero or more
 * viewer-specific metadata sections.
 */
export interface ViewerMetadata {
  /** Basic file information (always present) */
  fileInfo: FileInfoMetadata;
  /** Active viewer type identifier */
  viewerType: string;
  /** Viewer-specific metadata sections */
  sections: ViewerMetadataSection[];
}
