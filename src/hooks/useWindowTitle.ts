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

import {
  useWindowTitle as useSharedWindowTitle,
  setWindowTitle as setSharedWindowTitle,
} from "@core-suite/desktop-hooks";
import type { UseWindowTitleOptions } from "@core-suite/desktop-hooks";
export type { UseWindowTitleOptions } from "@core-suite/desktop-hooks";
import { createDesktopHookLogger } from "./desktopHookLogger";
import { APP_NAME } from "../utils/edition";

const log = createDesktopHookLogger("WindowTitle");

/**
 * Hook to manage window title with project name and unsaved indicator
 * 
 * Title formats:
 * - No project: "CORE-FFX"
 * - Project open: "ProjectName - CORE-FFX"
 * - Unsaved changes: "● ProjectName - CORE-FFX"
 */
export function useWindowTitle(options: UseWindowTitleOptions) {
  return useSharedWindowTitle(options, { appName: APP_NAME, log });
}

/**
 * Set window title directly (useful for one-off updates)
 */
export async function setWindowTitle(title: string): Promise<void> {
  return setSharedWindowTitle(title, { log });
}
