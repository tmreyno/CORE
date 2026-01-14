// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useTreeLoading - Manages loading state for tree nodes
 * 
 * Tracks which nodes are currently loading their children.
 */

import { createSignal } from "solid-js";

export interface UseTreeLoadingReturn {
  /** Set of loading node keys */
  loadingKeys: () => Set<string>;
  
  /** Check if a node is loading */
  isLoading: (key: string) => boolean;
  
  /** Start loading for a key */
  startLoading: (key: string) => void;
  
  /** Stop loading for a key */
  stopLoading: (key: string) => void;
  
  /** Execute an async operation with loading state */
  withLoading: <T>(key: string, operation: () => Promise<T>) => Promise<T>;
}

export function useTreeLoading(): UseTreeLoadingReturn {
  const [loadingKeys, setLoadingKeys] = createSignal<Set<string>>(new Set());

  const isLoading = (key: string): boolean => {
    return loadingKeys().has(key);
  };

  const startLoading = (key: string) => {
    setLoadingKeys(prev => {
      const next = new Set(prev);
      next.add(key);
      return next;
    });
  };

  const stopLoading = (key: string) => {
    setLoadingKeys(prev => {
      const next = new Set(prev);
      next.delete(key);
      return next;
    });
  };

  const withLoading = async <T>(key: string, operation: () => Promise<T>): Promise<T> => {
    startLoading(key);
    try {
      return await operation();
    } finally {
      stopLoading(key);
    }
  };

  return {
    loadingKeys,
    isLoading,
    startLoading,
    stopLoading,
    withLoading,
  };
}
