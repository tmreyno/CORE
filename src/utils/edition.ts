// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Build edition utilities.
 *
 * __APP_EDITION__ is injected at build time by Vite (see vite.config.ts).
 *   "full"    – CORE-FFX full suite (default)
 *   "acquire" – CORE Acquire (evidence acquisition only)
 */

import {
  getEditionAppName,
  getEditionAppShort,
  resolveAppEdition,
  type AppEdition,
} from "@core-suite/types/edition";

export type { AppEdition } from "@core-suite/types/edition";

export const APP_EDITION: AppEdition =
  resolveAppEdition(typeof __APP_EDITION__ !== "undefined" ? __APP_EDITION__ : undefined);

export const isFullEdition = () => APP_EDITION === "full";
export const isAcquireEdition = () => APP_EDITION === "acquire";

/**
 * App display name based on edition.
 */
export const APP_NAME = getEditionAppName(APP_EDITION);
export const APP_SHORT = getEditionAppShort(APP_EDITION);
