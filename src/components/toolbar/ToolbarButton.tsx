// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ToolbarButton - Consistent button component for toolbar actions
 * 
 * Variants:
 * - primary: Main actions (accent background)
 * - secondary: Secondary actions (border + bg)
 * - ghost: Subtle actions (transparent)
 * - icon: Icon-only button
 */

import { Component, Show, JSX } from "solid-js";

export type ToolbarButtonVariant = "primary" | "secondary" | "ghost" | "icon";

interface ToolbarButtonProps {
  variant?: ToolbarButtonVariant;
  onClick: () => void;
  disabled?: boolean;
  title?: string;
  ariaLabel?: string;
  class?: string;
  icon: Component<{ class?: string }>;
  children?: JSX.Element;
  badge?: string | number;
  showBadge?: boolean;
  compact?: boolean;
  highlight?: boolean; // For modified state, warnings, etc.
}

export const ToolbarButton: Component<ToolbarButtonProps> = (props) => {
  const variant = () => props.variant ?? "secondary";
  const compact = () => props.compact ?? false;
  
  // Base button styles
  const baseClass = "flex items-center justify-center gap-1.5 px-3 py-1.5 text-sm font-medium rounded-md transition-all duration-150 disabled:opacity-40 disabled:cursor-not-allowed focus:outline-none focus:ring-2 focus:ring-accent/50 focus:ring-offset-1 focus:ring-offset-bg";
  
  // Variant-specific styles
  const variantClass = () => {
    switch (variant()) {
      case "primary":
        return "bg-accent text-white hover:bg-accent-hover active:scale-[0.98] shadow-sm";
      case "secondary":
        return `bg-bg-secondary text-txt hover:bg-bg-hover active:bg-bg-active border border-border hover:border-border-strong ${
          props.highlight ? "border-warning text-warning" : ""
        }`;
      case "ghost":
        return "bg-transparent text-txt-secondary hover:text-txt hover:bg-bg-hover";
      case "icon":
        return "p-2 bg-transparent text-txt-muted hover:text-txt hover:bg-bg-hover";
    }
  };
  
  const Icon = props.icon;
  const hasBadge = () => props.showBadge && props.badge !== undefined;
  
  return (
    <button
      class={`${baseClass} ${variantClass()} ${props.class || ""} ${hasBadge() ? "relative" : ""}`}
      onClick={props.onClick}
      disabled={props.disabled}
      title={props.title}
      aria-label={props.ariaLabel}
    >
      <Icon class="w-4 h-4" />
      <Show when={!compact() && props.children}>
        {props.children}
      </Show>
      
      {/* Badge for counts, notifications, etc. */}
      <Show when={hasBadge()}>
        <span class="absolute -top-1.5 -right-1.5 flex items-center justify-center min-w-[18px] h-[18px] px-1 text-[10px] font-bold bg-accent text-white rounded-full shadow-sm">
          {props.badge}
        </span>
      </Show>
    </button>
  );
};
