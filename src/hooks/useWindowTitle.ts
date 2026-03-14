// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * @fileoverview Window title management hook
 * 
 * Manages the application window title to show:
 * - Current project name
 * - Unsaved changes indicator (*)
 * - App name suffix
 */

import { createEffect, on, type Accessor } from "solid-js";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { logger } from "../utils/logger";
import { APP_NAME } from "../utils/edition";

const log = logger.scope("WindowTitle");

export interface UseWindowTitleOptions {
  /** Project name (null if no project open) */
  projectName: Accessor<string | null>;
  /** Whether there are unsaved changes */
  modified: Accessor<boolean>;
  /** Project path (for showing file location in title) */
  projectPath?: Accessor<string | null>;
}

/**
 * Hook to manage window title with project name and unsaved indicator
 * 
 * Title formats:
 * - No project: "CORE-FFX"
 * - Project open: "ProjectName - CORE-FFX"
 * - Unsaved changes: "● ProjectName - CORE-FFX"
 */
export function useWindowTitle(options: UseWindowTitleOptions) {
  const { projectName, modified, projectPath } = options;

  log.debug("Hook initialized");

  // Update window title whenever project or modified state changes
  createEffect(on(
    () => ({ name: projectName(), isModified: modified(), path: projectPath?.() }),
    async ({ name, isModified }) => {
      log.debug(`Effect: name=${name}, isModified=${isModified}`);
      try {
        const window = getCurrentWindow();
        
        let title: string;
        
        if (!name) {
          // No project open
          title = APP_NAME;
        } else {
          // Project open - show name with optional unsaved indicator
          const unsavedMarker = isModified ? "● " : "";
          title = `${unsavedMarker}${name} - ${APP_NAME}`;
        }
        
        log.debug(`Setting title to "${title}"`);
        await window.setTitle(title);
      } catch (err) {
        log.warn("Failed to update window title:", err);
      }
    },
    { defer: false } // Run immediately on mount
  ));
}

/**
 * Set window title directly (useful for one-off updates)
 */
export async function setWindowTitle(title: string): Promise<void> {
  try {
    const window = getCurrentWindow();
    await window.setTitle(title);
  } catch (err) {
    log.warn("Failed to set window title:", err);
  }
}
