// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * TreeRow - A single row in the evidence tree
 * 
 * Standardized row component with consistent styling, indentation,
 * keyboard accessibility, and selection states.
 * 
 * This is the CANONICAL tree row component - all tree views should use this.
 */

import { Show, type JSX, createMemo, splitProps } from 'solid-js';
import { ExpandIcon } from './ExpandIcon';
import { TreeIcon } from './TreeIcon';
import { formatBytes } from '../../utils';
import { getPreference } from '../preferences';
import {
  TREE_ROW_BASE_CLASSES,
  TREE_ROW_SELECTED_CLASSES,
  TREE_ROW_NORMAL_CLASSES,
  getTreeIndent,
} from './constants';

export interface TreeRowProps {
  /** Display name */
  name: string;
  /** Full path (for tooltip) */
  path: string;
  /** Whether this is a directory */
  isDir: boolean;
  /** Size in bytes */
  size: number;
  /** Indentation depth (0 = root) */
  depth: number;
  /** Whether this row is selected */
  isSelected: boolean;
  /** Whether this row is expanded (for directories) */
  isExpanded: boolean;
  /** Whether this row is loading children */
  isLoading: boolean;
  /** Whether this directory has children (shows expand icon) */
  hasChildren: boolean;
  /** Click handler */
  onClick: () => void;
  /** Double-click handler (optional) */
  onDblClick?: () => void;
  /** Expand/collapse toggle handler (optional, defaults to onClick) */
  onToggle?: () => void;
  /** Entry type hint (for UFED, etc.) */
  entryType?: string;
  /** Hash value to show verification badge */
  hash?: string | null;
  /** Custom badge content */
  badge?: JSX.Element;
  /** Additional class names */
  class?: string;
  /** Data attributes for testing/debugging */
  'data-entry-path'?: string;
  'data-entry-addr'?: number;
}

export function TreeRow(props: TreeRowProps) {
  // Split props for cleaner handling of data attributes
  const [local, dataAttrs] = splitProps(props, [
    'name', 'path', 'isDir', 'size', 'depth', 'isSelected', 'isExpanded',
    'isLoading', 'hasChildren', 'onClick', 'onDblClick', 'onToggle',
    'entryType', 'hash', 'badge', 'class'
  ]);
  
  // Calculate padding based on depth using standardized indent (memoized)
  const paddingLeft = createMemo(() => getTreeIndent(local.depth));
  
  // Display name - conditionally strip extension based on preference
  const displayName = createMemo(() => {
    if (local.isDir || getPreference("showFileExtensions")) {
      return local.name;
    }
    // Strip the extension for files when showFileExtensions is false
    const lastDot = local.name.lastIndexOf(".");
    if (lastDot > 0) {
      return local.name.substring(0, lastDot);
    }
    return local.name;
  });
  
  // Row classes with selection and hover states - memoized for efficiency
  const rowClasses = createMemo(() => {
    const classes = [TREE_ROW_BASE_CLASSES];
    
    if (local.isSelected) {
      classes.push(TREE_ROW_SELECTED_CLASSES);
    } else {
      classes.push(TREE_ROW_NORMAL_CLASSES);
    }
    
    if (local.class) {
      classes.push(local.class);
    }
    
    return classes.join(' ');
  });
  
  // Handle keyboard navigation
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      local.onClick();
    }
    if (e.key === 'ArrowRight' && local.isDir && !local.isExpanded && local.onToggle) {
      e.preventDefault();
      local.onToggle();
    }
    if (e.key === 'ArrowLeft' && local.isDir && local.isExpanded && local.onToggle) {
      e.preventDefault();
      local.onToggle();
    }
  };
  
  // Handle expand toggle click (prevent event propagation)
  const handleToggleClick = (e: MouseEvent) => {
    e.stopPropagation();
    if (local.onToggle) {
      local.onToggle();
    } else {
      local.onClick();
    }
  };

  return (
    <div
      class={rowClasses()}
      style={{ 'padding-left': paddingLeft() }}
      onClick={local.onClick}
      onDblClick={local.onDblClick}
      onKeyDown={handleKeyDown}
      role="treeitem"
      aria-expanded={local.isDir ? local.isExpanded : undefined}
      aria-selected={local.isSelected}
      tabIndex={local.isSelected ? 0 : -1}
      title={local.path}
      data-tree-item
      data-entry-path={dataAttrs['data-entry-path']}
      data-entry-addr={dataAttrs['data-entry-addr']}
    >
      {/* Expand/collapse indicator */}
      <span 
        class="w-5 flex items-center justify-center shrink-0"
        style={{ visibility: local.hasChildren ? 'visible' : 'hidden' }}
        onClick={handleToggleClick}
        aria-hidden="true"
      >
        <ExpandIcon isLoading={local.isLoading} isExpanded={local.isExpanded} />
      </span>
      
      {/* File/folder icon */}
      <TreeIcon 
        name={local.name} 
        isDir={local.isDir} 
        isExpanded={local.isExpanded}
        entryType={local.entryType}
      />
      
      {/* File/folder name */}
      <span class="flex-1 truncate" title={local.name}>
        {displayName()}
      </span>
      
      {/* Entry type badge (UFED) */}
      <Show when={local.entryType}>
        <span class="text-[9px] px-1.5 py-0.5 rounded bg-bg-hover/80 text-txt-secondary shrink-0">
          {local.entryType}
        </span>
      </Show>
      
      {/* File/folder size - show for files always, directories if size > 0 */}
      <Show when={local.size > 0}>
        <span class={`tree-file-size text-[10px] tabular-nums shrink-0 ${local.isDir ? 'text-txt-muted/60' : 'text-txt-muted'}`}>
          {formatBytes(local.size)}
        </span>
      </Show>
      
      {/* Hash verification badge */}
      <Show when={local.hash}>
        <span 
          class="relative inline-flex text-green-400 shrink-0"
          title={`Verified: ${local.hash}`}
        >
          <span class="text-[10px]">✓</span>
          <span class="text-[10px] absolute left-[3px]">✓</span>
        </span>
      </Show>
      
      {/* Custom badge */}
      <Show when={local.badge}>
        {local.badge}
      </Show>
    </div>
  );
}
