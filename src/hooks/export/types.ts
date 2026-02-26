// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Shared types for export sub-hooks.
 */

import type { Activity } from "../../types/activity";

/** Export operation mode */
export type ExportMode = "physical" | "logical" | "native" | "tools";

/** Toast interface shared across all export sub-hooks */
export interface ExportToast {
  success: (title: string, message: string) => void;
  error: (title: string, message: string) => void;
  warning: (title: string, message: string) => void;
  info: (title: string, message: string) => void;
}

/** Activity callback options shared across export sub-hooks */
export interface ExportActivityCallbacks {
  onActivityCreate?: (activity: Activity) => void;
  onActivityUpdate?: (id: string, updates: Partial<Activity>) => void;
  onComplete?: (destination: string) => void;
}
