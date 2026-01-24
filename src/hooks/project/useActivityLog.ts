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
import type { ProjectStateSignals, ProjectStateSetters, ActivityLogger } from "./types";

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
    if (!proj || pendingEntries.length === 0) return;
    
    const limit = proj.activity_log_limit || 1000;
    let log = [...pendingEntries, ...proj.activity_log];
    if (log.length > limit) {
      log = log.slice(0, limit);
    }
    
    setters.setProject({ ...proj, activity_log: log } as FFXProject);
    markModified();
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

    // Add to pending batch
    pendingEntries.unshift(entry);
    
    // Schedule flush
    flushEntries();
  };

  return { logActivity };
}
