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
  HiOutlinePlusCircle,
} from "../icons";
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

interface DriveTreeBrowserProps {
  /** Called when a drive, folder, or file is selected as a source */
  onSelectSource: (path: string) => void;
  /** Paths already selected (to show visual indicator) */
  selectedPaths: () => Set<string> | string[];
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

  const selectedSet = createMemo(() => {
    const val = props.selectedPaths();
    if (val instanceof Set) return val;
    return new Set(val);
  });

  const externalDrives = createMemo(() => drives().filter(d => !d.isSystemDisk));
  const systemDrives = createMemo(() => drives().filter(d => d.isSystemDisk));

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
      try {
        const entries = await invoke<FsDirEntry[]>("list_directory", { path: dirPath });
        const children = new Map(dirChildren());
        children.set(dirPath, entries);
        setDirChildren(children);
      } catch {
        // Permission denied or inaccessible
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
          class="flex items-center gap-1.5 py-1 px-2 rounded cursor-pointer hover:bg-bg-hover text-xs group"
          classList={{ "bg-accent/10": isSelected() }}
          style={{ "padding-left": `${nodeProps.depth * 14 + 6}px` }}
          onClick={(e) => {
            e.stopPropagation();
            if (nodeProps.entry.isDir) {
              toggleExpand(nodeProps.entry.path);
            }
          }}
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
          <span class="flex-1 truncate text-txt">{nodeProps.entry.name}</span>

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
          class="flex items-center gap-2 py-1.5 px-2 rounded cursor-pointer hover:bg-bg-hover group"
          classList={{
            "opacity-60": driveProps.dimmed,
            "bg-accent/10": isSelected(),
          }}
          onClick={() => toggleExpand(drive.mountPoint)}
          title={`${drive.mountPoint} — ${drive.fileSystem} — ${formatDriveSize(drive.totalBytes)}\nClick to browse · Use + to add as source`}
        >
          {/* Expand chevron */}
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

          <Icon class="w-4 h-4 text-blue-400 shrink-0" />
          <div class="flex-1 min-w-0">
            <div class="text-xs text-txt truncate">
              {drive.name || basename(drive.mountPoint)}
              <Show when={drive.isSystemDisk}>
                <span class="ml-1 text-2xs text-warning">(System)</span>
              </Show>
            </div>
            <div class="text-2xs text-txt-muted truncate">
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
  };

  // ── Render ────────────────────────────────────────────────────────────────

  return (
    <div class="space-y-1">
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
          onClick={loadDrives}
          title="Refresh drives"
          disabled={drivesLoading()}
        >
          <HiOutlineArrowPath class="w-3.5 h-3.5" classList={{ "animate-spin": drivesLoading() }} />
        </button>
      </div>

      {/* Drive list */}
      <Show when={expanded()}>
        <div class="border border-border rounded-lg bg-bg-secondary max-h-64 overflow-y-auto">
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
              <div class="p-1 space-y-0.5">
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
    </div>
  );
}
