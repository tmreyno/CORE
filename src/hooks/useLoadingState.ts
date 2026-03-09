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

import { createSignal } from "solid-js";
import { logger } from "../utils/logger";

const log = logger.scope("LoadingState");

export interface LoadingStateReturn {
  isLoading: () => boolean;
  message: () => string;
  error: () => string | null;
  /** Wrap an async operation with loading/error state management */
  run: <T>(message: string, fn: () => Promise<T>) => Promise<T | undefined>;
  /** Manually set loading state (for operations that manage their own lifecycle) */
  setLoading: (loading: boolean, message?: string) => void;
  /** Clear any error */
  clearError: () => void;
}

export function useLoadingState(): LoadingStateReturn {
  const [isLoading, setIsLoading] = createSignal(false);
  const [message, setMessage] = createSignal("");
  const [error, setError] = createSignal<string | null>(null);

  let errorTimer: ReturnType<typeof setTimeout> | undefined;

  const clearError = () => {
    if (errorTimer) clearTimeout(errorTimer);
    setError(null);
  };

  const run = async <T,>(msg: string, fn: () => Promise<T>): Promise<T | undefined> => {
    clearError();
    setMessage(msg);
    setIsLoading(true);
    try {
      const result = await fn();
      setIsLoading(false);
      return result;
    } catch (err) {
      setIsLoading(false);
      const errMsg = err instanceof Error ? err.message : String(err);
      setError(errMsg);
      log.error(`Operation failed (${msg}):`, err);
      // Auto-clear error after 5 seconds
      errorTimer = setTimeout(() => setError(null), 5000);
      return undefined;
    }
  };

  const setLoadingManual = (loading: boolean, msg?: string) => {
    setIsLoading(loading);
    if (msg) setMessage(msg);
    if (!loading) setMessage("");
  };

  return {
    isLoading,
    message,
    error,
    run,
    setLoading: setLoadingManual,
    clearError,
  };
}
