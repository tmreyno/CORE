// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, createMemo, type Accessor } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { getBasename } from "../../utils/pathUtils";
import { logger } from "../../utils/logger";
import type { BinaryMetadataSection } from "../../types/viewerMetadata";
import type { BinaryInfo, BinaryViewerProps, ImportInfo } from "./types";
import { formatBadge, formatHex, formatTimestamp } from "./helpers";

const log = logger.scope("BinaryViewer");

export interface UseBinaryDataReturn {
  loading: Accessor<boolean>;
  error: Accessor<string | null>;
  info: Accessor<BinaryInfo | null>;
  showImports: Accessor<boolean>;
  setShowImports: (v: boolean) => void;
  showExports: Accessor<boolean>;
  setShowExports: (v: boolean) => void;
  showSections: Accessor<boolean>;
  setShowSections: (v: boolean) => void;
  importFilter: Accessor<string>;
  setImportFilter: (v: string) => void;
  filename: Accessor<string>;
  badge: Accessor<{ label: string; color: string } | null>;
  filteredImports: Accessor<ImportInfo[]>;
  totalImportFunctions: Accessor<number>;
  loadBinary: () => Promise<void>;
}

export function useBinaryData(props: BinaryViewerProps): UseBinaryDataReturn {
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [info, setInfo] = createSignal<BinaryInfo | null>(null);
  const [showImports, setShowImports] = createSignal(true);
  const [showExports, setShowExports] = createSignal(false);
  const [showSections, setShowSections] = createSignal(true);
  const [importFilter, setImportFilter] = createSignal("");

  const loadBinary = async () => {
    setLoading(true);
    setError(null);

    try {
      const data = await invoke<BinaryInfo>("binary_analyze", { path: props.path });
      setInfo(data);
    } catch (e) {
      log.error("Failed to analyze binary:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  createEffect(() => {
    if (props.path) {
      loadBinary();
    }
  });

  const filename = createMemo(() => getBasename(props.path) || props.path);
  const badge = createMemo(() => (info() ? formatBadge(info()!.format) : null));

  const filteredImports = createMemo(() => {
    const data = info();
    if (!data) return [];
    const query = importFilter().toLowerCase();
    if (!query) return data.imports;
    return data.imports.filter(
      (imp) =>
        imp.library.toLowerCase().includes(query) ||
        imp.functions.some((f) => f.toLowerCase().includes(query))
    );
  });

  const totalImportFunctions = createMemo(() => {
    const data = info();
    if (!data) return 0;
    return data.imports.reduce((sum, imp) => sum + imp.function_count, 0);
  });

  // Emit metadata section when binary info loads
  createEffect(() => {
    const data = info();
    if (!data || !props.onMetadata) return;
    const section: BinaryMetadataSection = {
      kind: "binary",
      format: data.format,
      architecture: data.architecture,
      entryPoint: data.entry_point !== null ? formatHex(data.entry_point) : undefined,
      sectionCount: data.sections.length,
      importCount: data.imports.reduce((sum, imp) => sum + imp.function_count, 0),
      exportCount: data.exports.length,
      isStripped: data.is_stripped,
      isDynamic: !data.is_stripped,
      subsystem: data.pe_subsystem || undefined,
      compiledDate: data.pe_timestamp !== null ? formatTimestamp(data.pe_timestamp) : undefined,
    };
    props.onMetadata(section);
  });

  return {
    loading,
    error,
    info,
    showImports,
    setShowImports,
    showExports,
    setShowExports,
    showSections,
    setShowSections,
    importFilter,
    setImportFilter,
    filename,
    badge,
    filteredImports,
    totalImportFunctions,
    loadBinary,
  };
}
