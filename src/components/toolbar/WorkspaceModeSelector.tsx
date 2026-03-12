// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * WorkspaceModeSelector — Compact dropdown for switching workspace modes
 * from the toolbar. Shows the active mode name and allows quick switching.
 */

import { Component, Show, createSignal } from "solid-js";
import { HiOutlineSquares2x2, HiOutlineCheckCircle, HiOutlineChevronDown } from "../icons";
import { WORKSPACE_PRESETS, type WorkspaceModePreset } from "../preferences";
import { DropdownMenu, type DropdownMenuElement } from "./DropdownMenu";

interface WorkspaceModeSelectorProps {
  /** Current workspace mode ID */
  activeModeId: string;
  /** Called when the user selects a mode */
  onModeChange: (modeId: string) => void;
  /** Optional: open settings panel to the workspace tab */
  onOpenSettings?: () => void;
  /** Compact mode (icon only) */
  compact?: boolean;
}

export const WorkspaceModeSelector: Component<WorkspaceModeSelectorProps> = (props) => {
  const [isOpen, setIsOpen] = createSignal(false);

  const activePreset = (): WorkspaceModePreset =>
    WORKSPACE_PRESETS.find(p => p.id === props.activeModeId) ?? WORKSPACE_PRESETS[0];

  const menuItems = (): DropdownMenuElement[] => {
    const items: DropdownMenuElement[] = WORKSPACE_PRESETS
      .filter(p => !p.isCustom)
      .map(preset => ({
        id: preset.id,
        label: preset.name,
        icon: props.activeModeId === preset.id ? HiOutlineCheckCircle : undefined,
        onClick: () => {
          props.onModeChange(preset.id);
          setIsOpen(false);
        },
      }));

    // Add custom mode entry
    items.push(
      { type: "divider" as const },
      {
        id: "custom",
        label: "Custom…",
        icon: props.activeModeId === "custom" ? HiOutlineCheckCircle : undefined,
        onClick: () => {
          if (props.onOpenSettings) {
            props.onOpenSettings();
          } else {
            props.onModeChange("custom");
          }
          setIsOpen(false);
        },
      }
    );

    return items;
  };

  return (
    <div class="relative">
      <button
        class="flex items-center gap-1.5 px-2 py-1.5 text-xs font-medium rounded-md transition-all
          bg-bg-secondary text-txt-secondary hover:text-txt hover:bg-bg-hover
          border border-border hover:border-border-strong
          focus:outline-none focus:ring-2 focus:ring-accent/50 focus:ring-offset-1 focus:ring-offset-bg"
        onClick={() => setIsOpen(!isOpen())}
        title={`Workspace: ${activePreset().name} — ${activePreset().description}`}
        aria-label={`Workspace mode: ${activePreset().name}`}
        aria-expanded={isOpen()}
        aria-haspopup="menu"
      >
        <HiOutlineSquares2x2 class="w-3.5 h-3.5 text-accent" />
        <Show when={!props.compact}>
          <span class="max-w-[100px] truncate">{activePreset().name}</span>
        </Show>
        <HiOutlineChevronDown class={`w-3 h-3 transition-transform ${isOpen() ? "rotate-180" : ""}`} />
      </button>

      <DropdownMenu
        isOpen={isOpen()}
        onClose={() => setIsOpen(false)}
        items={menuItems()}
        width="w-52"
      />
    </div>
  );
};
