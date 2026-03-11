// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * TabItem Component
 * 
 * Individual tab in the CenterPane tab bar showing:
 * - Tab type icon
 * - Title (truncated)
 * - Optional subtitle (when inactive)
 * - Close button (middle-click or button click)
 */

import { Component, Show } from "solid-js";
import { HiOutlineXMark } from "../icons";
import type { CenterTab } from "./CenterPane";
import { getTabTypeColor } from "./tabHelpers";

interface TabItemProps {
  tab: CenterTab;
  isActive: boolean;
  onSelect: () => void;
  onClose: () => void;
}

export const TabItem: Component<TabItemProps> = (props) => {
  const Icon = () => {
    const IconComponent = props.tab.icon;
    return IconComponent ? (
      <IconComponent class={`w-3.5 h-3.5 shrink-0 ${getTabTypeColor(props.tab.type)}`} />
    ) : null;
  };

  return (
    <div
      class={`flex items-center gap-1.5 px-2.5 py-1 rounded transition-all duration-150 cursor-pointer select-none group ${
        props.isActive
          ? "bg-bg text-txt border border-border shadow-sm"
          : "text-txt-muted hover:text-txt hover:bg-bg-hover"
      }`}
      onClick={props.onSelect}
      onMouseDown={(e) => {
        // Middle mouse button = close tab
        if (e.button === 1 && props.tab.closable !== false) {
          e.preventDefault();
          props.onClose();
        }
      }}
      title={props.tab.subtitle ? `${props.tab.title} — ${props.tab.subtitle}` : props.tab.title}
      role="tab"
      aria-selected={props.isActive}
    >
      <Icon />
      <span class="truncate max-w-[140px] font-medium">{props.tab.title}</span>
      <Show when={props.tab.subtitle && !props.isActive}>
        <span class="text-txt-muted/70 truncate max-w-[60px] text-2xs">
          {props.tab.subtitle}
        </span>
      </Show>
      <Show when={props.tab.closable !== false}>
        <button
          class={`ml-0.5 p-0.5 rounded transition-all ${
            props.isActive 
              ? "hover:bg-bg-hover opacity-60 hover:opacity-100" 
              : "hover:bg-bg-active opacity-0 group-hover:opacity-60 hover:!opacity-100"
          }`}
          onClick={(e) => {
            e.stopPropagation();
            props.onClose();
          }}
          title="Close tab (Middle-click)"
          aria-label={`Close ${props.tab.title}`}
        >
          <HiOutlineXMark class="w-3 h-3" />
        </button>
      </Show>
    </div>
  );
};
