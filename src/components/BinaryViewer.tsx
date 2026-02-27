// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * BinaryViewer - PE/ELF/Mach-O binary analysis viewer
 *
 * Analyzes executable files and displays:
 * - Format, architecture, entry point
 * - Sections with sizes and characteristics
 * - Import libraries and functions
 * - Export symbols
 * - Forensic indicators (PE timestamps, debug info, code signing)
 */

import { createSignal, createEffect, Show, For, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { getBasename } from "../utils/pathUtils";
import {
  HiOutlineExclamationTriangle,
} from "./icons";
import { ChipIcon, ChevronDownIcon, ChevronRightIcon, WarningIcon, SearchIcon } from "./icons";
import { formatBytes } from "../utils";
import { logger } from "../utils/logger";
import type { BinaryMetadataSection } from "../types/viewerMetadata";
const log = logger.scope("BinaryViewer");

// ============================================================================
// Types (matching Rust structs)
// ============================================================================

interface ImportInfo {
  library: string;
  functions: string[];
  function_count: number;
}

interface ExportInfo {
  name: string;
  ordinal: number | null;
  address: number;
}

interface SectionInfo {
  name: string;
  virtual_address: number;
  virtual_size: number;
  raw_size: number;
  characteristics: string;
}

interface BinaryInfo {
  path: string;
  format: string; // PE32, PE64, ELF32, ELF64, MachO32, MachO64, MachOFat
  architecture: string;
  is_64bit: boolean;
  entry_point: number | null;
  imports: ImportInfo[];
  exports: ExportInfo[];
  sections: SectionInfo[];
  file_size: number;
  // PE specific
  pe_timestamp: number | null;
  pe_checksum: number | null;
  pe_subsystem: string | null;
  // Mach-O specific
  macho_cpu_type: string | null;
  macho_filetype: string | null;
  // Security
  has_debug_info: boolean;
  is_stripped: boolean;
  has_code_signing: boolean;
}

// ============================================================================
// Props
// ============================================================================

interface BinaryViewerProps {
  /** Path to the binary file */
  path: string;
  /** Optional class name */
  class?: string;
  /** Callback to emit metadata section for right panel */
  onMetadata?: (section: BinaryMetadataSection) => void;
}

// ============================================================================
// Helpers
// ============================================================================

function formatHex(value: number | null): string {
  if (value === null) return "N/A";
  return `0x${value.toString(16).toUpperCase()}`;
}

function formatTimestamp(ts: number | null): string {
  if (ts === null) return "N/A";
  try {
    const d = new Date(ts * 1000);
    return d.toISOString().replace("T", " ").replace("Z", " UTC");
  } catch {
    return `0x${ts.toString(16)}`;
  }
}

function formatBadge(format: string): { label: string; color: string } {
  if (format.startsWith("PE")) return { label: format, color: "bg-blue-500/20 text-blue-400 border-blue-500/30" };
  if (format.startsWith("ELF")) return { label: format, color: "bg-green-500/20 text-green-400 border-green-500/30" };
  if (format.startsWith("MachO")) return { label: format, color: "bg-purple-500/20 text-purple-400 border-purple-500/30" };
  return { label: format, color: "bg-bg-secondary text-txt-muted border-border" };
}

// ============================================================================
// Component
// ============================================================================

export function BinaryViewer(props: BinaryViewerProps) {
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
  const badge = createMemo(() => info() ? formatBadge(info()!.format) : null);

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

  // Collapsible section header
  const SectionHeader = (p: { title: string; count?: number; open: boolean; onToggle: () => void }) => (
    <button
      class="flex items-center gap-2 w-full text-left py-1.5 px-2 rounded hover:bg-bg-hover"
      onClick={p.onToggle}
    >
      <Show when={p.open} fallback={<ChevronRightIcon class="w-4 h-4 text-txt-muted" />}>
        <ChevronDownIcon class="w-4 h-4 text-txt-muted" />
      </Show>
      <span class="text-sm font-medium text-txt">{p.title}</span>
      <Show when={p.count !== undefined}>
        <span class="text-xs text-txt-muted">({p.count})</span>
      </Show>
    </button>
  );

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

  return (
    <div class={`binary-viewer flex flex-col h-full ${props.class || ""}`}>
      {/* Toolbar */}
      <div class="flex items-center gap-2 p-2 border-b border-border bg-bg-secondary">
        <ChipIcon class="w-4 h-4 text-accent" />
        <span class="text-sm font-medium truncate" title={filename()}>{filename()}</span>
        <Show when={badge()}>
          <span class={`text-xs px-1.5 py-0.5 rounded border ${badge()!.color}`}>
            {badge()!.label}
          </span>
        </Show>
        <Show when={info()}>
          <span class="text-xs text-txt-muted">{info()!.architecture}</span>
          <span class="text-xs text-txt-muted">{formatBytes(info()!.file_size)}</span>
        </Show>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-y-auto">
        <Show
          when={!loading()}
          fallback={
            <div class="flex flex-col items-center justify-center h-full gap-2">
              <div class="animate-spin w-8 h-8 border-2 border-accent border-t-transparent rounded-full" />
              <span class="text-txt-muted">Analyzing binary...</span>
            </div>
          }
        >
          <Show
            when={!error()}
            fallback={
              <div class="flex flex-col items-center gap-3 text-txt-muted p-6 max-w-md mx-auto text-center">
                <HiOutlineExclamationTriangle class="w-10 h-10 text-warning" />
                <span class="font-medium text-txt">Not a recognized executable</span>
                <p class="text-sm leading-relaxed">
                  <span class="font-mono text-xs bg-bg-secondary px-1.5 py-0.5 rounded">{filename()}</span>{" "}
                  has a binary file extension but is not a PE, ELF, or Mach-O executable.
                  Use the <span class="font-semibold text-txt">Hex</span> button in the toolbar to inspect the raw bytes.
                </p>
                <button onClick={loadBinary} class="btn btn-secondary btn-sm mt-1">Retry Analysis</button>
              </div>
            }
          >
            <Show when={info()}>
              {(data) => (
                <div class="p-3 space-y-3">
                  {/* Overview */}
                  <div class="grid grid-cols-2 gap-3">
                    <div class="stat-box">
                      <div class="text-txt-muted text-xs">Format</div>
                      <div class="text-sm font-semibold text-txt">{data().format}</div>
                    </div>
                    <div class="stat-box">
                      <div class="text-txt-muted text-xs">Architecture</div>
                      <div class="text-sm font-semibold text-txt">{data().architecture} ({data().is_64bit ? "64-bit" : "32-bit"})</div>
                    </div>
                    <div class="stat-box">
                      <div class="text-txt-muted text-xs">Entry Point</div>
                      <div class="text-sm font-mono text-txt">{formatHex(data().entry_point)}</div>
                    </div>
                    <div class="stat-box">
                      <div class="text-txt-muted text-xs">File Size</div>
                      <div class="text-sm font-semibold text-txt">{formatBytes(data().file_size)}</div>
                    </div>
                  </div>

                  {/* PE-specific info */}
                  <Show when={data().pe_timestamp || data().pe_subsystem}>
                    <div class="card">
                      <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-2">PE Information</h3>
                      <Show when={data().pe_timestamp}>
                        <div class="flex gap-2 text-xs py-0.5">
                          <span class="text-txt-muted w-24">Compile Time</span>
                          <span class="text-accent font-mono">{formatTimestamp(data().pe_timestamp)}</span>
                        </div>
                      </Show>
                      <Show when={data().pe_subsystem}>
                        <div class="flex gap-2 text-xs py-0.5">
                          <span class="text-txt-muted w-24">Subsystem</span>
                          <span class="text-txt">{data().pe_subsystem}</span>
                        </div>
                      </Show>
                      <Show when={data().pe_checksum}>
                        <div class="flex gap-2 text-xs py-0.5">
                          <span class="text-txt-muted w-24">Checksum</span>
                          <span class="text-txt font-mono">{formatHex(data().pe_checksum)}</span>
                        </div>
                      </Show>
                    </div>
                  </Show>

                  {/* Mach-O specific */}
                  <Show when={data().macho_cpu_type || data().macho_filetype}>
                    <div class="card">
                      <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-2">Mach-O Information</h3>
                      <Show when={data().macho_cpu_type}>
                        <div class="flex gap-2 text-xs py-0.5">
                          <span class="text-txt-muted w-24">CPU Type</span>
                          <span class="text-txt">{data().macho_cpu_type}</span>
                        </div>
                      </Show>
                      <Show when={data().macho_filetype}>
                        <div class="flex gap-2 text-xs py-0.5">
                          <span class="text-txt-muted w-24">File Type</span>
                          <span class="text-txt">{data().macho_filetype}</span>
                        </div>
                      </Show>
                    </div>
                  </Show>

                  {/* Security Indicators */}
                  <div class="card">
                    <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-2 flex items-center gap-1">
                      <WarningIcon class="w-3 h-3" /> Security Indicators
                    </h3>
                    <div class="grid grid-cols-3 gap-2 text-xs">
                      <div class="flex items-center gap-1.5">
                        <span class={`w-2 h-2 rounded-full ${data().has_debug_info ? "bg-warning" : "bg-bg-hover"}`} />
                        <span class="text-txt-secondary">Debug Info</span>
                      </div>
                      <div class="flex items-center gap-1.5">
                        <span class={`w-2 h-2 rounded-full ${data().is_stripped ? "bg-error" : "bg-bg-hover"}`} />
                        <span class="text-txt-secondary">Stripped</span>
                      </div>
                      <div class="flex items-center gap-1.5">
                        <span class={`w-2 h-2 rounded-full ${data().has_code_signing ? "bg-success" : "bg-bg-hover"}`} />
                        <span class="text-txt-secondary">Code Signed</span>
                      </div>
                    </div>
                  </div>

                  {/* Sections */}
                  <Show when={data().sections.length > 0}>
                    <div>
                      <SectionHeader
                        title="Sections"
                        count={data().sections.length}
                        open={showSections()}
                        onToggle={() => setShowSections(!showSections())}
                      />
                      <Show when={showSections()}>
                        <table class="w-full text-xs mt-1">
                          <thead class="bg-bg-secondary">
                            <tr>
                              <th class="text-left p-1.5 text-txt-muted font-medium">Name</th>
                              <th class="text-left p-1.5 text-txt-muted font-medium">Virtual Addr</th>
                              <th class="text-left p-1.5 text-txt-muted font-medium">Virtual Size</th>
                              <th class="text-left p-1.5 text-txt-muted font-medium">Raw Size</th>
                              <th class="text-left p-1.5 text-txt-muted font-medium">Flags</th>
                            </tr>
                          </thead>
                          <tbody>
                            <For each={data().sections}>
                              {(sec) => (
                                <tr class="border-b border-border/30 hover:bg-bg-hover">
                                  <td class="p-1.5 font-mono text-txt">{sec.name}</td>
                                  <td class="p-1.5 font-mono text-txt-secondary">{formatHex(sec.virtual_address)}</td>
                                  <td class="p-1.5 text-txt-secondary">{formatBytes(sec.virtual_size)}</td>
                                  <td class="p-1.5 text-txt-secondary">{formatBytes(sec.raw_size)}</td>
                                  <td class="p-1.5 font-mono text-txt-muted">{sec.characteristics}</td>
                                </tr>
                              )}
                            </For>
                          </tbody>
                        </table>
                      </Show>
                    </div>
                  </Show>

                  {/* Imports */}
                  <Show when={data().imports.length > 0}>
                    <div>
                      <SectionHeader
                        title="Imports"
                        count={totalImportFunctions()}
                        open={showImports()}
                        onToggle={() => setShowImports(!showImports())}
                      />
                      <Show when={showImports()}>
                        {/* Import filter */}
                        <div class="relative mt-1 mb-2">
                          <SearchIcon class="w-3.5 h-3.5 absolute left-2 top-1/2 -translate-y-1/2 text-txt-muted" />
                          <input
                            type="text"
                            class="input-xs pl-7 w-full"
                            placeholder="Filter imports..."
                            value={importFilter()}
                            onInput={(e) => setImportFilter(e.currentTarget.value)}
                          />
                        </div>
                        <div class="space-y-1 max-h-64 overflow-y-auto">
                          <For each={filteredImports()}>
                            {(imp) => (
                              <div class="text-xs p-1.5 rounded bg-bg-secondary">
                                <div class="font-medium text-accent">{imp.library}</div>
                                <Show when={imp.functions.length > 0}>
                                  <div class="mt-0.5 pl-3 text-txt-muted space-y-0.5">
                                    <For each={imp.functions.slice(0, 20)}>
                                      {(fn) => <div class="font-mono">{fn}</div>}
                                    </For>
                                    <Show when={imp.functions.length > 20}>
                                      <div class="text-txt-muted italic">... and {imp.functions.length - 20} more</div>
                                    </Show>
                                  </div>
                                </Show>
                              </div>
                            )}
                          </For>
                        </div>
                      </Show>
                    </div>
                  </Show>

                  {/* Exports */}
                  <Show when={data().exports.length > 0}>
                    <div>
                      <SectionHeader
                        title="Exports"
                        count={data().exports.length}
                        open={showExports()}
                        onToggle={() => setShowExports(!showExports())}
                      />
                      <Show when={showExports()}>
                        <table class="w-full text-xs mt-1">
                          <thead class="bg-bg-secondary">
                            <tr>
                              <th class="text-left p-1.5 text-txt-muted font-medium">Name</th>
                              <th class="text-left p-1.5 text-txt-muted font-medium">Address</th>
                            </tr>
                          </thead>
                          <tbody>
                            <For each={data().exports.slice(0, 100)}>
                              {(exp) => (
                                <tr class="border-b border-border/30 hover:bg-bg-hover">
                                  <td class="p-1.5 font-mono text-txt">{exp.name}</td>
                                  <td class="p-1.5 font-mono text-txt-secondary">{formatHex(exp.address)}</td>
                                </tr>
                              )}
                            </For>
                          </tbody>
                        </table>
                        <Show when={data().exports.length > 100}>
                          <div class="text-xs text-txt-muted p-2 text-center">
                            Showing 100 of {data().exports.length} exports
                          </div>
                        </Show>
                      </Show>
                    </div>
                  </Show>
                </div>
              )}
            </Show>
          </Show>
        </Show>
      </div>
    </div>
  );
}
