// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * TreeEmptyState - Empty state display for tree views
 */

import type { JSX } from 'solid-js';
import { HiOutlineFolder } from '../icons';

export interface TreeEmptyStateProps {
  /** Main message to display */
  message: string;
  /** Optional secondary message */
  hint?: string;
  /** Custom icon (defaults to folder) */
  icon?: JSX.Element;
  /** Indent depth for alignment */
  depth?: number;
}

export function TreeEmptyState(props: TreeEmptyStateProps) {
  const paddingLeft = () => props.depth ? `${(props.depth + 1) * 16}px` : '32px';
  
  return (
    <div 
      class="flex flex-col gap-1 py-3 text-center"
      style={{ 'padding-left': paddingLeft() }}
    >
      <div class="flex items-center justify-center gap-2 text-txt-muted">
        {props.icon || <HiOutlineFolder class="w-5 h-5 opacity-50" />}
        <span class="text-sm">{props.message}</span>
      </div>
      {props.hint && (
        <span class="text-xs text-txt-muted">{props.hint}</span>
      )}
    </div>
  );
}
