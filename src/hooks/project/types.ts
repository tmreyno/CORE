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
  ProcessedDbIntegrity,
  FilterState,
} from "../../types/project";
import type { DiscoveredFile, HashHistoryEntry } from "../../types";
import type { ProcessedDatabase } from "../../types/processed";

// Re-export types for consumers
export type {
  FFXProject,
  ProjectSaveResult,
  ProjectLoadResult,
  ActivityLogEntry,
  ActivityCategory,
};

/** Options for building project state */
export interface BuildProjectOptions {
  rootPath: string;
  openTabs: Array<{ file: DiscoveredFile; id: string }>;
  activeTabPath: string | null;
  hashHistory: Map<string, HashHistoryEntry[]>;
  processedDatabases?: ProcessedDatabase[];
  selectedProcessedDb?: ProcessedDatabase | null;
  uiState?: Partial<ProjectUIState>;
  filterState?: FilterState;
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
  removeBookmark: (bookmarkId: string) => void;
}

/** Note management interface */
export interface NoteManager {
  addNote: (note: Omit<ProjectNote, 'id' | 'created_by' | 'created_at' | 'modified_at'>) => void;
  updateNote: (noteId: string, updates: Partial<Pick<ProjectNote, 'title' | 'content' | 'tags' | 'priority'>>) => void;
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
  createProject: (rootPath: string) => Promise<FFXProject>;
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
