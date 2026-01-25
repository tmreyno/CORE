// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ContainerHeader - Header row for forensic containers in the tree
 * 
 * Shows container type badge, filename, and expansion state.
 * Now includes selection checkbox and hash status indicator.
 */

import { Show, type JSX } from 'solid-js';
import { ExpandIcon } from './ExpandIcon';
import {
  HiOutlineCircleStack,
  HiOutlineArchiveBox,
  HiOutlineDevicePhoneMobile,
  HiOutlineDocument,
  HiOutlineServerStack,
  HiOutlineCube,
  HiOutlineHashtag,
} from '../icons';
import {
  getContainerIconColor,
  getContainerIconType,
} from '../ui/constants';
import type { FileStatus, FileHashInfo } from '../../hooks';
import type { HashHistoryEntry, ContainerInfo } from '../../types';
import { compareHashes } from '../../hooks/hashUtils';

/**
 * Extract stored hashes from already-loaded ContainerInfo.
 * This is a synchronous helper for components that already have ContainerInfo.
 */
function extractStoredHashesFromInfo(containerInfo: ContainerInfo | null): Array<{ algorithm: string; hash: string }> {
  if (!containerInfo) return [];
  
  return [
    ...(containerInfo.e01?.stored_hashes ?? []),
    ...(containerInfo.l01?.stored_hashes ?? []),
    ...(containerInfo.ad1?.companion_log?.md5_hash ? [{ algorithm: 'MD5', hash: containerInfo.ad1.companion_log.md5_hash }] : []),
    ...(containerInfo.ad1?.companion_log?.sha1_hash ? [{ algorithm: 'SHA-1', hash: containerInfo.ad1.companion_log.sha1_hash }] : []),
    ...(containerInfo.ad1?.companion_log?.sha256_hash ? [{ algorithm: 'SHA-256', hash: containerInfo.ad1.companion_log.sha256_hash }] : []),
    ...(containerInfo.ufed?.stored_hashes ?? []),
    ...(containerInfo.companion_log?.stored_hashes ?? [])
  ];
}

export interface ContainerHeaderProps {
  /** Container file name */
  name: string;
  /** Full path */
  path: string;
  /** Container type (ad1, e01, ufed, archive, etc.) */
  containerType: string;
  /** File size */
  size?: number;
  /** Whether this container is selected/active */
  isActive: boolean;
  /** Whether this container is expanded */
  isExpanded: boolean;
  /** Whether this container is loading */
  isLoading: boolean;
  /** Number of segments (for multi-part containers) */
  segmentCount?: number;
  /** Whether container has incomplete segments (e.g., missing AD1 parts) */
  isIncomplete?: boolean;
  /** Missing segment info for tooltip */
  incompleteMessage?: string;
  /** Click handler */
  onClick: () => void;
  /** Additional status indicator */
  statusIcon?: JSX.Element;
  
  // === Selection & Hash Props ===
  /** Whether this container is checked for batch operations */
  isChecked?: boolean;
  /** Toggle selection handler */
  onToggleSelection?: (e: MouseEvent) => void;
  /** Hash this container */
  onHash?: (e: MouseEvent) => void;
  /** Current hash status */
  fileStatus?: FileStatus;
  /** Computed hash info */
  fileHash?: FileHashInfo;
  /** Hash history entries */
  hashHistory?: HashHistoryEntry[];
  /** Container metadata info (for stored hashes) */
  fileInfo?: ContainerInfo;
  /** Whether hashing is in progress globally */
  busy?: boolean;
  /** Context menu handler */
  onContextMenu?: (e: MouseEvent) => void;
}

/** Get container type icon */
function getContainerIcon(type: string) {
  const iconClass = `flex items-center justify-center shrink-0 shrink-0`;
  const iconType = getContainerIconType(type);
  const color = getContainerIconColor(type);
  
  switch (iconType) {
    case 'ad1':
      return <HiOutlineCube class={`${iconClass} ${color}`} />;
    case 'e01':
    case 'l01':
      return <HiOutlineCircleStack class={`${iconClass} ${color}`} />;
    case 'raw':
      return <HiOutlineServerStack class={`${iconClass} ${color}`} />;
    case 'ufed':
      return <HiOutlineDevicePhoneMobile class={`${iconClass} ${color}`} />;
    case 'archive':
      return <HiOutlineArchiveBox class={`${iconClass} ${color}`} />;
    default:
      return <HiOutlineDocument class={`${iconClass} ${color}`} />;
  }
}

export function ContainerHeader(props: ContainerHeaderProps) {
  const rowClasses = () => {
    const base = [
      'flex items-center gap-0.5',
      'py-0.5 px-1.5',
      'cursor-pointer',
      'transition-colors duration-100',
      'focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:ring-inset',
    ];
    
    if (props.isActive) {
      base.push('bg-accent/20');
    } else {
      base.push('hover:bg-bg-secondary/50');
    }
    
    return base.join(' ');
  };
  
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      props.onClick();
    }
  };
  
  // Hash state helpers
  const isHashing = () => props.fileStatus?.status === "hashing";
  const hashProgress = () => props.fileStatus?.progress ?? 0;
  const hasHashResult = () => !!props.fileHash;
  const storedHashCount = () => {
    // Use extractStoredHashesFromInfo helper for consistent counting
    const stored = extractStoredHashesFromInfo(props.fileInfo ?? null);
    return stored.length;
  };
  const historyCount = () => props.hashHistory?.length ?? 0;
  const totalHashCount = () => storedHashCount() + (hasHashResult() ? 1 : 0) + historyCount();
  
  // Check if any COMPUTED hash matches a stored/acquired hash
  // Only returns true when an actual verification occurred (computed vs stored comparison)
  const hasVerifiedMatch = () => {
    // Extract all stored hashes using helper
    const storedHashes = extractStoredHashesFromInfo(props.fileInfo ?? null);
    const history = props.hashHistory ?? [];
    
    // Find computed/verified hashes in history (NOT stored ones)
    const computedHashes = history.filter(h => h.source === 'computed' || h.source === 'verified');
    
    // Check if any computed hash matches a stored hash using algorithm-aware comparison
    for (const stored of storedHashes) {
      const match = computedHashes.find(h => 
        compareHashes(h.hash, stored.hash, h.algorithm, stored.algorithm)
      );
      if (match) return true;
    }
    return false;
  };
  
  // Hash indicator icon based on state
  const hashIndicator = () => {
    if (isHashing()) {
      return (
        <span class="text-[10px] text-accent" title={`Hashing... ${hashProgress().toFixed(0)}%`}>
          {hashProgress().toFixed(0)}%
        </span>
      );
    }
    if (props.fileHash?.verified === false) {
      return (
        <span class="text-red-400 font-bold text-[11px]" title="Hash mismatch!">✗</span>
      );
    }
    if (hasVerifiedMatch() || props.fileHash?.verified === true) {
      return (
        <span class="relative inline-flex text-green-400" title="Hash verified">
          <span>✓</span>
          <span class="absolute left-[3px]">✓</span>
        </span>
      );
    }
    if (totalHashCount() > 0) {
      return (
        <span class="text-[10px] text-txt-secondary" title={`${totalHashCount()} hash(es) available`}>
          #{totalHashCount()}
        </span>
      );
    }
    return null;
  };
  
  return (
    <div
      class={rowClasses()}
      onClick={props.onClick}
      onKeyDown={handleKeyDown}
      onContextMenu={props.onContextMenu}
      role="treeitem"
      aria-expanded={props.isExpanded}
      aria-selected={props.isActive}
      tabIndex={props.isActive ? 0 : -1}
      title={props.path}
      data-tree-item
    >
      {/* Selection checkbox */}
      <Show when={props.onToggleSelection !== undefined}>
        <input
          type="checkbox"
          checked={props.isChecked || false}
          onClick={(e) => {
            e.stopPropagation();
            props.onToggleSelection?.(e);
          }}
          class="w-2.5 h-2.5 accent-accent cursor-pointer shrink-0"
          title={props.isChecked ? "Deselect container" : "Select container"}
        />
      </Show>
      
      {/* Expand indicator */}
      <span class="w-3 flex items-center justify-center shrink-0">
        <ExpandIcon isLoading={props.isLoading} isExpanded={props.isExpanded} />
      </span>
      
      {/* Container type icon */}
      {getContainerIcon(props.containerType)}
      
      {/* File name */}
      <span class="flex-1 truncate text-[12px] text-txt font-medium">
        {props.name}
      </span>
      
      {/* Segment count */}
      <Show when={props.segmentCount && props.segmentCount > 1}>
        <span class="text-[10px] text-txt-muted">
          {props.segmentCount} parts
        </span>
      </Show>
      
      {/* Incomplete container warning badge */}
      <Show when={props.isIncomplete}>
        <span 
          class="px-1 py-0.5 text-[9px] font-medium text-warning bg-warning/15 rounded"
          title={props.incompleteMessage || "Container has missing segments"}
        >
          ⚠ Incomplete
        </span>
      </Show>
      
      {/* Right-side items - grouped with minimal spacing */}
      <span class="flex items-center gap-0.5 shrink-0 ml-1">
        {/* Hash indicator */}
        <Show when={hashIndicator()}>
          {hashIndicator()}
        </Show>
        
        {/* Hash button */}
        <Show when={props.onHash !== undefined && !isHashing()}>
          <button
            onClick={(e) => {
              e.stopPropagation();
              props.onHash?.(e);
            }}
            disabled={props.busy}
            class="p-0.5 rounded text-txt-muted hover:text-accent hover:bg-bg-hover/50 transition-colors disabled:opacity-50"
            title="Hash this container"
          >
            <HiOutlineHashtag class="w-3.5 h-3.5" />
          </button>
        </Show>
        
        {/* Status icon */}
        <Show when={props.statusIcon}>
          {props.statusIcon}
        </Show>
      </span>
    </div>
  );
}
