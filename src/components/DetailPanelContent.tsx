// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, Show, createSignal, createEffect, createMemo } from "solid-js";
import {
  HiOutlineClipboardDocument,
  HiOutlineInformationCircle,
} from "./icons";
import { debounce } from "@solid-primitives/scheduled";
import type { DiscoveredFile, ContainerInfo, TreeEntry, HashHistoryEntry, HashAlgorithm, StoredHash } from "../types";
import type { FileStatus, FileHashInfo } from "../hooks";
import { formatBytes, formatOffsetLabel } from "../utils";
import { FileHeader } from "./detail-panel/FileHeader";
import { StatsRow } from "./detail-panel/StatsRow";
import { HashDisplay } from "./detail-panel/HashDisplay";
import { HashHistory } from "./detail-panel/HashHistory";
import { FileTree } from "./detail-panel/FileTree";

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
              <FileHeader file={file} />
              
              {/* Stats row - prioritize acquisition dates over filesystem dates */}
              <StatsRow 
                file={file}
                ad1Info={ad1Info}
                e01Info={e01Info}
                ufedInfo={ufedInfo}
                hasAcquiryDate={hasAcquiryDate}
              />
              
              {/* Hash progress and computed hash */}
              <HashDisplay
                fileHash={props.fileHash}
                isHashing={isHashing}
                currentProgress={currentProgress}
                selectedHashAlgorithm={props.selectedHashAlgorithm}
                storedHashes={props.storedHashes}
              />
              
              {/* Hash history - shows all computed and stored hashes for this session */}
              <HashHistory
                hashHistory={props.hashHistory}
                reversedHashHistory={reversedHashHistory}
              />
              
              {/* Container details - includes stored hashes */}
              <Show when={props.fileInfo}>
                <ContainerDetails info={props.fileInfo!} storedHashes={props.storedHashes} />
              </Show>
              
              {/* File tree */}
              <FileTree
                tree={props.tree}
                filteredTree={props.filteredTree}
                localTreeFilter={localTreeFilter}
                treeCount={treeCount}
                hasTree={hasTree}
                treeExceedsLimit={treeExceedsLimit}
                handleTreeFilterInput={handleTreeFilterInput}
              />
              
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

