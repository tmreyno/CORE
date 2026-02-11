// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * RegistryViewer - Windows Registry hive viewer for forensic analysis
 *
 * Displays registry hive files with:
 * - Expandable key tree navigation
 * - Values table for selected key
 * - Key metadata (timestamps, counts)
 * - Search/filter functionality
 */

import { createSignal, createEffect, Show, For, createMemo, on } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineExclamationTriangle,
} from "./icons";
import { SearchIcon, ChevronDownIcon, ChevronRightIcon } from "./icons";
import { logger } from "../utils/logger";
import type { RegistryMetadataSection } from "../types/viewerMetadata";
const log = logger.scope("RegistryViewer");

// ============================================================================
// Types (matching Rust structs)
// ============================================================================

interface RegistryKey {
  name: string;
  path: string;
  timestamp: string | null;
  subkeyCount: number;
  valueCount: number;
  hasSubkeys: boolean;
}

interface RegistryValue {
  name: string;
  dataType: string;
  data: string;
  size: number;
}

interface RegistryHiveInfo {
  path: string;
  rootKeyName: string;
  rootKeyPath: string;
  rootTimestamp: string | null;
  totalKeys: number;
  totalValues: number;
  rootSubkeyCount: number;
  rootValueCount: number;
}

interface RegistrySubkeysResponse {
  parentPath: string;
  subkeys: RegistryKey[];
}

interface RegistryKeyInfo {
  name: string;
  path: string;
  prettyPath: string;
  timestamp: string | null;
  subkeyCount: number;
  valueCount: number;
  values: RegistryValue[];
  subkeys: RegistryKey[];
}

// ============================================================================
// Props
// ============================================================================

interface RegistryViewerProps {
  /** Path to the registry hive file */
  path: string;
  /** Optional class name */
  class?: string;
  /** Callback to emit metadata section for right panel */
  onMetadata?: (section: RegistryMetadataSection) => void;
}

// ============================================================================
// Tree Node
// ============================================================================

interface TreeNode {
  key: RegistryKey;
  children: TreeNode[];
  loaded: boolean;
  expanded: boolean;
  depth: number;
}

// ============================================================================
// Helpers
// ============================================================================

function getDataTypeColor(type: string): string {
  switch (type) {
    case "REG_SZ":
    case "REG_EXPAND_SZ":
      return "text-green-400";
    case "REG_DWORD":
    case "REG_QWORD":
    case "REG_DWORD_BIG_ENDIAN":
      return "text-blue-400";
    case "REG_BINARY":
      return "text-orange-400";
    case "REG_MULTI_SZ":
      return "text-purple-400";
    case "REG_NONE":
      return "text-txt-muted";
    default:
      return "text-txt-secondary";
  }
}

function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

// ============================================================================
// Component
// ============================================================================

export function RegistryViewer(props: RegistryViewerProps) {
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [hiveInfo, setHiveInfo] = createSignal<RegistryHiveInfo | null>(null);
  const [treeNodes, setTreeNodes] = createSignal<TreeNode[]>([]);
  const [selectedPath, setSelectedPath] = createSignal<string>("");
  const [selectedKeyInfo, setSelectedKeyInfo] = createSignal<RegistryKeyInfo | null>(null);
  const [valuesLoading, setValuesLoading] = createSignal(false);
  const [searchQuery, setSearchQuery] = createSignal("");

  // Load hive info and root subkeys
  const loadHive = async () => {
    setLoading(true);
    setError(null);

    try {
      const info = await invoke<RegistryHiveInfo>("registry_get_info", { path: props.path });
      setHiveInfo(info);

      // Load root subkeys
      const rootSubkeys = await invoke<RegistrySubkeysResponse>("registry_get_subkeys", {
        hivePath: props.path,
        keyPath: "",
      });

      const nodes: TreeNode[] = rootSubkeys.subkeys.map((sk) => ({
        key: sk,
        children: [],
        loaded: false,
        expanded: false,
        depth: 0,
      }));
      setTreeNodes(nodes);

      // Auto-select root
      if (nodes.length > 0) {
        selectKey(nodes[0]);
      }
    } catch (e) {
      log.error("Failed to load registry hive:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  // Toggle expand/collapse a tree node
  const toggleNode = async (node: TreeNode) => {
    if (!node.key.hasSubkeys) return;

    if (!node.loaded) {
      try {
        const response = await invoke<RegistrySubkeysResponse>("registry_get_subkeys", {
          hivePath: props.path,
          keyPath: node.key.path,
        });

        node.children = response.subkeys.map((sk) => ({
          key: sk,
          children: [],
          loaded: false,
          expanded: false,
          depth: node.depth + 1,
        }));
        node.loaded = true;
      } catch (e) {
        log.error("Failed to load subkeys for:", node.key.path, e);
        return;
      }
    }

    node.expanded = !node.expanded;
    // Force reactivity by replacing the tree
    setTreeNodes([...treeNodes()]);
  };

  // Select a key and load its info
  const selectKey = async (node: TreeNode) => {
    setSelectedPath(node.key.path);
    setValuesLoading(true);

    try {
      const keyInfo = await invoke<RegistryKeyInfo>("registry_get_key_info", {
        hivePath: props.path,
        keyPath: node.key.path,
      });
      setSelectedKeyInfo(keyInfo);
    } catch (e) {
      log.error("Failed to load key info:", e);
    } finally {
      setValuesLoading(false);
    }
  };

  // Flatten tree for rendering
  const flatTree = createMemo((): { node: TreeNode; depth: number }[] => {
    const result: { node: TreeNode; depth: number }[] = [];
    const walk = (nodes: TreeNode[], depth: number) => {
      for (const node of nodes) {
        result.push({ node, depth });
        if (node.expanded && node.children.length > 0) {
          walk(node.children, depth + 1);
        }
      }
    };
    walk(treeNodes(), 0);
    return result;
  });

  // Filter values by search
  const filteredValues = createMemo(() => {
    const info = selectedKeyInfo();
    if (!info) return [];
    const q = searchQuery().toLowerCase();
    if (!q) return info.values;
    return info.values.filter(
      (v) =>
        v.name.toLowerCase().includes(q) ||
        v.data.toLowerCase().includes(q) ||
        v.dataType.toLowerCase().includes(q)
    );
  });

  // Load on mount / path change
  createEffect(on(() => props.path, () => loadHive()));

  // Emit metadata section when hive info or selection changes
  createEffect(() => {
    const info = hiveInfo();
    if (!info || !props.onMetadata) return;
    const keyInfo = selectedKeyInfo();
    const section: RegistryMetadataSection = {
      kind: "registry",
      hiveName: info.rootKeyName,
      hiveType: info.rootKeyName,
      rootKeyName: info.rootKeyName,
      totalKeys: info.totalKeys,
      totalValues: info.totalValues,
      lastModified: info.rootTimestamp || undefined,
      selectedKeyPath: selectedPath() || undefined,
      selectedKeyInfo: keyInfo ? {
        subkeyCount: keyInfo.subkeyCount,
        valueCount: keyInfo.valueCount,
        lastModified: keyInfo.timestamp || undefined,
        className: undefined,
      } : undefined,
    };
    props.onMetadata(section);
  });

  return (
    <div class={`flex flex-col h-full bg-bg ${props.class || ""}`}>
      {/* Loading */}
      <Show when={loading()}>
        <div class="flex flex-col items-center justify-center h-full gap-4">
          <div class="relative w-12 h-12">
            <div class="absolute inset-0 rounded-full border-2 border-border opacity-30" />
            <div class="absolute inset-0 rounded-full border-2 border-accent border-t-transparent animate-spin" />
          </div>
          <p class="text-txt-secondary text-sm">Loading registry hive...</p>
        </div>
      </Show>

      {/* Error */}
      <Show when={error()}>
        <div class="m-4 p-4 rounded-lg bg-error/10 border border-error/30">
          <div class="flex items-start gap-3">
            <HiOutlineExclamationTriangle class="w-5 h-5 text-error shrink-0 mt-0.5" />
            <div>
              <p class="text-error font-medium text-sm">Failed to load registry hive</p>
              <p class="text-txt-muted text-xs mt-1">{error()}</p>
            </div>
          </div>
        </div>
      </Show>

      {/* Main Content */}
      <Show when={!loading() && !error() && hiveInfo()}>
        {/* Header bar */}
        <div class="panel-header gap-3 border-b border-border">
          <span class="badge badge-warning text-xs">Registry</span>
          <span class="text-sm text-txt font-medium truncate">{hiveInfo()!.rootKeyName}</span>
          <span class="text-xs text-txt-muted">
            {hiveInfo()!.totalKeys.toLocaleString()} keys · {hiveInfo()!.totalValues.toLocaleString()} values
          </span>
          <Show when={hiveInfo()!.rootTimestamp}>
            <span class="text-xs text-txt-muted ml-auto">{hiveInfo()!.rootTimestamp}</span>
          </Show>
        </div>

        {/* Split: Tree + Values */}
        <div class="flex flex-1 min-h-0">
          {/* Left: Tree Navigation */}
          <div class="w-64 border-r border-border flex flex-col">
            <div class="p-2 text-xs text-txt-muted font-medium uppercase tracking-wider border-b border-border">
              Registry Keys
            </div>
            <div class="flex-1 overflow-y-auto p-1">
              <For each={flatTree()}>
                {({ node, depth }) => (
                  <div
                    class={`flex items-center gap-1 px-1 py-0.5 rounded cursor-pointer text-xs hover:bg-bg-hover ${
                      selectedPath() === node.key.path ? "bg-accent/20 text-accent" : "text-txt"
                    }`}
                    style={{ "padding-left": `${depth * 16 + 4}px` }}
                    onClick={() => selectKey(node)}
                  >
                    {/* Expand/collapse toggle */}
                    <button
                      class="w-4 h-4 flex items-center justify-center shrink-0"
                      onClick={(e) => {
                        e.stopPropagation();
                        toggleNode(node);
                      }}
                    >
                      <Show when={node.key.hasSubkeys}>
                        <Show when={node.expanded} fallback={<ChevronRightIcon class="w-3 h-3" />}>
                          <ChevronDownIcon class="w-3 h-3" />
                        </Show>
                      </Show>
                    </button>
                    {/* Key icon */}
                    <svg class="w-3.5 h-3.5 shrink-0 text-yellow-500" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                      <path stroke-linecap="round" stroke-linejoin="round" d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z" />
                    </svg>
                    {/* Key name */}
                    <span class="truncate" title={node.key.path}>{node.key.name}</span>
                    {/* Count badge */}
                    <Show when={node.key.subkeyCount > 0}>
                      <span class="text-[10px] text-txt-muted ml-auto">{node.key.subkeyCount}</span>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </div>

          {/* Right: Values Panel */}
          <div class="flex-1 flex flex-col min-w-0">
            {/* Selected key header */}
            <Show when={selectedKeyInfo()}>
              <div class="px-3 py-2 border-b border-border bg-bg-secondary">
                <div class="flex items-center gap-2">
                  <span class="text-sm text-txt font-medium truncate">{selectedKeyInfo()!.prettyPath}</span>
                  <Show when={selectedKeyInfo()!.timestamp}>
                    <span class="text-xs text-txt-muted ml-auto shrink-0">{selectedKeyInfo()!.timestamp}</span>
                  </Show>
                </div>
                <div class="flex items-center gap-3 mt-1">
                  <span class="text-xs text-txt-muted">{selectedKeyInfo()!.subkeyCount} subkeys</span>
                  <span class="text-xs text-txt-muted">{selectedKeyInfo()!.valueCount} values</span>
                </div>
              </div>

              {/* Search bar */}
              <div class="px-3 py-2 border-b border-border">
                <div class="relative">
                  <SearchIcon class="absolute left-2 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-txt-muted" />
                  <input
                    class="input-sm w-full pl-7"
                    placeholder="Filter values..."
                    value={searchQuery()}
                    onInput={(e) => setSearchQuery(e.currentTarget.value)}
                  />
                </div>
              </div>

              {/* Values table */}
              <div class="flex-1 overflow-auto">
                <Show when={valuesLoading()}>
                  <div class="flex items-center justify-center py-8">
                    <div class="w-6 h-6 rounded-full border-2 border-accent border-t-transparent animate-spin" />
                  </div>
                </Show>
                <Show when={!valuesLoading()}>
                  <table class="w-full text-xs">
                    <thead class="sticky top-0 bg-bg-panel z-10">
                      <tr class="text-left text-txt-muted border-b border-border">
                        <th class="px-3 py-1.5 font-medium">Name</th>
                        <th class="px-3 py-1.5 font-medium w-32">Type</th>
                        <th class="px-3 py-1.5 font-medium">Data</th>
                        <th class="px-3 py-1.5 font-medium w-16 text-right">Size</th>
                      </tr>
                    </thead>
                    <tbody>
                      <For each={filteredValues()} fallback={
                        <tr>
                          <td colspan="4" class="px-3 py-4 text-center text-txt-muted italic">
                            {searchQuery() ? "No matching values" : "No values"}
                          </td>
                        </tr>
                      }>
                        {(val) => (
                          <tr class="border-b border-border/50 hover:bg-bg-hover">
                            <td class="px-3 py-1.5 text-txt font-mono truncate max-w-[200px]" title={val.name}>
                              {val.name}
                            </td>
                            <td class={`px-3 py-1.5 font-mono ${getDataTypeColor(val.dataType)}`}>
                              {val.dataType}
                            </td>
                            <td class="px-3 py-1.5 text-txt-secondary font-mono truncate max-w-[400px]" title={val.data}>
                              {val.data}
                            </td>
                            <td class="px-3 py-1.5 text-txt-muted text-right">
                              {formatSize(val.size)}
                            </td>
                          </tr>
                        )}
                      </For>
                    </tbody>
                  </table>
                </Show>
              </div>
            </Show>

            {/* No selection placeholder */}
            <Show when={!selectedKeyInfo() && !valuesLoading()}>
              <div class="flex items-center justify-center h-full text-txt-muted text-sm">
                Select a key from the tree to view its values
              </div>
            </Show>
          </div>
        </div>
      </Show>
    </div>
  );
}
