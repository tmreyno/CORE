// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Portable mode hook.
 *
 * Exposes reactive signals for the portable status and config.
 * The status is NOT queried on mount — call `check()` explicitly
 * (e.g., when a project is opened/created) so no backend calls
 * happen before the user starts working.  The result is cached.
 */

import { createSignal } from "solid-js";
import {
  getPortableStatus,
  type PortableConfig,
  type PortableStatus,
} from "../api/portable";
import { logger } from "../utils/logger";

const log = logger.scope("PortableMode");

export interface UsePortableModeReturn {
  /** Whether portable mode is active */
  isPortable: () => boolean;
  /** Portable configuration (null if not portable) */
  config: () => PortableConfig | null;
  /** Full status object */
  status: () => PortableStatus | null;
  /** Whether the status has been checked yet */
  ready: () => boolean;
  /** Query the backend for portable status (safe to call multiple times — cached after first success) */
  check: () => void;
}

/**
 * Hook to detect and expose portable mode status.
 *
 * Call once in the top-level App component. Invoke `check()` when
 * appropriate (e.g., on project load) — portable mode cannot change
 * at runtime, so the result is cached after the first successful query.
 */
export function usePortableMode(): UsePortableModeReturn {
  const [status, setStatus] = createSignal<PortableStatus | null>(null);
  const [ready, setReady] = createSignal(false);

  log.debug("Hook initialized");

  async function check() {
    if (ready()) return; // already checked
    try {
      const result = await getPortableStatus();
      setStatus(result);
      log.info(`Portable status: ${result.isPortable ? "ACTIVE" : "inactive"}${result.config?.detectionReason ? ` (${result.config.detectionReason})` : ""}`);
    } catch (e) {
      log.warn("Failed to check portable mode:", e);
      setStatus({ isPortable: false, config: null });
    }
    setReady(true);
  }

  return {
    isPortable: () => status()?.isPortable ?? false,
    config: () => status()?.config ?? null,
    status: () => status(),
    ready: () => ready(),
    check,
  };
}
