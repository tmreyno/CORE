// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Activity logging functionality for project
 * Uses batched updates to reduce state churn from rapid logging
 */

import { debounce } from "@solid-primitives/scheduled";
import type { FFXProject, ActivityCategory, ActivityLogEntry } from "../../types/project";
import { createActivityEntry } from "../../types/project";
import { logger } from "../../utils/logger";
import type { ProjectStateSignals, ProjectStateSetters, ActivityLogger } from "./types";
import { dbSync } from "./useProjectDbSync";

const log = logger.scope("ActivityLog");

/**
 * Create activity logging function with batched updates
 */
export function createActivityLogger(
  signals: ProjectStateSignals,
  setters: ProjectStateSetters,
  markModified: () => void
): ActivityLogger {
  // Batch pending log entries to reduce state updates
  let pendingEntries: ActivityLogEntry[] = [];
  
  // Flush batched entries to project state
  const flushEntries = debounce(() => {
    const proj = signals.project();
    log.debug(`flushEntries called, pendingEntries=${pendingEntries.length}, hasProject=${!!proj}, loading=${signals.loading()}`);
    if (!proj || pendingEntries.length === 0) return;
    
    // Don't flush during project loading - entries will be discarded
    // The loaded project already has its own activity log
    if (signals.loading()) {
      log.debug("Skipping flush during project load");
      pendingEntries = [];
      return;
    }
    
    const limit = proj.activity_log_limit || 1000;
    let log_entries = [...pendingEntries, ...proj.activity_log];
    if (log_entries.length > limit) {
      log_entries = log_entries.slice(0, limit);
    }
    
    log.debug(`Updating project with ${log_entries.length} log entries (no markModified — entries already in .ffxdb)`);
    setters.setProject({ ...proj, activity_log: log_entries } as FFXProject);
    // NOTE: Do NOT call markModified() here. Activity entries are already
    // persisted to the .ffxdb file immediately via dbSync.insertActivity().
    // Calling markModified would re-dirty the project right after autosave
    // clears the flag, creating an infinite save→dirty→save loop.
    // The in-memory activity_log update is for UI display only; the .cffx
    // file will naturally include the latest log on the next real save
    // triggered by a user action (bookmark, note, hash, setting change, etc.).
    pendingEntries = [];
  }, 500); // Batch entries over 500ms window

  const logActivity = (
    category: ActivityCategory,
    action: string,
    description: string,
    filePath?: string,
    details?: Record<string, unknown>
  ) => {
    const proj = signals.project();
    if (!proj || !proj.settings?.track_activity) return;

    const entry = createActivityEntry(
      signals.currentUser(),
      category,
      action,
      description,
      filePath,
      details
    );

    // Write-through to .ffxdb immediately (fire-and-forget)
    dbSync.insertActivity(entry);

    // Add to pending batch for in-memory project state
    pendingEntries.unshift(entry);
    
    // Schedule flush
    flushEntries();
  };

  return { logActivity };
}
