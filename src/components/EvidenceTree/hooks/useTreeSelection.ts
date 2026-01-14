// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useTreeSelection - Manages selection state for tree nodes
 * 
 * Provides consistent selection tracking with support for single
 * and multi-select modes.
 */

import { createSignal } from "solid-js";

export interface UseTreeSelectionReturn {
  /** Currently selected key(s) */
  selectedKey: () => string | null;
  
  /** Check if a node is selected */
  isSelected: (key: string) => boolean;
  
  /** Select a node */
  select: (key: string) => void;
  
  /** Clear selection */
  clearSelection: () => void;
}

export function useTreeSelection(): UseTreeSelectionReturn {
  const [selectedKey, setSelectedKey] = createSignal<string | null>(null);

  const isSelected = (key: string): boolean => {
    return selectedKey() === key;
  };

  const select = (key: string) => {
    setSelectedKey(key);
  };

  const clearSelection = () => {
    setSelectedKey(null);
  };

  return {
    selectedKey,
    isSelected,
    select,
    clearSelection,
  };
}
