// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * SidebarButton — A single icon button in the sidebar with optional badge.
 */

import { type Component, Show } from "solid-js";
import type { JSX } from "solid-js";

export interface SidebarButtonProps {
  active?: boolean;
  disabled?: boolean;
  warning?: boolean;
  badge?: number | string;
  badgeColor?: "accent" | "warning" | "success";
  title: string;
  shortcut?: string;
  onClick?: () => void;
  onContextMenu?: (e: MouseEvent) => void;
  children: JSX.Element;
}

const baseClass =
  "flex items-center justify-center p-1 rounded-md transition-all duration-150 cursor-pointer relative focus:outline-none focus:ring-2 focus:ring-accent/50 group";

export const SidebarButton: Component<SidebarButtonProps> = (props) => {
  const stateClass = () => {
    if (props.disabled) return "opacity-40 cursor-not-allowed";
    if (props.active) return "bg-accent text-white shadow-sm";
    if (props.warning) return "text-warning hover:text-warning hover:bg-warning/10";
    return "text-txt-secondary hover:text-txt hover:bg-bg-hover";
  };

  const badgeColorClass = () => {
    switch (props.badgeColor) {
      case "warning": return "bg-warning text-bg";
      case "success": return "bg-success text-white";
      default: return "bg-accent text-white";
    }
  };

  const fullTitle = () => (props.shortcut ? `${props.title} (${props.shortcut})` : props.title);

  return (
    <button
      class={`${baseClass} ${stateClass()}`}
      onClick={() => { if (!props.disabled) props.onClick?.(); }}
      onContextMenu={props.onContextMenu}
      disabled={props.disabled}
      title={fullTitle()}
      aria-label={props.title}
    >
      {props.children}
      <Show when={props.badge !== undefined && props.badge !== 0}>
        <span
          class={`absolute -top-1 -right-1 flex items-center justify-center min-w-[14px] h-3.5 px-0.5 text-2xs leading-tight font-bold rounded-full animate-pulse ${badgeColorClass()}`}
        >
          {props.badge}
        </span>
      </Show>
    </button>
  );
};
