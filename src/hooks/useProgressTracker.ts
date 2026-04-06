// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Compatibility wrapper over @core-suite/desktop-hooks.
 * Keeps local imports stable while the shared implementation lives in core-shared.
 */

export {
  createProgressTracker,
  type ProgressSnapshot,
  type SmoothedStats,
} from "@core-suite/desktop-hooks";
