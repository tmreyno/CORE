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

import { Show, JSX, Switch, Match, createMemo } from "solid-js";
import { getBasename } from "../../../utils";
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

// =============================================================================
// Container Type Utilities - Memoized lookups
// =============================================================================

/** Icon class for container type icons */
const ICON_CLASS = "w-4 h-4";

/** Container type icon map (static, no need to recreate) */
const CONTAINER_ICONS: Record<string, (className: string) => JSX.Element> = {
  'ad1': (cls) => <HiOutlineFolderOpen class={`${cls} text-blue-400`} />,
  'e01': (cls) => <HiOutlineCircleStack class={`${cls} text-purple-400`} />,
  'raw': (cls) => <HiOutlineCircleStack class={`${cls} text-purple-400`} />,
  'l01': (cls) => <HiOutlineCircleStack class={`${cls} text-purple-400`} />,
  'zip': (cls) => <HiOutlineArchiveBox class={`${cls} text-yellow-400`} />,
  '7z': (cls) => <HiOutlineArchiveBox class={`${cls} text-yellow-400`} />,
  'rar': (cls) => <HiOutlineArchiveBox class={`${cls} text-yellow-400`} />,
  'tar': (cls) => <HiOutlineArchiveBox class={`${cls} text-yellow-400`} />,
  'gz': (cls) => <HiOutlineArchiveBox class={`${cls} text-yellow-400`} />,
  'ufdr': (cls) => <HiOutlineDevicePhoneMobile class={`${cls} text-green-400`} />,
  'cellebrite': (cls) => <HiOutlineDevicePhoneMobile class={`${cls} text-green-400`} />,
};

/** Container type label map */
const TYPE_LABELS: Record<string, string> = {
  'ad1': 'AD1',
  'e01': 'E01',
  'raw': 'RAW',
  'l01': 'L01',
  'zip': 'ZIP',
  '7z': '7Z',
  'rar': 'RAR',
  'ufdr': 'UFDR',
};

/**
 * Get container type icon based on container type
 */
function getContainerIcon(type: string): JSX.Element {
  const key = type.toLowerCase();
  const iconFn = CONTAINER_ICONS[key];
  return iconFn ? iconFn(ICON_CLASS) : <HiOutlineFolderOpen class={ICON_CLASS} />;
}

/**
 * Get container type label for display
 */
function getContainerTypeLabel(type: string): string {
  return TYPE_LABELS[type.toLowerCase()] ?? type.toUpperCase();
}

// =============================================================================
// Container Node Component
// =============================================================================

/**
 * Container Node - renders a container header with icon, name, and stats
 */
export function ContainerNode(props: ContainerNodeProps): JSX.Element {
  // Memoized computed values
  const fileName = createMemo(() => getBasename(props.container.path) || props.container.path);
  const typeLabel = createMemo(() => getContainerTypeLabel(props.container.container_type));
  const containerIcon = createMemo(() => getContainerIcon(props.container.container_type));
  const formattedSize = createMemo(() => {
    const size = props.container.size;
    return size !== undefined && size > 0 ? formatBytes(size) : null;
  });
  
  // Memoized row classes
  const rowClasses = createMemo(() => props.isSelected 
    ? `${TREE_ROW_BASE_CLASSES} ${TREE_ROW_SELECTED_CLASSES}`
    : `${TREE_ROW_BASE_CLASSES} ${TREE_ROW_NORMAL_CLASSES}`);
  
  // Memoized partition/item text
  const hasPartitions = createMemo(() => props.partitionCount !== undefined && props.partitionCount > 0);
  const hasEntries = createMemo(() => props.entryCount !== undefined);

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
        {containerIcon()}
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
          <Match when={hasPartitions()}>
            <span>{props.partitionCount} partition{props.partitionCount !== 1 ? 's' : ''}</span>
          </Match>
          <Match when={hasEntries()}>
            <span>{(props.entryCount ?? 0).toLocaleString()} item{props.entryCount !== 1 ? 's' : ''}</span>
          </Match>
        </Switch>
        <Show when={formattedSize()}>
          <span>• {formattedSize()}</span>
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
