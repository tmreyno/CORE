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

import {
  useCloseConfirmation as useSharedCloseConfirmation,
  confirmUnsavedChanges as confirmSharedUnsavedChanges,
} from "@core-suite/desktop-hooks";
export type { UseCloseConfirmationOptions } from "@core-suite/desktop-hooks";
import { logger } from "../utils/logger";

const log = logger.scope("CloseConfirmation");

/**
 * Hook to show confirmation dialog when closing window with unsaved changes
 * 
 * Uses Tauri's close_requested event to intercept window close and show
 * a native dialog asking the user to save or discard changes.
 */
export function useCloseConfirmation(options: UseCloseConfirmationOptions) {
  return useSharedCloseConfirmation(options, { log });
}

/**
 * Show a standalone confirmation dialog for unsaved changes
 * (useful for navigation away from pages, not window close)
 */
export async function confirmUnsavedChanges(options?: {
  title?: string;
  message?: string;
}): Promise<"save" | "discard" | "cancel"> {
  return confirmSharedUnsavedChanges(options, { log });
}
