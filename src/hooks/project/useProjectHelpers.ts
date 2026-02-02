// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Search, UI state, and processed database management for project
 */

import { debounce } from "@solid-primitives/scheduled";
import type { FFXProject, ProjectUIState, RecentSearch, ProcessedDbIntegrity } from "../../types/project";
import { nowISO } from "../../types/project";
import type {
  ProjectStateSignals,
  ProjectStateSetters,
  ActivityLogger,
  SearchHistoryManager,
  UIStateManager,
  ProcessedDbManager,
} from "./types";

/**
 * Create search history management
 */
export function createSearchHistoryManager(
  signals: ProjectStateSignals,
  setters: ProjectStateSetters,
  markModified: () => void,
  logger: ActivityLogger
): SearchHistoryManager {
  /**
   * Add a recent search
   */
  const addRecentSearch = (query: string, resultCount: number) => {
    console.log(`[DEBUG] SearchHistory: addRecentSearch called, query="${query}", resultCount=${resultCount}`);
    const proj = signals.project();
    if (!proj) {
      console.log("[DEBUG] SearchHistory: No project, skipping");
      return;
    }

    const entry: RecentSearch = {
      query,
      timestamp: nowISO(),
      result_count: resultCount,
    };

    const maxRecent = proj.settings?.max_recent_items || 50;
    let searches = [entry, ...proj.recent_searches.filter(s => s.query !== query)];
    if (searches.length > maxRecent) {
      searches = searches.slice(0, maxRecent);
    }

    setters.setProject({
      ...proj,
      recent_searches: searches,
    } as FFXProject);

    logger.logActivity('search', 'perform', `Searched: "${query}" (${resultCount} results)`);
    markModified();
  };

  return { addRecentSearch };
}

/**
 * Create UI state management with debounced updates
 * UI state updates (panel resizes, etc.) can be frequent - debounce to reduce state churn
 */
export function createUIStateManager(
  signals: ProjectStateSignals,
  setters: ProjectStateSetters,
  markModified: () => void
): UIStateManager {
  // Collect pending UI state updates
  let pendingUpdates: Partial<ProjectUIState> = {};
  
  // Flush debounced updates to project state
  const flushUpdates = debounce(() => {
    const proj = signals.project();
    console.log("[DEBUG] UIState: flushUpdates called, pendingUpdates=", Object.keys(pendingUpdates), "hasProject=", !!proj);
    if (!proj || Object.keys(pendingUpdates).length === 0) return;

    console.log("[DEBUG] UIState: Flushing UI state updates, calling markModified");
    setters.setProject({
      ...proj,
      ui_state: {
        ...proj.ui_state,
        ...pendingUpdates,
      },
    } as FFXProject);
    markModified();
    pendingUpdates = {};
  }, 300); // 300ms debounce for UI state updates

  /**
   * Update UI state in project (debounced)
   */
  const updateUIState = (updates: Partial<ProjectUIState>) => {
    const proj = signals.project();
    if (!proj) return;

    // Merge with pending updates
    pendingUpdates = { ...pendingUpdates, ...updates };
    flushUpdates();
  };

  return { updateUIState };
}

/**
 * Create processed database integrity management
 */
export function createProcessedDbManager(
  signals: ProjectStateSignals,
  setters: ProjectStateSetters,
  markModified: () => void,
  logger: ActivityLogger
): ProcessedDbManager {
  /**
   * Update processed database integrity
   */
  const updateProcessedDbIntegrity = (path: string, integrity: ProcessedDbIntegrity) => {
    const proj = signals.project();
    if (!proj) return;

    setters.setProject({
      ...proj,
      processed_databases: {
        ...proj.processed_databases,
        integrity: {
          ...proj.processed_databases.integrity,
          [path]: integrity,
        },
      },
    } as FFXProject);

    logger.logActivity('database', 'verify', `Updated integrity for: ${path}`, path, {
      status: integrity.status,
      changes: integrity.changes,
    });
    markModified();
  };

  return { updateProcessedDbIntegrity };
}

/**
 * Project locations manager
 */
export interface ProjectLocationsManager {
  updateLocations: (locations: import("../../types/project").ProjectLocations) => void;
}

export function createProjectLocationsManager(
  signals: ProjectStateSignals,
  setters: ProjectStateSetters,
  markModified: () => void,
  logger: ActivityLogger
): ProjectLocationsManager {
  /**
   * Update project locations (evidence path, processed db path, case docs path)
   */
  const updateLocations = (locations: import("../../types/project").ProjectLocations) => {
    console.log(`[DEBUG] ProjectLocations: Updating locations`, locations);
    const proj = signals.project();
    if (!proj) {
      console.log("[DEBUG] ProjectLocations: No project, skipping");
      return;
    }

    setters.setProject({
      ...proj,
      locations,
    } as FFXProject);

    logger.logActivity('project', 'update', `Updated project locations: evidence=${locations.evidence_path}`, undefined, {
      evidence_path: locations.evidence_path,
      processed_db_path: locations.processed_db_path,
      case_documents_path: locations.case_documents_path,
    });
    markModified();
  };

  return { updateLocations };
}
