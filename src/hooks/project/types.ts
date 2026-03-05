// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Types specific to the useProject hook
 */

import type { Accessor } from "solid-js";
import type {
  FFXProject,
  ProjectSaveResult,
  ProjectLoadResult,
  ActivityLogEntry,
  ActivityCategory,
  ProjectUIState,
  ProjectBookmark,
  ProjectNote,
  ProjectTag,
  ProcessedDbIntegrity,
  FilterState,
  ProjectTabType,
} from "../../types/project";
import type { DiscoveredFile, HashHistoryEntry, ContainerInfo, CaseDocument } from "../../types";
import type { ProcessedDatabase, AxiomCaseInfo, ArtifactCategorySummary } from "../../types/processed";
import type { SelectedEntry } from "../../components/EvidenceTree";

// Re-export types for consumers
export type {
  FFXProject,
  ProjectSaveResult,
  ProjectLoadResult,
  ActivityLogEntry,
  ActivityCategory,
};

/** Serializable center pane tab for project save */
export interface CenterTabForSave {
  id: string;
  type: ProjectTabType;
  title: string;
  subtitle?: string;
  /** For evidence tabs */
  file?: DiscoveredFile;
  /** For case document tabs - the path */
  documentPath?: string;
  /** For entry tabs (files inside containers) */
  entry?: SelectedEntry;
  /** For processed database tabs */
  processedDb?: ProcessedDatabase;
  /** For evidence collection tabs */
  collectionId?: string;
  /** Whether the collection is read-only */
  collectionReadOnly?: boolean;
  /** Whether this is the collection list view */
  collectionListView?: boolean;
}

/** Options for building project state */
export interface BuildProjectOptions {
  rootPath: string;
  /** Project name (defaults to folder name if not provided) */
  projectName?: string;
  /** New center pane tabs (unified tab system) */
  centerTabs?: CenterTabForSave[];
  /** Active tab ID in center pane */
  activeTabId?: string | null;
  /** View mode for center pane */
  viewMode?: string;
  /** @deprecated Use centerTabs instead - legacy open tabs */
  openTabs?: Array<{ file: DiscoveredFile; id: string }>;
  /** @deprecated Use activeTabId instead */
  activeTabPath?: string | null;
  hashHistory: Map<string, HashHistoryEntry[]>;
  processedDatabases?: ProcessedDatabase[];
  selectedProcessedDb?: ProcessedDatabase | null;
  uiState?: Partial<ProjectUIState>;
  filterState?: FilterState;
  /** Evidence cache to avoid re-scanning/re-loading on project open */
  evidenceCache?: {
    discoveredFiles: DiscoveredFile[];
    fileInfoMap: Map<string, ContainerInfo>;
    fileHashMap: Map<string, { algorithm: string; hash: string; verified?: boolean | null }>;
  };
  /** Processed databases cache with AXIOM data */
  processedDbCache?: {
    databases: ProcessedDatabase[];
    axiomCaseInfo: Record<string, AxiomCaseInfo>;
    artifactCategories: Record<string, ArtifactCategorySummary[]>;
    detailViewType?: string | null;
  };
  /** Case documents cache */
  caseDocumentsCache?: {
    documents: CaseDocument[];
    searchPath: string;
  };
}

/** Project state signals (read-only) */
export interface ProjectStateSignals {
  project: Accessor<FFXProject | null>;
  projectPath: Accessor<string | null>;
  modified: Accessor<boolean>;
  error: Accessor<string | null>;
  loading: Accessor<boolean>;
  currentUser: Accessor<string>;
  currentSessionId: Accessor<string | null>;
  autoSaveEnabled: Accessor<boolean>;
  lastAutoSave: Accessor<Date | null>;
}

/** Activity logging interface */
export interface ActivityLogger {
  logActivity: (
    category: ActivityCategory,
    action: string,
    description: string,
    filePath?: string,
    details?: Record<string, unknown>
  ) => void;
}

/** Bookmark management interface */
export interface BookmarkManager {
  addBookmark: (bookmark: Omit<ProjectBookmark, 'id' | 'created_by' | 'created_at'>) => void;
  updateBookmark: (bookmarkId: string, updates: Partial<Pick<ProjectBookmark, 'name' | 'color' | 'tags' | 'notes'>>) => void;
  removeBookmark: (bookmarkId: string) => void;
}

/** Note management interface */
export interface NoteManager {
  addNote: (note: Omit<ProjectNote, 'id' | 'created_by' | 'created_at' | 'modified_at'>) => void;
  updateNote: (noteId: string, updates: Partial<Pick<ProjectNote, 'title' | 'content' | 'tags' | 'priority'>>) => void;
  removeNote: (noteId: string) => void;
}

/** Tag management interface */
export interface TagManager {
  addTag: (tag: Omit<ProjectTag, 'id' | 'created_at'>) => ProjectTag | undefined;
  updateTag: (tagId: string, updates: Partial<Pick<ProjectTag, 'name' | 'color' | 'description'>>) => void;
  removeTag: (tagId: string) => void;
}

/** Auto-save management interface */
export interface AutoSaveManager {
  setAutoSaveEnabled: (enabled: boolean) => void;
  setAutoSaveCallback: (callback: () => Promise<void>) => void;
  startAutoSave: () => void;
  stopAutoSave: () => void;
}

/** Project I/O interface */
export interface ProjectIO {
  checkProjectExists: (rootPath: string) => Promise<string | null>;
  getDefaultProjectPath: (rootPath: string) => Promise<string>;
  createProject: (rootPath: string, projectName?: string, ownerName?: string) => Promise<FFXProject>;
  saveProject: (options: BuildProjectOptions, customPath?: string) => Promise<ProjectSaveResult>;
  saveProjectAs: (options: BuildProjectOptions) => Promise<ProjectSaveResult>;
  loadProject: (customPath?: string) => Promise<{ project: FFXProject | null; error?: string; warnings?: string[] }>;
  clearProject: () => void;
}

/** Processed database interface */
export interface ProcessedDbManager {
  updateProcessedDbIntegrity: (path: string, integrity: ProcessedDbIntegrity) => void;
}

/** Search history interface */
export interface SearchHistoryManager {
  addRecentSearch: (query: string, resultCount: number) => void;
}

/** UI state interface */
export interface UIStateManager {
  updateUIState: (updates: Partial<ProjectUIState>) => void;
}

/** Internal state setters (for composition) */
export interface ProjectStateSetters {
  setProject: (project: FFXProject | null | ((prev: FFXProject | null) => FFXProject | null)) => void;
  setProjectPath: (path: string | null) => void;
  setModified: (modified: boolean) => void;
  setError: (error: string | null) => void;
  setLoading: (loading: boolean) => void;
  setCurrentUser: (user: string) => void;
  setCurrentSessionId: (sessionId: string | null) => void;
  setAutoSaveEnabled: (enabled: boolean) => void;
  setLastAutoSave: (date: Date | null) => void;
}
