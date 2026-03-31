// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * DriveTreeBrowser — Inline drive/volume browser with lazy-loaded directory trees.
 *
 * Used in the ExportSourceSection for physical/logical acquisition modes.
 * Shows mounted drives with expandable directory trees for selecting
 * source drives, folders, or files.
 */

import {
  Show,
  For,
  createSignal,
  createMemo,
  onMount,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { CoreSpinner } from "@core-suite/icons";
import {
  HiOutlineCircleStack,
  HiOutlineArrowPath,
  HiOutlineComputerDesktop,
  HiOutlineServer,
  HiOutlineChevronRight,
  HiOutlineChevronDown,
  HiOutlinePlusCircle,
} from "../icons";
import { ExpandIcon } from "../tree/ExpandIcon";
import { TreeIcon } from "../tree/TreeIcon";
import {
  TREE_ROW_BASE_CLASSES,
  TREE_ROW_SELECTED_CLASSES,
  TREE_ROW_NORMAL_CLASSES,
  getTreeIndent,
} from "../tree/constants";
import { createContextMenu, ContextMenu, type ContextMenuItem } from "../ContextMenu";
import type { DriveInfo } from "../../api/drives";
import { listDrives, formatDriveSize } from "../../api/drives";
import { formatBytes } from "../../utils";
import type { ExportMode } from "../../hooks/export/types";

// =============================================================================
// Types
// =============================================================================

/** Matches the Rust DirEntry struct (serde camelCase) */
interface FsDirEntry {
  name: string;
  path: string;
  isDir: boolean;
  size: number;
  modified: number | null;
}

interface DriveTreeBrowserProps {
  /** Called when a drive, folder, or file is selected as a source */
  onSelectSource: (path: string) => void;
  /** Called when a source is selected with a specific acquisition mode (from context menu) */
  onAcquireSource?: (path: string, mode: ExportMode) => void;
  /** Paths already selected (to show visual indicator) */
  selectedPaths: () => Set<string> | string[];
  /** When true, the drive list fills available height instead of capping at max-h-64 */
  fillHeight?: boolean;
  /** Pre-fetched drives to avoid redundant scanning. When provided and non-empty, skips the initial list_drives call. */
  initialDrives?: DriveInfo[];
}

// =============================================================================
// Helpers
// =============================================================================

const basename = (path: string): string => {
  const parts = path.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || path;
};

const driveIcon = (drive: DriveInfo) => {
  if (drive.isRemovable) return HiOutlineCircleStack;
  if (drive.isSystemDisk) return HiOutlineComputerDesktop;
  return HiOutlineServer;
};

// =============================================================================
// Component
// =============================================================================

export function DriveTreeBrowser(props: DriveTreeBrowserProps) {
  const [drives, setDrives] = createSignal<DriveInfo[]>([]);
  const [drivesLoading, setDrivesLoading] = createSignal(false);
  const [expanded, setExpanded] = createSignal(true);

  // Tree expansion state
  const [expandedPaths, setExpandedPaths] = createSignal<Set<string>>(new Set());
  const [dirChildren, setDirChildren] = createSignal<Map<string, FsDirEntry[]>>(new Map());
  const [loadingPaths, setLoadingPaths] = createSignal<Set<string>>(new Set());
  const [errorPaths, setErrorPaths] = createSignal<Map<string, string>>(new Map());

  const selectedSet = createMemo(() => {
    const val = props.selectedPaths();
    if (val instanceof Set) return val;
    return new Set(val);
  });

  const externalDrives = createMemo(() => drives().filter(d => !d.isSystemDisk));
  const systemDrives = createMemo(() => drives().filter(d => d.isSystemDisk));

  const contextMenu = createContextMenu();

  // ── Context menu ──────────────────────────────────────────────────────────

  const buildContextMenuItems = (path: string, isDir: boolean): ContextMenuItem[] => {
    const isSelected = selectedSet().has(path);
    const items: ContextMenuItem[] = [
      {
        id: "toggle-select",
        label: isSelected ? "Deselect" : "Select",
        icon: isSelected ? "➖" : "✅",
        onSelect: () => props.onSelectSource(path),
      },
      { id: "sep1", label: "", separator: true },
      {
        id: "acquire-e01",
        label: "Acquire as E01 (Physical)",
        icon: "💿",
        onSelect: () => {
          if (!selectedSet().has(path)) props.onSelectSource(path);
          props.onAcquireSource?.(path, "physical");
        },
      },
      {
        id: "acquire-l01",
        label: "Acquire as L01 (Logical)",
        icon: "📦",
        onSelect: () => {
          if (!selectedSet().has(path)) props.onSelectSource(path);
          props.onAcquireSource?.(path, "logical");
        },
      },
      {
        id: "export-native",
        label: "Export (7z / Copy)",
        icon: "📤",
        onSelect: () => {
          if (!selectedSet().has(path)) props.onSelectSource(path);
          props.onAcquireSource?.(path, "native");
        },
      },
    ];

    if (isDir) {
      items.push(
        { id: "sep2", label: "", separator: true },
        {
          id: "expand",
          label: expandedPaths().has(path) ? "Collapse" : "Expand",
          icon: "📂",
          onSelect: () => toggleExpand(path),
        },
      );
    }

    items.push(
      { id: "sep3", label: "", separator: true },
      {
        id: "copy-path",
        label: "Copy Path",
        icon: "📋",
        onSelect: () => navigator.clipboard.writeText(path),
      },
    );

    return items;
  };

  const handleDriveContextMenu = (drive: DriveInfo, e: MouseEvent) => {
    contextMenu.open(e, buildContextMenuItems(drive.mountPoint, true));
  };

  const handleTreeContextMenu = (entry: FsDirEntry, e: MouseEvent) => {
    e.stopPropagation();
    contextMenu.open(e, buildContextMenuItems(entry.path, entry.isDir));
  };

  // ── Drive loading ─────────────────────────────────────────────────────────

  /** Serialise drive list to detect real changes (mount points + sizes). */
  const driveFingerprint = (list: DriveInfo[]): string =>
    list.map((d) => `${d.mountPoint}|${d.totalBytes}|${d.availableBytes}|${d.isReadOnly}`).sort().join(";");

  const loadDrives = async (background = false) => {
    // Only show the loading spinner on the first fetch
    if (!background) setDrivesLoading(true);
    try {
      const list = await listDrives();
      // Skip reactive update when nothing changed (prevents flicker)
      if (driveFingerprint(list) !== driveFingerprint(drives())) {
        setDrives(list);
      }
    } catch {
      // Silently handle
    } finally {
      if (!background) setDrivesLoading(false);
    }
  };

  onMount(() => {
    // Use pre-fetched drives if available, otherwise fetch fresh
    if (props.initialDrives && props.initialDrives.length > 0) {
      setDrives(props.initialDrives);
    } else {
      loadDrives(false);
    }
  });

  // ── Tree expansion ────────────────────────────────────────────────────────

  const toggleExpand = async (dirPath: string) => {
    const exp = new Set(expandedPaths());
    if (exp.has(dirPath)) {
      exp.delete(dirPath);
      setExpandedPaths(exp);
      return;
    }

    if (!dirChildren().has(dirPath)) {
      const loading = new Set(loadingPaths());
      loading.add(dirPath);
      setLoadingPaths(loading);
      // Clear any previous error for this path
      const prevErrors = new Map(errorPaths());
      prevErrors.delete(dirPath);
      setErrorPaths(prevErrors);
      try {
        const entries = await invoke<FsDirEntry[]>("list_directory", { path: dirPath });
        const children = new Map(dirChildren());
        children.set(dirPath, entries);
        setDirChildren(children);
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        const errors = new Map(errorPaths());
        errors.set(dirPath, msg.toLowerCase().includes("permission") ? "Permission denied" : "Cannot access directory");
        setErrorPaths(errors);
      } finally {
        const l = new Set(loadingPaths());
        l.delete(dirPath);
        setLoadingPaths(l);
      }
    }

    exp.add(dirPath);
    setExpandedPaths(exp);
  };

  // ── Directory tree node (recursive) ───────────────────────────────────────

  const DirTreeNode = (nodeProps: { entry: FsDirEntry; depth: number }) => {
    const isExpanded = () => expandedPaths().has(nodeProps.entry.path);
    const isLoading = () => loadingPaths().has(nodeProps.entry.path);
    const isSelected = () => selectedSet().has(nodeProps.entry.path);
    const children = () => dirChildren().get(nodeProps.entry.path) || [];

    return (
      <>
        <div
          role="treeitem"
          tabIndex={isSelected() ? 0 : -1}
          aria-expanded={nodeProps.entry.isDir ? isExpanded() : undefined}
          aria-selected={isSelected()}
          class={`${TREE_ROW_BASE_CLASSES} group ${isSelected() ? TREE_ROW_SELECTED_CLASSES : TREE_ROW_NORMAL_CLASSES}`}
          style={{ "padding-left": getTreeIndent(nodeProps.depth) }}
          onContextMenu={(e) => handleTreeContextMenu(nodeProps.entry, e)}
          onClick={(e) => {
            e.stopPropagation();
            if (nodeProps.entry.isDir) {
              toggleExpand(nodeProps.entry.path);
            }
          }}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              if (nodeProps.entry.isDir) toggleExpand(nodeProps.entry.path);
            }
          }}
        >
          {/* Expand/collapse indicator */}
          <span
            class="w-5 flex items-center justify-center shrink-0"
            style={{ visibility: nodeProps.entry.isDir ? "visible" : "hidden" }}
          >
            <ExpandIcon isLoading={isLoading()} isExpanded={isExpanded()} />
          </span>

          {/* File/folder icon */}
          <TreeIcon name={nodeProps.entry.name} isDir={nodeProps.entry.isDir} isExpanded={isExpanded()} />

          {/* Name */}
          <span class="flex-1 truncate">{nodeProps.entry.name}</span>

          {/* File size */}
          <Show when={!nodeProps.entry.isDir && nodeProps.entry.size > 0}>
            <span class="text-2xs text-txt-muted tabular-nums shrink-0">
              {formatBytes(nodeProps.entry.size)}
            </span>
          </Show>

          {/* Add button (visible on hover) */}
          <button
            class="icon-btn-sm opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
            onClick={(e) => {
              e.stopPropagation();
              props.onSelectSource(nodeProps.entry.path);
            }}
            title={`Add ${nodeProps.entry.isDir ? "folder" : "file"} as source`}
            aria-label={`Add ${nodeProps.entry.isDir ? "folder" : "file"} ${nodeProps.entry.name} as source`}
          >
            <HiOutlinePlusCircle class="w-3.5 h-3.5 text-accent" />
          </button>
        </div>

        {/* Recursive children */}
        <Show when={nodeProps.entry.isDir && isExpanded()}>
          <Show
            when={children().length > 0}
            fallback={
              <Show when={!isLoading()}>
                <div
                  class="text-compact italic"
                  classList={{
                    "text-error/70": !!errorPaths().get(nodeProps.entry.path),
                    "text-txt-muted": !errorPaths().get(nodeProps.entry.path),
                  }}
                  style={{ "padding-left": `${(nodeProps.depth + 2) * 10 + 20}px` }}
                >
                  {errorPaths().get(nodeProps.entry.path) || "Empty"}
                </div>
              </Show>
            }
          >
            <For each={children()}>
              {(child) => <DirTreeNode entry={child} depth={nodeProps.depth + 1} />}
            </For>
          </Show>
        </Show>
      </>
    );
  };

  // ── Drive row with expandable tree ────────────────────────────────────────

  const DriveRow = (driveProps: { drive: DriveInfo; dimmed?: boolean }) => {
    const { drive } = driveProps;
    const Icon = driveIcon(drive);
    const driveExpanded = () => expandedPaths().has(drive.mountPoint);
    const driveLoading = () => loadingPaths().has(drive.mountPoint);
    const driveChildren = () => dirChildren().get(drive.mountPoint) || [];
    const isSelected = () => selectedSet().has(drive.mountPoint);

    return (
      <>
        <div
          role="treeitem"
          tabIndex={0}
          aria-expanded={driveExpanded()}
          aria-selected={isSelected()}
          class={`flex items-center gap-1 py-1 pr-1 text-compact leading-tight cursor-pointer transition-colors duration-100 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:ring-inset group ${driveProps.dimmed ? 'opacity-60' : ''} ${isSelected() ? TREE_ROW_SELECTED_CLASSES : TREE_ROW_NORMAL_CLASSES}`}
          style={{ "padding-left": getTreeIndent(0) }}
          onClick={() => toggleExpand(drive.mountPoint)}
          onContextMenu={(e) => handleDriveContextMenu(drive, e)}
          onKeyDown={(e) => {
            if (e.key === "Enter" || e.key === " ") {
              e.preventDefault();
              toggleExpand(drive.mountPoint);
            }
          }}
          title={`${drive.mountPoint} — ${drive.fileSystem} — ${formatDriveSize(drive.totalBytes)}\nClick to browse · Right-click for options`}
        >
          {/* Expand chevron */}
          <span class="w-5 flex items-center justify-center shrink-0">
            <ExpandIcon isLoading={driveLoading()} isExpanded={driveExpanded()} />
          </span>

          <Icon class="w-4 h-4 text-blue-400 shrink-0" />
          <div class="flex-1 min-w-0">
            <div class="text-xs text-txt truncate">
              {drive.name || basename(drive.mountPoint)}
              <Show when={drive.isSystemDisk}>
                <span class="ml-1 text-2xs text-warning">(System)</span>
              </Show>
            </div>
            <div class="text-xs text-txt-muted truncate">
              {drive.mountPoint} · {drive.fileSystem.toUpperCase()} · {formatDriveSize(drive.totalBytes)}
              {drive.isRemovable ? " · USB" : ""}
            </div>
          </div>
          <Show when={drive.isReadOnly}>
            <span class="text-2xs text-warning" title="Read-only">RO</span>
          </Show>

          {/* Add drive as source */}
          <button
            class="icon-btn-sm opacity-0 group-hover:opacity-100 transition-opacity shrink-0"
            onClick={(e) => {
              e.stopPropagation();
              props.onSelectSource(drive.mountPoint);
            }}
            title="Add drive as source"
            aria-label={`Add drive ${drive.name || drive.mountPoint} as source`}
          >
            <HiOutlinePlusCircle class="w-3.5 h-3.5 text-accent" />
          </button>
        </div>

        {/* Drive directory tree */}
        <Show when={driveExpanded()}>
          <Show
            when={driveChildren().length > 0}
            fallback={
              <Show when={!driveLoading()}>
                <div
                  class="text-compact italic"
                  classList={{
                    "text-error/70": !!errorPaths().get(drive.mountPoint),
                    "text-txt-muted": !errorPaths().get(drive.mountPoint),
                  }}
                  style={{ "padding-left": `${2 * 10 + 20}px` }}
                >
                  {errorPaths().get(drive.mountPoint) || "Empty"}
                </div>
              </Show>
            }
          >
            <For each={driveChildren()}>
              {(entry) => <DirTreeNode entry={entry} depth={1} />}
            </For>
          </Show>
        </Show>
      </>
    );
  };

  // ── Render ────────────────────────────────────────────────────────────────

  return (
    <div class={props.fillHeight ? "flex flex-col flex-1 min-h-0 gap-1" : "space-y-1"}>
      {/* Section header */}
      <div class="flex items-center justify-between">
        <button
          class="flex items-center gap-1.5 text-xs font-medium text-txt-muted hover:text-txt transition-colors"
          onClick={() => setExpanded(v => !v)}
        >
          <Show
            when={expanded()}
            fallback={<HiOutlineChevronRight class="w-3 h-3" />}
          >
            <HiOutlineChevronDown class="w-3 h-3" />
          </Show>
          <HiOutlineCircleStack class="w-3.5 h-3.5" />
          Drives & Volumes
          <span class="text-2xs text-txt-muted">({drives().length})</span>
        </button>
        <button
          class="icon-btn-sm"
          onClick={() => loadDrives(false)}
          title="Refresh drives"
          aria-label="Refresh drives"
          disabled={drivesLoading()}
        >
          <Show when={drivesLoading()} fallback={<HiOutlineArrowPath class="w-3.5 h-3.5" />}><CoreSpinner size={14} /></Show>
        </button>
      </div>

      {/* Drive list */}
      <Show when={expanded()}>
        <div class={`border border-border rounded-lg bg-bg-secondary overflow-y-auto ${props.fillHeight ? "flex-1 min-h-0" : "max-h-64"}`}>
          <Show
            when={!drivesLoading()}
            fallback={
              <div class="px-3 py-4 text-xs text-txt-muted text-center">
                Scanning drives…
              </div>
            }
          >
            <Show
              when={drives().length > 0}
              fallback={
                <div class="px-3 py-4 text-xs text-txt-muted text-center">
                  No drives detected. Connect a drive and click refresh.
                </div>
              }
            >
              <div class="p-1 space-y-0.5" role="tree" aria-label="Drive browser">
                {/* External / removable drives first */}
                <For each={externalDrives()}>
                  {(drive) => <DriveRow drive={drive} />}
                </For>
                {/* System drives (dimmed) */}
                <For each={systemDrives()}>
                  {(drive) => <DriveRow drive={drive} dimmed />}
                </For>
              </div>
            </Show>
          </Show>
        </div>
      </Show>

      {/* Context menu */}
      <ContextMenu
        items={contextMenu.items()}
        position={contextMenu.position()}
        onClose={contextMenu.close}
      />
    </div>
  );
}
