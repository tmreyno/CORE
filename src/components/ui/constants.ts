// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * UI Constants - Container utilities and re-exports
 * 
 * This file provides:
 * - Container icon color mapping
 * - Container type detection utilities
 * - Re-exports from src/constants/ui.ts
 */

// =============================================================================
// Re-exports from src/constants/ui.ts
// =============================================================================

export {
  // Heights
  BAR_HEIGHT_SMALL,
  BAR_HEIGHT_BASE,
  BAR_HEIGHT_LG,
  CONTAINER_HEADER_HEIGHT,
  
  // Sidebar sizes
  SIDEBAR_WIDTH,
  SIDEBAR_MIN_WIDTH,
  SIDEBAR_MAX_WIDTH,
  
  // Icon sizes (pixels)
  ICON_SIZE_MICRO,
  ICON_SIZE_COMPACT,
  ICON_SIZE_SMALL,
  ICON_SIZE_BASE,
  ICON_SIZE_LG,
  
  // Z-index
  Z_INDEX,
  
  // Types
  type ContainerType,
  type StatusType,
  type FileCategory,
  type TransferPhase,
  type ShortcutKey,
  type TreeDensity,
  
  // Helper functions
  getContainerType,
  getContainerTextColor,
  getContainerBadgeClass,
  getStatusTextColor,
  getTransferPhaseColor,
  getFileCategory,
  applyTreeDensity,
  
  // Data
  CONTAINER_TEXT_COLORS,
  CONTAINER_BADGE_CLASSES,
  STATUS_TEXT_COLORS,
  TRANSFER_PHASE_COLORS,
  DEFAULT_SHORTCUTS,
  TREE_DENSITY_PRESETS,
} from '../../constants/ui';

// Aliases for backward compatibility
export { BAR_HEIGHT_SMALL as BAR_HEIGHT_COMPACT } from '../../constants/ui';
export { BAR_HEIGHT_LG as BAR_HEIGHT_LARGE } from '../../constants/ui';
export { ICON_SIZE_LG as ICON_SIZE_LARGE } from '../../constants/ui';

// =============================================================================
// Container Icon Utilities
// =============================================================================

/** Container icon colors by type */
export const CONTAINER_ICON_COLORS = {
  ad1: 'text-blue-400',
  e01: 'text-green-400',
  l01: 'text-yellow-400',
  raw: 'text-purple-400',
  ufed: 'text-cyan-400',
  archive: 'text-orange-400',
  default: 'text-txt-secondary',
} as const;

/** Get container icon color based on type string */
export function getContainerIconColor(type: string): string {
  const lower = type.toLowerCase();
  if (lower.includes('ad1')) return CONTAINER_ICON_COLORS.ad1;
  if (lower.includes('e01') || lower.includes('ewf') || lower.includes('encase')) return CONTAINER_ICON_COLORS.e01;
  if (lower.includes('l01') || lower.includes('lx01') || lower.includes('lvf')) return CONTAINER_ICON_COLORS.l01;
  if (lower.includes('raw') || lower.includes('dd') || lower.includes('img') || lower.includes('001')) return CONTAINER_ICON_COLORS.raw;
  if (lower.includes('ufed') || lower.includes('ufd')) return CONTAINER_ICON_COLORS.ufed;
  if (lower.includes('zip') || lower.includes('7z') || lower.includes('rar') || lower.includes('tar') || lower.includes('archive')) return CONTAINER_ICON_COLORS.archive;
  return CONTAINER_ICON_COLORS.default;
}

/** Container icon type enumeration */
export type ContainerIconType = 'ad1' | 'e01' | 'l01' | 'raw' | 'ufed' | 'archive' | 'default';

/** Get container icon type based on type string */
export function getContainerIconType(type: string): ContainerIconType {
  const lower = type.toLowerCase();
  if (lower.includes('ad1')) return 'ad1';
  if (lower.includes('e01') || lower.includes('ewf') || lower.includes('encase')) return 'e01';
  if (lower.includes('l01') || lower.includes('lx01') || lower.includes('lvf')) return 'l01';
  if (lower.includes('raw') || lower.includes('dd') || lower.includes('img') || lower.includes('001')) return 'raw';
  if (lower.includes('ufed') || lower.includes('ufd')) return 'ufed';
  if (lower.includes('zip') || lower.includes('7z') || lower.includes('rar') || lower.includes('tar') || lower.includes('archive')) return 'archive';
  return 'default';
}
