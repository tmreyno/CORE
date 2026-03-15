// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Portable mode hook.
 *
 * Queries the backend on mount to detect whether the app is running in
 * portable mode (from removable media or with a `portable.marker` file).
 * Exposes reactive signals for the portable status and config.
 */

import { createSignal, onMount } from "solid-js";
import {
  getPortableStatus,
  type PortableConfig,
  type PortableStatus,
} from "../api/portable";

export interface UsePortableModeReturn {
  /** Whether portable mode is active */
  isPortable: () => boolean;
  /** Portable configuration (null if not portable) */
  config: () => PortableConfig | null;
  /** Full status object */
  status: () => PortableStatus | null;
  /** Whether the status has been checked yet */
  ready: () => boolean;
}

/**
 * Hook to detect and expose portable mode status.
 *
 * Call once in the top-level App component. The status is queried from
 * the backend on mount and cached — portable mode cannot change at runtime.
 */
export function usePortableMode(): UsePortableModeReturn {
  const [status, setStatus] = createSignal<PortableStatus | null>(null);
  const [ready, setReady] = createSignal(false);

  onMount(async () => {
    try {
      const result = await getPortableStatus();
      setStatus(result);
    } catch (e) {
      console.warn("Failed to check portable mode:", e);
      setStatus({ isPortable: false, config: null });
    }
    setReady(true);
  });

  return {
    isPortable: () => status()?.isPortable ?? false,
    config: () => status()?.config ?? null,
    status: () => status(),
    ready: () => ready(),
  };
}
