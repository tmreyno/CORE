// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ContainerHeader - Header row for forensic containers in the tree
 * 
 * Shows container type badge, filename, and expansion state.
 */

import { Show } from 'solid-js';
import { ExpandIcon } from './ExpandIcon';
import {
  HiOutlineCircleStack,
  HiOutlineArchiveBox,
  HiOutlineDevicePhoneMobile,
  HiOutlineDocument,
  HiOutlineServerStack,
  HiOutlineCube,
} from '../icons';
import {
  UI_ICON_COMPACT,
  getContainerIconColor,
  getContainerBadgeColor,
  getContainerIconType,
  CONTAINER_HEADER_NAME_CLASSES,
  CONTAINER_HEADER_BADGE_CLASSES,
  CONTAINER_HEADER_SEGMENT_CLASSES,
  CONTAINER_HEADER_EXPAND_CLASSES,
} from '../ui/constants';

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
  /** Click handler */
  onClick: () => void;
  /** Additional status indicator */
  statusIcon?: any;
}

/** Get container type icon */
function getContainerIcon(type: string) {
  const iconClass = `${UI_ICON_COMPACT} shrink-0`;
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

/** Get container type badge color */
function getBadgeClass(type: string): string {
  return getContainerBadgeColor(type);
}

export function ContainerHeader(props: ContainerHeaderProps) {
  const rowClasses = () => {
    const base = [
      'flex items-center gap-1',
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
  
  // Get shortened type for badge
  const badgeType = () => {
    const lower = props.containerType.toLowerCase();
    if (lower.includes('ad1')) return 'AD1';
    if (lower.includes('e01')) return 'E01';
    if (lower.includes('l01')) return 'L01';
    if (lower.includes('raw')) return 'RAW';
    if (lower.includes('ufed')) return 'UFED';
    if (lower.includes('zip')) return 'ZIP';
    if (lower.includes('7z')) return '7Z';
    if (lower.includes('tar')) return 'TAR';
    return props.containerType.toUpperCase().slice(0, 4);
  };
  
  return (
    <div
      class={rowClasses()}
      onClick={props.onClick}
      onKeyDown={handleKeyDown}
      role="treeitem"
      aria-expanded={props.isExpanded}
      aria-selected={props.isActive}
      tabIndex={props.isActive ? 0 : -1}
      title={props.path}
      data-tree-item
    >
      {/* Expand indicator */}
      <span class={CONTAINER_HEADER_EXPAND_CLASSES}>
        <ExpandIcon isLoading={props.isLoading} isExpanded={props.isExpanded} />
      </span>
      
      {/* Container type icon */}
      {getContainerIcon(props.containerType)}
      
      {/* File name */}
      <span class={CONTAINER_HEADER_NAME_CLASSES}>
        {props.name}
      </span>
      
      {/* Container type badge */}
      <span class={`${CONTAINER_HEADER_BADGE_CLASSES} ${getBadgeClass(props.containerType)}`}>
        {badgeType()}
      </span>
      
      {/* Segment count */}
      <Show when={props.segmentCount && props.segmentCount > 1}>
        <span class={CONTAINER_HEADER_SEGMENT_CLASSES}>
          {props.segmentCount} parts
        </span>
      </Show>
      
      {/* Status icon */}
      <Show when={props.statusIcon}>
        {props.statusIcon}
      </Show>
    </div>
  );
}
