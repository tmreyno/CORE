// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// =============================================================================
// FFX PROJECT FILE TYPES (.cffx)
// =============================================================================
// Comprehensive project state for saving/restoring entire application state
// including user activity, open directories, reports, and processed databases.

import type { ProcessedDatabase } from './processed';

// -----------------------------------------------------------------------------
// VERSION & CONSTANTS
// -----------------------------------------------------------------------------

/** Current project file format version */
export const PROJECT_FILE_VERSION = 2;

/** File extension for FFX project files */
export const PROJECT_FILE_EXTENSION = ".cffx";

/** Auto-save interval in milliseconds (5 minutes) */
export const AUTO_SAVE_INTERVAL_MS = 5 * 60 * 1000;

// -----------------------------------------------------------------------------
// USER & SESSION TYPES
// -----------------------------------------------------------------------------

/** User/examiner identity for activity tracking */
export interface ProjectUser {
  /** Username (from OS or configured) */
  username: string;
  /** Display name (optional) */
  display_name?: string;
  /** Machine name */
  hostname?: string;
  /** When this user first opened the project */
  first_access: string;
  /** When this user last accessed the project */
  last_access: string;
}

/** A recorded activity/action in the project */
export interface ActivityLogEntry {
  /** Unique ID for this entry */
  id: string;
  /** ISO timestamp */
  timestamp: string;
  /** User who performed the action */
  user: string;
  /** Category of action */
  category: ActivityCategory;
  /** Action type */
  action: string;
  /** Human-readable description */
  description: string;
  /** Related file path (if applicable) */
  file_path?: string;
  /** Additional details */
  details?: Record<string, unknown>;
}

/** Categories of activity for filtering/display */
export type ActivityCategory =
  | 'file'        // File operations (open, close, select)
  | 'hash'        // Hash computations and verifications
  | 'search'      // Searches performed
  | 'export'      // Exports/reports generated
  | 'bookmark'    // Bookmarks added/removed
  | 'note'        // Notes created/edited
  | 'tag'         // Tags applied/removed
  | 'database'    // Processed database operations
  | 'project'     // Project save/load
  | 'system';     // System events

/** Session record for tracking work sessions */
export interface ProjectSession {
  /** Unique session ID */
  session_id: string;
  /** User for this session */
  user: string;
  /** Session start time */
  started_at: string;
  /** Session end time (null if current session) */
  ended_at: string | null;
  /** Duration in seconds (computed on end) */
  duration_seconds?: number;
  /** Machine/hostname */
  hostname?: string;
  /** App version used */
  app_version: string;
  /** Brief summary of work done */
  summary?: string;
}

// -----------------------------------------------------------------------------
// EVIDENCE & FILE STATE
// -----------------------------------------------------------------------------

/** Tab type for center pane tabs */
export type ProjectTabType = "evidence" | "document" | "entry" | "export" | "processed";

/** State for an open center pane tab */
export interface ProjectTab {
  /** Unique tab identifier */
  id: string;
  /** Tab type */
  type: ProjectTabType;
  /** File path (absolute) - for evidence files */
  file_path: string;
  /** Display name */
  name: string;
  /** Subtitle (e.g., container type) */
  subtitle?: string;
  /** Tab order (0-based) */
  order: number;
  /** Container type - for evidence tabs */
  container_type?: string;
  /** Document path - for case document tabs */
  document_path?: string;
  /** Container entry path - for entry tabs (files inside containers) */
  entry_path?: string;
  /** Parent container path - for entry tabs */
  entry_container_path?: string;
  /** Entry name - for entry tabs */
  entry_name?: string;
  /** Processed database path - for processed db tabs */
  processed_db_path?: string;
  /** Processed database type */
  processed_db_type?: string;
  /** Scroll position in file list */
  scroll_position?: number;
  /** Last viewed timestamp */
  last_viewed?: string;
}

/** Expanded/collapsed tree node state */
export interface TreeNodeState {
  /** Node path/identifier */
  path: string;
  /** Whether expanded */
  expanded: boolean;
  /** Child states (recursive) */
  children?: TreeNodeState[];
}

/** Hash record for a file */
export interface ProjectFileHash {
  /** Hash algorithm */
  algorithm: string;
  /** Computed hash value */
  hash_value: string;
  /** When computed */
  computed_at: string;
  /** Verification state */
  verification?: {
    /** Result of verification */
    result: 'match' | 'mismatch' | 'pending';
    /** What it was verified against */
    verified_against: string;
    /** When verified */
    verified_at: string;
  };
}

/** Hash history for all files */
export interface ProjectHashHistory {
  /** Map of file path to hash records */
  files: Record<string, ProjectFileHash[]>;
}

/** Selection state for files */
export interface FileSelectionState {
  /** Selected file paths */
  selected_paths: string[];
  /** Active (focused) file path */
  active_path: string | null;
  /** Last selection timestamp */
  timestamp: string;
}

// -----------------------------------------------------------------------------
// EVIDENCE CACHE TYPES (for avoiding re-scan/re-load on project open)
// -----------------------------------------------------------------------------

/** Cached discovered file - serializable version of DiscoveredFile */
export interface CachedDiscoveredFile {
  /** File path (absolute) */
  path: string;
  /** Filename */
  filename: string;
  /** Container type (ad1, e01, l01, ufed, archive, raw) */
  container_type: string;
  /** File size in bytes */
  size: number;
  /** Number of segments (for multi-part E01/AD1) */
  segment_count?: number;
  /** Created timestamp */
  created?: string;
  /** Modified timestamp */
  modified?: string;
}

/** Cached container info - serializable version of ContainerInfo */
export interface CachedContainerInfo {
  /** Container type indicator */
  container: string;
  /** AD1 specific info (JSON representation) */
  ad1?: unknown;
  /** E01 physical image info */
  e01?: unknown;
  /** L01 logical image info */
  l01?: unknown;
  /** Raw image info */
  raw?: unknown;
  /** Archive info */
  archive?: unknown;
  /** UFED info */
  ufed?: unknown;
  /** Notes */
  note?: string | null;
  /** Companion log info */
  companion_log?: unknown;
}

/** Cached file hash result - serializable version of FileHashInfo */
export interface CachedFileHash {
  /** Hash algorithm used */
  algorithm: string;
  /** Computed hash value */
  hash: string;
  /** Verification result (null = not verified, true = verified match, false = mismatch) */
  verified?: boolean | null;
  /** When the hash was computed */
  computed_at?: string;
}

/** Complete evidence cache state for project persistence */
export interface EvidenceCache {
  /** Discovered evidence files */
  discovered_files: CachedDiscoveredFile[];
  /** Loaded container info for each file (by path) */
  file_info: Record<string, CachedContainerInfo>;
  /** Computed hashes for each file (by path) */
  computed_hashes: Record<string, CachedFileHash>;
  /** When the cache was last updated */
  cached_at: string;
  /** Whether the cache is considered valid */
  valid: boolean;
}

// -----------------------------------------------------------------------------
// CASE DOCUMENTS CACHE
// -----------------------------------------------------------------------------

/** Cached case document - serializable for project persistence */
export interface CachedCaseDocument {
  /** Full path to the document */
  path: string;
  /** Filename */
  filename: string;
  /** Document type (ChainOfCustody, EvidenceIntake, etc.) */
  document_type: string;
  /** File size in bytes */
  size: number;
  /** File format (PDF, DOCX, TXT, etc.) */
  format: string;
  /** Case number extracted from filename */
  case_number?: string | null;
  /** Evidence ID extracted from filename */
  evidence_id?: string | null;
  /** Last modified timestamp */
  modified?: string | null;
}

/** Cache state for case documents */
export interface CaseDocumentsCache {
  /** Discovered case documents */
  documents: CachedCaseDocument[];
  /** Path that was searched */
  search_path: string;
  /** When the cache was last updated */
  cached_at: string;
  /** Whether the cache is valid */
  valid: boolean;
}

// -----------------------------------------------------------------------------
// PREVIEW CACHE (for extracted container files)
// -----------------------------------------------------------------------------

/** Cache entry for an extracted preview file */
export interface PreviewCacheEntry {
  /** Unique key: containerPath::entryPath */
  key: string;
  /** Path to the container file */
  container_path: string;
  /** Path within the container */
  entry_path: string;
  /** Path to the extracted temp file */
  temp_path: string;
  /** Size of the entry */
  entry_size: number;
  /** When extracted */
  extracted_at: string;
  /** Whether the temp file still exists (validated on load) */
  valid?: boolean;
}

/** Preview cache state for project */
export interface PreviewCache {
  /** Map of containerPath::entryPath -> cache entry */
  entries: PreviewCacheEntry[];
  /** When cache was last updated */
  cached_at: string;
  /** Project cache directory */
  cache_dir?: string;
}

// -----------------------------------------------------------------------------
// PROCESSED DATABASE STATE
// -----------------------------------------------------------------------------

/** Integrity record for a processed database */
export interface ProcessedDbIntegrity {
  /** Database file path */
  path: string;
  /** File size in bytes */
  file_size: number;
  /** Baseline hash when first loaded */
  baseline_hash: string;
  /** When baseline was established */
  baseline_timestamp: string;
  /** Current hash (may differ from baseline if work was done) */
  current_hash?: string;
  /** When current hash was computed */
  current_hash_timestamp?: string;
  /** Verification status */
  status: 'unchanged' | 'modified' | 'new_baseline' | 'not_verified';
  /** Work metrics from database */
  metrics?: ProcessedDbWorkMetrics;
  /** Detected changes since baseline */
  changes?: string[];
}

/** Work metrics extracted from processed database */
export interface ProcessedDbWorkMetrics {
  /** Total scans/analyses run */
  total_scans: number;
  /** Last scan date */
  last_scan_date: string | null;
  /** Total jobs/tasks run */
  total_jobs: number;
  /** Last job date */
  last_job_date: string | null;
  /** Total examiner notes */
  total_notes: number;
  /** Total tagged items */
  total_tagged_items: number;
  /** Total users who have worked on this */
  total_users: number;
  /** User names who have worked on this */
  user_names: string[];
}

/** State for processed databases in project */
export interface ProcessedDatabaseState {
  /** List of loaded database paths */
  loaded_paths: string[];
  /** Currently selected database path */
  selected_path: string | null;
  /** Detail view state */
  detail_view_type: string | null;
  /** Integrity records for each database */
  integrity: Record<string, ProcessedDbIntegrity>;
  /** Cached metadata (to avoid re-querying on load) */
  cached_metadata?: Record<string, Partial<ProcessedDatabase>>;
  /** Full database objects cache (complete ProcessedDatabase objects) */
  cached_databases?: ProcessedDatabase[];
  /** Cached AXIOM case info (by path) */
  cached_axiom_case_info?: Record<string, unknown>;
  /** Cached artifact categories (by path) */
  cached_artifact_categories?: Record<string, unknown[]>;
}

// -----------------------------------------------------------------------------
// BOOKMARKS & NOTES
// -----------------------------------------------------------------------------

/** A bookmark in the project */
export interface ProjectBookmark {
  /** Unique bookmark ID */
  id: string;
  /** Type of bookmark target */
  target_type: 'file' | 'artifact' | 'search_result' | 'location';
  /** Path to bookmarked item */
  target_path: string;
  /** Display name */
  name: string;
  /** User who created */
  created_by: string;
  /** When created */
  created_at: string;
  /** Color/category */
  color?: string;
  /** Tags */
  tags?: string[];
  /** Notes */
  notes?: string;
  /** Additional context */
  context?: Record<string, unknown>;
}

/** A note/annotation in the project */
export interface ProjectNote {
  /** Unique note ID */
  id: string;
  /** What the note is attached to */
  target_type: 'file' | 'artifact' | 'database' | 'case' | 'general';
  /** Path to target item */
  target_path?: string;
  /** Note title */
  title: string;
  /** Note content (supports markdown) */
  content: string;
  /** User who created */
  created_by: string;
  /** When created */
  created_at: string;
  /** When last modified */
  modified_at: string;
  /** Tags */
  tags?: string[];
  /** Priority/importance */
  priority?: 'low' | 'normal' | 'high' | 'critical';
}

/** A tag definition */
export interface ProjectTag {
  /** Tag ID */
  id: string;
  /** Tag name */
  name: string;
  /** Color (hex) */
  color: string;
  /** Description */
  description?: string;
  /** When created */
  created_at: string;
}

// -----------------------------------------------------------------------------
// REPORTS & EXPORTS
// -----------------------------------------------------------------------------

/** Record of a generated report */
export interface ProjectReportRecord {
  /** Unique report ID */
  id: string;
  /** Report title */
  title: string;
  /** Report type */
  report_type: 'summary' | 'detailed' | 'hash_verification' | 'timeline' | 'custom';
  /** Output format */
  format: 'json' | 'markdown' | 'html' | 'pdf' | 'csv';
  /** Output file path (if saved) */
  output_path?: string;
  /** When generated */
  generated_at: string;
  /** User who generated */
  generated_by: string;
  /** Files/items included */
  included_items?: string[];
  /** Report configuration used */
  config?: Record<string, unknown>;
  /** Status */
  status: 'pending' | 'completed' | 'failed';
  /** Error message if failed */
  error?: string;
}

// -----------------------------------------------------------------------------
// UI STATE
// -----------------------------------------------------------------------------

/** Complete UI state for restoration */
export interface ProjectUIState {
  /** Left panel width (pixels) */
  left_panel_width: number;
  /** Right panel width (pixels) */
  right_panel_width: number;
  /** Left panel collapsed state */
  left_panel_collapsed: boolean;
  /** Right panel collapsed state */
  right_panel_collapsed: boolean;
  /** Active left panel tab */
  left_panel_tab: 'evidence' | 'processed' | 'casedocs' | 'activity' | 'bookmarks';
  /** Current view mode for detail panel */
  detail_view_mode: string;
  /** Tree node expansion state */
  tree_state: TreeNodeState[];
  /** Scroll positions by panel/view */
  scroll_positions: Record<string, number>;
  /** Window dimensions (for responsive restore) */
  window_dimensions?: {
    width: number;
    height: number;
  };
  /** Custom theme/preferences */
  preferences?: {
    theme?: 'light' | 'dark' | 'auto';
    font_size?: number;
    show_hidden_files?: boolean;
    confirm_on_close?: boolean;
  };
  /** Expanded container paths in the evidence tree */
  expanded_containers?: string[];
  /** Currently selected entry in container viewer */
  selected_entry?: {
    containerPath: string;
    entryPath: string;
    name: string;
  } | null;
  /** Entry content view mode (auto, hex, text, document) */
  entry_content_view_mode?: 'auto' | 'hex' | 'text' | 'document';
  /** Case documents search path */
  case_documents_path?: string;
  /** Comprehensive tree expansion state for all container types */
  tree_expansion_state?: {
    containers: string[];
    vfs: string[];
    archive: string[];
    lazy: string[];
    ad1: string[];
    selectedKey: string | null;
  };
}

// -----------------------------------------------------------------------------
// SEARCH & FILTER STATE
// -----------------------------------------------------------------------------

/** Saved search query */
export interface SavedSearch {
  /** Unique search ID */
  id: string;
  /** Search name */
  name: string;
  /** Search query/pattern */
  query: string;
  /** Search type */
  search_type: 'filename' | 'content' | 'hash' | 'metadata' | 'keyword';
  /** Is regex */
  is_regex: boolean;
  /** Case sensitive */
  case_sensitive: boolean;
  /** Target scope */
  scope: 'all' | 'selected' | 'evidence' | 'processed';
  /** When created */
  created_at: string;
  /** Times used */
  use_count: number;
  /** Last used */
  last_used?: string;
}

/** Recent search record */
export interface RecentSearch {
  /** Search query */
  query: string;
  /** When performed */
  timestamp: string;
  /** Results count */
  result_count: number;
}

/** Filter state */
export interface FilterState {
  /** Active type filter */
  type_filter: string | null;
  /** Active status filter */
  status_filter: string | null;
  /** Active search query */
  search_query: string | null;
  /** Sort field */
  sort_by: string;
  /** Sort direction */
  sort_direction: 'asc' | 'desc';
}

// -----------------------------------------------------------------------------
// DIRECTORIES & PATHS
// -----------------------------------------------------------------------------

/** Project locations configuration (set during project setup wizard) */
export interface ProjectLocations {
  /** Root project directory */
  project_root: string;
  /** Path to evidence files directory */
  evidence_path: string;
  /** Path to processed databases directory */
  processed_db_path: string;
  /** Path to case documents directory (COC, forms, etc.) */
  case_documents_path?: string;
  /** Whether locations were auto-discovered or manually set */
  auto_discovered: boolean;
  /** When locations were configured */
  configured_at: string;
  /** Count of discovered evidence files */
  evidence_file_count?: number;
  /** Count of discovered processed databases */
  processed_db_count?: number;
  /** Whether to load stored hashes on project open */
  load_stored_hashes?: boolean;
}

/** Open directory state */
export interface OpenDirectory {
  /** Directory path */
  path: string;
  /** When opened */
  opened_at: string;
  /** Whether recursive scan was used */
  recursive: boolean;
  /** File count discovered */
  file_count: number;
  /** Total size */
  total_size: number;
  /** Last scan timestamp */
  last_scanned: string;
}

/** Recent directory record */
export interface RecentDirectory {
  /** Directory path */
  path: string;
  /** Times opened */
  open_count: number;
  /** Last opened */
  last_opened: string;
  /** Display name */
  name: string;
}

// -----------------------------------------------------------------------------
// MAIN PROJECT TYPE
// -----------------------------------------------------------------------------

/** Complete FFX Project file structure */
export interface FFXProject {
  // === Metadata ===
  /** Project file format version */
  version: number;
  /** Project unique identifier */
  project_id: string;
  /** Project name */
  name: string;
  /** Project description */
  description?: string;
  /** Root directory path */
  root_path: string;
  /** When project was created */
  created_at: string;
  /** When project was last saved */
  saved_at: string;
  /** App version that created this project */
  created_by_version: string;
  /** App version that last saved this project */
  saved_by_version: string;

  // === Users & Sessions ===
  /** Users who have accessed this project */
  users: ProjectUser[];
  /** Current user */
  current_user?: string;
  /** Session history */
  sessions: ProjectSession[];
  /** Current session ID */
  current_session_id?: string;
  /** Activity log */
  activity_log: ActivityLogEntry[];
  /** Max activity log entries to keep */
  activity_log_limit?: number;

  // === Evidence State ===
  /** Project locations (evidence, processed databases) */
  locations?: ProjectLocations;
  /** Open directories */
  open_directories: OpenDirectory[];
  /** Recent directories */
  recent_directories: RecentDirectory[];
  /** Open tabs */
  tabs: ProjectTab[];
  /** Active tab path (legacy) */
  active_tab_path: string | null;
  /** Center pane state (new unified tab system) */
  center_pane_state?: {
    /** Active tab ID */
    active_tab_id: string | null;
    /** Current view mode */
    view_mode: string;
  };
  /** File selection state */
  file_selection: FileSelectionState;
  /** Hash history for files */
  hash_history: ProjectHashHistory;
  /** Evidence cache (discovered files, loaded info, computed hashes) to avoid re-scan/re-load */
  evidence_cache?: EvidenceCache;
  /** Case documents cache to avoid re-discovery on load */
  case_documents_cache?: CaseDocumentsCache;
  /** Preview cache for extracted container files */
  preview_cache?: PreviewCache;

  // === Processed Databases ===
  /** Processed database state */
  processed_databases: ProcessedDatabaseState;

  // === Bookmarks & Notes ===
  /** Bookmarks */
  bookmarks: ProjectBookmark[];
  /** Notes */
  notes: ProjectNote[];
  /** Tag definitions */
  tags: ProjectTag[];

  // === Reports ===
  /** Generated report history */
  reports: ProjectReportRecord[];

  // === Searches ===
  /** Saved searches */
  saved_searches: SavedSearch[];
  /** Recent searches */
  recent_searches: RecentSearch[];
  /** Current filter state */
  filter_state: FilterState;

  // === UI State ===
  /** UI state for restoration */
  ui_state: ProjectUIState;

  // === Settings ===
  /** Project-specific settings */
  settings?: {
    /** Auto-save enabled */
    auto_save: boolean;
    /** Auto-save interval (ms) */
    auto_save_interval: number;
    /** Default hash algorithm */
    default_hash_algorithm: string;
    /** Verify hashes on load */
    verify_hashes_on_load: boolean;
    /** Track activity */
    track_activity: boolean;
    /** Max recent items to keep */
    max_recent_items: number;
  };

  // === Custom Data ===
  /** Custom key-value data for extensibility */
  custom_data?: Record<string, unknown>;
}

// -----------------------------------------------------------------------------
// RESULT TYPES
// -----------------------------------------------------------------------------

/** Result of loading a project */
export interface ProjectLoadResult {
  success: boolean;
  project?: FFXProject;
  error?: string;
  /** Warnings (e.g., version mismatch, missing files) */
  warnings?: string[];
}

/** Result of saving a project */
export interface ProjectSaveResult {
  success: boolean;
  path?: string;
  error?: string;
  /** Bytes written */
  bytes_written?: number;
}

// -----------------------------------------------------------------------------
// FACTORY FUNCTIONS
// -----------------------------------------------------------------------------

/** Generate a unique ID */
export function generateId(): string {
  return `${Date.now().toString(36)}-${Math.random().toString(36).slice(2, 9)}`;
}

/** Get current ISO timestamp */
export function nowISO(): string {
  return new Date().toISOString();
}

/** Create default UI state */
export function createDefaultUIState(): ProjectUIState {
  return {
    left_panel_width: 320,
    right_panel_width: 280,
    left_panel_collapsed: false,
    right_panel_collapsed: true,
    left_panel_tab: 'evidence',
    detail_view_mode: 'info',
    tree_state: [],
    scroll_positions: {},
    preferences: {
      theme: 'auto',
      confirm_on_close: true,
    },
    expanded_containers: [],
    selected_entry: null,
    entry_content_view_mode: 'auto',
    case_documents_path: undefined,
  };
}

/** Create default filter state */
export function createDefaultFilterState(): FilterState {
  return {
    type_filter: null,
    status_filter: null,
    search_query: null,
    sort_by: 'name',
    sort_direction: 'asc',
  };
}

/** Create default project settings */
export function createDefaultSettings(): NonNullable<FFXProject['settings']> {
  return {
    auto_save: true,
    auto_save_interval: AUTO_SAVE_INTERVAL_MS,
    default_hash_algorithm: 'SHA-256',
    verify_hashes_on_load: false,
    track_activity: true,
    max_recent_items: 50,
  };
}

/** Create a new empty project */
export function createEmptyProject(
  rootPath: string,
  username: string,
  appVersion: string,
  projectName?: string
): FFXProject {
  const now = nowISO();
  const projectId = generateId();
  const sessionId = generateId();
  const name = projectName || rootPath.split('/').pop() || 'Untitled Project';

  return {
    // Metadata
    version: PROJECT_FILE_VERSION,
    project_id: projectId,
    name,
    root_path: rootPath,
    created_at: now,
    saved_at: now,
    created_by_version: appVersion,
    saved_by_version: appVersion,

    // Users & Sessions
    users: [{
      username,
      first_access: now,
      last_access: now,
    }],
    current_user: username,
    sessions: [{
      session_id: sessionId,
      user: username,
      started_at: now,
      ended_at: null,
      app_version: appVersion,
    }],
    current_session_id: sessionId,
    activity_log: [{
      id: generateId(),
      timestamp: now,
      user: username,
      category: 'project',
      action: 'create',
      description: `Project created: ${name}`,
    }],

    // Evidence State
    open_directories: [],
    recent_directories: [],
    tabs: [],
    active_tab_path: null,
    file_selection: {
      selected_paths: [],
      active_path: null,
      timestamp: now,
    },
    hash_history: { files: {} },

    // Processed Databases
    processed_databases: {
      loaded_paths: [],
      selected_path: null,
      detail_view_type: null,
      integrity: {},
    },

    // Bookmarks & Notes
    bookmarks: [],
    notes: [],
    tags: [],

    // Reports
    reports: [],

    // Searches
    saved_searches: [],
    recent_searches: [],
    filter_state: createDefaultFilterState(),

    // UI State
    ui_state: createDefaultUIState(),

    // Settings
    settings: createDefaultSettings(),
  };
}

/** Create an activity log entry */
export function createActivityEntry(
  user: string,
  category: ActivityCategory,
  action: string,
  description: string,
  filePath?: string,
  details?: Record<string, unknown>
): ActivityLogEntry {
  return {
    id: generateId(),
    timestamp: nowISO(),
    user,
    category,
    action,
    description,
    file_path: filePath,
    details,
  };
}
