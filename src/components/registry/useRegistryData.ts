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
import { logger } from "../../utils/logger";
import { commands, type HashSourceInput } from "../../api/commands";
import type { RegistryMetadataSection } from "../../types/viewerMetadata";
import { isTauri } from "../../utils/platform";
import type {
  RegistryHiveInfo,
  RegistrySubkeysResponse,
  RegistryKeyInfo,
  TreeNode,
} from "./types";

const log = logger.scope("RegistryViewer");
const BROWSER_REGISTRY_VIEW_MESSAGE =
  "Registry evidence viewing is available in the desktop app.";

export interface UseRegistryDataOptions {
  path: () => string;
  source?: () => HashSourceInput | null | undefined;
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
  let hiveGeneration = 0;
  let keyGeneration = 0;
  let treeGeneration = 0;

  // ── Load hive info and root subkeys ──
  const loadHive = async () => {
    const generation = ++hiveGeneration;
    keyGeneration++;
    treeGeneration++;
    setLoading(true);
    setError(null);

    try {
      if (!isTauri) {
        throw new Error(BROWSER_REGISTRY_VIEW_MESSAGE);
      }

      const source = opts.source?.();
      const info = source
        ? await commands.registry.getInfoSource<RegistryHiveInfo>(source)
        : await commands.registry.getInfo<RegistryHiveInfo>(opts.path());
      if (generation !== hiveGeneration) return;
      setHiveInfo(info);

      const rootSubkeys = source
        ? await commands.registry.getSubkeysSource<RegistrySubkeysResponse>(
            source,
            "",
          )
        : await commands.registry.getSubkeys<RegistrySubkeysResponse>(
            opts.path(),
            "",
          );
      if (generation !== hiveGeneration) return;

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
        void selectKey(nodes[0], generation);
      }
    } catch (e) {
      if (generation !== hiveGeneration) return;
      log.error("Failed to load registry hive:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      if (generation === hiveGeneration) {
        setLoading(false);
      }
    }
  };

  // ── Toggle expand/collapse a tree node ──
  const toggleNode = async (node: TreeNode) => {
    if (!node.key.hasSubkeys) return;
    const hiveToken = hiveGeneration;

    if (!node.loaded) {
      const generation = ++treeGeneration;
      try {
        if (!isTauri) {
          throw new Error(BROWSER_REGISTRY_VIEW_MESSAGE);
        }

        const source = opts.source?.();
        const response = source
          ? await commands.registry.getSubkeysSource<RegistrySubkeysResponse>(
              source,
              node.key.path,
            )
          : await commands.registry.getSubkeys<RegistrySubkeysResponse>(
              opts.path(),
              node.key.path,
            );
        if (hiveToken !== hiveGeneration || generation !== treeGeneration) return;

        node.children = response.subkeys.map((sk) => ({
          key: sk,
          children: [],
          loaded: false,
          expanded: false,
          depth: node.depth + 1,
        }));
        node.loaded = true;
      } catch (e) {
        if (hiveToken !== hiveGeneration || generation !== treeGeneration) return;
        log.error("Failed to load subkeys for:", node.key.path, e);
        return;
      }
    }

    if (hiveToken !== hiveGeneration) return;

    node.expanded = !node.expanded;
    // Force reactivity by replacing the tree
    setTreeNodes([...treeNodes()]);
  };

  // ── Select a key and load its info ──
  const selectKey = async (node: TreeNode, hiveToken = hiveGeneration) => {
    const generation = ++keyGeneration;
    setSelectedPath(node.key.path);
    setValuesLoading(true);

    try {
      if (!isTauri) {
        throw new Error(BROWSER_REGISTRY_VIEW_MESSAGE);
      }

      const source = opts.source?.();
      const keyInfo = source
        ? await commands.registry.getKeyInfoSource<RegistryKeyInfo>(
            source,
            node.key.path,
          )
        : await commands.registry.getKeyInfo<RegistryKeyInfo>(
            opts.path(),
            node.key.path,
          );
      if (hiveToken !== hiveGeneration || generation !== keyGeneration) return;
      setSelectedKeyInfo(keyInfo);
    } catch (e) {
      if (hiveToken !== hiveGeneration || generation !== keyGeneration) return;
      log.error("Failed to load key info:", e);
    } finally {
      if (hiveToken === hiveGeneration && generation === keyGeneration) {
        setValuesLoading(false);
      }
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
  createEffect(
    on(
      () => `${opts.path()}|${JSON.stringify(opts.source?.() ?? null)}`,
      () => loadHive(),
    ),
  );

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
