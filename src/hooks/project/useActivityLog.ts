// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Activity logging functionality for project
 */

import type { FFXProject, ActivityCategory } from "../../types/project";
import { createActivityEntry } from "../../types/project";
import type { ProjectStateSignals, ProjectStateSetters, ActivityLogger } from "./types";

/**
 * Create activity logging function
 */
export function createActivityLogger(
  signals: ProjectStateSignals,
  setters: ProjectStateSetters,
  markModified: () => void
): ActivityLogger {
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

    // Add entry, respecting limit
    const limit = proj.activity_log_limit || 1000;
    let log = [entry, ...proj.activity_log];
    if (log.length > limit) {
      log = log.slice(0, limit);
    }

    setters.setProject({ ...proj, activity_log: log } as FFXProject);
    markModified();
  };

  return { logActivity };
}
