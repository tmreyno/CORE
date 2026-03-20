// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * DriveSourcePanel — Left sidebar panel for browsing drives, volumes, and
 * directories. Users can browse the filesystem, select items, and send them
 * to the Acquire & Export panel via right-click context menu or the "Export"
 * button.
 */

import {
  Component,
  Show,
  For,
  createSignal,
  createMemo,
  onMount,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { open } from "@tauri-apps/plugin-dialog";
import {
  HiOutlineCircleStack,
  HiOutlineArrowPath,
  HiOutlineComputerDesktop,
  HiOutlineServer,
  HiOutlineCheckCircle,
  HiOutlineXMark,
  HiOutlineArrowUpTray,
  HiOutlineDocumentPlus,
  HiOutlineFolderPlus,
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

export interface DriveSourcePanelProps {
  /** Called when user wants to open selected sources in the export panel */
  onExportSources: (paths: string[], mode?: "physical" | "logical" | "native", destination?: string) => void;
  /** Called when a single source is added (auto-send on check) */
  onSourceAdd?: (path: string) => void;
  /** Called when a single source is removed (auto-remove on uncheck) */
  onSourceRemove?: (path: string) => void;
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

const DriveSourcePanel: Component<DriveSourcePanelProps> = (props) => {
  const [drives, setDrives] = createSignal<DriveInfo[]>([]);
  const [drivesLoading, setDrivesLoading] = createSignal(false);
  const [drivesExpanded, setDrivesExpanded] = createSignal(true);

  // Tree expansion state
  const [expandedPaths, setExpandedPaths] = createSignal<Set<string>>(new Set());
  const [dirChildren, setDirChildren] = createSignal<Map<string, FsDirEntry[]>>(new Map());
  const [loadingPaths, setLoadingPaths] = createSignal<Set<string>>(new Set());

  // Selected sources
  const [selectedPaths, setSelectedPaths] = createSignal<Set<string>>(new Set());

  const contextMenu = createContextMenu();

  const selectedCount = createMemo(() => selectedPaths().size);

  // Separate system drives and external/removable drives
  const externalDrives = createMemo(() =>
    drives().filter(d => !d.isSystemDisk),
  );
  const systemDrives = createMemo(() =>
    drives().filter(d => d.isSystemDisk),
  );

  // ── Drive loading ─────────────────────────────────────────────────────────

  const loadDrives = async () => {
    setDrivesLoading(true);
    try {
      const list = await listDrives();
      setDrives(list);
    } catch {
      // Silently handle
    } finally {
      setDrivesLoading(false);
    }
  };

  onMount(() => {
    loadDrives();
  });

  // ── Tree expansion ────────────────────────────────────────────────────────

  const toggleExpand = async (dirPath: string) => {
    const expanded = new Set(expandedPaths());

    if (expanded.has(dirPath)) {
      expanded.delete(dirPath);
      setExpandedPaths(expanded);
      return;
    }

    // Load children if not cached
    if (!dirChildren().has(dirPath)) {
      const loading = new Set(loadingPaths());
      loading.add(dirPath);
      setLoadingPaths(loading);
      try {
        const entries = await invoke<FsDirEntry[]>("list_directory", { path: dirPath });
        const children = new Map(dirChildren());
        children.set(dirPath, entries);
        setDirChildren(children);
      } catch {
        // Permission denied or inaccessible — silently handle
      } finally {
        const l = new Set(loadingPaths());
        l.delete(dirPath);
        setLoadingPaths(l);
      }
    }

    expanded.add(dirPath);
    setExpandedPaths(expanded);
  };

  // ── Selection ─────────────────────────────────────────────────────────────

  const toggleSelect = (path: string) => {
    const next = new Set<string>(selectedPaths());
    if (next.has(path)) {
      next.delete(path);
      props.onSourceRemove?.(path);
    } else {
      next.add(path);
      props.onSourceAdd?.(path);
    }
    setSelectedPaths(next);
  };

  const clearSelection = () => {
    for (const path of selectedPaths()) {
      props.onSourceRemove?.(path);
    }
    setSelectedPaths(new Set<string>());
  };

  // ── File/Folder Dialogs ───────────────────────────────────────────────────

  const handleAddFiles = async () => {
    const selected = await open({
      multiple: true,
      directory: false,
      title: "Select Files",
    });
    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      const next = new Set<string>(selectedPaths());
      for (const p of paths) {
        if (!next.has(p)) {
          next.add(p);
          props.onSourceAdd?.(p);
        }
      }
      setSelectedPaths(next);
    }
  };

  const handleAddFolder = async () => {
    const selected = await open({
      multiple: false,
      directory: true,
      title: "Select Folder",
    });
    if (selected) {
      const path = selected as string;
      const next = new Set<string>(selectedPaths());
      if (!next.has(path)) {
        next.add(path);
        props.onSourceAdd?.(path);
      }
      setSelectedPaths(next);
    }
  };

  // ── Context menu ──────────────────────────────────────────────────────────

  const buildContextMenuItems = (path: string, isDir: boolean): ContextMenuItem[] => {
    const isSelected = selectedPaths().has(path);
    const items: ContextMenuItem[] = [
      {
        id: "toggle-select",
        label: isSelected ? "Deselect" : "Select",
        icon: isSelected ? "➖" : "✅",
        onSelect: () => toggleSelect(path),
      },
      { id: "sep1", label: "", separator: true },
      {
        id: "export-e01",
        label: "Acquire as E01 (Physical)",
        icon: "💿",
        onSelect: () => props.onExportSources([path], "physical"),
      },
      {
        id: "export-l01",
        label: "Acquire as L01 (Logical)",
        icon: "📦",
        onSelect: () => props.onExportSources([path], "logical"),
      },
      {
        id: "export-native",
        label: "Export (7z / Copy)",
        icon: "📤",
        onSelect: () => props.onExportSources([path], "native"),
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
    e.preventDefault();
    contextMenu.open(e, buildContextMenuItems(drive.mountPoint, true));
  };

  const handleTreeContextMenu = (entry: FsDirEntry, e: MouseEvent) => {
    e.preventDefault();
    e.stopPropagation();
    contextMenu.open(e, buildContextMenuItems(entry.path, entry.isDir));
  };

  // ── Export selected ───────────────────────────────────────────────────────

  const handleExportSelected = () => {
    // Items auto-sent on check; just focus the export panel
    props.onExportSources([]);
  };

  // ── Directory tree node (recursive) ───────────────────────────────────────

  const DirTreeNode = (nodeProps: { entry: FsDirEntry; depth: number }) => {
    const isExpanded = () => expandedPaths().has(nodeProps.entry.path);
    const isLoading = () => loadingPaths().has(nodeProps.entry.path);
    const isSelected = () => selectedPaths().has(nodeProps.entry.path);
    const children = () => dirChildren().get(nodeProps.entry.path) || [];

    return (
      <>
        <div
          class={`${TREE_ROW_BASE_CLASSES} ${isSelected() ? TREE_ROW_SELECTED_CLASSES : TREE_ROW_NORMAL_CLASSES}`}
          style={{ "padding-left": getTreeIndent(nodeProps.depth) }}
          onClick={(e) => {
            e.stopPropagation();
            if (nodeProps.entry.isDir) {
              toggleExpand(nodeProps.entry.path);
            } else {
              toggleSelect(nodeProps.entry.path);
            }
          }}
          onContextMenu={(e) => handleTreeContextMenu(nodeProps.entry, e)}
          title={`${nodeProps.entry.path}\nClick ${nodeProps.entry.isDir ? "to expand" : "to select"} · Right-click for options`}
        >
          {/* Expand/collapse indicator */}
          <span
            class="w-5 flex items-center justify-center shrink-0"
            style={{ visibility: nodeProps.entry.isDir ? "visible" : "hidden" }}
          >
            <ExpandIcon isLoading={isLoading()} isExpanded={isExpanded()} />
          </span>

          {/* Selection indicator */}
          <Show when={isSelected()}>
            <HiOutlineCheckCircle class="w-3.5 h-3.5 text-accent shrink-0" />
          </Show>

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
        </div>

        {/* Recursive children */}
        <Show when={nodeProps.entry.isDir && isExpanded()}>
          <Show
            when={children().length > 0}
            fallback={
              <Show when={!isLoading()}>
                <div
                  class="text-compact text-txt-muted italic"
                  style={{ "padding-left": `${(nodeProps.depth + 2) * 10 + 20}px` }}
                >
                  Empty
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

  // ── Render ────────────────────────────────────────────────────────────────

  return (
    <div class="flex flex-col h-full bg-bg">
      {/* Panel header */}
      <div class="flex items-center justify-between px-3 py-2 border-b border-border bg-bg-secondary shrink-0">
        <span class="text-xs font-medium text-txt uppercase tracking-wider">Sources</span>
        <div class="flex items-center gap-1">
          <button
            class="icon-btn-sm"
            onClick={handleAddFiles}
            title="Add files"
          >
            <HiOutlineDocumentPlus class="w-4 h-4" />
          </button>
          <button
            class="icon-btn-sm"
            onClick={handleAddFolder}
            title="Add folder"
          >
            <HiOutlineFolderPlus class="w-4 h-4" />
          </button>
          <button
            class="icon-btn-sm"
            onClick={loadDrives}
            title="Refresh drives"
            disabled={drivesLoading()}
          >
            <HiOutlineArrowPath class="w-4 h-4" classList={{ "animate-spin": drivesLoading() }} />
          </button>
        </div>
      </div>

      {/* Selection bar — shown when items are selected */}
      <Show when={selectedCount() > 0}>
        <div class="flex items-center justify-between px-3 py-1.5 border-b border-border shrink-0"
          style={{ background: "color-mix(in srgb, var(--color-accent) 8%, var(--color-bg-secondary))" }}
        >
          <span class="text-xs text-accent font-medium">
            {selectedCount()} in export
          </span>
          <div class="flex items-center gap-1">
            <button
              class="btn-sm text-xs px-2 py-0.5"
              onClick={handleExportSelected}
              title="View Acquire & Export panel"
            >
              <HiOutlineArrowUpTray class="w-3 h-3 mr-1 inline" />
              View
            </button>
            <button
              class="icon-btn-sm"
              onClick={clearSelection}
              title="Clear selection"
            >
              <HiOutlineXMark class="w-3 h-3" />
            </button>
          </div>
        </div>
      </Show>

      {/* Scrollable content */}
      <div class="flex-1 overflow-y-auto">
        {/* ── Drives Section ─────────────────────────────────── */}
        <div class="border-b border-border/30">
          <button
            class="flex items-center gap-1.5 w-full px-2.5 py-1.5 text-xs font-semibold text-txt-muted uppercase tracking-wider cursor-pointer bg-transparent border-none hover:bg-bg-hover transition-colors select-none"
            onClick={() => setDrivesExpanded(v => !v)}
          >
            <span class="text-2xs transition-transform" classList={{ "rotate-90": drivesExpanded() }}>▶</span>
            <HiOutlineCircleStack class="w-3.5 h-3.5 text-txt-muted" />
            <span class="flex-1 text-left">Drives & Volumes</span>
            <span class="text-2xs text-txt-muted font-normal">{drives().length}</span>
          </button>

          <Show when={drivesExpanded()}>
            <Show
              when={!drivesLoading()}
              fallback={
                <div class="px-3 py-3 text-xs text-txt-muted text-center">
                  Scanning drives…
                </div>
              }
            >
              <Show
                when={drives().length > 0}
                fallback={
                  <div class="px-3 py-3 text-xs text-txt-muted text-center">
                    No drives detected
                  </div>
                }
              >
                <div class="py-0.5">
                  {/* External / removable drives first */}
                  <For each={externalDrives()}>
                    {(drive) => {
                      const Icon = driveIcon(drive);
                      const driveExpanded = () => expandedPaths().has(drive.mountPoint);
                      const driveLoading = () => loadingPaths().has(drive.mountPoint);
                      const driveChildren = () => dirChildren().get(drive.mountPoint) || [];
                      const driveSelected = () => selectedPaths().has(drive.mountPoint);

                      return (
                        <>
                          <button
                            class={`flex items-center gap-1 py-1 pr-1 text-compact leading-tight cursor-pointer transition-colors duration-100 w-full text-left bg-transparent border-none focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:ring-inset ${driveSelected() ? TREE_ROW_SELECTED_CLASSES : TREE_ROW_NORMAL_CLASSES}`}
                            style={{ "padding-left": getTreeIndent(0) }}
                            onContextMenu={(e) => handleDriveContextMenu(drive, e)}
                            onClick={() => toggleExpand(drive.mountPoint)}
                            title={`${drive.mountPoint} — ${drive.fileSystem} — ${formatDriveSize(drive.totalBytes)}\nClick to browse · Right-click for options`}
                          >
                            <span class="w-5 flex items-center justify-center shrink-0">
                              <ExpandIcon isLoading={driveLoading()} isExpanded={driveExpanded()} />
                            </span>

                            <Show when={driveSelected()}>
                              <HiOutlineCheckCircle class="w-3.5 h-3.5 text-accent shrink-0" />
                            </Show>

                            <Icon class="w-4 h-4 text-blue-400 shrink-0" />
                            <div class="flex-1 min-w-0">
                              <div class="truncate">
                                {drive.name || basename(drive.mountPoint)}
                              </div>
                              <div class="text-txt-muted truncate">
                                {drive.mountPoint} · {drive.fileSystem.toUpperCase()} · {formatDriveSize(drive.totalBytes)}
                                {drive.isRemovable ? " · USB" : ""}
                              </div>
                            </div>
                            <Show when={drive.isReadOnly}>
                              <span class="text-2xs text-warning" title="Read-only">RO</span>
                            </Show>
                          </button>

                          {/* Drive directory tree */}
                          <Show when={driveExpanded()}>
                            <Show
                              when={driveChildren().length > 0}
                              fallback={
                                <Show when={!driveLoading()}>
                                  <div
                                    class="text-compact text-txt-muted italic"
                                    style={{ "padding-left": `${2 * 10 + 20}px` }}
                                  >
                                    Empty or inaccessible
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
                    }}
                  </For>

                  {/* System drives (dimmed) */}
                  <For each={systemDrives()}>
                    {(drive) => {
                      const Icon = driveIcon(drive);
                      const driveExpanded = () => expandedPaths().has(drive.mountPoint);
                      const driveLoading = () => loadingPaths().has(drive.mountPoint);
                      const driveChildren = () => dirChildren().get(drive.mountPoint) || [];
                      const driveSelected = () => selectedPaths().has(drive.mountPoint);

                      return (
                        <>
                          <button
                            class={`flex items-center gap-1 py-1 pr-1 text-compact leading-tight cursor-pointer transition-colors duration-100 w-full text-left bg-transparent border-none opacity-60 focus:outline-none focus-visible:ring-2 focus-visible:ring-accent/50 focus-visible:ring-inset ${driveSelected() ? TREE_ROW_SELECTED_CLASSES : TREE_ROW_NORMAL_CLASSES}`}
                            style={{ "padding-left": getTreeIndent(0) }}
                            onContextMenu={(e) => handleDriveContextMenu(drive, e)}
                            onClick={() => toggleExpand(drive.mountPoint)}
                            title={`${drive.mountPoint} (System) — ${drive.fileSystem}\nClick to browse · Right-click for options`}
                          >
                            <span class="w-5 flex items-center justify-center shrink-0">
                              <ExpandIcon isLoading={driveLoading()} isExpanded={driveExpanded()} />
                            </span>

                            <Show when={driveSelected()}>
                              <HiOutlineCheckCircle class="w-3.5 h-3.5 text-accent shrink-0" />
                            </Show>

                            <Icon class="w-4 h-4 text-txt-muted shrink-0" />
                            <div class="flex-1 min-w-0">
                              <div class="truncate">
                                {drive.name || basename(drive.mountPoint)}
                                <span class="ml-1 text-2xs text-warning">(System)</span>
                              </div>
                              <div class="text-txt-muted truncate">
                                {drive.mountPoint} · {drive.fileSystem.toUpperCase()}
                              </div>
                            </div>
                          </button>

                          <Show when={driveExpanded()}>
                            <Show
                              when={driveChildren().length > 0}
                              fallback={
                                <Show when={!driveLoading()}>
                                  <div
                                    class="text-compact text-txt-muted italic"
                                    style={{ "padding-left": `${2 * 10 + 20}px` }}
                                  >
                                    Empty or inaccessible
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
                    }}
                  </For>
                </div>
              </Show>
            </Show>
          </Show>
        </div>

        {/* Empty state when no drives */}
        <Show when={!drivesLoading() && drives().length === 0}>
          <div class="flex flex-col items-center justify-center py-8 text-txt-muted text-sm gap-2">
            <HiOutlineCircleStack class="w-8 h-8 opacity-30" />
            <span>No drives detected</span>
            <button class="btn-text text-xs" onClick={loadDrives}>
              Refresh
            </button>
          </div>
        </Show>

        {/* ── Selected Items Section ────────────────────────── */}
        <Show when={selectedCount() > 0}>
          <div class="border-t border-border/30">
            <div class="flex items-center gap-1.5 px-2.5 py-1.5">
              <HiOutlineCheckCircle class="w-3.5 h-3.5 text-accent shrink-0" />
              <span class="text-xs font-semibold text-txt-muted uppercase tracking-wider flex-1">
                Export Sources
              </span>
              <span class="text-2xs text-txt-muted">{selectedCount()}</span>
            </div>
            <div class="py-0.5">
              <For each={[...selectedPaths()]}>
                {(path) => {
                  const name = basename(path);
                  const isInTree = () => {
                    // Check if this path is a drive mount point or appears in loaded tree
                    return drives().some(d => d.mountPoint === path) ||
                      [...dirChildren().values()].some(children =>
                        children.some(c => c.path === path)
                      );
                  };
                  return (
                    <div
                      class={`${TREE_ROW_BASE_CLASSES} ${TREE_ROW_SELECTED_CLASSES} group`}
                      style={{ "padding-left": getTreeIndent(0) }}
                      title={path}
                    >
                      <TreeIcon name={name} isDir={isInTree()} isExpanded={false} />
                      <div class="flex-1 min-w-0">
                        <div class="truncate">{name}</div>
                        <div class="text-txt-muted truncate">{path}</div>
                      </div>
                      <button
                        class="icon-btn-sm opacity-0 group-hover:opacity-100 shrink-0"
                        onClick={(e) => {
                          e.stopPropagation();
                          toggleSelect(path);
                        }}
                        title="Remove from selection"
                      >
                        <HiOutlineXMark class="w-3 h-3" />
                      </button>
                    </div>
                  );
                }}
              </For>
            </div>
          </div>
        </Show>
      </div>

      {/* Context menu overlay */}
      <ContextMenu
        items={contextMenu.items()}
        position={contextMenu.position()}
        onClose={contextMenu.close}
      />
    </div>
  );
};

export default DriveSourcePanel;
