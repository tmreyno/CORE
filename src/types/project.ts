// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// =============================================================================
// FFX PROJECT FILE TYPES (.cffx)
// =============================================================================
// Local compatibility wrapper so CORE-FFX keeps stable imports while the
// canonical project manifest and persistence contracts live in @core-suite/types.

export type {
  ActivityCategory,
  ActivityLogEntry,
  CachedCaseDocument,
  CachedContainerInfo,
  CachedDiscoveredFile,
  CachedFileHash,
  CaseDocumentsCache,
  EvidenceCache,
  FileSelectionState,
  FilterState,
  MergeSourceRecord,
  OpenDirectory,
  PreviewCache,
  PreviewCacheEntry,
  ProcessedDbIntegrity,
  ProcessedDbWorkMetrics,
  ProjectBookmark,
  ProjectFileHash,
  ProjectHashHistory,
  ProjectLocations,
  ProjectNote,
  ProjectReportRecord,
  ProjectSaveResult,
  ProjectSession,
  ProjectTab,
  ProjectTabType,
  ProjectTag,
  ProjectUIState,
  ProjectUser,
  RecentDirectory,
  RecentSearch,
  SavedSearch,
  TreeNodeState,
} from "@core-suite/types/project-file";

export type {
  FFXCenterPaneState,
  FFXProcessedDatabaseState as ProcessedDatabaseState,
  FFXProject,
  FFXProjectLoadResult as ProjectLoadResult,
  FFXProjectSettings,
} from "@core-suite/types/project";

export {
  AUTO_SAVE_INTERVAL_MS,
  PROJECT_FILE_EXTENSION,
  PROJECT_FILE_VERSION,
  createActivityEntry,
  createDefaultFilterState,
  createDefaultSettings,
  createDefaultUIState,
  createEmptyProject,
  generateId,
  nowISO,
} from "@core-suite/types/project";
