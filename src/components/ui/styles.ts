// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Style Constants - Shared CSS class strings for consistent UI styling
 */

/** Common input styling classes */
export const inputStyles = {
  base: "w-full px-3 py-2.5 bg-surface border border-border/50 rounded-lg transition-all",
  focus: "focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20",
  small: "px-2.5 py-2 text-sm",
  error: "border-error/50 focus:border-error focus:ring-error/20",
  disabled: "opacity-50 cursor-not-allowed",
} as const;

/** Combined input class string */
export const inputClass = `${inputStyles.base} ${inputStyles.focus}`;
export const inputClassSm = `${inputStyles.base} ${inputStyles.focus} ${inputStyles.small}`;

/** Card/panel styling classes */
export const cardStyles = {
  base: "bg-surface/50 border border-border/30 rounded-xl",
  padded: "p-4 bg-surface/50 border border-border/30 rounded-xl",
  large: "p-6 bg-surface/50 border border-border/30 rounded-2xl",
  interactive: "hover:border-accent/30 hover:bg-surface/60 transition-colors cursor-pointer",
} as const;

/** Text styling classes */
export const textStyles = {
  muted: "text-xs text-txt/50",
  mutedSm: "text-sm text-txt/60",
  secondary: "text-sm text-txt/70",
  label: "text-xs text-txt/50 font-medium",
  error: "text-xs text-error",
  success: "text-xs text-success",
} as const;

/** Badge styling classes */
export const badgeStyles = {
  default: "px-2 py-0.5 text-xs rounded-full font-medium",
  accent: "px-2 py-0.5 text-xs rounded-full font-medium bg-accent/10 text-accent",
  success: "px-2 py-0.5 text-xs rounded-full font-medium bg-success/10 text-success",
  warning: "px-2 py-0.5 text-xs rounded-full font-medium bg-warning/10 text-warning",
  error: "px-2 py-0.5 text-xs rounded-full font-medium bg-error/10 text-error",
  info: "px-2 py-0.5 text-xs rounded-full font-medium bg-blue-500/10 text-blue-400",
} as const;

/** Button styling classes */
export const buttonStyles = {
  base: "px-4 py-2 rounded-lg font-medium transition-all",
  primary: "px-4 py-2 rounded-lg font-medium bg-accent text-white hover:bg-accent/90 transition-all",
  secondary: "px-4 py-2 rounded-lg font-medium bg-surface border border-border/50 hover:bg-surface/80 transition-all",
  ghost: "px-4 py-2 rounded-lg font-medium hover:bg-surface/50 transition-all",
  small: "px-3 py-1.5 text-sm rounded-lg font-medium transition-all",
  icon: "p-2 rounded-lg hover:bg-surface/50 transition-colors",
} as const;
