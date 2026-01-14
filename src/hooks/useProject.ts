// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * @fileoverview Project management hook - re-exports from modular implementation
 * 
 * The implementation has been split into focused modules:
 * - project/types.ts - Type definitions
 * - project/useProjectState.ts - Core signal management
 * - project/useActivityLog.ts - Activity logging
 * - project/useBookmarks.ts - Bookmark management
 * - project/useNotes.ts - Note management
 * - project/useAutoSave.ts - Auto-save timer
 * - project/useProjectIO.ts - Save/load operations
 * - project/useProjectHelpers.ts - Search, UI state, processed DB management
 * - project/index.ts - Main composition
 */

// Re-export everything from the modular implementation
export {
  useProject,
  type FFXProject,
  type ProjectSaveResult,
  type ProjectLoadResult,
  type ActivityLogEntry,
  type ActivityCategory,
  type BuildProjectOptions,
} from "./project";
