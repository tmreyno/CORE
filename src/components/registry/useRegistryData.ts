// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useRegistryData — state management hook for the registry hive viewer.
 *
 * Manages hive loading, tree node expansion, key selection, value filtering,
 * and metadata emission.
 */

import { createSignal, createEffect, createMemo, on } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { logger } from "../../utils/logger";
import type { RegistryMetadataSection } from "../../types/viewerMetadata";
import type {
  RegistryHiveInfo,
  RegistrySubkeysResponse,
  RegistryKeyInfo,
  TreeNode,
} from "./types";

const log = logger.scope("RegistryViewer");

export interface UseRegistryDataOptions {
  path: () => string;
  onMetadata?: (section: RegistryMetadataSection) => void;
}

export function useRegistryData(opts: UseRegistryDataOptions) {
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [hiveInfo, setHiveInfo] = createSignal<RegistryHiveInfo | null>(null);
  const [treeNodes, setTreeNodes] = createSignal<TreeNode[]>([]);
  const [selectedPath, setSelectedPath] = createSignal<string>("");
  const [selectedKeyInfo, setSelectedKeyInfo] = createSignal<RegistryKeyInfo | null>(null);
  const [valuesLoading, setValuesLoading] = createSignal(false);
  const [searchQuery, setSearchQuery] = createSignal("");

  // ── Load hive info and root subkeys ──
  const loadHive = async () => {
    setLoading(true);
    setError(null);

    try {
      const info = await invoke<RegistryHiveInfo>("registry_get_info", {
        path: opts.path(),
      });
      setHiveInfo(info);

      const rootSubkeys = await invoke<RegistrySubkeysResponse>("registry_get_subkeys", {
        hivePath: opts.path(),
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

  // ── Toggle expand/collapse a tree node ──
  const toggleNode = async (node: TreeNode) => {
    if (!node.key.hasSubkeys) return;

    if (!node.loaded) {
      try {
        const response = await invoke<RegistrySubkeysResponse>("registry_get_subkeys", {
          hivePath: opts.path(),
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

  // ── Select a key and load its info ──
  const selectKey = async (node: TreeNode) => {
    setSelectedPath(node.key.path);
    setValuesLoading(true);

    try {
      const keyInfo = await invoke<RegistryKeyInfo>("registry_get_key_info", {
        hivePath: opts.path(),
        keyPath: node.key.path,
      });
      setSelectedKeyInfo(keyInfo);
    } catch (e) {
      log.error("Failed to load key info:", e);
    } finally {
      setValuesLoading(false);
    }
  };

  // ── Flatten tree for rendering ──
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

  // ── Filter values by search ──
  const filteredValues = createMemo(() => {
    const info = selectedKeyInfo();
    if (!info) return [];
    const q = searchQuery().toLowerCase();
    if (!q) return info.values;
    return info.values.filter(
      (v) =>
        v.name.toLowerCase().includes(q) ||
        v.data.toLowerCase().includes(q) ||
        v.dataType.toLowerCase().includes(q),
    );
  });

  // ── Load on path change ──
  createEffect(on(() => opts.path(), () => loadHive()));

  // ── Emit metadata ──
  createEffect(() => {
    const info = hiveInfo();
    if (!info || !opts.onMetadata) return;
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
      selectedKeyInfo: keyInfo
        ? {
            subkeyCount: keyInfo.subkeyCount,
            valueCount: keyInfo.valueCount,
            lastModified: keyInfo.timestamp || undefined,
            className: undefined,
          }
        : undefined,
    };
    opts.onMetadata(section);
  });

  return {
    loading,
    error,
    hiveInfo,
    treeNodes,
    selectedPath,
    selectedKeyInfo,
    valuesLoading,
    searchQuery,
    setSearchQuery,
    flatTree,
    filteredValues,
    toggleNode,
    selectKey,
  };
}
