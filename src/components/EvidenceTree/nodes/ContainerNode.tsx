// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ContainerNode - Container header components for EvidenceTree
 * 
 * Renders container headers with type-specific icons and statistics.
 */

import { Show, JSX, Switch, Match } from "solid-js";
import { 
  HiOutlineFolderOpen,
  HiOutlineArchiveBox,
  HiOutlineCircleStack,
  HiOutlineDevicePhoneMobile,
} from "../../icons";
import { 
  ExpandIcon,
  TREE_ROW_BASE_CLASSES,
  TREE_ROW_SELECTED_CLASSES,
  TREE_ROW_NORMAL_CLASSES,
  getTreeIndent,
} from "../../tree";
import { formatBytes } from "../../../utils";

/** Container info passed to ContainerNode */
export interface ContainerInfo {
  path: string;
  container_type: string;
  size?: number;
}

export interface ContainerNodeProps {
  container: ContainerInfo;
  isSelected: boolean;
  isExpanded: boolean;
  isLoading: boolean;
  entryCount?: number;
  partitionCount?: number;
  onClick: () => void;
  onToggle: () => void;
  onRemove: () => void;
}

/**
 * Get container type icon based on container type
 */
function getContainerIcon(type: string): JSX.Element {
  const iconClass = "w-4 h-4";
  
  switch (type.toLowerCase()) {
    case 'ad1':
      return <HiOutlineFolderOpen class={`${iconClass} text-blue-400`} />;
    case 'e01':
    case 'raw':
    case 'l01':
      return <HiOutlineCircleStack class={`${iconClass} text-purple-400`} />;
    case 'zip':
    case '7z':
    case 'rar':
    case 'tar':
    case 'gz':
      return <HiOutlineArchiveBox class={`${iconClass} text-yellow-400`} />;
    case 'ufdr':
    case 'cellebrite':
      return <HiOutlineDevicePhoneMobile class={`${iconClass} text-green-400`} />;
    default:
      return <HiOutlineFolderOpen class={iconClass} />;
  }
}

/**
 * Get container type label for display
 */
function getContainerTypeLabel(type: string): string {
  switch (type.toLowerCase()) {
    case 'ad1': return 'AD1';
    case 'e01': return 'E01';
    case 'raw': return 'RAW';
    case 'l01': return 'L01';
    case 'zip': return 'ZIP';
    case '7z': return '7Z';
    case 'rar': return 'RAR';
    case 'ufdr': return 'UFDR';
    default: return type.toUpperCase();
  }
}

/**
 * Container Node - renders a container header with icon, name, and stats
 */
export function ContainerNode(props: ContainerNodeProps): JSX.Element {
  const fileName = () => props.container.path.split('/').pop() || props.container.path;
  const typeLabel = () => getContainerTypeLabel(props.container.container_type);
  
  const rowClasses = () => props.isSelected 
    ? `${TREE_ROW_BASE_CLASSES} ${TREE_ROW_SELECTED_CLASSES}`
    : `${TREE_ROW_BASE_CLASSES} ${TREE_ROW_NORMAL_CLASSES}`;

  return (
    <div 
      class={rowClasses()}
      onClick={props.onClick}
      style={{ "padding-left": getTreeIndent(0) }}
      role="treeitem"
      aria-expanded={props.isExpanded}
      aria-selected={props.isSelected}
      tabIndex={0}
      data-tree-item
      data-container-path={props.container.path}
    >
      {/* Expand/collapse toggle */}
      <span 
        class="w-4 text-xs text-txt-muted flex items-center justify-center cursor-pointer"
        onClick={(e) => { e.stopPropagation(); props.onToggle(); }}
        aria-hidden="true"
      >
        <ExpandIcon isLoading={props.isLoading} isExpanded={props.isExpanded} />
      </span>

      {/* Container type icon */}
      <span class="text-base" aria-hidden="true">
        {getContainerIcon(props.container.container_type)}
      </span>

      {/* Container name */}
      <span class="text-sm text-txt truncate flex-1" title={props.container.path}>
        {fileName()}
      </span>

      {/* Container metadata badge */}
      <span class="text-xs text-txt-muted flex items-center gap-1">
        <span class="px-1.5 py-0.5 bg-bg-panel rounded text-txt-secondary">
          {typeLabel()}
        </span>
        <Switch>
          <Match when={props.partitionCount !== undefined && props.partitionCount > 0}>
            <span>{props.partitionCount} partition{props.partitionCount !== 1 ? 's' : ''}</span>
          </Match>
          <Match when={props.entryCount !== undefined}>
            <span>{(props.entryCount ?? 0).toLocaleString()} item{props.entryCount !== 1 ? 's' : ''}</span>
          </Match>
        </Switch>
        <Show when={props.container.size !== undefined && props.container.size > 0}>
          <span>• {formatBytes(props.container.size ?? 0)}</span>
        </Show>
      </span>

      {/* Remove button */}
      <button
        class="ml-2 p-1 text-txt-muted hover:text-red-400 hover:bg-bg-panel rounded opacity-0 group-hover:opacity-100 transition-opacity"
        onClick={(e) => { e.stopPropagation(); props.onRemove(); }}
        title="Remove container"
        aria-label="Remove container"
      >
        ×
      </button>
    </div>
  );
}
