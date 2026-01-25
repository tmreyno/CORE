// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * QuickActionsBar - Profile-specific quick actions bar
 *
 * Features:
 * - Shows quick actions from the active workspace profile
 * - Displays keyboard shortcuts
 * - Compact horizontal bar below the main toolbar
 * - Customizable per profile
 */

import { Component, For, Show, createMemo } from "solid-js";
import type { QuickAction } from "../hooks/useWorkspaceProfiles";
import {
  HiOutlineBolt,
  HiOutlineFingerPrint,
  HiOutlineMagnifyingGlass,
  HiOutlineDocumentText,
  HiOutlineFolderOpen,
  HiOutlineArrowUpTray,
  HiOutlineArchiveBoxArrowDown,
  HiOutlineCheckBadge,
  HiOutlineEye,
  HiOutlineTag,
  HiOutlineClipboardDocumentList,
  HiOutlineTableCells,
  HiOutlineChartBar,
  HiOutlineDocumentDuplicate,
  HiOutlineDocumentMagnifyingGlass,
  HiOutlineArrowPath,
  HiOutlineCog6Tooth,
  HiOutlineBookmark,
} from "./icons";
import { Kbd } from "./ui/Kbd";

// =============================================================================
// Types
// =============================================================================

export interface QuickActionsBarProps {
  /** Quick actions from the active profile */
  actions?: QuickAction[];
  /** Handler when an action is triggered */
  onAction?: (action: QuickAction) => void;
  /** Whether the bar is visible */
  visible?: boolean;
  /** Compact mode */
  compact?: boolean;
  /** Additional custom actions */
  customActions?: QuickAction[];
  /** Loading state */
  loading?: boolean;
}

// =============================================================================
// Icon Mapping
// =============================================================================

const ICON_MAP: Record<string, Component<{ class?: string }>> = {
  bolt: HiOutlineBolt,
  fingerprint: HiOutlineFingerPrint,
  search: HiOutlineMagnifyingGlass,
  document: HiOutlineDocumentText,
  folder: HiOutlineFolderOpen,
  export: HiOutlineArrowUpTray,
  extract: HiOutlineArchiveBoxArrowDown,
  verify: HiOutlineCheckBadge,
  view: HiOutlineEye,
  tag: HiOutlineTag,
  report: HiOutlineClipboardDocumentList,
  table: HiOutlineTableCells,
  chart: HiOutlineChartBar,
  duplicate: HiOutlineDocumentDuplicate,
  inspect: HiOutlineDocumentMagnifyingGlass,
  refresh: HiOutlineArrowPath,
  settings: HiOutlineCog6Tooth,
  bookmark: HiOutlineBookmark,
};

const getActionIcon = (iconName: string): Component<{ class?: string }> => {
  return ICON_MAP[iconName.toLowerCase()] || HiOutlineBolt;
};

// =============================================================================
// Default Quick Actions
// =============================================================================

const DEFAULT_ACTIONS: QuickAction[] = [
  { id: "hash", name: "Hash Files", icon: "fingerprint", command: "hash_selected", shortcut: "⌘H" },
  { id: "search", name: "Search", icon: "search", command: "open_search", shortcut: "⌘F" },
  { id: "export", name: "Export", icon: "export", command: "export_selected", shortcut: "⌘E" },
  { id: "verify", name: "Verify", icon: "verify", command: "verify_hashes", shortcut: "⌘V" },
  { id: "report", name: "Report", icon: "report", command: "generate_report", shortcut: "⌘P" },
];

// =============================================================================
// QuickActionButton Component
// =============================================================================

interface QuickActionButtonProps {
  action: QuickAction;
  onClick: () => void;
  compact?: boolean;
}

const QuickActionButton: Component<QuickActionButtonProps> = (props) => {
  const IconComp = () => {
    const Icon = getActionIcon(props.action.icon);
    return <Icon class="w-4 h-4" />;
  };

  return (
    <button
      onClick={props.onClick}
      class="flex items-center gap-1.5 px-2.5 py-1 rounded-md text-txt-secondary hover:text-txt hover:bg-bg-hover active:bg-bg-active transition-colors group"
      title={props.action.shortcut ? `${props.action.name} (${props.action.shortcut})` : props.action.name}
    >
      <IconComp />
      <Show when={!props.compact}>
        <span class="text-xs font-medium">{props.action.name}</span>
      </Show>
      <Show when={props.action.shortcut && !props.compact}>
        <Kbd keys={props.action.shortcut} muted size="sm" class="ml-1" />
      </Show>
    </button>
  );
};

// =============================================================================
// QuickActionsBar Component
// =============================================================================

export const QuickActionsBar: Component<QuickActionsBarProps> = (props) => {
  // Combine profile actions with custom actions
  const allActions = createMemo(() => {
    const profileActions = props.actions || [];
    const customActions = props.customActions || [];
    
    // If no actions provided, use defaults
    if (profileActions.length === 0 && customActions.length === 0) {
      return DEFAULT_ACTIONS;
    }
    
    return [...profileActions, ...customActions];
  });

  // Handle action click
  const handleActionClick = (action: QuickAction) => {
    props.onAction?.(action);
  };

  // Don't render if not visible or no actions
  if (props.visible === false || allActions().length === 0) {
    return null;
  }

  return (
    <div class="flex items-center gap-1 px-3 py-1 bg-bg-secondary/50 border-b border-border/50 overflow-x-auto">
      {/* Quick Actions Label */}
      <div class="flex items-center gap-1 pr-2 border-r border-border/30 mr-1">
        <HiOutlineBolt class="w-3.5 h-3.5 text-accent" />
        <Show when={!props.compact}>
          <span class="text-[10px] font-medium text-txt-muted uppercase tracking-wider">
            Quick
          </span>
        </Show>
      </div>

      {/* Loading State */}
      <Show when={props.loading}>
        <div class="flex items-center gap-2 px-2">
          <div class="w-4 h-4 border-2 border-accent/30 border-t-accent rounded-full animate-spin" />
          <span class="text-xs text-txt-muted">Loading...</span>
        </div>
      </Show>

      {/* Actions */}
      <Show when={!props.loading}>
        <div class="flex items-center gap-0.5">
          <For each={allActions()}>
            {(action) => (
              <QuickActionButton
                action={action}
                onClick={() => handleActionClick(action)}
                compact={props.compact}
              />
            )}
          </For>
        </div>
      </Show>

      {/* Spacer */}
      <div class="flex-1" />

      {/* Keyboard Shortcut Hint */}
      <Show when={!props.compact}>
        <div class="text-xs text-txt-muted/60 flex items-center gap-1.5">
          Press <Kbd keys="?" muted size="sm" /> for shortcuts
        </div>
      </Show>
    </div>
  );
};

export default QuickActionsBar;
