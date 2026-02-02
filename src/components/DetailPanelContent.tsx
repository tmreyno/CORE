// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, Show, createSignal, createEffect, createMemo } from "solid-js";
import {
  HiOutlineFolder,
  HiOutlineDocument,
  HiOutlineClipboardDocument,
  HiOutlineDocumentDuplicate,
  HiOutlineExclamationTriangle,
  HiOutlineInformationCircle,
  HiOutlineLockClosed,
  HiOutlineCheck,
} from "./icons";
import { debounce } from "@solid-primitives/scheduled";
import type { DiscoveredFile, ContainerInfo, TreeEntry, HashHistoryEntry, HashAlgorithm, StoredHash } from "../types";
import type { FileStatus, FileHashInfo } from "../hooks";
import { formatBytes, typeClass, formatOffsetLabel } from "../utils";
import { getContainerTypeIcon } from "./tree";

// =============================================================================
// Shared Memoization Utilities for Metadata Display
// =============================================================================

interface DetailPanelContentProps {
  activeFile: DiscoveredFile | null;
  fileInfo: ContainerInfo | undefined;
  fileHash: FileHashInfo | undefined;
  fileStatus: FileStatus | undefined;
  tree: TreeEntry[];
  filteredTree: TreeEntry[];
  treeFilter: string;
  onTreeFilterChange: (filter: string) => void;
  selectedHashAlgorithm: HashAlgorithm;
  hashHistory: HashHistoryEntry[];
  storedHashes: StoredHash[];
  busy: boolean;
  onLoadInfo: () => void;
  formatHashDate: (timestamp: string) => string;
}

export function DetailPanelContent(props: DetailPanelContentProps) {
  // Local state for immediate input feedback, with debounced propagation
  const [localTreeFilter, setLocalTreeFilter] = createSignal(props.treeFilter);
  
  // Debounced filter update (150ms delay)
  const debouncedFilterChange = debounce((value: string) => {
    props.onTreeFilterChange(value);
  }, 150);
  
  // Sync local filter when prop changes (e.g., from external clear)
  createEffect(() => {
    setLocalTreeFilter(props.treeFilter);
  });
  
  const handleTreeFilterInput = (value: string) => {
    setLocalTreeFilter(value);
    debouncedFilterChange(value);
  };
  
  // ==========================================================================
  // Memoized computed values for status checks (avoid repeated property access)
  // ==========================================================================
  const isHashing = createMemo(() => props.fileStatus?.status === "hashing");
  const currentProgress = createMemo(() => props.fileStatus?.progress ?? 0);
  
  // Memoized container type accessors (avoid deep property access in JSX)
  const ad1Info = createMemo(() => props.fileInfo?.ad1);
  const e01Info = createMemo(() => props.fileInfo?.e01);
  const ufedInfo = createMemo(() => props.fileInfo?.ufed);
  
  // Memoized date accessors
  const acquiryDate = createMemo(() => 
    e01Info()?.acquiry_date || 
    ad1Info()?.companion_log?.acquisition_date || 
    ufedInfo()?.extraction_info?.start_time
  );
  const hasAcquiryDate = createMemo(() => !!acquiryDate());
  
  // Memoized tree info
  const treeCount = createMemo(() => props.tree.length);
  const hasTree = createMemo(() => treeCount() > 0);
  const treeExceedsLimit = createMemo(() => treeCount() > 500);
  
  // Memoized hash history (reversed once, not on every render)
  const reversedHashHistory = createMemo(() => props.hashHistory.slice().reverse());
  
  return (
    <main class="flex flex-col flex-1 min-h-0 overflow-y-auto bg-bg p-4">
      <Show 
        when={props.activeFile} 
        keyed
        fallback={
          <div class="flex flex-col items-center justify-center flex-1 text-txt-muted gap-2">
            <HiOutlineClipboardDocument class="w-12 h-12 opacity-50" />
            <p>Select a file to view details</p>
          </div>
        }
      >
        {(file) => {
          return (
            <div class="flex flex-col gap-4">
              {/* Header */}
              <div class="flex flex-col gap-1">
                <span class={`inline-flex items-center gap-1.5 text-xs px-2 py-0.5 rounded w-fit ${typeClass(file.container_type)}`}>
                  {(() => {
                    const IconComponent = getContainerTypeIcon(file.container_type);
                    return <IconComponent class="w-3 h-3" />;
                  })()} {file.container_type}
                </span>
                <h2 class="text-lg font-semibold text-txt truncate" title={file.filename}>{file.filename}</h2>
                <p class="text-xs text-txt-muted truncate" title={file.path}>{file.path}</p>
              </div>
              
              {/* Stats row - prioritize acquisition dates over filesystem dates */}
              <div class="grid grid-cols-2 sm:grid-cols-3 lg:grid-cols-4 gap-2 p-3 bg-bg-panel/50 rounded-lg border border-border/50">
                <div class="flex flex-col gap-0.5">
                  <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>Size</span>
                  <span class="text-sm text-txt font-medium" title={`${file.size.toLocaleString()} bytes`}>{formatBytes(file.size)}</span>
                </div>
                <Show when={file.segment_count}>
                  <div class="flex flex-col gap-0.5">
                    <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>Segments</span>
                    <span class="text-sm text-txt font-medium" title={`${file.segment_count} segments`}>{file.segment_count}</span>
                  </div>
                </Show>
                
                {/* E01: Show acquisition date from header */}
                <Show when={e01Info()?.acquiry_date}>
                  <div class="flex flex-col gap-0.5">
                    <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>Acquired</span>
                    <span class="text-sm text-txt font-medium" title={`Acquisition date from E01 header: ${e01Info()!.acquiry_date}`}>{e01Info()!.acquiry_date}</span>
                  </div>
                </Show>
                
                {/* AD1: Show acquisition date from companion log */}
                <Show when={ad1Info()?.companion_log?.acquisition_date}>
                  <div class="flex flex-col gap-0.5">
                    <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>Acquired</span>
                    <span class="text-sm text-txt font-medium" title={`Acquisition date from AD1 companion log: ${ad1Info()!.companion_log!.acquisition_date}`}>{ad1Info()!.companion_log!.acquisition_date}</span>
                  </div>
                </Show>
                
                {/* UFED: Show extraction date */}
                <Show when={ufedInfo()?.extraction_info?.start_time}>
                  <div class="flex flex-col gap-0.5">
                    <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>Extracted</span>
                    <span class="text-sm text-txt font-medium" title={`Extraction date from UFED metadata: ${ufedInfo()!.extraction_info!.start_time}`}>{ufedInfo()!.extraction_info!.start_time}</span>
                  </div>
                </Show>
                
                {/* Fallback to filesystem dates only if no container date */}
                <Show when={!hasAcquiryDate()}>
                  <Show when={file.created}>
                    <div class="flex flex-col gap-0.5">
                      <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>File Created</span>
                      <span class="text-sm text-txt font-medium" title={`Filesystem date (when file was created on disk): ${file.created}`}>{file.created}</span>
                    </div>
                  </Show>
                  <Show when={file.modified}>
                    <div class="flex flex-col gap-0.5">
                      <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>File Modified</span>
                      <span class="text-sm text-txt font-medium" title={`Filesystem date (when file was last modified): ${file.modified}`}>{file.modified}</span>
                    </div>
                  </Show>
                </Show>
                
                <Show when={ad1Info()}>
                  <div class="flex flex-col gap-0.5">
                    <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>Items</span>
                    <span class="text-sm text-txt font-medium" title={`${ad1Info()!.item_count.toLocaleString()} items in AD1 container`}>{ad1Info()!.item_count.toLocaleString()}</span>
                  </div>
                </Show>
                <Show when={e01Info()}>
                  <div class="flex flex-col gap-0.5">
                    <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>Chunks</span>
                    <span class="text-sm text-txt font-medium" title={`${e01Info()!.chunk_count.toLocaleString()} compressed chunks`}>{e01Info()!.chunk_count.toLocaleString()}</span>
                  </div>
                  <div class="flex flex-col gap-0.5">
                    <span class={`text-[10px] leading-tight text-txt-muted uppercase tracking-wider`}>Sectors</span>
                    <span class="text-sm text-txt font-medium" title={`${e01Info()!.sector_count.toLocaleString()} sectors`}>{e01Info()!.sector_count.toLocaleString()}</span>
                  </div>
                </Show>
              </div>
              
              {/* Hash progress */}
              <Show when={isHashing()}>
                <div class="progress-card">
                  <div class="progress-header">
                    <span class="progress-title"><HiOutlineLockClosed class="w-4 h-4" /> Hashing with {props.selectedHashAlgorithm.toUpperCase()}...</span>
                    <span class="progress-value">{currentProgress().toFixed(1)}%</span>
                  </div>
                  <div class="progress-bar">
                    <div class="progress-fill" style={{ width: `${currentProgress()}%` }} />
                  </div>
                </div>
              </Show>
              
              {/* Computed hash card */}
              <Show when={props.fileHash && !isHashing()}>
                <div class={`rounded-lg border p-3 ${props.fileHash!.verified === true ? 'bg-green-900/20 border-green-500/50' : props.fileHash!.verified === false ? 'bg-red-900/20 border-red-500/50' : 'bg-bg-panel/50 border-border/50'}`}>
                  <div class="flex items-center gap-2 mb-2">
                    <span class="text-sm text-accent font-medium"><HiOutlineLockClosed class="w-4 h-4 inline" /> {props.fileHash!.algorithm}</span>
                    <Show when={props.fileHash!.verified === true}>
                      <span class="inline-flex items-center gap-1 text-xs text-green-400 font-semibold">
                        <span class="relative inline-flex">
                          <span>✓</span>
                          <span class="absolute left-[3px]">✓</span>
                        </span>
                        <span class="ml-1">VERIFIED</span>
                      </span>
                    </Show>
                    <Show when={props.fileHash!.verified === false}>
                      <span class="text-xs text-red-400 font-semibold flex items-center gap-1">
                        <span class="font-bold">✗</span>
                        MISMATCH
                      </span>
                    </Show>
                    <Show when={props.fileHash!.verified === null}>
                      <span class="text-xs text-txt-secondary flex items-center gap-1"><HiOutlineCheck class="w-3 h-3" /> Computed</span>
                    </Show>
                    <button class="ml-auto text-sm hover:bg-bg-hover p-1 rounded flex items-center" onClick={() => navigator.clipboard.writeText(props.fileHash!.hash)} title="Copy hash">
                      <HiOutlineDocumentDuplicate class="w-4 h-4" />
                    </button>
                  </div>
                  <code class="block text-xs font-mono text-txt-tertiary break-all bg-bg/50 p-2 rounded">{props.fileHash!.hash}</code>
                  <Show when={props.fileHash!.verified === true && props.storedHashes.some(sh => sh.algorithm.toLowerCase() === props.fileHash!.algorithm.toLowerCase())}>
                    <div class="mt-2 text-xs text-green-400 flex items-center gap-1">
                      <span class="relative inline-flex">
                        <span>✓</span>
                        <span class="absolute left-[3px]">✓</span>
                      </span>
                      <span class="ml-1">Hash matches stored value from container/companion</span>
                    </div>
                  </Show>
                  <Show when={props.fileHash!.verified === true && !props.storedHashes.some(sh => sh.algorithm.toLowerCase() === props.fileHash!.algorithm.toLowerCase())}>
                    <div class="mt-2 text-xs text-green-400 flex items-center gap-1">
                      <span class="relative inline-flex">
                        <span>✓</span>
                        <span class="absolute left-[3px]">✓</span>
                      </span>
                      <span class="ml-1">Hash matches previous computation (self-verified)</span>
                    </div>
                  </Show>
                  <Show when={props.fileHash!.verified === false}>
                    <div class="mt-2 text-xs text-red-400 flex items-center gap-1">
                      <HiOutlineExclamationTriangle class="w-3 h-3" /> Computed hash does NOT match stored hash!
                    </div>
                  </Show>
                  <Show when={props.fileHash!.verified === null && props.hashHistory.length === 0}>
                    <div class="mt-2 text-xs text-txt-muted">No stored hash or history to verify against</div>
                  </Show>
                </div>
              </Show>
              
              {/* Hash history - shows all computed and stored hashes for this session */}
              <Show when={props.hashHistory.length > 0}>
                <div class="info-card">
                  <div class="flex items-center justify-between mb-2">
                    <span class="info-card-title">🕒 Hash History ({props.hashHistory.length})</span>
                  </div>
                  <div class="flex flex-col gap-1 max-h-40 overflow-y-auto">
                    <For each={reversedHashHistory()}>
                      {(entry) => {
                        const isStored = entry.source === "stored";
                        const isVerified = entry.source === "verified";
                        const entryDate = entry.timestamp instanceof Date ? entry.timestamp : new Date(entry.timestamp);
                        // Check if this is the epoch fallback (no real date available)
                        const hasValidDate = entryDate.getTime() > 0;
                        
                        // Format short date (e.g., "Jan 16, 2026")
                        const shortDateStr = hasValidDate 
                          ? entryDate.toLocaleDateString(undefined, { month: "short", day: "numeric", year: "numeric" })
                          : "Container";
                        
                        // Build detailed tooltip
                        const tooltipParts = [
                          `Algorithm: ${entry.algorithm}`,
                          `Hash: ${entry.hash}`,
                          `Source: ${isStored ? "Stored in container" : isVerified ? "Verified copy" : "Computed"}`,
                        ];
                        if (hasValidDate) {
                          tooltipParts.push(`Date: ${entryDate.toLocaleString()}`);
                        }
                        if (entry.verified === true) {
                          tooltipParts.push("Status: Verified ✓");
                        } else if (entry.verified === false) {
                          tooltipParts.push("Status: MISMATCH ✗");
                        }
                        if (entry.verified_against) {
                          tooltipParts.push(`Compared against: ${entry.verified_against.substring(0, 16)}...`);
                        }
                        const tooltipText = tooltipParts.join("\n");
                        
                        return (
                          <div 
                            class={`hash-row ${entry.verified === true ? 'hash-row-verified' : entry.verified === false ? 'hash-row-failed' : isStored ? 'hash-row-stored' : 'hash-row-neutral'}`}
                            title={tooltipText}
                          >
                            <span class="text-txt-muted w-24 shrink-0 text-xs cursor-help">
                              {isStored ? '◆ ' : isVerified ? '⟳ ' : ''}{shortDateStr}
                            </span>
                            <span class="text-accent w-12 shrink-0 uppercase">{entry.algorithm}</span>
                            <span class={`w-16 shrink-0 text-xs ${isStored ? 'text-amber-400' : isVerified ? 'text-green-400' : 'text-txt-muted'}`}>
                              {isStored ? 'stored' : isVerified ? 'verified' : entry.source}
                            </span>
                            <code class="text-txt-secondary font-mono truncate flex-1">{entry.hash}</code>
                            <Show when={entry.verified === true}>
                              <span class="relative inline-flex text-green-400" title="Verified match">
                                <span>✓</span>
                                <span class="absolute left-[3px]">✓</span>
                              </span>
                            </Show>
                            <Show when={entry.verified === false}><span class="text-red-400 font-bold" title="Hash mismatch!">✗</span></Show>
                            <button class="text-txt-muted hover:text-txt-tertiary p-0.5 flex items-center" onClick={() => navigator.clipboard.writeText(entry.hash)} title="Copy hash">
                              <HiOutlineDocumentDuplicate class="w-3 h-3" />
                            </button>
                          </div>
                        );
                      }}
                    </For>
                  </div>
                </div>
              </Show>
              
              {/* Container details - includes stored hashes */}
              <Show when={props.fileInfo}>
                <ContainerDetails info={props.fileInfo!} storedHashes={props.storedHashes} />
              </Show>
              
              {/* File tree */}
              <Show when={hasTree()}>
                <div class="info-card">
                  <div class="flex items-center justify-between mb-2">
                    <span class="info-card-title">
                      <HiOutlineFolder class="w-4 h-4 text-yellow-500" /> File Tree ({treeCount()})
                    </span>
                    <input 
                      type="text" 
                      class="text-xs px-2 py-1 rounded bg-bg border border-border text-txt placeholder-txt-muted focus:border-accent focus:outline-none w-32" 
                      placeholder="Filter..." 
                      value={localTreeFilter()} 
                      onInput={(e) => handleTreeFilterInput(e.currentTarget.value)} 
                    />
                  </div>
                  <div class="flex flex-col gap-0.5 max-h-64 overflow-y-auto bg-bg/50 rounded p-1">
                    <For each={props.filteredTree}>
                      {(entry) => (
                        <div class={`flex items-center gap-1.5 px-2 py-1 text-xs rounded hover:bg-bg-panel ${entry.is_dir ? "text-accent" : "text-txt-tertiary"}`} title={entry.path}>
                          <span class="w-4 shrink-0 flex items-center justify-center">
                            {entry.is_dir ? <HiOutlineFolder class={`w-3 h-3 text-yellow-500`} /> : <HiOutlineDocument class="w-3 h-3" />}
                          </span>
                          <span class="truncate flex-1" title={entry.path}>{entry.path}</span>
                          <span class="text-txt-muted w-16 text-right shrink-0">{entry.is_dir ? "" : formatBytes(entry.size)}</span>
                        </div>
                      )}
                    </For>
                    <Show when={treeExceedsLimit()}>
                      <div class="text-center text-xs text-txt-muted py-2">Showing first 500 of {treeCount()} items</div>
                    </Show>
                  </div>
                </div>
              </Show>
              
              {/* Action buttons */}
              <div class="flex flex-wrap gap-2 pt-2">
                <Show when={!props.fileInfo}>
                  <button 
                    class="btn-sm" 
                    onClick={props.onLoadInfo} 
                    disabled={props.busy}
                  >
                    <HiOutlineInformationCircle class="w-3 h-3" /> Load Info
                  </button>
                </Show>
              </div>
            </div>
          );
        }}
      </Show>
    </main>
  );
}

// Container details sub-component
// ============================================================================
// Common Info Row Component - Single source of truth for rendering detail rows
// ============================================================================

type RowType = 'normal' | 'highlight' | 'device' | 'full-width' | 'hash' | 'warning';
type RowFormat = 'text' | 'bytes' | 'mono' | 'notes' | 'list' | 'warning';

interface InfoField {
  label: string;
  value: string | number | undefined | null;
  type?: RowType;
  format?: RowFormat;
  condition?: boolean; // Override automatic truthy check
}

// Renders a single info row with consistent styling
function InfoRow(props: InfoField) {
  // Skip if no value (unless condition explicitly set)
  const shouldShow = () => {
    if (props.condition !== undefined) return props.condition;
    return props.value !== undefined && props.value !== null && props.value !== '';
  };
  
  const formatValue = () => {
    const val = props.value;
    if (val === undefined || val === null) return '';
    if (props.format === 'bytes' && typeof val === 'number') return formatBytes(val);
    return String(val);
  };
  
  const rowClass = () => {
    const base = 'flex items-start gap-2 py-1 px-1.5 text-xs rounded';
    if (props.type === 'highlight') return `${base} bg-accent/20`;
    if (props.type === 'device') return `${base} bg-amber-900/10`;
    if (props.type === 'full-width') return `${base} col-span-2`;
    if (props.type === 'hash') return `${base} bg-bg-panel/50 col-span-2`;
    if (props.type === 'warning' || props.format === 'warning') return `${base} bg-red-900/20 col-span-2`;
    return base;
  };
  
  const valueClass = () => {
    const base = 'text-txt flex-1';
    if (props.format === 'mono') return `${base} font-mono text-[10px] leading-tight`;
    if (props.format === 'notes') return `${base} text-txt-secondary italic`;
    if (props.format === 'list') return `${base} text-txt-secondary`;
    if (props.type === 'hash') return `${base} font-mono text-[10px] leading-tight break-all`;
    if (props.format === 'warning') return `${base} text-red-400`;
    return base;
  };
  
  return (
    <Show when={shouldShow()}>
      <div class={rowClass()}>
        <span class="text-txt-muted shrink-0 w-24">{props.label}</span>
        <span class={valueClass()}>{formatValue()}</span>
      </div>
    </Show>
  );
}

// Renders multiple info rows from a field array
function InfoRows(props: { fields: InfoField[] }) {
  return (
    <For each={props.fields}>
      {(field) => <InfoRow {...field} />}
    </For>
  );
}

// ============================================================================
// Normalize container info to common field structure
// ============================================================================

function normalizeContainerFields(info: ContainerInfo, storedHashes: StoredHash[]): InfoField[] {
  const fields: InfoField[] = [];
  
  // AD1
  if (info.ad1) {
    const ad1 = info.ad1;
    const log = ad1.companion_log;
    const vol = ad1.volume;
    
    // Show warning if segments are missing
    if (ad1.missing_segments && ad1.missing_segments.length > 0) {
      fields.push({
        label: '⚠ Incomplete',
        value: `Missing ${ad1.missing_segments.length} segment(s): ${ad1.missing_segments.join(', ')}`,
        type: 'full-width',
        format: 'warning'
      });
    }
    
    fields.push(
      { label: 'Format', value: `AD1 (${ad1.logical.signature})` },
      { label: 'Version', value: ad1.logical.image_version },
      { label: 'Segments', value: `${ad1.segment_files?.length ?? 0} / ${ad1.segment.segment_number}${ad1.missing_segments?.length ? ' (incomplete)' : ''}` },
      { label: 'Total Size', value: ad1.total_size, format: 'bytes' },
      { label: 'Items', value: ad1.item_count },
      // Case metadata from companion log
      { label: 'Case #', value: log?.case_number, type: 'highlight' },
      { label: 'Evidence #', value: log?.evidence_number, type: 'highlight' },
      { label: 'Examiner', value: log?.examiner },
      { label: 'Acquired', value: log?.acquisition_date },
      // Volume/system info from header
      { label: 'Volume', value: vol?.volume_label },
      { label: 'Filesystem', value: vol?.filesystem },
      { label: 'OS', value: vol?.os_info },
      { label: 'Block Size', value: vol?.block_size, format: 'bytes' },
      // Technical details
      { label: 'Chunk Size', value: ad1.logical.zlib_chunk_size, format: 'bytes' },
      { label: 'Source', value: ad1.logical.data_source_name, type: 'full-width' },
      // Notes (hashes now displayed via storedHashes at end)
      { label: 'Notes', value: log?.notes, type: 'full-width', format: 'notes' },
    );
  }
  
  // E01
  if (info.e01) {
    const e01 = info.e01;
    fields.push(
      { label: 'Format', value: e01.format_version },
      { label: 'Segments', value: e01.segment_count },
      { label: 'Total Size', value: e01.total_size, format: 'bytes' },
      { label: 'Compression', value: e01.compression },
      { label: 'Bytes/Sector', value: e01.bytes_per_sector },
      { label: 'Sectors/Chunk', value: e01.sectors_per_chunk },
      { label: 'Case #', value: e01.case_number, type: 'highlight' },
      { label: 'Evidence #', value: e01.evidence_number, type: 'highlight' },
      { label: 'Examiner', value: e01.examiner_name },
      { label: 'Acquired', value: e01.acquiry_date },
      { label: 'System Date', value: e01.system_date },
      { label: 'Model', value: e01.model, type: 'device' },
      { label: 'Serial #', value: e01.serial_number, type: 'device' },
      { label: 'Description', value: e01.description, type: 'full-width' },
      { label: 'Notes', value: e01.notes, type: 'full-width', format: 'notes' },
      // Stored hashes now displayed via storedHashes at end
    );
  }
  
  // L01 (Logical Evidence - uses same EwfInfo type as E01)
  if (info.l01) {
    const l01 = info.l01;
    fields.push(
      { label: 'Format', value: l01.format_version },
      { label: 'Segments', value: l01.segment_count },
      { label: 'Total Size', value: l01.total_size, format: 'bytes' },
      { label: 'Compression', value: l01.compression },
      { label: 'Bytes/Sector', value: l01.bytes_per_sector },
      { label: 'Sectors/Chunk', value: l01.sectors_per_chunk },
      { label: 'Case #', value: l01.case_number, type: 'highlight' },
      { label: 'Evidence #', value: l01.evidence_number, type: 'highlight' },
      { label: 'Examiner', value: l01.examiner_name },
      { label: 'Acquired', value: l01.acquiry_date },
      { label: 'System Date', value: l01.system_date },
      { label: 'Model', value: l01.model, type: 'device' },
      { label: 'Serial #', value: l01.serial_number, type: 'device' },
      { label: 'Description', value: l01.description, type: 'full-width' },
      { label: 'Notes', value: l01.notes, type: 'full-width', format: 'notes' },
    );
  }
  
  // Raw
  if (info.raw) {
    const raw = info.raw;
    fields.push(
      { label: 'Format', value: 'Raw Image' },
      { label: 'Segments', value: raw.segment_count },
      { label: 'Total Size', value: raw.total_size, format: 'bytes' },
    );
    if (raw.segment_count > 1) {
      const segList = raw.segment_names.slice(0, 5).join(', ') + 
        (raw.segment_count > 5 ? ` (+${raw.segment_count - 5} more)` : '');
      fields.push({ label: 'Segment Files', value: segList, type: 'full-width', format: 'list' });
    }
  }
  
  // Archive (ZIP/7z)
  if (info.archive) {
    const archive = info.archive;
    fields.push(
      { label: 'Format', value: `${archive.format}${archive.version ? ` v${archive.version}` : ''}` },
      { label: 'Segments', value: archive.segment_count },
      { label: 'Total Size', value: archive.total_size, format: 'bytes' },
      { label: 'Entries', value: archive.entry_count },
      { label: 'AES Encrypted', value: archive.aes_encrypted ? 'Yes' : undefined, type: 'highlight' },
      { label: 'Encrypted Headers', value: archive.encrypted_headers ? 'Filenames Hidden' : undefined, type: 'highlight' },
    );
    if (archive.start_header_crc_valid !== undefined && archive.start_header_crc_valid !== null) {
      fields.push({ 
        label: 'Header CRC', 
        value: archive.start_header_crc_valid ? '✓ Valid' : '✗ Invalid',
        type: archive.start_header_crc_valid ? 'normal' : 'highlight',
        condition: true 
      });
    }
    fields.push(
      { label: 'Central Dir', value: archive.central_dir_offset ? `@ ${archive.central_dir_offset.toLocaleString()}` : undefined },
      { label: 'Next Header', value: archive.next_header_offset ? formatOffsetLabel(archive.next_header_offset) : undefined },
    );
    if (archive.segment_count > 1) {
      const segList = archive.segment_names.slice(0, 5).join(', ') + 
        (archive.segment_count > 5 ? ` (+${archive.segment_count - 5} more)` : '');
      fields.push({ label: 'Segment Files', value: segList, type: 'full-width', format: 'list' });
    }
  }
  
  // UFED (Cellebrite)
  if (info.ufed) {
    const ufed = info.ufed;
    const allFiles: string[] = [...ufed.associated_files.map(f => f.filename)];
    if (ufed.collection_info?.ufdx_path) {
      const ufdxName = ufed.collection_info.ufdx_path.split('/').pop() || ufed.collection_info.ufdx_path.split('\\').pop();
      if (ufdxName && !allFiles.includes(ufdxName)) allFiles.push(ufdxName);
    }
    
    fields.push(
      { label: 'Format', value: `UFED (${ufed.format})` },
      { label: 'Total Size', value: ufed.size, format: 'bytes' },
      { label: 'Extraction', value: ufed.extraction_info?.extraction_type },
      { label: 'Tool', value: ufed.extraction_info?.acquisition_tool ? 
        `${ufed.extraction_info.acquisition_tool}${ufed.extraction_info.tool_version ? ` v${ufed.extraction_info.tool_version}` : ''}` : undefined },
      { label: 'Case #', value: ufed.case_info?.case_identifier, type: 'highlight' },
      { label: 'Evidence #', value: ufed.case_info?.device_name || ufed.evidence_number, type: 'highlight' },
      { label: 'Examiner', value: ufed.case_info?.examiner_name },
      { label: 'Acquired', value: ufed.extraction_info?.start_time },
      { label: 'Completed', value: ufed.extraction_info?.end_time },
      { label: 'Device', value: ufed.device_info?.full_name || ufed.device_hint, type: 'device' },
      { label: 'Model', value: ufed.device_info?.model, type: 'device' },
      { label: 'Serial #', value: ufed.device_info?.serial_number, type: 'device' },
      { label: 'IMEI', value: ufed.device_info?.imei ? 
        `${ufed.device_info.imei}${ufed.device_info.imei2 ? ` / ${ufed.device_info.imei2}` : ''}` : undefined, type: 'device' },
      { label: 'OS', value: ufed.device_info?.os_version ? 
        `${ufed.device_info.vendor ? `${ufed.device_info.vendor} ` : ''}${ufed.device_info.os_version}` : undefined, type: 'device' },
      { label: 'Connection', value: ufed.extraction_info?.connection_type, type: 'full-width' },
      { label: 'Location', value: ufed.case_info?.location, type: 'full-width' },
      { label: 'GUID', value: ufed.extraction_info?.guid, type: 'full-width', format: 'mono' },
      { label: 'Files', value: allFiles.length > 0 ? allFiles.join(', ') : undefined, type: 'full-width' },
    );
  }
  
  // Companion log
  if (info.companion_log) {
    const log = info.companion_log;
    fields.push(
      { label: 'Created By', value: log.created_by },
      { label: 'Case #', value: log.case_number, type: 'highlight' },
      { label: 'Evidence #', value: log.evidence_number, type: 'highlight' },
      { label: 'Examiner', value: log.examiner },
      { label: 'Acquired', value: log.acquisition_started },
      { label: 'Source', value: log.unique_description, type: 'full-width' },
      { label: 'Notes', value: log.notes, type: 'full-width', format: 'notes' },
    );
  }
  
  // Add all stored hashes (unified display for all container types)
  if (storedHashes && storedHashes.length > 0) {
    for (const sh of storedHashes) {
      const algo = sh.algorithm?.toUpperCase() || 'HASH';
      const hash = sh.hash || '';
      const sourceLabel = sh.source === 'container' ? '◆' : sh.source === 'companion' ? '◇' : '▣';
      const verifyIcon = sh.verified === true ? ' ✓' : sh.verified === false ? ' ✗' : '';
      // Show filename if available (UFED has per-file hashes)
      const filenameLabel = sh.filename ? ` (${sh.filename})` : '';
      fields.push({ 
        label: `${sourceLabel} ${algo}${verifyIcon}${filenameLabel}`, 
        value: hash, 
        type: 'hash' 
      });
    }
  }
  
  return fields;
}

// ============================================================================
// Container Details Component - Uses common template with memoization
// ============================================================================

function ContainerDetails(props: { info: ContainerInfo; storedHashes: StoredHash[] }) {
  // Memoize the field normalization to avoid recomputation on every render
  const fields = createMemo(() => normalizeContainerFields(props.info, props.storedHashes));
  
  return (
    <div class="info-card">
      <div class="info-card-title">
        <HiOutlineClipboardDocument class="w-4 h-4" /> Container Details
      </div>
      <div class="grid grid-cols-1 sm:grid-cols-2 gap-0.5">
        <InfoRows fields={fields()} />
      </div>
    </div>
  );
}

