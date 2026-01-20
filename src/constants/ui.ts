/**
 * =============================================================================
 * UI Logic Constants
 * =============================================================================
 * This file contains ONLY logic-related constants:
 *   - Numeric values for calculations (bar heights, dimensions)
 *   - Type definitions
 *   - Function helpers
 *   - Status/container mappings
 * 
 * For styling, use:
 *   - Tailwind classes directly in JSX for layout/spacing
 *   - CSS classes from components.css for reusable patterns
 *   - CSS variables from variables.css (via Tailwind or direct)
 * 
 * See: src/styles/variables.css, src/styles/components.css
 * =============================================================================
 */

import { getExtension } from '../utils';

// =============================================================================
// NUMERIC VALUES (for calculations, not styling)
// =============================================================================

/** Bar height in pixels - small (24px) */
export const BAR_HEIGHT_SMALL = 24;

/** Bar height in pixels - base (32px) */
export const BAR_HEIGHT_BASE = 32;

/** Bar height in pixels - large (40px) */
export const BAR_HEIGHT_LG = 40;

/** Container header row height (pixels) */
export const CONTAINER_HEADER_HEIGHT = 20;

/** Settings panel width in pixels */
export const SETTINGS_PANEL_WIDTH = 640;

/** Settings panel max height as viewport percentage */
export const SETTINGS_PANEL_MAX_HEIGHT_VH = 75;

/** Sidebar default width */
export const SIDEBAR_WIDTH = 280;

/** Sidebar min width */
export const SIDEBAR_MIN_WIDTH = 200;

/** Sidebar max width */
export const SIDEBAR_MAX_WIDTH = 400;

// =============================================================================
// ICON SIZE NUMERIC VALUES (for SVG viewBox, calculations)
// =============================================================================

/** Icon size in pixels - micro (12px) */
export const ICON_SIZE_MICRO = 12;

/** Icon size in pixels - compact (14px) */
export const ICON_SIZE_COMPACT = 14;

/** Icon size in pixels - small (16px) */
export const ICON_SIZE_SMALL = 16;

/** Icon size in pixels - base (20px) */
export const ICON_SIZE_BASE = 20;

/** Icon size in pixels - large (24px) */
export const ICON_SIZE_LG = 24;

// =============================================================================
// Z-INDEX SCALE (numeric for programmatic use)
// =============================================================================

export const Z_INDEX = {
  base: 0,
  dropdown: 10,
  sticky: 20,
  fixed: 30,
  modalBackdrop: 40,
  modal: 50,
  popover: 60,
  tooltip: 70,
  notification: 80,
} as const;

// =============================================================================
// STATUS COLORS - CSS class mappings
// =============================================================================

/**
 * Status to CSS text color class mapping.
 * Use: `text-${STATUS_COLORS[status]}` or directly in className
 */
export const STATUS_TEXT_COLORS = {
  success: 'text-green-400',
  warning: 'text-yellow-400',
  error: 'text-red-400',
  info: 'text-blue-400',
  accent: 'text-accent',
  muted: 'text-txt-muted',
  pending: 'text-txt-muted',
  active: 'text-accent',
  cancelled: 'text-amber-400',
  verifying: 'text-purple-400',
} as const;

export type StatusType = keyof typeof STATUS_TEXT_COLORS;

/**
 * Get status text color class
 */
export function getStatusTextColor(status: StatusType): string {
  return STATUS_TEXT_COLORS[status] ?? STATUS_TEXT_COLORS.muted;
}

// =============================================================================
// CONTAINER TYPE MAPPINGS
// =============================================================================

/** Container type identifiers */
export type ContainerType = 'ad1' | 'e01' | 'l01' | 'raw' | 'ufed' | 'archive' | 'default';

/**
 * Container type to CSS text color class mapping
 */
export const CONTAINER_TEXT_COLORS: Record<ContainerType, string> = {
  ad1: 'text-type-ad1',
  e01: 'text-type-e01',
  l01: 'text-type-l01',
  raw: 'text-type-raw',
  ufed: 'text-type-ufed',
  archive: 'text-type-archive',
  default: 'text-txt-secondary',
};

/**
 * Container type to badge CSS class mapping
 */
export const CONTAINER_BADGE_CLASSES: Record<ContainerType, string> = {
  ad1: 'badge-ad1',
  e01: 'badge-e01',
  l01: 'badge-l01',
  raw: 'badge-raw',
  ufed: 'badge-ufed',
  archive: 'badge-archive',
  default: 'badge-muted',
};

/**
 * Detect container type from filename/type string
 */
export function getContainerType(type: string): ContainerType {
  const lower = type.toLowerCase();
  if (lower.includes('ad1')) return 'ad1';
  if (lower.includes('e01') || lower.includes('ewf') || lower.includes('encase')) return 'e01';
  if (lower.includes('l01') || lower.includes('lx01') || lower.includes('lvf')) return 'l01';
  if (lower.includes('raw') || lower.includes('dd') || lower.includes('img') || lower.includes('001')) return 'raw';
  if (lower.includes('ufed') || lower.includes('ufd')) return 'ufed';
  if (lower.includes('zip') || lower.includes('7z') || lower.includes('rar') || lower.includes('tar') || lower.includes('archive')) return 'archive';
  return 'default';
}

/**
 * Get container text color class based on type string
 */
export function getContainerTextColor(type: string): string {
  return CONTAINER_TEXT_COLORS[getContainerType(type)];
}

/**
 * Get container badge class based on type string
 */
export function getContainerBadgeClass(type: string): string {
  return CONTAINER_BADGE_CLASSES[getContainerType(type)];
}

// =============================================================================
// TRANSFER PHASE COLORS
// =============================================================================

export type TransferPhase = 'scanning' | 'copying' | 'verifying' | 'completed' | 'failed' | 'cancelled' | 'pending';

/**
 * Transfer phase to CSS text color class mapping
 */
export const TRANSFER_PHASE_COLORS: Record<TransferPhase, string> = {
  scanning: 'text-blue-400',
  copying: 'text-accent',
  verifying: 'text-purple-400',
  completed: 'text-green-400',
  failed: 'text-red-400',
  cancelled: 'text-amber-400',
  pending: 'text-txt-muted',
};

/**
 * Get transfer phase text color class
 */
export function getTransferPhaseColor(phase: TransferPhase): string {
  return TRANSFER_PHASE_COLORS[phase] ?? TRANSFER_PHASE_COLORS.pending;
}

// =============================================================================
// KEYBOARD SHORTCUTS (logic, not styling)
// =============================================================================

export const DEFAULT_SHORTCUTS = {
  toggleSidebar: 'mod+b',
  toggleDetails: 'mod+d',
  search: 'mod+f',
  settings: 'mod+,',
  refresh: 'mod+r',
  newTab: 'mod+t',
  closeTab: 'mod+w',
  nextTab: 'mod+tab',
  prevTab: 'mod+shift+tab',
} as const;

export type ShortcutKey = keyof typeof DEFAULT_SHORTCUTS;

// =============================================================================
// TREE DENSITY PRESETS (numeric values for density settings)
// =============================================================================

export const TREE_DENSITY_PRESETS = {
  compact: {
    iconSize: 14,
    gap: 2,
    padding: '1px 2px',
  },
  default: {
    iconSize: 16,
    gap: 4,
    padding: '2px 4px',
  },
  comfortable: {
    iconSize: 18,
    gap: 6,
    padding: '4px 6px',
  },
} as const;

export type TreeDensity = keyof typeof TREE_DENSITY_PRESETS;

/**
 * Apply tree density CSS variables to document
 */
export function applyTreeDensity(density: TreeDensity): void {
  const preset = TREE_DENSITY_PRESETS[density];
  document.documentElement.style.setProperty('--tree-icon-size', `${preset.iconSize}px`);
  document.documentElement.style.setProperty('--tree-item-gap', `${preset.gap}px`);
  document.documentElement.style.setProperty('--tree-item-padding', preset.padding);
}

// =============================================================================
// FILE TYPE DETECTION (logic helpers)
// =============================================================================

const IMAGE_EXTENSIONS = new Set(['jpg', 'jpeg', 'png', 'gif', 'bmp', 'webp', 'svg', 'ico', 'tiff', 'tif']);
const VIDEO_EXTENSIONS = new Set(['mp4', 'avi', 'mov', 'mkv', 'webm', 'wmv', 'flv', 'm4v']);
const AUDIO_EXTENSIONS = new Set(['mp3', 'wav', 'flac', 'aac', 'ogg', 'm4a', 'wma']);
const DOCUMENT_EXTENSIONS = new Set(['pdf', 'doc', 'docx', 'xls', 'xlsx', 'ppt', 'pptx', 'odt', 'ods', 'odp']);
const ARCHIVE_EXTENSIONS = new Set(['zip', '7z', 'rar', 'tar', 'gz', 'bz2', 'xz']);
const CODE_EXTENSIONS = new Set(['js', 'ts', 'jsx', 'tsx', 'py', 'java', 'c', 'cpp', 'h', 'cs', 'go', 'rs', 'rb', 'php']);

export type FileCategory = 'image' | 'video' | 'audio' | 'document' | 'archive' | 'code' | 'text' | 'binary';

/**
 * Get file category from extension
 */
export function getFileCategory(filename: string): FileCategory {
  const ext = getExtension(filename);
  if (IMAGE_EXTENSIONS.has(ext)) return 'image';
  if (VIDEO_EXTENSIONS.has(ext)) return 'video';
  if (AUDIO_EXTENSIONS.has(ext)) return 'audio';
  if (DOCUMENT_EXTENSIONS.has(ext)) return 'document';
  if (ARCHIVE_EXTENSIONS.has(ext)) return 'archive';
  if (CODE_EXTENSIONS.has(ext)) return 'code';
  if (['txt', 'md', 'json', 'xml', 'csv', 'log', 'ini', 'cfg', 'yaml', 'yml'].includes(ext)) return 'text';
  return 'binary';
}

