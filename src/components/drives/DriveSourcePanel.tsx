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
import {
  HiOutlineCircleStack,
  HiOutlineDocument,
  HiOutlineArrowPath,
  HiOutlineComputerDesktop,
  HiOutlineServer,
  HiOutlineChevronRight,
  HiOutlineChevronDown,
  HiOutlineFolder,
  HiOutlineFolderOpen,
  HiOutlineCheckCircle,
  HiOutlineXMark,
  HiOutlineArrowUpTray,
} from "../icons";
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
  onExportSources: (paths: string[], mode?: "physical" | "logical" | "native") => void;
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
    } else {
      next.add(path);
    }
    setSelectedPaths(next);
  };

  const clearSelection = () => {
    setSelectedPaths(new Set<string>());
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
    const paths = [...selectedPaths()];
    if (paths.length > 0) {
      props.onExportSources(paths);
      clearSelection();
    }
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
          class="acquire-tree-row"
          classList={{
            "acquire-tree-selected": isSelected(),
          }}
          style={{ "padding-left": `${nodeProps.depth * 14 + 6}px` }}
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
          {/* Expand chevron for directories */}
          <Show
            when={nodeProps.entry.isDir}
            fallback={<span class="w-4 shrink-0" />}
          >
            <span class="w-4 h-4 flex items-center justify-center shrink-0 text-txt-muted">
              <Show
                when={!isLoading()}
                fallback={
                  <svg class="w-3 h-3 animate-spin" viewBox="0 0 24 24" fill="none">
                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" />
                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                  </svg>
                }
              >
                <Show
                  when={isExpanded()}
                  fallback={<HiOutlineChevronRight class="w-3 h-3" />}
                >
                  <HiOutlineChevronDown class="w-3 h-3" />
                </Show>
              </Show>
            </span>
          </Show>

          {/* Selection indicator */}
          <Show when={isSelected()}>
            <HiOutlineCheckCircle class="w-3.5 h-3.5 text-accent shrink-0" />
          </Show>

          {/* File/folder icon */}
          <Show
            when={nodeProps.entry.isDir}
            fallback={<HiOutlineDocument class="w-3.5 h-3.5 text-txt-secondary shrink-0" />}
          >
            <Show
              when={isExpanded()}
              fallback={<HiOutlineFolder class="w-3.5 h-3.5 text-amber-400 shrink-0" />}
            >
              <HiOutlineFolderOpen class="w-3.5 h-3.5 text-amber-400 shrink-0" />
            </Show>
          </Show>

          {/* Name */}
          <span class="flex-1 truncate text-xs text-txt">{nodeProps.entry.name}</span>

          {/* File size */}
          <Show when={!nodeProps.entry.isDir && nodeProps.entry.size > 0}>
            <span class="text-2xs text-txt-muted tabular-nums shrink-0 mr-1">
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
                  class="text-2xs text-txt-muted italic"
                  style={{ "padding-left": `${(nodeProps.depth + 1) * 14 + 24}px` }}
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
        <span class="text-xs font-medium text-txt uppercase tracking-wider">Drives & Volumes</span>
        <button
          class="icon-btn-sm"
          onClick={loadDrives}
          title="Refresh drives"
          disabled={drivesLoading()}
        >
          <HiOutlineArrowPath class="w-4 h-4" classList={{ "animate-spin": drivesLoading() }} />
        </button>
      </div>

      {/* Selection bar — shown when items are selected */}
      <Show when={selectedCount() > 0}>
        <div class="flex items-center justify-between px-3 py-1.5 border-b border-border shrink-0"
          style={{ background: "color-mix(in srgb, var(--color-accent) 8%, var(--color-bg-secondary))" }}
        >
          <span class="text-2xs text-accent font-medium">
            {selectedCount()} selected
          </span>
          <div class="flex items-center gap-1">
            <button
              class="btn-sm text-2xs px-2 py-0.5"
              onClick={handleExportSelected}
              title="Open selected in Acquire & Export"
            >
              <HiOutlineArrowUpTray class="w-3 h-3 mr-1 inline" />
              Export
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
            class="flex items-center gap-1.5 w-full px-2.5 py-1.5 text-compact font-semibold text-txt-muted uppercase tracking-wider cursor-pointer bg-transparent border-none hover:bg-bg-hover transition-colors select-none"
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
                <div class="px-3 py-3 text-2xs text-txt-muted text-center">
                  Scanning drives…
                </div>
              }
            >
              <Show
                when={drives().length > 0}
                fallback={
                  <div class="px-3 py-3 text-2xs text-txt-muted text-center">
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
                            class="flex items-center gap-2 w-full px-2.5 py-1 text-left cursor-pointer bg-transparent border-none hover:bg-bg-hover transition-colors"
                            classList={{ "acquire-tree-selected": driveSelected() }}
                            onContextMenu={(e) => handleDriveContextMenu(drive, e)}
                            onClick={() => toggleExpand(drive.mountPoint)}
                            title={`${drive.mountPoint} — ${drive.fileSystem} — ${formatDriveSize(drive.totalBytes)}\nClick to browse · Right-click for options`}
                          >
                            <span class="w-4 h-4 flex items-center justify-center shrink-0 text-txt-muted">
                              <Show
                                when={!driveLoading()}
                                fallback={
                                  <svg class="w-3 h-3 animate-spin" viewBox="0 0 24 24" fill="none">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" />
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                                  </svg>
                                }
                              >
                                <Show
                                  when={driveExpanded()}
                                  fallback={<HiOutlineChevronRight class="w-3 h-3" />}
                                >
                                  <HiOutlineChevronDown class="w-3 h-3" />
                                </Show>
                              </Show>
                            </span>

                            <Show when={driveSelected()}>
                              <HiOutlineCheckCircle class="w-3.5 h-3.5 text-accent shrink-0" />
                            </Show>

                            <Icon class="w-4 h-4 text-blue-400 shrink-0" />
                            <div class="flex-1 min-w-0">
                              <div class="text-xs text-txt truncate">
                                {drive.name || basename(drive.mountPoint)}
                              </div>
                              <div class="text-2xs text-txt-muted truncate">
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
                                  <div class="text-2xs text-txt-muted italic px-6 py-1">
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
                            class="flex items-center gap-2 w-full px-2.5 py-1 text-left cursor-pointer bg-transparent border-none hover:bg-bg-hover transition-colors opacity-60"
                            classList={{ "acquire-tree-selected": driveSelected() }}
                            onContextMenu={(e) => handleDriveContextMenu(drive, e)}
                            onClick={() => toggleExpand(drive.mountPoint)}
                            title={`${drive.mountPoint} (System) — ${drive.fileSystem}\nClick to browse · Right-click for options`}
                          >
                            <span class="w-4 h-4 flex items-center justify-center shrink-0 text-txt-muted">
                              <Show
                                when={!driveLoading()}
                                fallback={
                                  <svg class="w-3 h-3 animate-spin" viewBox="0 0 24 24" fill="none">
                                    <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" />
                                    <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z" />
                                  </svg>
                                }
                              >
                                <Show
                                  when={driveExpanded()}
                                  fallback={<HiOutlineChevronRight class="w-3 h-3" />}
                                >
                                  <HiOutlineChevronDown class="w-3 h-3" />
                                </Show>
                              </Show>
                            </span>

                            <Show when={driveSelected()}>
                              <HiOutlineCheckCircle class="w-3.5 h-3.5 text-accent shrink-0" />
                            </Show>

                            <Icon class="w-4 h-4 text-txt-muted shrink-0" />
                            <div class="flex-1 min-w-0">
                              <div class="text-xs text-txt truncate">
                                {drive.name || basename(drive.mountPoint)}
                                <span class="ml-1 text-2xs text-warning">(System)</span>
                              </div>
                              <div class="text-2xs text-txt-muted truncate">
                                {drive.mountPoint} · {drive.fileSystem.toUpperCase()}
                              </div>
                            </div>
                          </button>

                          <Show when={driveExpanded()}>
                            <Show
                              when={driveChildren().length > 0}
                              fallback={
                                <Show when={!driveLoading()}>
                                  <div class="text-2xs text-txt-muted italic px-6 py-1">
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
