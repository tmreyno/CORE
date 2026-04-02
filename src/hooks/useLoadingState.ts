// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useLoadingState — reactive loading state for wrapping async operations.
 *
 * Usage:
 *   const loading = useLoadingState();
 *   await loading.run("Scanning evidence…", () => scanForFiles());
 *
 * Then in JSX:
 *   <LoadingOverlay
 *     isLoading={loading.isLoading}
 *     message={loading.message}
 *     error={loading.error}
 *   />
 */

import { useLoadingState as useSharedLoadingState } from "@core-suite/desktop-hooks";
export type { LoadingStateReturn } from "@core-suite/desktop-hooks";
import { logger } from "../utils/logger";

const log = logger.scope("LoadingState");
export function useLoadingState() {
  return useSharedLoadingState({ log });
}
