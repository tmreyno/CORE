// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Core project state signals and utility functions
 */

import { createSignal } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { FFXProject } from "../../types/project";
import { logger } from "../../utils/logger";
import type { ProjectStateSignals, ProjectStateSetters } from "./types";

const log = logger.scope("ProjectState");

/** Get current username from environment */
export async function getCurrentUsername(): Promise<string> {
  try {
    const username = await invoke<string>("get_current_username");
    return username;
  } catch {
    return "unknown";
  }
}

/** Get current app version */
export async function getAppVersion(): Promise<string> {
  try {
    const version = await invoke<string>("get_app_version");
    return version;
  } catch {
    return "0.1.0";
  }
}

/**
 * Create all project state signals
 * Returns both accessors (for reading) and setters (for internal use)
 */
export function createProjectState(): { signals: ProjectStateSignals; setters: ProjectStateSetters } {
  // Core State
  const [project, setProject] = createSignal<FFXProject | null>(null);
  const [projectPath, setProjectPath] = createSignal<string | null>(null);
  const [modified, setModified] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [loading, setLoading] = createSignal(false);

  // User/Session State
  const [currentUser, setCurrentUser] = createSignal<string>("unknown");
  const [currentSessionId, setCurrentSessionId] = createSignal<string | null>(null);

  // Auto-save State
  const [autoSaveEnabled, setAutoSaveEnabled] = createSignal(true);
  const [lastAutoSave, setLastAutoSave] = createSignal<Date | null>(null);

  // Initialize current user
  getCurrentUsername().then(setCurrentUser);

  return {
    signals: {
      project,
      projectPath,
      modified,
      error,
      loading,
      currentUser,
      currentSessionId,
      autoSaveEnabled,
      lastAutoSave,
    },
    setters: {
      setProject,
      setProjectPath,
      setModified,
      setError,
      setLoading,
      setCurrentUser,
      setCurrentSessionId,
      setAutoSaveEnabled,
      setLastAutoSave,
    },
  };
}

/**
 * Mark project as modified
 * Suppresses modification during project loading to prevent false dirty states
 */
export function createMarkModified(
  signals: ProjectStateSignals,
  setters: ProjectStateSetters
): () => void {
  return () => {
    // Don't mark as modified while loading a project
    if (signals.loading()) {
      log.debug("markModified: Suppressed during loading");
      return;
    }
    
    const hasProject = signals.project();
    const hasPath = signals.projectPath();
    log.debug(`markModified called: hasProject=${!!hasProject}, hasPath=${hasPath}, currentModified=${signals.modified()}`);
    if (hasProject || hasPath) {
      log.debug("markModified: Setting modified to TRUE");
      setters.setModified(true);
    } else {
      log.debug("markModified: No project/path, NOT setting modified");
    }
  };
}
