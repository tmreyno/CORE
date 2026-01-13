// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Global UI Constants - Shared styling and configuration for all components
 * 
 * This file defines the standard look and feel for:
 * - Font sizes, icon sizes, gaps, padding
 * - Bar heights for toolbars, tabs, filters
 * - Button, badge, and common component styling
 * - Tab bars, breadcrumbs, filter bars, panel headers
 * 
 * All components should import from here to maintain consistency.
 */

// =============================================================================
// Font Sizes
// =============================================================================

/** Standard compact font size (Tailwind) - 12px */
export const UI_FONT_COMPACT = 'text-[12px]';

/** Standard small font size (Tailwind) - 12px */
export const UI_FONT_SMALL = 'text-xs';

/** Standard base font size (Tailwind) - 14px */
export const UI_FONT_BASE = 'text-sm';

// =============================================================================
// Icon Sizes
// =============================================================================

/** Standard compact icon size (Tailwind) - matches text-[10px] */
export const UI_ICON_COMPACT = 'w-2.5 h-2.5';

/** Standard small icon size (Tailwind) - matches text-xs */
export const UI_ICON_SMALL = 'w-3 h-3';

/** Standard base icon size (Tailwind) - matches text-sm */
export const UI_ICON_BASE = 'w-4 h-4';

// =============================================================================
// Gaps
// =============================================================================

/** Standard compact gap (Tailwind) */
export const UI_GAP_COMPACT = 'gap-0.5';

/** Standard small gap (Tailwind) */
export const UI_GAP_SMALL = 'gap-1';

/** Standard base gap (Tailwind) */
export const UI_GAP_BASE = 'gap-1.5';

// =============================================================================
// Padding
// =============================================================================

/** Standard compact padding (Tailwind) */
export const UI_PADDING_COMPACT = 'px-1 py-px';

/** Standard small padding (Tailwind) */
export const UI_PADDING_SMALL = 'px-1.5 py-0.5';

/** Standard base padding (Tailwind) */
export const UI_PADDING_BASE = 'px-2 py-1';

// =============================================================================
// Bar Heights
// =============================================================================

/** Standard bar height - compact (pixels) */
export const UI_BAR_HEIGHT_COMPACT = 20;

/** Standard bar height - small (pixels) */
export const UI_BAR_HEIGHT_SMALL = 24;

/** Standard bar height - base (pixels) */
export const UI_BAR_HEIGHT_BASE = 32;

// =============================================================================
// Filter Bar Styling
// =============================================================================

/** Filter bar container classes */
export const FILTER_BAR_CLASSES = `flex flex-wrap items-center ${UI_GAP_COMPACT} px-1.5 py-0.5 border-b border-border bg-bg-secondary/50`;

/** Filter button base classes */
export const FILTER_BUTTON_BASE = `flex items-center ${UI_GAP_COMPACT} ${UI_PADDING_COMPACT} ${UI_FONT_COMPACT} rounded transition-colors`;

/** Filter button active state */
export const FILTER_BUTTON_ACTIVE = 'bg-accent text-white';

/** Filter button inactive state */
export const FILTER_BUTTON_INACTIVE = 'text-txt-secondary hover:bg-bg-hover hover:text-txt';

// =============================================================================
// Tab Bar Styling
// =============================================================================

/** Tab bar container classes */
export const TAB_BAR_CLASSES = `flex items-center justify-between border-b border-border bg-bg/50 h-6 min-h-[${UI_BAR_HEIGHT_SMALL}px]`;

/** Tab item base classes */
export const TAB_ITEM_BASE = `group relative flex items-center ${UI_GAP_COMPACT} px-1.5 py-0.5 ${UI_FONT_COMPACT} cursor-pointer transition-colors select-none border-r border-border/50`;

/** Tab item active state */
export const TAB_ITEM_ACTIVE = 'bg-bg-secondary text-txt';

/** Tab item inactive state */
export const TAB_ITEM_INACTIVE = 'bg-transparent text-txt-secondary hover:bg-bg-secondary/50 hover:text-txt';

// =============================================================================
// Breadcrumb Styling
// =============================================================================

/** Breadcrumb nav classes */
export const BREADCRUMB_CLASSES = `flex items-center ${UI_GAP_COMPACT} ${UI_FONT_COMPACT} text-txt-secondary min-w-0 overflow-hidden`;

/** Breadcrumb item classes */
export const BREADCRUMB_ITEM_BASE = `flex items-center ${UI_GAP_COMPACT} ${UI_PADDING_COMPACT} rounded transition-colors min-w-0`;

/** Breadcrumb active item */
export const BREADCRUMB_ITEM_ACTIVE = 'text-txt font-medium cursor-default';

/** Breadcrumb clickable item */
export const BREADCRUMB_ITEM_CLICKABLE = 'hover:bg-bg-secondary hover:text-txt';

// =============================================================================
// Panel Tab Styling (Side panel tabs - Evidence/Processed)
// =============================================================================

/** Panel tabs container */
export const PANEL_TABS_CLASSES = `flex ${UI_GAP_COMPACT} px-1.5 py-1 bg-bg border-b border-border shrink-0`;

/** Panel tab base classes */
export const PANEL_TAB_BASE = `flex-1 flex items-center justify-center ${UI_GAP_SMALL} px-2 py-1 ${UI_FONT_COMPACT} font-medium rounded transition-colors cursor-pointer border border-transparent whitespace-nowrap`;

/** Panel tab active state */
export const PANEL_TAB_ACTIVE = 'bg-bg-secondary border-border text-txt';

/** Panel tab inactive state */
export const PANEL_TAB_INACTIVE = 'bg-transparent text-txt-secondary hover:bg-bg-secondary/50 hover:text-txt';

// =============================================================================
// Panel Header Styling
// =============================================================================

/** Panel header container */
export const PANEL_HEADER_CLASSES = `flex items-center justify-between px-2 py-1 border-b border-border/50`;

/** Panel header title */
export const PANEL_HEADER_TITLE = `flex items-center ${UI_GAP_SMALL} ${UI_FONT_COMPACT} font-semibold text-txt-tertiary`;

/** Panel header badge/count */
export const PANEL_HEADER_BADGE = `${UI_FONT_COMPACT} text-txt-muted`;

// =============================================================================
// Button Styling
// =============================================================================

/** Button base - compact */
export const BUTTON_COMPACT = `flex items-center ${UI_GAP_COMPACT} ${UI_PADDING_COMPACT} ${UI_FONT_COMPACT} rounded transition-colors`;

/** Button base - small */
export const BUTTON_SMALL = `flex items-center ${UI_GAP_SMALL} ${UI_PADDING_SMALL} ${UI_FONT_SMALL} rounded transition-colors`;

/** Button primary state */
export const BUTTON_PRIMARY = 'bg-accent text-white hover:bg-accent-hover';

/** Button secondary state */
export const BUTTON_SECONDARY = 'bg-bg-hover text-txt hover:bg-bg-active';

/** Button ghost state */
export const BUTTON_GHOST = 'text-txt-secondary hover:bg-bg-secondary hover:text-txt';

/** Button danger state */
export const BUTTON_DANGER = 'text-red-400 hover:bg-red-500/20';

// =============================================================================
// Badge Styling
// =============================================================================

/** Badge base classes */
export const BADGE_BASE = `${UI_FONT_COMPACT} px-1 py-px rounded font-medium`;

/** Badge info */
export const BADGE_INFO = 'bg-cyan-500/20 text-cyan-400';

/** Badge success */
export const BADGE_SUCCESS = 'bg-green-500/20 text-green-400';

/** Badge warning */
export const BADGE_WARNING = 'bg-yellow-500/20 text-yellow-400';

/** Badge error */
export const BADGE_ERROR = 'bg-red-500/20 text-red-400';

/** Badge muted */
export const BADGE_MUTED = 'bg-bg-hover text-txt-secondary';

// =============================================================================
// Text Colors
// =============================================================================

/** Primary text color */
export const TEXT_PRIMARY = 'text-txt';

/** Secondary text color */
export const TEXT_SECONDARY = 'text-txt-secondary';

/** Muted text color */
export const TEXT_MUTED = 'text-txt-muted';

/** Accent text color */
export const TEXT_ACCENT = 'text-accent';

/** Success text color */
export const TEXT_SUCCESS = 'text-green-400';

/** Warning text color */
export const TEXT_WARNING = 'text-yellow-400';

/** Error text color */
export const TEXT_ERROR = 'text-red-400';

// =============================================================================
// Container Type Styling (forensic container icons/colors)
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

/** Container badge colors by type */
export const CONTAINER_BADGE_COLORS = {
  ad1: 'bg-blue-500/20 text-blue-400',
  e01: 'bg-green-500/20 text-green-400',
  l01: 'bg-yellow-500/20 text-yellow-400',
  raw: 'bg-purple-500/20 text-purple-400',
  ufed: 'bg-cyan-500/20 text-cyan-400',
  archive: 'bg-orange-500/20 text-orange-400',
  default: 'bg-bg-hover text-txt-secondary',
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

/** Get container badge color based on type string */
export function getContainerBadgeColor(type: string): string {
  const lower = type.toLowerCase();
  if (lower.includes('ad1')) return CONTAINER_BADGE_COLORS.ad1;
  if (lower.includes('e01') || lower.includes('ewf')) return CONTAINER_BADGE_COLORS.e01;
  if (lower.includes('l01')) return CONTAINER_BADGE_COLORS.l01;
  if (lower.includes('raw') || lower.includes('dd')) return CONTAINER_BADGE_COLORS.raw;
  if (lower.includes('ufed')) return CONTAINER_BADGE_COLORS.ufed;
  if (lower.includes('zip') || lower.includes('7z') || lower.includes('archive')) return CONTAINER_BADGE_COLORS.archive;
  return CONTAINER_BADGE_COLORS.default;
}

/** Container type to icon name mapping */
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

// =============================================================================
// Container Header Styling
// =============================================================================

/** Container header row height (pixels) */
export const CONTAINER_HEADER_HEIGHT = 20;

/** Container header font size - filename */
export const CONTAINER_HEADER_FONT = 'text-[12px]';

/** Container header badge font size */
export const CONTAINER_HEADER_BADGE_FONT = 'text-[10px]';

/** Container header filename classes */
export const CONTAINER_HEADER_NAME_CLASSES = `flex-1 truncate ${CONTAINER_HEADER_FONT} text-txt font-medium`;

/** Container header badge base classes */
export const CONTAINER_HEADER_BADGE_CLASSES = `${CONTAINER_HEADER_BADGE_FONT} px-1 py-px rounded font-medium`;

/** Container header segment count classes */
export const CONTAINER_HEADER_SEGMENT_CLASSES = `${CONTAINER_HEADER_BADGE_FONT} text-txt-muted`;

/** Container header expand icon wrapper */
export const CONTAINER_HEADER_EXPAND_CLASSES = 'w-3 flex items-center justify-center shrink-0';
