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

import { Show, type JSX } from 'solid-js';
import { ExpandIcon } from './ExpandIcon';
import { TreeIcon } from './TreeIcon';
import { formatBytes } from '../../utils';
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
  // Calculate padding based on depth using standardized indent
  const paddingLeft = () => getTreeIndent(props.depth);
  
  // Row classes with selection and hover states - using standardized constants
  const rowClasses = () => {
    const classes = [TREE_ROW_BASE_CLASSES];
    
    if (props.isSelected) {
      classes.push(TREE_ROW_SELECTED_CLASSES);
    } else {
      classes.push(TREE_ROW_NORMAL_CLASSES);
    }
    
    if (props.class) {
      classes.push(props.class);
    }
    
    return classes.join(' ');
  };
  
  // Handle keyboard navigation
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      props.onClick();
    }
    if (e.key === 'ArrowRight' && props.isDir && !props.isExpanded && props.onToggle) {
      e.preventDefault();
      props.onToggle();
    }
    if (e.key === 'ArrowLeft' && props.isDir && props.isExpanded && props.onToggle) {
      e.preventDefault();
      props.onToggle();
    }
  };
  
  // Handle expand toggle click (prevent event propagation)
  const handleToggleClick = (e: MouseEvent) => {
    e.stopPropagation();
    if (props.onToggle) {
      props.onToggle();
    } else {
      props.onClick();
    }
  };

  return (
    <div
      class={rowClasses()}
      style={{ 'padding-left': paddingLeft() }}
      onClick={props.onClick}
      onDblClick={props.onDblClick}
      onKeyDown={handleKeyDown}
      role="treeitem"
      aria-expanded={props.isDir ? props.isExpanded : undefined}
      aria-selected={props.isSelected}
      tabIndex={props.isSelected ? 0 : -1}
      title={props.path}
      data-tree-item
      data-entry-path={props['data-entry-path']}
      data-entry-addr={props['data-entry-addr']}
    >
      {/* Expand/collapse indicator */}
      <span 
        class="w-4 flex items-center justify-center shrink-0"
        style={{ visibility: props.hasChildren ? 'visible' : 'hidden' }}
        onClick={handleToggleClick}
        aria-hidden="true"
      >
        <ExpandIcon isLoading={props.isLoading} isExpanded={props.isExpanded} />
      </span>
      
      {/* File/folder icon */}
      <TreeIcon 
        name={props.name} 
        isDir={props.isDir} 
        isExpanded={props.isExpanded}
        entryType={props.entryType}
      />
      
      {/* File/folder name */}
      <span class="flex-1 truncate" title={props.name}>
        {props.name}
      </span>
      
      {/* Entry type badge (UFED) */}
      <Show when={props.entryType}>
        <span class="text-[9px] px-1.5 py-0.5 rounded bg-bg-hover/80 text-txt-secondary shrink-0">
          {props.entryType}
        </span>
      </Show>
      
      {/* File size */}
      <Show when={!props.isDir && props.size > 0}>
        <span class="text-[10px] text-txt-muted tabular-nums shrink-0">
          {formatBytes(props.size)}
        </span>
      </Show>
      
      {/* Hash verification badge */}
      <Show when={props.hash}>
        <span 
          class="w-3.5 h-3.5 rounded-full bg-green-500/20 text-green-400 flex items-center justify-center shrink-0"
          title={`Verified: ${props.hash}`}
        >
          <svg class="w-2.5 h-2.5" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M16.707 5.293a1 1 0 010 1.414l-8 8a1 1 0 01-1.414 0l-4-4a1 1 0 011.414-1.414L8 12.586l7.293-7.293a1 1 0 011.414 0z" clip-rule="evenodd" />
          </svg>
        </span>
      </Show>
      
      {/* Custom badge */}
      <Show when={props.badge}>
        {props.badge}
      </Show>
    </div>
  );
}
