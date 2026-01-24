// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Tree Constants - Styling and configuration for tree components
 * 
 * This file defines the standard look and feel for:
 * - EvidenceTree (main evidence tree)
 * - TreePanel (UFED/AD1 file tree)
 * - VirtualList/VirtualTree (virtualized trees)
 * 
 * Uses direct Tailwind classes for styling consistency.
 */

import { getExtension } from '../../utils';

// =============================================================================
// Tree Layout Constants
// =============================================================================

/** Standard row height for tree items (pixels) */
export const TREE_ROW_HEIGHT = 18;

/** Standard row height for compact mode (pixels) */
export const TREE_ROW_HEIGHT_COMPACT = 16;

/** Standard row height for comfortable mode (pixels) */
export const TREE_ROW_HEIGHT_COMFORTABLE = 22;

/** Indentation per depth level (pixels) */
export const TREE_INDENT_SIZE = 10;

/** Icon size (Tailwind class) */
export const TREE_ICON_SIZE = 'w-3 h-3';

/** Expand icon size (Tailwind class) */
export const TREE_EXPAND_ICON_SIZE = 'w-3 h-3';

// Note: CONTAINER_HEADER_HEIGHT is defined in ui/constants.ts and re-exported here

/** Default overscan for virtual lists */
export const VIRTUAL_LIST_OVERSCAN = 5;

// =============================================================================
// Tree Info Bar Styling
// =============================================================================

/** Info bar classes (used for container summaries) */
export const TREE_INFO_BAR_CLASSES = 'flex items-center gap-1.5 py-1 text-[11px] leading-tight text-txt-muted';

/** Info bar padding (style value for info rows below headers) */
export const TREE_INFO_BAR_PADDING = `${TREE_INDENT_SIZE * 1.5}px`;

// =============================================================================
// Tree Row Classes
// =============================================================================

/** Base row classes (shared by all tree rows) */
export const TREE_ROW_BASE_CLASSES = [
  'flex items-center gap-1',
  'py-px pr-1',
  'text-[11px] leading-tight cursor-pointer',
  'transition-colors duration-100',
  'focus:outline-none',
  'focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:ring-inset',
].join(' ');

/** Row classes when selected */
export const TREE_ROW_SELECTED_CLASSES = 'bg-accent/40 text-white';

/** Row classes when not selected (normal state) */
export const TREE_ROW_NORMAL_CLASSES = 'text-txt-tertiary hover:bg-bg-hover/60';

/** Row classes when active/focused container */
export const TREE_ROW_ACTIVE_CLASSES = 'bg-accent/15 border-l-2 border-accent';

/** Disabled/loading row classes */
export const TREE_ROW_DISABLED_CLASSES = 'opacity-50 cursor-not-allowed';

// =============================================================================
// Tree Text Styling
// =============================================================================

/** Primary text (file/folder names) */
export const TREE_TEXT_PRIMARY = 'text-txt';

/** Secondary text (metadata, sizes) */
export const TREE_TEXT_SECONDARY = 'text-txt-secondary';

/** Muted text (hints, timestamps) */
export const TREE_TEXT_MUTED = 'text-txt-muted';

/** File size text */
export const TREE_TEXT_SIZE = 'text-[11px] leading-tight text-txt-muted tabular-nums';

/** Entry type badge text */
export const TREE_TEXT_BADGE = 'text-[11px] leading-tight px-1 py-px rounded';

// =============================================================================
// File Icon Colors
// =============================================================================

/** Folder icon color */
export const ICON_COLOR_FOLDER = 'text-amber-400';

/** Folder open icon color */
export const ICON_COLOR_FOLDER_OPEN = 'text-amber-500';

/** Default file icon color */
export const ICON_COLOR_FILE = 'text-txt-secondary';

/** Image file icon color */
export const ICON_COLOR_IMAGE = 'text-pink-400';

/** Document file icon color */
export const ICON_COLOR_DOCUMENT = 'text-blue-400';

/** Spreadsheet file icon color */
export const ICON_COLOR_SPREADSHEET = 'text-green-400';

/** Archive file icon color */
export const ICON_COLOR_ARCHIVE = 'text-yellow-400';

/** Database file icon color */
export const ICON_COLOR_DATABASE = 'text-purple-400';

/** Audio file icon color */
export const ICON_COLOR_AUDIO = 'text-accent';

/** Video file icon color */
export const ICON_COLOR_VIDEO = 'text-purple-400';

/** Code file icon color */
export const ICON_COLOR_CODE = 'text-emerald-400';

/** Executable file icon color */
export const ICON_COLOR_EXECUTABLE = 'text-red-400';

/** System file icon color */
export const ICON_COLOR_SYSTEM = 'text-txt-muted';

// =============================================================================
// Container Type Colors
// =============================================================================

/** AD1 container colors */
export const CONTAINER_COLOR_AD1 = {
  icon: 'text-blue-400',
  badge: 'bg-blue-500/20 text-blue-400',
  border: 'border-blue-500/30',
};

/** E01/EnCase container colors */
export const CONTAINER_COLOR_E01 = {
  icon: 'text-green-400',
  badge: 'bg-green-500/20 text-green-400',
  border: 'border-green-500/30',
};

/** L01/Logical evidence container colors */
export const CONTAINER_COLOR_L01 = {
  icon: 'text-yellow-400',
  badge: 'bg-yellow-500/20 text-yellow-400',
  border: 'border-yellow-500/30',
};

/** Raw/DD image container colors */
export const CONTAINER_COLOR_RAW = {
  icon: 'text-purple-400',
  badge: 'bg-purple-500/20 text-purple-400',
  border: 'border-purple-500/30',
};

/** UFED/Cellebrite container colors */
export const CONTAINER_COLOR_UFED = {
  icon: 'text-accent',
  badge: 'bg-accent/20 text-accent',
  border: 'border-accent/30',
};

/** Archive container colors */
export const CONTAINER_COLOR_ARCHIVE = {
  icon: 'text-orange-400',
  badge: 'bg-orange-500/20 text-orange-400',
  border: 'border-orange-500/30',
};

/** Default container colors */
export const CONTAINER_COLOR_DEFAULT = {
  icon: 'text-txt-secondary',
  badge: 'bg-bg-hover text-txt-secondary',
  border: 'border-border',
};

// =============================================================================
// Helper Functions
// =============================================================================

/**
 * Get container type color configuration
 */
export function getContainerColors(type: string) {
  const lower = type.toLowerCase();
  
  if (lower.includes('ad1')) return CONTAINER_COLOR_AD1;
  if (lower.includes('e01') || lower.includes('ewf') || lower.includes('encase')) return CONTAINER_COLOR_E01;
  if (lower.includes('l01') || lower.includes('lx01') || lower.includes('lvf')) return CONTAINER_COLOR_L01;
  if (lower.includes('raw') || lower.includes('dd') || lower.includes('img') || lower.includes('001')) return CONTAINER_COLOR_RAW;
  if (lower.includes('ufed') || lower.includes('ufd')) return CONTAINER_COLOR_UFED;
  if (lower.includes('zip') || lower.includes('7z') || lower.includes('rar') || lower.includes('tar') || lower.includes('archive')) return CONTAINER_COLOR_ARCHIVE;
  
  return CONTAINER_COLOR_DEFAULT;
}

/**
 * Calculate padding-left for a given depth level
 */
export function getTreeIndent(depth: number): string {
  return `${(depth + 1) * TREE_INDENT_SIZE}px`;
}

/**
 * Get combined row classes based on state
 */
export function getTreeRowClasses(options: {
  isSelected?: boolean;
  isActive?: boolean;
  isDisabled?: boolean;
  customClass?: string;
}): string {
  const classes = [TREE_ROW_BASE_CLASSES];
  
  if (options.isDisabled) {
    classes.push(TREE_ROW_DISABLED_CLASSES);
  } else if (options.isSelected) {
    classes.push(TREE_ROW_SELECTED_CLASSES);
  } else if (options.isActive) {
    classes.push(TREE_ROW_ACTIVE_CLASSES);
  } else {
    classes.push(TREE_ROW_NORMAL_CLASSES);
  }
  
  if (options.customClass) {
    classes.push(options.customClass);
  }
  
  return classes.join(' ');
}

/**
 * Get icon color class based on file extension
 */
export function getFileIconColor(name: string): string {
  const ext = getExtension(name);
  
  // Images
  if (['jpg', 'jpeg', 'png', 'gif', 'bmp', 'ico', 'webp', 'svg', 'tiff', 'heic'].includes(ext)) {
    return ICON_COLOR_IMAGE;
  }
  
  // Documents
  if (['doc', 'docx', 'pdf', 'txt', 'rtf', 'md', 'odt'].includes(ext)) {
    return ICON_COLOR_DOCUMENT;
  }
  
  // Spreadsheets
  if (['xls', 'xlsx', 'csv', 'ods'].includes(ext)) {
    return ICON_COLOR_SPREADSHEET;
  }
  
  // Archives
  if (['zip', 'rar', '7z', 'tar', 'gz', 'bz2', 'xz', 'zst'].includes(ext)) {
    return ICON_COLOR_ARCHIVE;
  }
  
  // Databases
  if (['db', 'sqlite', 'sqlite3', 'mdb', 'accdb', 'sql'].includes(ext)) {
    return ICON_COLOR_DATABASE;
  }
  
  // Audio
  if (['mp3', 'wav', 'flac', 'm4a', 'aac', 'ogg', 'wma'].includes(ext)) {
    return ICON_COLOR_AUDIO;
  }
  
  // Video
  if (['mp4', 'avi', 'mkv', 'mov', 'wmv', 'flv', 'webm'].includes(ext)) {
    return ICON_COLOR_VIDEO;
  }
  
  // Code
  if (['js', 'ts', 'jsx', 'tsx', 'py', 'rs', 'go', 'java', 'c', 'cpp', 'h', 'css', 'html', 'json', 'xml', 'yaml', 'yml'].includes(ext)) {
    return ICON_COLOR_CODE;
  }
  
  // Executables
  if (['exe', 'msi', 'app', 'dmg', 'deb', 'rpm'].includes(ext)) {
    return ICON_COLOR_EXECUTABLE;
  }
  
  // System
  if (['dll', 'sys', 'ini', 'cfg', 'conf', 'reg', 'dat'].includes(ext)) {
    return ICON_COLOR_SYSTEM;
  }
  
  return ICON_COLOR_FILE;
}
