// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Per-project SQLite database (.ffxdb) types
 *
 * These types mirror the Rust `project_db` module structs and are used
 * for IPC with the Tauri backend. The .ffxdb file lives alongside the
 * .cffx manifest in the case folder.
 *
 * Naming convention: all fields use camelCase (Rust uses #[serde(rename_all = "camelCase")])
 */

// -----------------------------------------------------------------------------
// Constants
// -----------------------------------------------------------------------------

/** File extension for project databases */
export const PROJECT_DB_EXTENSION = ".ffxdb";

// -----------------------------------------------------------------------------
// Activity Log
// -----------------------------------------------------------------------------

/** Activity log entry — immutable audit trail of examiner actions */
export interface DbActivityEntry {
  id: string;
  timestamp: string;
  user: string;
  category: string;
  action: string;
  description: string;
  filePath?: string;
  /** JSON-encoded additional details */
  details?: string;
}

/** Query parameters for activity log filtering */
export interface ActivityQuery {
  category?: string;
  user?: string;
  since?: string;
  until?: string;
  search?: string;
  limit?: number;
  offset?: number;
}

// -----------------------------------------------------------------------------
// Sessions & Users
// -----------------------------------------------------------------------------

/** Session record — tracks examiner work sessions */
export interface DbProjectSession {
  sessionId: string;
  user: string;
  startedAt: string;
  endedAt?: string;
  durationSeconds?: number;
  hostname?: string;
  appVersion: string;
  summary?: string;
}

/** User/examiner record */
export interface DbProjectUser {
  username: string;
  displayName?: string;
  hostname?: string;
  firstAccess: string;
  lastAccess: string;
}

// -----------------------------------------------------------------------------
// Evidence Files
// -----------------------------------------------------------------------------

/** Evidence file record — discovered forensic containers */
export interface DbEvidenceFile {
  id: string;
  path: string;
  filename: string;
  containerType: string;
  totalSize: number;
  segmentCount: number;
  discoveredAt: string;
  created?: string;
  modified?: string;
}

// -----------------------------------------------------------------------------
// Hashes & Verifications
// -----------------------------------------------------------------------------

/** Hash record — immutable audit trail of hash computations */
export interface DbProjectHash {
  id: string;
  fileId: string;
  algorithm: string;
  hashValue: string;
  computedAt: string;
  segmentIndex?: number;
  segmentName?: string;
  /** 'computed', 'stored', 'imported' */
  source: string;
}

/** Verification record — hash verification audit trail */
export interface DbProjectVerification {
  id: string;
  hashId: string;
  verifiedAt: string;
  /** 'match' or 'mismatch' */
  result: string;
  expectedHash: string;
  actualHash: string;
}

// -----------------------------------------------------------------------------
// Bookmarks & Notes
// -----------------------------------------------------------------------------

/** Bookmark record */
export interface DbBookmark {
  id: string;
  /** 'file', 'artifact', 'search_result', 'location' */
  targetType: string;
  targetPath: string;
  name: string;
  createdBy: string;
  createdAt: string;
  color?: string;
  notes?: string;
  /** JSON-encoded additional context */
  context?: string;
}

/** Note/annotation record */
export interface DbNote {
  id: string;
  /** 'file', 'artifact', 'database', 'case', 'general' */
  targetType: string;
  targetPath?: string;
  title: string;
  /** Supports markdown */
  content: string;
  createdBy: string;
  createdAt: string;
  modifiedAt: string;
  priority?: string;
}

// -----------------------------------------------------------------------------
// Tags
// -----------------------------------------------------------------------------

/** Tag definition record */
export interface DbTag {
  id: string;
  name: string;
  /** Hex color string */
  color: string;
  description?: string;
  createdAt: string;
}

/** Tag assignment (many-to-many) */
export interface DbTagAssignment {
  tagId: string;
  /** 'bookmark', 'note', 'file', 'artifact' */
  targetType: string;
  targetId: string;
  assignedAt: string;
  assignedBy: string;
}

// -----------------------------------------------------------------------------
// Tabs & UI State
// -----------------------------------------------------------------------------

/** Tab record for UI state */
export interface DbProjectTab {
  id: string;
  tabType: string;
  filePath: string;
  name: string;
  subtitle?: string;
  tabOrder: number;
  /** JSON-encoded extra fields (containerType, entryPath, etc.) */
  extra?: string;
}

// -----------------------------------------------------------------------------
// Reports
// -----------------------------------------------------------------------------

/** Report record */
export interface DbReportRecord {
  id: string;
  title: string;
  reportType: string;
  format: string;
  outputPath?: string;
  generatedAt: string;
  generatedBy: string;
  status: string;
  error?: string;
  /** JSON-encoded config */
  config?: string;
}

// -----------------------------------------------------------------------------
// Searches
// -----------------------------------------------------------------------------

/** Saved search record */
export interface DbSavedSearch {
  id: string;
  name: string;
  query: string;
  searchType: string;
  isRegex: boolean;
  caseSensitive: boolean;
  scope: string;
  createdAt: string;
  useCount: number;
  lastUsed?: string;
}

/** Recent search record */
export interface DbRecentSearch {
  query: string;
  timestamp: string;
  resultCount: number;
}

// -----------------------------------------------------------------------------
// Case Documents
// -----------------------------------------------------------------------------

/** Case document record */
export interface DbCaseDocument {
  id: string;
  path: string;
  filename: string;
  documentType: string;
  size: number;
  format: string;
  caseNumber?: string;
  evidenceId?: string;
  modified?: string;
  discoveredAt: string;
}

// -----------------------------------------------------------------------------
// Statistics
// -----------------------------------------------------------------------------

/** Project database statistics summary */
export interface ProjectDbStats {
  totalActivities: number;
  totalSessions: number;
  totalUsers: number;
  totalEvidenceFiles: number;
  totalHashes: number;
  totalVerifications: number;
  totalBookmarks: number;
  totalNotes: number;
  totalTags: number;
  totalReports: number;
  totalSavedSearches: number;
  totalCaseDocuments: number;
  totalProcessedDatabases: number;
  totalAxiomCases: number;
  totalArtifactCategories: number;
  totalExports: number;
  totalCustodyRecords: number;
  totalClassifications: number;
  totalExtractions: number;
  totalViewerHistory: number;
  totalAnnotations: number;
  totalRelationships: number;
  totalCocItems: number;
  totalCocTransfers: number;
  totalEvidenceCollections: number;
  totalCollectedItems: number;
  dbSizeBytes: number;
  schemaVersion: number;
}

// -----------------------------------------------------------------------------
// Processed Databases (read-only snapshots of AXIOM, PA, etc.)
// -----------------------------------------------------------------------------

/** A registered processed database (AXIOM, Cellebrite PA, X-Ways, etc.) */
export interface DbProcessedDatabase {
  id: string;
  /** Path to the processed database folder or file */
  path: string;
  /** Display name */
  name: string;
  /** 'MagnetAxiom', 'CellebritePA', 'XWays', 'Autopsy', 'EnCase', 'FTK', 'GenericSqlite', 'Unknown' */
  dbType: string;
  caseNumber?: string;
  examiner?: string;
  createdDate?: string;
  totalSize: number;
  artifactCount?: number;
  notes?: string;
  /** When this record was registered in CORE-FFX */
  registeredAt: string;
  /** JSON-encoded metadata snapshot */
  metadataJson?: string;
}

/** Integrity record for a processed database file */
export interface DbProcessedDbIntegrity {
  id: string;
  /** FK to processed database */
  processedDbId: string;
  /** Database file path */
  filePath: string;
  fileSize: number;
  /** Hash when first loaded */
  baselineHash: string;
  baselineTimestamp: string;
  /** Most recent hash check */
  currentHash?: string;
  currentHashTimestamp?: string;
  /** 'unchanged', 'modified', 'new_baseline', 'not_verified' */
  status: string;
  /** JSON array of detected changes */
  changesJson?: string;
}

/** Work metrics extracted from a processed database */
export interface DbProcessedDbMetrics {
  id: string;
  processedDbId: string;
  totalScans: number;
  lastScanDate?: string;
  totalJobs: number;
  lastJobDate?: string;
  totalNotes: number;
  totalTaggedItems: number;
  totalUsers: number;
  /** JSON array of user names */
  userNamesJson?: string;
  /** When these metrics were captured */
  capturedAt: string;
}

// -----------------------------------------------------------------------------
// AXIOM-Specific Types
// -----------------------------------------------------------------------------

/** AXIOM case information snapshot */
export interface DbAxiomCaseInfo {
  id: string;
  processedDbId: string;
  caseName: string;
  caseNumber?: string;
  caseType?: string;
  description?: string;
  examiner?: string;
  agency?: string;
  axiomVersion?: string;
  searchStart?: string;
  searchEnd?: string;
  searchDuration?: string;
  searchOutcome?: string;
  outputFolder?: string;
  totalArtifacts: number;
  casePath?: string;
  capturedAt: string;
  /** JSON-encoded keyword info */
  keywordInfoJson?: string;
}

/** AXIOM evidence source */
export interface DbAxiomEvidenceSource {
  id: string;
  axiomCaseId: string;
  name: string;
  evidenceNumber?: string;
  /** 'image', 'mobile', 'cloud', etc. */
  sourceType: string;
  path?: string;
  hash?: string;
  size?: number;
  acquired?: string;
  /** JSON array of search types */
  searchTypesJson?: string;
}

/** AXIOM search result (artifact type + hit count) */
export interface DbAxiomSearchResult {
  id: string;
  axiomCaseId: string;
  artifactType: string;
  hitCount: number;
}

/** Artifact category summary (works for any processed DB type) */
export interface DbArtifactCategory {
  id: string;
  processedDbId: string;
  category: string;
  artifactType: string;
  count: number;
}

// -----------------------------------------------------------------------------
// v3: Forensic Workflow Types
// -----------------------------------------------------------------------------

/** Export history record */
export interface DbExportRecord {
  id: string;
  /** 'file', 'archive', 'report', etc. */
  exportType: string;
  /** JSON-encoded array of source file paths */
  sourcePathsJson: string;
  destination: string;
  startedAt: string;
  completedAt?: string;
  initiatedBy: string;
  /** 'pending', 'in_progress', 'completed', 'failed', 'cancelled' */
  status: string;
  totalFiles: number;
  totalBytes: number;
  archiveName?: string;
  archiveFormat?: string;
  compressionLevel?: string;
  encrypted: boolean;
  manifestHash?: string;
  error?: string;
  /** JSON-encoded export options */
  optionsJson?: string;
}

/** Chain of custody record */
export interface DbCustodyRecord {
  id: string;
  /** 'received', 'transferred', 'returned', 'acquired', etc. */
  action: string;
  fromPerson: string;
  toPerson: string;
  date: string;
  time?: string;
  location?: string;
  purpose?: string;
  notes?: string;
  /** JSON-encoded array of evidence IDs involved */
  evidenceIdsJson?: string;
  recordedBy: string;
  recordedAt: string;
}

// -----------------------------------------------------------------------------
// COC Items (v4 — detailed chain-of-custody evidence items)
// -----------------------------------------------------------------------------

/** A COC item — detailed evidence item with custody tracking */
export interface DbCocItem {
  id: string;
  cocNumber: string;
  /** FK to evidence_files.id — links this COC entry to a discovered container */
  evidenceFileId?: string;
  /** NOT NULL in schema — must always be provided (empty string OK) */
  caseNumber: string;
  /** NOT NULL in schema — must always be provided (empty string OK) */
  evidenceId: string;
  description: string;
  /** NOT NULL in schema — must always be provided (empty string OK) */
  itemType: string;
  make?: string;
  model?: string;
  serialNumber?: string;
  capacity?: string;
  condition: string;
  /** NOT NULL in schema — must always be provided (empty string OK) */
  acquisitionDate: string;
  /** NOT NULL in schema — must always be provided (empty string OK) */
  enteredCustodyDate: string;
  /** NOT NULL in schema — must always be provided (empty string OK) */
  submittedBy: string;
  /** NOT NULL in schema — must always be provided (empty string OK) */
  receivedBy: string;
  receivedLocation?: string;
  storageLocation?: string;
  reasonSubmitted?: string;
  /** JSON-encoded array of { algorithm, value } hash objects */
  intakeHashesJson?: string;
  notes?: string;
  disposition?: string;
  dispositionDate?: string;
  dispositionNotes?: string;
  createdAt: string;
  modifiedAt: string;
  /** Immutability status: 'draft', 'locked', 'voided' */
  status: string;
  /** When the item was locked (ISO 8601) */
  lockedAt?: string;
  /** Who locked the item (initials) */
  lockedBy?: string;
}

/** A COC transfer — custody handoff record for a COC item */
export interface DbCocTransfer {
  id: string;
  /** FK to coc_items.id */
  cocItemId: string;
  timestamp: string;
  releasedBy: string;
  receivedBy: string;
  purpose: string;
  location?: string;
  method?: string;
  notes?: string;
}

// -----------------------------------------------------------------------------
// Evidence Collections (v4 — scene/collection-level records)
// -----------------------------------------------------------------------------

/** An evidence collection — documents a collection event/scene */
export interface DbEvidenceCollection {
  id: string;
  /** NOT NULL in schema — must always be provided (empty string OK) */
  caseNumber: string;
  /** NOT NULL in schema — must always be provided (empty string OK) */
  collectionDate: string;
  /** NOT NULL in schema — must always be provided (empty string OK) */
  collectionLocation: string;
  /** NOT NULL in schema — must always be provided (empty string OK) */
  collectingOfficer: string;
  /** NOT NULL in schema — must always be provided (empty string OK) */
  authorization: string;
  authorizationDate?: string;
  authorizingAuthority?: string;
  /** JSON-encoded array of witness names */
  witnessesJson?: string;
  documentationNotes?: string;
  conditions?: string;
  /** Status lifecycle: draft → complete → locked */
  status: string;
  /** Number of collected items (populated by summary queries) */
  itemCount?: number;
  createdAt: string;
  modifiedAt: string;
}

/** A collected item — individual item gathered during a collection event */
export interface DbCollectedItem {
  id: string;
  /** FK to evidence_collections.id */
  collectionId: string;
  /** FK to coc_items.id — cross-references this item to its COC record */
  cocItemId?: string;
  /** FK to evidence_files.id — links this item to a discovered container */
  evidenceFileId?: string;
  itemNumber: string;
  description: string;
  /** NOT NULL in schema — must always be provided (empty string OK) */
  foundLocation: string;
  /** NOT NULL in schema — must always be provided (empty string OK) */
  itemType: string;
  make?: string;
  model?: string;
  serialNumber?: string;
  condition: string;
  /** NOT NULL in schema — must always be provided (empty string OK) */
  packaging: string;
  /** JSON-encoded array of photo reference strings */
  photoRefsJson?: string;
  notes?: string;

  // --- Per-Item Collection Info (v8) ---
  itemCollectionDatetime?: string;
  itemSystemDatetime?: string;
  itemCollectingOfficer?: string;
  itemAuthorization?: string;

  // --- Device Identification (v8) ---
  deviceType?: string;
  deviceTypeOther?: string;
  storageInterface?: string;
  storageInterfaceOther?: string;
  brand?: string;
  color?: string;
  imei?: string;
  otherIdentifiers?: string;

  // --- Location (v8) ---
  building?: string;
  room?: string;
  locationOther?: string;

  // --- Forensic Image (v8) ---
  imageFormat?: string;
  imageFormatOther?: string;
  acquisitionMethod?: string;
  acquisitionMethodOther?: string;

  // --- Additional Info (v8) ---
  storageNotes?: string;
}

// -----------------------------------------------------------------------------
// COC Amendments & Audit Trail (v5 — immutability model)
// -----------------------------------------------------------------------------

/** A COC amendment — tracks a single field change on a COC item */
export interface DbCocAmendment {
  id: string;
  /** FK to coc_items.id */
  cocItemId: string;
  /** Which field was amended */
  fieldName: string;
  /** Value before amendment */
  oldValue: string;
  /** Value after amendment */
  newValue: string;
  /** Initials of the person making the amendment */
  amendedByInitials: string;
  /** Timestamp of amendment (ISO 8601) */
  amendedAt: string;
  /** Reason for amendment */
  reason?: string;
}

/** A COC audit log entry — immutable record of a COC action */
export interface DbCocAuditEntry {
  id: string;
  /** FK to coc_items.id (nullable for system-level actions) */
  cocItemId?: string;
  /** Action: 'created', 'amended', 'locked', 'voided', 'transfer_added', etc. */
  action: string;
  /** Who performed the action */
  performedBy: string;
  /** When the action was performed (ISO 8601) */
  performedAt: string;
  /** Human-readable summary */
  summary: string;
  /** JSON-encoded details */
  detailsJson?: string;
}

/** File classification (examiner-assigned labels) */
export interface DbFileClassification {
  id: string;
  filePath: string;
  containerPath?: string;
  /** 'relevant', 'irrelevant', 'privileged', 'contraband', 'responsive', etc. */
  classification: string;
  customLabel?: string;
  classifiedBy: string;
  classifiedAt: string;
  notes?: string;
  /** 'high', 'medium', 'low' */
  confidence?: string;
}

/** Extraction log entry (immutable audit trail) */
export interface DbExtractionRecord {
  id: string;
  containerPath: string;
  entryPath: string;
  outputPath: string;
  extractedBy: string;
  extractedAt: string;
  entrySize: number;
  /** 'preview', 'export', 'analysis', 'report' */
  purpose: string;
  hashValue?: string;
  hashAlgorithm?: string;
  /** 'success', 'failed', 'partial' */
  status: string;
  error?: string;
}

/** Viewer history entry */
export interface DbViewerHistoryEntry {
  id: string;
  filePath: string;
  containerPath?: string;
  /** 'hex', 'text', 'image', 'document', 'spreadsheet', etc. */
  viewerType: string;
  viewedBy: string;
  openedAt: string;
  closedAt?: string;
  durationSeconds?: number;
}

/** Annotation (hex/document viewer highlights and comments) */
export interface DbAnnotation {
  id: string;
  filePath: string;
  containerPath?: string;
  /** 'highlight', 'comment', 'bookmark', 'region' */
  annotationType: string;
  offsetStart?: number;
  offsetEnd?: number;
  lineStart?: number;
  lineEnd?: number;
  label: string;
  content?: string;
  /** Hex color like '#ff0000' */
  color?: string;
  createdBy: string;
  createdAt: string;
  modifiedAt: string;
}

/** Evidence relationship (links between evidence files) */
export interface DbEvidenceRelationship {
  id: string;
  sourcePath: string;
  targetPath: string;
  /** 'contains', 'derived_from', 'related_to', 'duplicate_of', etc. */
  relationshipType: string;
  description?: string;
  createdBy: string;
  createdAt: string;
}

/** Full-text search result */
export interface FtsSearchResult {
  /** 'notes', 'bookmarks', 'activity_log' */
  source: string;
  id: string;
  /** Matched text snippet with <mark> highlights */
  snippet: string;
  /** BM25 relevance rank (lower = better match) */
  rank: number;
}

/** Generic JSON-driven form submission (schema v6) */
export interface DbFormSubmission {
  id: string;
  templateId: string;
  templateVersion: string;
  caseNumber?: string;
  /** JSON blob containing the form field values */
  dataJson: string;
  /** 'draft' | 'complete' | 'locked' */
  status: string;
  createdAt: string;
  updatedAt: string;
}
