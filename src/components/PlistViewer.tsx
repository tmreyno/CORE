// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * PlistViewer - Apple property list viewer for forensic analysis
 *
 * Renders plist files as searchable, navigable tables showing:
 * - Key paths, value types, and value previews
 * - Format detection (Binary/XML)
 * - Search/filter functionality
 */

import { createSignal, createEffect, Show, For, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineExclamationTriangle,
} from "./icons";
import { SearchIcon, ChevronDownIcon, ChevronRightIcon, CopyIcon } from "./icons";
import { logger } from "../utils/logger";
const log = logger.scope("PlistViewer");

// ============================================================================
// Types (matching Rust structs)
// ============================================================================

interface FlatPlistEntry {
  key_path: string;
  value_type: string;
  value_preview: string;
}

interface PlistInfo {
  path: string;
  format: string;
  root_type: string;
  entry_count: number;
  entries: FlatPlistEntry[];
}

// ============================================================================
// Props
// ============================================================================

interface PlistViewerProps {
  /** Path to the plist file */
  path: string;
  /** Optional class name */
  class?: string;
}

// ============================================================================
// Helpers
// ============================================================================

function getTypeColor(type: string): string {
  if (type === "String") return "text-green-400";
  if (type === "Integer" || type === "Real") return "text-blue-400";
  if (type === "Boolean") return "text-yellow-400";
  if (type === "Date") return "text-purple-400";
  if (type === "Data") return "text-orange-400";
  if (type.startsWith("Array")) return "text-cyan-400";
  if (type.startsWith("Dictionary")) return "text-pink-400";
  return "text-txt-muted";
}

function getDepth(keyPath: string): number {
  return keyPath.split("/").filter(Boolean).length - 1;
}

function getKeyName(keyPath: string): string {
  const parts = keyPath.split("/").filter(Boolean);
  return parts[parts.length - 1] || keyPath;
}

// ============================================================================
// Component
// ============================================================================

export function PlistViewer(props: PlistViewerProps) {
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [plistInfo, setPlistInfo] = createSignal<PlistInfo | null>(null);
  const [searchQuery, setSearchQuery] = createSignal("");
  const [collapsedPaths, setCollapsedPaths] = createSignal<Set<string>>(new Set());
  const [copiedPath, setCopiedPath] = createSignal<string | null>(null);

  const loadPlist = async () => {
    setLoading(true);
    setError(null);

    try {
      const info = await invoke<PlistInfo>("plist_read", { path: props.path });
      setPlistInfo(info);
    } catch (e) {
      log.error("Failed to parse plist:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  createEffect(() => {
    if (props.path) {
      loadPlist();
    }
  });

  const filename = createMemo(() => props.path.split("/").pop() || props.path);

  // Filter entries based on search
  const filteredEntries = createMemo(() => {
    const info = plistInfo();
    if (!info) return [];
    const query = searchQuery().toLowerCase();
    if (!query) return info.entries;
    return info.entries.filter(
      (e) =>
        e.key_path.toLowerCase().includes(query) ||
        e.value_preview.toLowerCase().includes(query) ||
        e.value_type.toLowerCase().includes(query)
    );
  });

  // Visible entries based on collapsed state
  const visibleEntries = createMemo(() => {
    const entries = filteredEntries();
    if (searchQuery()) return entries; // Show all when searching
    const collapsed = collapsedPaths();
    return entries.filter((entry) => {
      // Check if any parent path is collapsed
      const parts = entry.key_path.split("/").filter(Boolean);
      for (let i = 1; i < parts.length; i++) {
        const parentPath = parts.slice(0, i).join("/");
        if (collapsed.has(parentPath)) return false;
      }
      return true;
    });
  });

  const toggleCollapse = (keyPath: string) => {
    const collapsed = new Set(collapsedPaths());
    if (collapsed.has(keyPath)) {
      collapsed.delete(keyPath);
    } else {
      collapsed.add(keyPath);
    }
    setCollapsedPaths(collapsed);
  };

  const isCollapsed = (keyPath: string) => collapsedPaths().has(keyPath);
  const isContainer = (type: string) => type.startsWith("Array") || type.startsWith("Dictionary");

  const copyKeyPath = async (keyPath: string) => {
    try {
      await navigator.clipboard.writeText(keyPath);
      setCopiedPath(keyPath);
      setTimeout(() => setCopiedPath(null), 1500);
    } catch {
      // Clipboard might not be available
    }
  };

  return (
    <div class={`plist-viewer flex flex-col h-full ${props.class || ""}`}>
      {/* Toolbar */}
      <div class="flex items-center gap-2 p-2 border-b border-border bg-bg-secondary">
        <svg class="w-4 h-4 text-accent" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M9 12h6M9 16h6M13 4H6a2 2 0 00-2 2v12a2 2 0 002 2h12a2 2 0 002-2V9l-7-5z" />
        </svg>
        <span class="text-sm font-medium truncate" title={filename()}>{filename()}</span>
        <Show when={plistInfo()}>
          <span class="text-xs text-txt-muted">
            {plistInfo()!.format} · {plistInfo()!.root_type} · {plistInfo()!.entry_count} entries
          </span>
        </Show>
        <div class="flex-1" />
        {/* Search */}
        <div class="relative">
          <SearchIcon class="w-3.5 h-3.5 absolute left-2 top-1/2 -translate-y-1/2 text-txt-muted" />
          <input
            type="text"
            class="input-xs pl-7 w-48"
            placeholder="Search keys/values..."
            value={searchQuery()}
            onInput={(e) => setSearchQuery(e.currentTarget.value)}
          />
        </div>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-auto">
        <Show
          when={!loading()}
          fallback={
            <div class="flex flex-col items-center justify-center h-full gap-2">
              <div class="animate-spin w-8 h-8 border-2 border-accent border-t-transparent rounded-full" />
              <span class="text-txt-muted">Parsing plist...</span>
            </div>
          }
        >
          <Show
            when={!error()}
            fallback={
              <div class="flex flex-col items-center gap-2 text-error p-4">
                <HiOutlineExclamationTriangle class="w-12 h-12" />
                <span class="font-medium">Failed to parse plist</span>
                <span class="text-sm text-txt-muted">{error()}</span>
                <button onClick={loadPlist} class="btn btn-secondary mt-2">Retry</button>
              </div>
            }
          >
            <Show
              when={visibleEntries().length > 0}
              fallback={
                <div class="flex flex-col items-center justify-center h-full gap-2 text-txt-muted">
                  <span>No entries {searchQuery() ? "match your search" : "found"}</span>
                </div>
              }
            >
              <table class="w-full text-xs">
                <thead class="bg-bg-secondary sticky top-0 z-10">
                  <tr>
                    <th class="text-left p-2 text-txt-muted font-medium border-b border-border">Key</th>
                    <th class="text-left p-2 text-txt-muted font-medium border-b border-border w-28">Type</th>
                    <th class="text-left p-2 text-txt-muted font-medium border-b border-border">Value</th>
                  </tr>
                </thead>
                <tbody>
                  <For each={visibleEntries()}>
                    {(entry) => {
                      const depth = getDepth(entry.key_path);
                      const keyName = getKeyName(entry.key_path);
                      const container = isContainer(entry.value_type);
                      const collapsed = isCollapsed(entry.key_path);

                      return (
                        <tr class="border-b border-border/30 hover:bg-bg-hover group">
                          <td class="p-1.5 font-mono">
                            <div class="flex items-center gap-1" style={{ "padding-left": `${depth * 16}px` }}>
                              <Show when={container}>
                                <button
                                  class="p-0.5 rounded hover:bg-bg-active"
                                  onClick={() => toggleCollapse(entry.key_path)}
                                >
                                  <Show when={collapsed} fallback={<ChevronDownIcon class="w-3 h-3 text-txt-muted" />}>
                                    <ChevronRightIcon class="w-3 h-3 text-txt-muted" />
                                  </Show>
                                </button>
                              </Show>
                              <Show when={!container}>
                                <span class="w-4" />
                              </Show>
                              <span class="text-txt">{keyName}</span>
                              <button
                                class="opacity-0 group-hover:opacity-100 p-0.5 rounded hover:bg-bg-active"
                                onClick={() => copyKeyPath(entry.key_path)}
                                title="Copy key path"
                              >
                                <CopyIcon class="w-3 h-3 text-txt-muted" />
                              </button>
                              <Show when={copiedPath() === entry.key_path}>
                                <span class="text-[10px] text-success">Copied!</span>
                              </Show>
                            </div>
                          </td>
                          <td class={`p-1.5 ${getTypeColor(entry.value_type)}`}>
                            {entry.value_type}
                          </td>
                          <td class="p-1.5 text-txt-secondary font-mono truncate max-w-xs" title={entry.value_preview}>
                            {entry.value_preview}
                          </td>
                        </tr>
                      );
                    }}
                  </For>
                </tbody>
              </table>
            </Show>
          </Show>
        </Show>
      </div>

      {/* Status bar */}
      <Show when={plistInfo() && !loading()}>
        <div class="flex items-center gap-3 px-3 py-1 border-t border-border bg-bg-secondary text-[11px] text-txt-muted">
          <span>
            {searchQuery()
              ? `${filteredEntries().length} of ${plistInfo()!.entry_count} entries`
              : `${visibleEntries().length} visible of ${plistInfo()!.entry_count} total`}
          </span>
          <span>Format: {plistInfo()!.format}</span>
        </div>
      </Show>
    </div>
  );
}
