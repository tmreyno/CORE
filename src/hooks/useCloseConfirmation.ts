// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * @fileoverview Close confirmation hook for unsaved changes
 * 
 * Prevents accidental data loss by:
 * - Intercepting window close events
 * - Showing confirmation dialog when there are unsaved changes
 * - Allowing save before close
 */

import { onMount, onCleanup, type Accessor } from "solid-js";
import { getCurrentWindow, type CloseRequestedEvent } from "@tauri-apps/api/window";
import { confirm } from "@tauri-apps/plugin-dialog";
import { logger } from "../utils/logger";

const log = logger.scope("CloseConfirmation");

export interface UseCloseConfirmationOptions {
  /** Whether there are unsaved changes */
  hasUnsavedChanges: Accessor<boolean>;
  /** Callback to save changes (returns true if save was successful) */
  onSave?: () => Promise<boolean>;
  /** Optional callback when close is confirmed */
  onClose?: () => void;
  /** Title for the confirmation dialog */
  dialogTitle?: string;
  /** Message for the confirmation dialog */
  dialogMessage?: string;
}

/**
 * Hook to show confirmation dialog when closing window with unsaved changes
 * 
 * Uses Tauri's close_requested event to intercept window close and show
 * a native dialog asking the user to save or discard changes.
 */
export function useCloseConfirmation(options: UseCloseConfirmationOptions) {
  const {
    hasUnsavedChanges,
    onSave,
    onClose,
    dialogTitle = "Unsaved Changes",
    dialogMessage = "You have unsaved changes. Do you want to save before closing?",
  } = options;

  let unlisten: (() => void) | undefined;

  onMount(async () => {
    log.debug("Setting up close listener");
    try {
      const window = getCurrentWindow();
      
      // Listen for close requested event
      unlisten = await window.onCloseRequested(async (event: CloseRequestedEvent) => {
        const unsaved = hasUnsavedChanges();
        log.debug(`Close requested, hasUnsavedChanges=${unsaved}`);
        
        // If no unsaved changes, allow close
        if (!unsaved) {
          log.debug("No unsaved changes, allowing close");
          onClose?.();
          return;
        }

        // Prevent the close while we show the dialog
        log.debug("Showing confirmation dialog");
        event.preventDefault();

        // Show confirmation dialog
        const shouldSave = await confirm(dialogMessage, {
          title: dialogTitle,
          kind: "warning",
          okLabel: "Save & Close",
          cancelLabel: "Discard Changes",
        });

        log.debug(`User chose shouldSave=${shouldSave}`);
        if (shouldSave && onSave) {
          // Try to save
          const saved = await onSave();
          log.debug(`Save result=${saved}`);
          if (saved) {
            // Save successful, close window
            onClose?.();
            await window.close();
          }
          // If save failed, don't close (user can retry)
        } else {
          // User chose to discard or there's no save handler
          log.debug("Discarding changes and closing");
          onClose?.();
          await window.close();
        }
      });
      log.debug("Close listener setup complete");
    } catch (err) {
      log.warn("Failed to setup close confirmation:", err);
    }
  });

  onCleanup(() => {
    unlisten?.();
  });
}

/**
 * Show a standalone confirmation dialog for unsaved changes
 * (useful for navigation away from pages, not window close)
 */
export async function confirmUnsavedChanges(options?: {
  title?: string;
  message?: string;
}): Promise<"save" | "discard" | "cancel"> {
  const title = options?.title ?? "Unsaved Changes";
  const message = options?.message ?? "You have unsaved changes. What would you like to do?";
  
  try {
    const result = await confirm(message, {
      title,
      kind: "warning",
      okLabel: "Save Changes",
      cancelLabel: "Discard",
    });
    
    return result ? "save" : "discard";
  } catch {
    return "cancel";
  }
}
