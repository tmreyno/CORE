// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useTreeExpansion - Manages expansion state for tree nodes
 * 
 * Provides consistent expansion tracking across all container types.
 */

import { createSignal } from "solid-js";

export interface UseTreeExpansionReturn {
  /** Set of expanded node keys */
  expandedKeys: () => Set<string>;
  
  /** Check if a node is expanded */
  isExpanded: (key: string) => boolean;
  
  /** Toggle expansion state */
  toggle: (key: string) => void;
  
  /** Expand a node */
  expand: (key: string) => void;
  
  /** Collapse a node */
  collapse: (key: string) => void;
  
  /** Expand multiple nodes */
  expandAll: (keys: string[]) => void;
  
  /** Collapse all nodes */
  collapseAll: () => void;
}

export function useTreeExpansion(): UseTreeExpansionReturn {
  const [expandedKeys, setExpandedKeys] = createSignal<Set<string>>(new Set());

  const isExpanded = (key: string): boolean => {
    return expandedKeys().has(key);
  };

  const toggle = (key: string) => {
    setExpandedKeys(prev => {
      const next = new Set(prev);
      if (next.has(key)) {
        next.delete(key);
      } else {
        next.add(key);
      }
      return next;
    });
  };

  const expand = (key: string) => {
    setExpandedKeys(prev => {
      const next = new Set(prev);
      next.add(key);
      return next;
    });
  };

  const collapse = (key: string) => {
    setExpandedKeys(prev => {
      const next = new Set(prev);
      next.delete(key);
      return next;
    });
  };

  const expandAll = (keys: string[]) => {
    setExpandedKeys(prev => {
      const next = new Set(prev);
      keys.forEach(key => next.add(key));
      return next;
    });
  };

  const collapseAll = () => {
    setExpandedKeys(new Set<string>());
  };

  return {
    expandedKeys,
    isExpanded,
    toggle,
    expand,
    collapse,
    expandAll,
    collapseAll,
  };
}
