// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * DropdownMenu - Reusable dropdown menu component for toolbar
 * 
 * Features:
 * - Click-outside overlay to close
 * - Keyboard navigation support
 * - Menu items with icons and shortcuts
 * - Optional dividers
 */

import { Component, Show, For } from "solid-js";

export interface DropdownMenuItem {
  id: string;
  label: string;
  icon?: Component<{ class?: string }>;
  shortcut?: string;
  onClick: () => void;
  variant?: "default" | "danger";
}

export interface DropdownMenuDivider {
  type: "divider";
}

export type DropdownMenuElement = DropdownMenuItem | DropdownMenuDivider;

interface DropdownMenuProps {
  isOpen: boolean;
  onClose: () => void;
  items: DropdownMenuElement[];
  position?: "left" | "right";
  width?: string;
}

export const DropdownMenu: Component<DropdownMenuProps> = (props) => {
  const position = () => props.position ?? "left";
  const width = () => props.width ?? "w-48";
  
  const positionClass = () => {
    return position() === "left" ? "left-0" : "right-0";
  };
  
  const isDivider = (item: DropdownMenuElement): item is DropdownMenuDivider => {
    return "type" in item && item.type === "divider";
  };
  
  return (
    <Show when={props.isOpen}>
      {/* Click outside overlay to close */}
      <div 
        class="fixed inset-0 z-[9]"
        onClick={() => props.onClose()}
      />
      
      {/* Dropdown menu */}
      <div 
        class={`absolute top-full ${positionClass()} mt-1 ${width()} bg-bg-panel border border-border rounded-lg shadow-lg z-dropdown py-1`}
        role="menu"
      >
        <For each={props.items}>
          {(item) => (
            <Show
              when={!isDivider(item)}
              fallback={
                <div class="h-px bg-border my-1" role="separator" />
              }
            >
              {(() => {
                const menuItem = item as DropdownMenuItem;
                const Icon = menuItem.icon;
                const textColor = menuItem.variant === "danger" ? "text-error" : "text-txt";
                
                return (
                  <button
                    class={`w-full flex items-center gap-2 px-3 py-2 text-sm ${textColor} hover:bg-bg-hover transition-colors text-left`}
                    onClick={() => {
                      props.onClose();
                      menuItem.onClick();
                    }}
                    role="menuitem"
                  >
                    <Show when={Icon} fallback={<div class="w-4 h-4" />}>
                      {Icon && <Icon class="w-4 h-4 text-txt-muted shrink-0" />}
                    </Show>
                    <div class="flex-1">
                      <span>{menuItem.label}</span>
                      <Show when={menuItem.shortcut}>
                        <span class="text-txt-muted text-xs ml-2">{menuItem.shortcut}</span>
                      </Show>
                    </div>
                  </button>
                );
              })()}
            </Show>
          )}
        </For>
      </div>
    </Show>
  );
};
