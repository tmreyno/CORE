// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Search, UI state, and processed database management for project
 */

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
    const proj = signals.project();
    if (!proj) return;

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
 * Create UI state management
 */
export function createUIStateManager(
  signals: ProjectStateSignals,
  setters: ProjectStateSetters,
  markModified: () => void
): UIStateManager {
  /**
   * Update UI state in project
   */
  const updateUIState = (updates: Partial<ProjectUIState>) => {
    const proj = signals.project();
    if (!proj) return;

    setters.setProject({
      ...proj,
      ui_state: {
        ...proj.ui_state,
        ...updates,
      },
    } as FFXProject);
    markModified();
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
