// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Tauri Commands API
 *
 * Type-safe wrappers for Tauri invoke calls. Use these instead of raw invoke()
 * calls for better type safety and centralized command management.
 *
 * @example
 * ```tsx
 * import { commands } from '../api/commands';
 *
 * // Container operations
 * const info = await commands.container.getInfo(path);
 * const children = await commands.container.getChildren(path, parentId);
 *
 * // Hash operations
 * await commands.hash.compute(path, 'SHA-256');
 *
 * // Database operations
 * await commands.database.upsertFile(fileData);
 * ```
 */

import { invoke } from "@tauri-apps/api/core";
import type {
  LazyLoadResult,
  ContainerSummary,
  LazyLoadConfig,
} from "../types/lazy-loading";
import type { FileTypeInfo, ParsedMetadata } from "../types/viewer";

// =============================================================================
// Type Definitions
// =============================================================================

/** Operation types for recovery */
export type OperationType =
  "hash_verification" | "extraction" | "indexing" | "scan";

/** Recovery operation states */
export type RecoveryState =
  "pending" | "in_progress" | "paused" | "completed" | "failed" | "cancelled";

/** Notification types */
export type NotificationType = "info" | "success" | "warning" | "error";

/** Hash algorithm types */
export type HashAlgorithm =
  | "MD5"
  | "SHA-1"
  | "SHA-256"
  | "SHA-512"
  | "BLAKE3"
  | "BLAKE2"
  | "XXH3"
  | "XXH64"
  | "CRC32";

/** Source-aware hash input for local files and container entries */
export interface HashSourceInput {
  path?: string;
  containerPath?: string;
  entryPath?: string;
  nestedArchivePath?: string;
  containerType?: string;
  size?: number;
  dataAddr?: number | null;
  itemAddr?: number | null;
}

/** Source-aware hash result */
export interface HashSourceResult {
  sourceRef: EvidenceSourceRef;
  sourceId: string;
  path?: string;
  containerPath?: string;
  entryPath?: string;
  containerType?: string;
  algorithm: string;
  hash: string;
  bytesHashed: number;
  durationMs: number;
  throughputMbs?: number;
}

export interface ExifGpsCoordinates {
  latitude: number;
  longitude: number;
  altitude: number | null;
  latitude_ref: string;
  longitude_ref: string;
}

export interface ExifMetadata {
  path: string;
  make: string | null;
  model: string | null;
  software: string | null;
  lens_model: string | null;
  exposure_time: string | null;
  f_number: string | null;
  iso: number | null;
  focal_length: string | null;
  flash: string | null;
  date_time_original: string | null;
  date_time_digitized: string | null;
  date_time: string | null;
  gps_timestamp: string | null;
  gps: ExifGpsCoordinates | null;
  width: number | null;
  height: number | null;
  orientation: number | null;
  color_space: string | null;
  image_unique_id: string | null;
  owner_name: string | null;
  serial_number: string | null;
  raw_tags: [string, string][];
}

/** Evidence file record used by the per-project .ffxdb database */
export interface ProjectDbEvidenceFile {
  id: string;
  path: string;
  filename: string;
  containerType: string;
  totalSize: number;
  segmentCount: number;
  discoveredAt: string;
  created?: string | null;
  modified?: string | null;
}

/** Project DB hash record */
export interface ProjectDbHashRecord {
  id: string;
  fileId: string;
  sourceId?: string | null;
  sourceRefJson?: string | null;
  algorithm: string;
  hashValue: string;
  computedAt: string;
  segmentIndex?: number | null;
  segmentName?: string | null;
  source: string;
}

/** Aggregated project DB hash facts by algorithm */
export interface DbHashAlgorithmSummary {
  algorithm: string;
  count: number;
  evidenceFileCount: number;
  sourceCount: number;
  latestComputedAt?: string | null;
}

/** Project DB hash verification record */
export interface ProjectDbVerificationRecord {
  id: string;
  hashId: string;
  verifiedAt: string;
  result: string;
  expectedHash: string;
  actualHash: string;
}

/** Project DB annotation record for offset- or line-based review findings */
export interface ProjectDbAnnotationRecord {
  id: string;
  filePath: string;
  containerPath?: string | null;
  annotationType: string;
  offsetStart?: number | null;
  offsetEnd?: number | null;
  lineStart?: number | null;
  lineEnd?: number | null;
  label: string;
  content?: string | null;
  color?: string | null;
  createdBy: string;
  createdAt: string;
  modifiedAt: string;
}

/** Project DB full-text search result across notes, bookmarks, annotations, artifacts, and analysis facts */
export interface ProjectDbFtsSearchResult {
  source: string;
  id: string;
  snippet: string;
  rank: number;
}

/** Aggregated hash verification facts by result status */
export interface DbVerificationResultSummary {
  result: string;
  count: number;
  hashCount: number;
  latestVerifiedAt?: string | null;
}

/** Request to compute a hash source and persist it to .ffxdb */
export interface ProjectDbHashSourceRequest {
  source: HashSourceInput;
  algorithm: HashAlgorithm | string;
  fileId?: string;
  evidenceFile?: ProjectDbEvidenceFile;
  hashRecordSource?: string;
}

/** Result from DB-aware source hashing */
export interface ProjectDbHashSourceResult {
  hashResult: HashSourceResult;
  hashRecord: ProjectDbHashRecord;
}

/** Bounded artifact extraction options */
export interface ArtifactExtractionOptions {
  headerBytes: number;
  previewBytes: number;
}

/** Stable evidence source reference emitted by backend artifact extraction */
export type EvidenceSourceRef =
  | { kind: "localFile"; path: string }
  | {
      kind: "containerEntry";
      containerPath: string;
      entryPath: string;
      containerType: string;
    }
  | {
      kind: "nestedContainerEntry";
      containerPath: string;
      nestedContainerPath: string;
      entryPath: string;
      containerType?: string;
    }
  | {
      kind: "vfsEntry";
      containerPath: string;
      entryPath: string;
      containerType?: string;
    };

/** Normalized artifact record shared by viewer/search/report engines */
export interface NormalizedArtifact {
  id: string;
  sourceRef: EvidenceSourceRef;
  sourceId: string;
  name: string;
  extension?: string;
  size: number;
  mimeType?: string;
  typeDescription: string;
  category: string;
  confidence: string;
  isText: boolean;
  contentPreview?: string;
  metadata: Record<string, string>;
}

/** Persisted normalized artifact record in .ffxdb */
export interface DbNormalizedArtifact {
  id: string;
  evidenceFileId?: string | null;
  sourceId: string;
  sourceRefJson: string;
  name: string;
  extension?: string | null;
  size: number;
  mimeType?: string | null;
  typeDescription: string;
  category: string;
  confidence: string;
  isText: boolean;
  contentPreview?: string | null;
  metadataJson?: string | null;
  extractedAt: string;
  extractor: string;
}

/** Aggregated normalized artifact facts by category */
export interface DbArtifactCategorySummary {
  category: string;
  count: number;
  totalSize: number;
  textCount: number;
  latestExtractedAt?: string | null;
}

/** Aggregated normalized artifact facts by evidence file */
export interface DbArtifactEvidenceSummary {
  evidenceFileId?: string | null;
  count: number;
  totalSize: number;
  textCount: number;
  categoryCount: number;
  latestExtractedAt?: string | null;
}

/** Aggregated normalized artifact facts by extractor engine */
export interface DbArtifactExtractorSummary {
  extractor: string;
  count: number;
  totalSize: number;
  textCount: number;
  categoryCount: number;
  evidenceFileCount: number;
  latestExtractedAt?: string | null;
}

/** Request to extract and persist a normalized artifact */
export interface ProjectDbExtractArtifactRequest {
  source: HashSourceInput;
  options?: ArtifactExtractionOptions | null;
  evidenceFileId?: string;
  evidenceFile?: ProjectDbEvidenceFile;
  extractor?: string;
}

/** Result from DB-aware artifact extraction */
export interface ProjectDbExtractArtifactResult {
  artifact: NormalizedArtifact;
  record: DbNormalizedArtifact;
}

/** Request to collect known OS identity artifacts from mixed evidence sources */
export interface ProjectDbCollectSystemIdentityRequest {
  sources: HashSourceInput[];
  options?: ArtifactExtractionOptions | null;
  evidenceFileId?: string;
  evidenceFile?: ProjectDbEvidenceFile;
  extractor?: string;
}

/** Per-source extraction failure from system identity collection */
export interface ProjectDbSystemIdentityCollectionError {
  sourceId: string;
  error: string;
}

/** Result from collecting known OS identity artifacts */
export interface ProjectDbCollectSystemIdentityResult {
  scanned: number;
  matched: number;
  inserted: number;
  skipped: number;
  records: DbNormalizedArtifact[];
  errors: ProjectDbSystemIdentityCollectionError[];
}

/** Request to collect known driver/module binary artifacts from mixed evidence sources */
export interface ProjectDbCollectBinaryArtifactsRequest {
  sources: HashSourceInput[];
  options?: ArtifactExtractionOptions | null;
  evidenceFileId?: string;
  evidenceFile?: ProjectDbEvidenceFile;
  extractor?: string;
}

/** Per-source extraction failure from driver/module binary collection */
export interface ProjectDbBinaryArtifactCollectionError {
  sourceId: string;
  error: string;
}

/** Result from collecting known driver/module binary artifacts */
export interface ProjectDbCollectBinaryArtifactsResult {
  scanned: number;
  matched: number;
  inserted: number;
  skipped: number;
  records: DbNormalizedArtifact[];
  errors: ProjectDbBinaryArtifactCollectionError[];
}

/** Bounded byte/source analysis options for hex and data review */
export interface SourceAnalysisOptions {
  offset?: number;
  length?: number;
  entropyWindowBytes?: number;
}

/** Binary source metadata used before choosing inline or ranged reads */
export interface ViewerBinaryInfo {
  path: string;
  size: number;
  maxInlineBytes: number;
  supportsRangeReads: boolean;
}

/** Ranged binary chunk returned as base64 for source-backed viewers */
export interface ViewerBinaryBase64Chunk {
  path: string;
  offset: number;
  bytesRead: number;
  totalSize: number;
  eof: boolean;
  data: string;
}

/** Ranged text chunk returned for source-backed viewers */
export interface ViewerTextChunk {
  path: string;
  offset: number;
  bytesRead: number;
  totalSize: number;
  eof: boolean;
  text: string;
}

/** Detected source signature */
export interface SourceSignature {
  offset: number;
  description: string;
  mimeType: string;
  extensions: string[];
  category: string;
  confidence: string;
  magicHex: string;
}

/** Entropy for one contiguous byte window */
export interface EntropyWindow {
  offset: number;
  length: number;
  entropy: number;
}

/** Text-like indicator extracted from analyzed source bytes */
export interface SourceIndicator {
  indicatorType: string;
  value: string;
  offset: number;
  length: number;
  confidence: string;
}

/** Source-aware byte analysis result for hex/data review engines */
export interface SourceAnalysis {
  sourceRef: EvidenceSourceRef;
  sourceId: string;
  totalSize: number;
  offset: number;
  bytesAnalyzed: number;
  magicHex: string;
  signatures: SourceSignature[];
  entropy: number;
  entropyWindows: EntropyWindow[];
  histogram: number[];
  printableBytes: number;
  nulBytes: number;
  highBitBytes: number;
  printableRatio: number;
  isLikelyText: boolean;
  indicators: SourceIndicator[];
  asciiPreview: string;
}

/** Persisted source-analysis record in .ffxdb */
export interface DbSourceAnalysisRecord {
  id: string;
  evidenceFileId?: string | null;
  sourceId: string;
  sourceRefJson: string;
  totalSize: number;
  offset: number;
  bytesAnalyzed: number;
  magicHex: string;
  signatureCount: number;
  primarySignature?: string | null;
  primaryMimeType?: string | null;
  primaryCategory?: string | null;
  entropy: number;
  printableRatio: number;
  isLikelyText: boolean;
  asciiPreview?: string | null;
  signaturesJson?: string | null;
  entropyWindowsJson?: string | null;
  histogramJson?: string | null;
  indicatorsJson?: string | null;
  analyzedAt: string;
  analyzer: string;
}

/** Aggregated persisted source-analysis facts by primary category */
export interface DbSourceAnalysisCategorySummary {
  category: string;
  count: number;
  evidenceFileCount: number;
  avgEntropy: number;
  textLikeCount: number;
  latestAnalyzedAt?: string | null;
}

/** Request to analyze and persist source byte facts */
export interface ProjectDbAnalyzeSourceRequest {
  source: HashSourceInput;
  options?: SourceAnalysisOptions | null;
  evidenceFileId?: string;
  evidenceFile?: ProjectDbEvidenceFile;
  analyzer?: string;
}

/** Result from DB-aware source analysis */
export interface ProjectDbAnalyzeSourceResult {
  analysis: SourceAnalysis;
  record: DbSourceAnalysisRecord;
}

/** Recovery operation interface */
export interface RecoveryOperation {
  id: string;
  operation_type: OperationType;
  state: RecoveryState;
  progress: number;
  created_at: string;
  updated_at: string;
  error_message?: string;
  metadata?: Record<string, unknown>;
}

/** Recovery stats interface */
export interface RecoveryStats {
  total: number;
  pending: number;
  in_progress: number;
  completed: number;
  failed: number;
}

/** File record for database */
export interface FileRecord {
  path: string;
  name: string;
  size: number;
  modified?: string;
  created?: string;
  file_type?: string;
}

/** Hash record for database */
export interface HashRecord {
  file_path: string;
  algorithm: HashAlgorithm;
  hash_value: string;
  computed_at: string;
}

/** Verification record for database */
export interface VerificationRecord {
  file_path: string;
  algorithm: HashAlgorithm;
  expected_hash: string;
  actual_hash: string;
  verified_at: string;
  is_match: boolean;
}

/** Extraction item */
export interface ExtractionItem {
  source_path: string;
  entry_path: string;
  destination_path: string;
}

/** Health check result */
export interface HealthStatus {
  status: "healthy" | "degraded" | "unhealthy";
  uptime_seconds: number;
  version: string;
  platform: string;
  memory_used_mb: number;
}

// =============================================================================
// Container Commands
// =============================================================================

export const containerCommands = {
  /**
   * Get container summary (entry count, type, lazy loading recommendation)
   */
  getSummary: (path: string): Promise<ContainerSummary> =>
    invoke<ContainerSummary>("container_get_summary", { path }),

  /**
   * Get root-level children of a container (V2 API - fast, cached)
   */
  getRootChildren: (path: string): Promise<LazyLoadResult> =>
    invoke<LazyLoadResult>("container_get_root_children_v2", {
      containerPath: path,
    }),

  /**
   * Get children at a specific address (V2 API - fast, cached)
   * Use entry.first_child_addr to get children of a directory
   */
  getChildrenAtAddr: (
    path: string,
    addr: number,
    parentPath: string,
  ): Promise<LazyLoadResult> =>
    invoke<LazyLoadResult>("container_get_children_at_addr_v2", {
      containerPath: path,
      addr,
      parentPath,
    }),

  /**
   * Get lazy load settings
   */
  getSettings: (): Promise<LazyLoadConfig> =>
    invoke<LazyLoadConfig>("get_lazy_load_settings"),

  /**
   * Update lazy load settings
   */
  updateSettings: (config: Partial<LazyLoadConfig>): Promise<void> =>
    invoke("update_lazy_load_settings", { config }),
};

// =============================================================================
// Hash Commands
// =============================================================================

export const hashCommands = {
  /**
   * Hash a local file or container entry through the shared byte-source engine.
   */
  computeSource: (
    source: HashSourceInput,
    algorithm: HashAlgorithm | string,
  ): Promise<HashSourceResult> =>
    invoke<HashSourceResult>("hash_source", { source, algorithm }),

  /**
   * Hash a local file or container entry and persist the immutable hash record
   * to the active project database.
   */
  computeSourceAndInsert: (
    request: ProjectDbHashSourceRequest,
  ): Promise<ProjectDbHashSourceResult> =>
    invoke<ProjectDbHashSourceResult>("project_db_hash_source_and_insert", {
      request,
    }),

  /**
   * Summarize stored project DB hashes by algorithm.
   */
  summarizeByAlgorithm: (): Promise<DbHashAlgorithmSummary[]> =>
    invoke<DbHashAlgorithmSummary[]>(
      "project_db_summarize_hashes_by_algorithm",
    ),

  /**
   * Read all stored hashes for an evidence file.
   */
  getForFile: (fileId: string): Promise<ProjectDbHashRecord[]> =>
    invoke<ProjectDbHashRecord[]>("project_db_get_hashes_for_file", { fileId }),

  /**
   * Read all stored hashes for a source-aware byte target.
   */
  getForSource: (sourceId: string): Promise<ProjectDbHashRecord[]> =>
    invoke<ProjectDbHashRecord[]>("project_db_get_hashes_for_source", {
      sourceId,
    }),

  /**
   * Read the latest stored hash for a source-aware byte target and algorithm.
   */
  getLatestForSource: (
    sourceId: string,
    algorithm: HashAlgorithm | string,
  ): Promise<ProjectDbHashRecord | null> =>
    invoke<ProjectDbHashRecord | null>(
      "project_db_get_latest_hash_for_source",
      {
        sourceId,
        algorithm,
      },
    ),

  /**
   * Insert a hash verification record in the active project DB.
   */
  insertVerification: (
    verification: ProjectDbVerificationRecord,
  ): Promise<void> =>
    invoke("project_db_insert_verification", { v: verification }),

  /**
   * Read verification records for a stored hash.
   */
  getVerificationsForHash: (
    hashId: string,
  ): Promise<ProjectDbVerificationRecord[]> =>
    invoke<ProjectDbVerificationRecord[]>(
      "project_db_get_verifications_for_hash",
      {
        hashId,
      },
    ),

  /**
   * Summarize stored hash verifications by result status.
   */
  summarizeVerificationsByResult: (): Promise<DbVerificationResultSummary[]> =>
    invoke<DbVerificationResultSummary[]>(
      "project_db_summarize_verifications_by_result",
    ),

  /**
   * Queue operations
   */
  queue: {
    resume: (): Promise<void> => invoke("hash_queue_resume"),
    pause: (): Promise<void> => invoke("hash_queue_pause"),
    clearCompleted: (): Promise<void> => invoke("hash_queue_clear_completed"),
  },
};

// =============================================================================
// Artifact Commands
// =============================================================================

export const artifactCommands = {
  /**
   * Extract a normalized artifact record from a local file or container entry.
   */
  extractSource: (
    source: HashSourceInput,
    options?: ArtifactExtractionOptions,
  ): Promise<NormalizedArtifact> =>
    invoke<NormalizedArtifact>("artifact_extract_source", { source, options }),

  /**
   * Insert or replace a normalized artifact record in the active project DB.
   */
  upsert: (artifact: DbNormalizedArtifact): Promise<void> =>
    invoke("project_db_upsert_artifact", { artifact }),

  /**
   * Read a normalized artifact record by ID.
   */
  get: (id: string): Promise<DbNormalizedArtifact | null> =>
    invoke<DbNormalizedArtifact | null>("project_db_get_artifact", { id }),

  /**
   * List normalized artifacts across the active project.
   */
  list: (limit?: number): Promise<DbNormalizedArtifact[]> =>
    invoke<DbNormalizedArtifact[]>("project_db_list_artifacts", { limit }),

  /**
   * List normalized artifacts for an evidence file.
   */
  listForEvidence: (evidenceFileId: string): Promise<DbNormalizedArtifact[]> =>
    invoke<DbNormalizedArtifact[]>("project_db_list_artifacts_for_evidence", {
      evidenceFileId,
    }),

  /**
   * List normalized artifacts by category.
   */
  listByCategory: (
    category: string,
    limit?: number,
  ): Promise<DbNormalizedArtifact[]> =>
    invoke<DbNormalizedArtifact[]>("project_db_list_artifacts_by_category", {
      category,
      limit,
    }),

  /**
   * Summarize normalized artifacts by category.
   */
  summarizeByCategory: (): Promise<DbArtifactCategorySummary[]> =>
    invoke<DbArtifactCategorySummary[]>(
      "project_db_summarize_artifacts_by_category",
    ),

  /**
   * Summarize normalized artifacts by evidence file.
   */
  summarizeByEvidence: (): Promise<DbArtifactEvidenceSummary[]> =>
    invoke<DbArtifactEvidenceSummary[]>(
      "project_db_summarize_artifacts_by_evidence",
    ),

  /**
   * Summarize normalized artifacts by extractor engine.
   */
  summarizeByExtractor: (): Promise<DbArtifactExtractorSummary[]> =>
    invoke<DbArtifactExtractorSummary[]>(
      "project_db_summarize_artifacts_by_extractor",
    ),

  /**
   * Extract a normalized artifact and persist it to the active project DB.
   */
  extractSourceAndInsert: (
    request: ProjectDbExtractArtifactRequest,
  ): Promise<ProjectDbExtractArtifactResult> =>
    invoke<ProjectDbExtractArtifactResult>(
      "project_db_extract_artifact_source",
      {
        request,
      },
    ),

  /**
   * Extract and persist known Windows/Linux/macOS system identity artifacts
   * from a batch of candidate evidence sources.
   */
  collectSystemIdentitySources: (
    request: ProjectDbCollectSystemIdentityRequest,
  ): Promise<ProjectDbCollectSystemIdentityResult> =>
    invoke<ProjectDbCollectSystemIdentityResult>(
      "project_db_collect_system_identity_sources",
      {
        request,
      },
    ),

  /**
   * Extract and persist known driver/module binary artifacts from a batch of
   * candidate evidence sources.
   */
  collectBinaryArtifactSources: (
    request: ProjectDbCollectBinaryArtifactsRequest,
  ): Promise<ProjectDbCollectBinaryArtifactsResult> =>
    invoke<ProjectDbCollectBinaryArtifactsResult>(
      "project_db_collect_binary_artifact_sources",
      {
        request,
      },
    ),
};

// =============================================================================
// Source Analysis Commands
// =============================================================================

export const sourceAnalysisCommands = {
  /**
   * Read a persisted source-analysis record by ID.
   */
  get: (id: string): Promise<DbSourceAnalysisRecord | null> =>
    invoke<DbSourceAnalysisRecord | null>("project_db_get_source_analysis", {
      id,
    }),

  /**
   * List persisted source-analysis records.
   */
  list: (limit?: number): Promise<DbSourceAnalysisRecord[]> =>
    invoke<DbSourceAnalysisRecord[]>("project_db_list_source_analyses", {
      limit,
    }),

  /**
   * Summarize persisted source analyses by primary signature category.
   */
  summarizeByCategory: (): Promise<DbSourceAnalysisCategorySummary[]> =>
    invoke<DbSourceAnalysisCategorySummary[]>(
      "project_db_summarize_source_analyses_by_category",
    ),

  /**
   * Analyze a local file or container entry and persist the source-analysis record.
   */
  analyzeSourceAndInsert: (
    request: ProjectDbAnalyzeSourceRequest,
  ): Promise<ProjectDbAnalyzeSourceResult> =>
    invoke<ProjectDbAnalyzeSourceResult>(
      "project_db_analyze_source_and_insert",
      {
        request,
      },
    ),
};

// =============================================================================
// Project DB Status Commands
// =============================================================================

export const projectDbCommands = {
  /**
   * True when the active window has an open per-project .ffxdb database.
   */
  isOpen: (): Promise<boolean> => invoke<boolean>("project_db_is_open"),

  rebuildFts: (): Promise<void> => invoke("project_db_rebuild_fts"),

  searchFts: (
    query: string,
    limit?: number,
  ): Promise<ProjectDbFtsSearchResult[]> =>
    invoke<ProjectDbFtsSearchResult[]>("project_db_fts_search", {
      query,
      limit,
    }),

  annotations: {
    insert: (annotation: ProjectDbAnnotationRecord): Promise<void> =>
      invoke("project_db_insert_annotation", { ann: annotation }),

    update: (annotation: ProjectDbAnnotationRecord): Promise<void> =>
      invoke("project_db_update_annotation", { ann: annotation }),

    getForPath: (filePath: string): Promise<ProjectDbAnnotationRecord[]> =>
      invoke<ProjectDbAnnotationRecord[]>(
        "project_db_get_annotations_for_path",
        {
          filePath,
        },
      ),

    getAll: (): Promise<ProjectDbAnnotationRecord[]> =>
      invoke<ProjectDbAnnotationRecord[]>("project_db_get_all_annotations"),

    delete: (id: string): Promise<void> =>
      invoke("project_db_delete_annotation", { id }),
  },
};

// =============================================================================
// Viewer / Source Analysis Commands
// =============================================================================

export const viewerCommands = {
  detectType: (path: string): Promise<FileTypeInfo> =>
    invoke<FileTypeInfo>("viewer_detect_type", { path }),

  detectTypeSource: (source: HashSourceInput): Promise<FileTypeInfo> =>
    invoke<FileTypeInfo>("viewer_detect_type_source", { source }),

  parseHeader: (path: string): Promise<ParsedMetadata> =>
    invoke<ParsedMetadata>("viewer_parse_header", { path }),

  parseHeaderSource: (source: HashSourceInput): Promise<ParsedMetadata> =>
    invoke<ParsedMetadata>("viewer_parse_header_source", { source }),

  /**
   * Analyze a local file for hex/data review.
   */
  analyzePath: (
    path: string,
    options?: SourceAnalysisOptions,
  ): Promise<SourceAnalysis> =>
    invoke<SourceAnalysis>("viewer_analyze_path", { path, options }),

  /**
   * Analyze a local file or container entry through the shared byte-source engine.
   */
  analyzeSource: (
    source: HashSourceInput,
    options?: SourceAnalysisOptions,
  ): Promise<SourceAnalysis> =>
    invoke<SourceAnalysis>("viewer_analyze_source", { source, options }),

  getBinaryInfo: (path: string): Promise<ViewerBinaryInfo> =>
    invoke<ViewerBinaryInfo>("viewer_get_binary_info", { path }),

  getBinaryInfoSource: (source: HashSourceInput): Promise<ViewerBinaryInfo> =>
    invoke<ViewerBinaryInfo>("viewer_get_binary_info_source", { source }),

  readBinaryBase64: (path: string): Promise<string> =>
    invoke<string>("viewer_read_binary_base64", { path }),

  readBinarySourceBase64: (source: HashSourceInput): Promise<string> =>
    invoke<string>("viewer_read_binary_source_base64", { source }),

  readTextSource: (
    source: HashSourceInput,
    offset: number,
    maxChars: number,
  ): Promise<ViewerTextChunk> =>
    invoke<ViewerTextChunk>("viewer_read_text_source", {
      source,
      offset,
      maxChars,
    }),

  readBinaryBase64Chunk: (
    path: string,
    offset: number,
    size: number,
  ): Promise<ViewerBinaryBase64Chunk> =>
    invoke<ViewerBinaryBase64Chunk>("viewer_read_binary_base64_chunk", {
      path,
      offset,
      size,
    }),

  readBinarySourceBase64Chunk: (
    source: HashSourceInput,
    offset: number,
    size: number,
  ): Promise<ViewerBinaryBase64Chunk> =>
    invoke<ViewerBinaryBase64Chunk>("viewer_read_binary_source_base64_chunk", {
      source,
      offset,
      size,
    }),
};

// =============================================================================
// Generic Document Commands
// =============================================================================

export const documentCommands = {
  read: <T = unknown>(path: string): Promise<T> =>
    invoke<T>("document_read", { path }),

  readSource: <T = unknown>(source: HashSourceInput): Promise<T> =>
    invoke<T>("document_read_source", { source }),

  getMetadata: <T = unknown>(path: string): Promise<T> =>
    invoke<T>("document_get_metadata", { path }),

  getMetadataSource: <T = unknown>(source: HashSourceInput): Promise<T> =>
    invoke<T>("document_get_metadata_source", { source }),

  detectContentFormat: <T = unknown>(path: string): Promise<T> =>
    invoke<T>("detect_content_format", { path }),

  detectContentFormatSource: <T = unknown>(
    source: HashSourceInput,
  ): Promise<T> => invoke<T>("detect_content_format_source", { source }),
};

// =============================================================================
// Image / Metadata Commands
// =============================================================================

export const imageCommands = {
  /**
   * Extract EXIF metadata from a local image file.
   */
  extractExif: (path: string): Promise<ExifMetadata> =>
    invoke<ExifMetadata>("exif_extract", { path }),

  /**
   * Extract EXIF metadata from a local file or supported container entry.
   */
  extractExifSource: (source: HashSourceInput): Promise<ExifMetadata> =>
    invoke<ExifMetadata>("exif_extract_source", { source }),
};

// =============================================================================
// Binary Analysis Commands
// =============================================================================

export const binaryCommands = {
  /**
   * Analyze a PE/ELF/Mach-O binary from a local file path.
   */
  analyze: <T = unknown>(path: string): Promise<T> =>
    invoke<T>("binary_analyze", { path }),

  /**
   * Analyze a PE/ELF/Mach-O binary from a local file or supported container entry.
   */
  analyzeSource: <T = unknown>(source: HashSourceInput): Promise<T> =>
    invoke<T>("binary_analyze_source", { source }),
};

// =============================================================================
// Registry Viewer Commands
// =============================================================================

export const registryCommands = {
  getInfo: <T = unknown>(path: string): Promise<T> =>
    invoke<T>("registry_get_info", { path }),

  getInfoSource: <T = unknown>(source: HashSourceInput): Promise<T> =>
    invoke<T>("registry_get_info_source", { source }),

  getSubkeys: <T = unknown>(hivePath: string, keyPath: string): Promise<T> =>
    invoke<T>("registry_get_subkeys", { hivePath, keyPath }),

  getSubkeysSource: <T = unknown>(
    source: HashSourceInput,
    keyPath: string,
  ): Promise<T> =>
    invoke<T>("registry_get_subkeys_source", { source, keyPath }),

  getKeyInfo: <T = unknown>(hivePath: string, keyPath: string): Promise<T> =>
    invoke<T>("registry_get_key_info", { hivePath, keyPath }),

  getKeyInfoSource: <T = unknown>(
    source: HashSourceInput,
    keyPath: string,
  ): Promise<T> =>
    invoke<T>("registry_get_key_info_source", { source, keyPath }),
};

// =============================================================================
// PLIST Commands
// =============================================================================

export const plistCommands = {
  /**
   * Read and flatten a plist from a local file path.
   */
  read: <T = unknown>(path: string): Promise<T> =>
    invoke<T>("plist_read", { path }),

  /**
   * Read and flatten a plist from a local file or supported container entry.
   */
  readSource: <T = unknown>(source: HashSourceInput): Promise<T> =>
    invoke<T>("plist_read_source", { source }),
};

// =============================================================================
// Email Commands
// =============================================================================

export const emailCommands = {
  parseEml: <T = unknown>(path: string): Promise<T> =>
    invoke<T>("email_parse_eml", { path }),

  parseEmlSource: <T = unknown>(source: HashSourceInput): Promise<T> =>
    invoke<T>("email_parse_eml_source", { source }),

  parseMbox: <T = unknown>(path: string, maxMessages?: number): Promise<T> =>
    invoke<T>("email_parse_mbox", { path, maxMessages }),

  parseMboxSource: <T = unknown>(
    source: HashSourceInput,
    maxMessages?: number,
  ): Promise<T> =>
    invoke<T>("email_parse_mbox_source", { source, maxMessages }),

  parseMsg: <T = unknown>(path: string): Promise<T> =>
    invoke<T>("email_parse_msg", { path }),

  parseMsgSource: <T = unknown>(source: HashSourceInput): Promise<T> =>
    invoke<T>("email_parse_msg_source", { source }),
};

// =============================================================================
// PST/OST Commands
// =============================================================================

export const pstCommands = {
  getFolders: <T = unknown>(path: string): Promise<T> =>
    invoke<T>("pst_get_folders", { path }),

  getFoldersSource: <T = unknown>(source: HashSourceInput): Promise<T> =>
    invoke<T>("pst_get_folders_source", { source }),

  getMessages: <T = unknown>(
    path: string,
    folderNodeId: number,
    offset?: number,
    limit?: number,
  ): Promise<T> =>
    invoke<T>("pst_get_messages", { path, folderNodeId, offset, limit }),

  getMessagesSource: <T = unknown>(
    source: HashSourceInput,
    folderNodeId: number,
    offset?: number,
    limit?: number,
  ): Promise<T> =>
    invoke<T>("pst_get_messages_source", {
      source,
      folderNodeId,
      offset,
      limit,
    }),

  getMessageDetail: <T = unknown>(
    path: string,
    messageNodeId: number,
  ): Promise<T> => invoke<T>("pst_get_message_detail", { path, messageNodeId }),

  getMessageDetailSource: <T = unknown>(
    source: HashSourceInput,
    messageNodeId: number,
  ): Promise<T> =>
    invoke<T>("pst_get_message_detail_source", { source, messageNodeId }),
};

// =============================================================================
// Spreadsheet Commands
// =============================================================================

export const spreadsheetCommands = {
  info: <T = unknown>(path: string): Promise<T> =>
    invoke<T>("spreadsheet_info", { path }),

  infoSource: <T = unknown>(source: HashSourceInput): Promise<T> =>
    invoke<T>("spreadsheet_info_source", { source }),

  readSheet: <T = unknown>(
    path: string,
    sheetName: string,
    startRow?: number,
    maxRows?: number,
  ): Promise<T> =>
    invoke<T>("spreadsheet_read_sheet", { path, sheetName, startRow, maxRows }),

  readSheetSource: <T = unknown>(
    source: HashSourceInput,
    sheetName: string,
    startRow?: number,
    maxRows?: number,
  ): Promise<T> =>
    invoke<T>("spreadsheet_read_sheet_source", {
      source,
      sheetName,
      startRow,
      maxRows,
    }),
};

// =============================================================================
// Office Commands
// =============================================================================

export const officeCommands = {
  readDocument: <T = unknown>(path: string): Promise<T> =>
    invoke<T>("office_read_document", { path }),

  readDocumentSource: <T = unknown>(source: HashSourceInput): Promise<T> =>
    invoke<T>("office_read_document_source", { source }),
};

// =============================================================================
// SQLite Viewer Commands
// =============================================================================

export const sqliteCommands = {
  getInfo: <T = unknown>(path: string): Promise<T> =>
    invoke<T>("database_get_info", { path }),

  getInfoSource: <T = unknown>(source: HashSourceInput): Promise<T> =>
    invoke<T>("database_get_info_source", { source }),

  getTableSchema: <T = unknown>(
    dbPath: string,
    tableName: string,
  ): Promise<T> =>
    invoke<T>("database_get_table_schema", { dbPath, tableName }),

  getTableSchemaSource: <T = unknown>(
    source: HashSourceInput,
    tableName: string,
  ): Promise<T> =>
    invoke<T>("database_get_table_schema_source", { source, tableName }),

  queryTable: <T = unknown>(
    dbPath: string,
    tableName: string,
    page: number,
    pageSize: number,
  ): Promise<T> =>
    invoke<T>("database_query_table", { dbPath, tableName, page, pageSize }),

  queryTableSource: <T = unknown>(
    source: HashSourceInput,
    tableName: string,
    page: number,
    pageSize: number,
  ): Promise<T> =>
    invoke<T>("database_query_table_source", {
      source,
      tableName,
      page,
      pageSize,
    }),
};

// =============================================================================
// Database Commands
// =============================================================================

export const databaseCommands = {
  /**
   * Insert or update a file record
   */
  upsertFile: (file: FileRecord): Promise<void> =>
    invoke("db_upsert_file", { file }),

  /**
   * Insert a hash record
   */
  insertHash: (hash: HashRecord): Promise<void> =>
    invoke("db_insert_hash", { hash }),

  /**
   * Insert a verification record
   */
  insertVerification: (verification: VerificationRecord): Promise<void> =>
    invoke("db_insert_verification", { verification }),

  /**
   * Save open tabs to session
   */
  saveOpenTabs: (sessionId: string, tabs: string[]): Promise<void> =>
    invoke("db_save_open_tabs", { sessionId, tabs }),

  /**
   * Set a setting value
   */
  setSetting: (key: string, value: string): Promise<void> =>
    invoke("db_set_setting", { key, value }),
};

// =============================================================================
// System Commands
// =============================================================================

export const systemCommands = {
  /**
   * Open path in system file manager
   */
  openPath: (path: string): Promise<void> =>
    invoke("plugin:opener|open_path", { path }),
};

// =============================================================================
// Unified Commands Export
// =============================================================================

/**
 * Unified commands API for all Tauri backend operations.
 * Provides type-safe wrappers with organized namespaces.
 */
export const commands = {
  container: containerCommands,
  hash: hashCommands,
  artifact: artifactCommands,
  sourceAnalysis: sourceAnalysisCommands,
  projectDb: projectDbCommands,
  viewer: viewerCommands,
  document: documentCommands,
  image: imageCommands,
  binary: binaryCommands,
  registry: registryCommands,
  plist: plistCommands,
  email: emailCommands,
  pst: pstCommands,
  spreadsheet: spreadsheetCommands,
  office: officeCommands,
  sqlite: sqliteCommands,
  database: databaseCommands,
  system: systemCommands,
} as const;

export default commands;
