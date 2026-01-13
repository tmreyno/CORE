// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import type { DiscoveredFile, ContainerInfo, HashHistoryEntry } from "../types";
import type { FileStatus, FileHashInfo } from "../hooks";
import { formatBytes, typeClass } from "../utils";
import { getContainerTypeIcon } from "./tree";
import {
  HiOutlineFolderOpen,
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
  HiOutlineLockClosed,
  HiOutlineDocumentText,
  HiOutlineHashtag,
} from "./icons";

interface FileRowProps {
  file: DiscoveredFile;
  index: number;
  isSelected: boolean;
  isActive: boolean;
  isFocused: boolean;
  isHovered: boolean;
  fileStatus: FileStatus | undefined;
  fileInfo: ContainerInfo | undefined;
  fileHash: FileHashInfo | undefined;
  hashHistory: HashHistoryEntry[];
  busy: boolean;
  onSelect: () => void;
  onToggleSelection: () => void;
  onHash: () => void;
  onMouseEnter: () => void;
  onMouseLeave: () => void;
  onContextMenu?: (e: MouseEvent) => void;
}

export function FileRow(props: FileRowProps) {
  // Check if container is incomplete (missing segments)
  const isIncomplete = () => (props.fileInfo?.ad1?.missing_segments?.length ?? 0) > 0;
  
  // Get total container size (all segments combined) when available
  const totalContainerSize = () => {
    const info = props.fileInfo;
    if (info?.ad1?.total_size) return info.ad1.total_size;
    if (info?.e01?.total_size) return info.e01.total_size;
    if (info?.l01?.total_size) return info.l01.total_size;
    if (info?.raw?.total_size) return info.raw.total_size;
    if (info?.archive?.total_size) return info.archive.total_size;
    return null;
  };
  
  // Display size: use total container size if available, otherwise first segment size
  const displaySize = () => totalContainerSize() ?? props.file.size;
  const hasMultipleSegments = () => (props.file.segment_count ?? 1) > 1;
  const sizeLabel = () => {
    const total = totalContainerSize();
    if (total && hasMultipleSegments()) {
      return `Total: ${formatBytes(total)} (${props.file.segment_count} segments, first segment: ${formatBytes(props.file.size)})`;
    }
    return `${formatBytes(displaySize())}`;
  };
  
  // Unified hash indicator logic
  const storedHashCount = () => (props.fileInfo?.e01?.stored_hashes?.length ?? 0) + (props.fileInfo?.companion_log?.stored_hashes?.length ?? 0);
  const historyCount = () => props.hashHistory?.length ?? 0;
  const totalHashCount = () => storedHashCount() + (props.fileHash ? 1 : 0) + historyCount();
  const isHashing = () => props.fileStatus?.status === "hashing" && !props.fileHash && (props.fileStatus?.progress ?? 0) < 95;
  const isCompleting = () => props.fileStatus?.status === "hashing" && (props.fileStatus?.progress ?? 0) >= 95 && !props.fileHash;
  const hashProgress = () => props.fileStatus?.progress ?? 0;
  const chunksProcessed = () => props.fileStatus?.chunksProcessed;
  const chunksTotal = () => props.fileStatus?.chunksTotal;
  const hasChunkProgress = () => chunksTotal() !== undefined && chunksTotal()! > 0 && hashProgress() < 100;
  
  // Format chunk progress for display
  const formatChunks = (count: number) => {
    if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M`;
    if (count >= 1000) return `${(count / 1000).toFixed(0)}k`;
    return count.toString();
  };
  
  // Check if any hash matches exist (stored vs history, or history vs history)
  const hasVerifiedMatch = () => {
    const storedHashes = [
      ...(props.fileInfo?.e01?.stored_hashes ?? []), 
      ...(props.fileInfo?.ufed?.stored_hashes ?? []),
      ...(props.fileInfo?.companion_log?.stored_hashes ?? [])
    ];
    const history = props.hashHistory ?? [];
    
    // Check if any stored hash matches any history hash
    for (const stored of storedHashes) {
      const match = history.find(h => 
        h.algorithm.toLowerCase() === stored.algorithm.toLowerCase() && 
        h.hash.toLowerCase() === stored.hash.toLowerCase()
      );
      if (match) return true;
    }
    
    // Check if any history entries match each other (same algorithm, same hash, different times)
    for (let i = 0; i < history.length; i++) {
      for (let j = i + 1; j < history.length; j++) {
        if (history[i].algorithm.toLowerCase() === history[j].algorithm.toLowerCase() &&
            history[i].hash.toLowerCase() === history[j].hash.toLowerCase()) {
          return true;
        }
      }
    }
    
    return false;
  };
  
  // Determine hash state
  const hashState = () => {
    // Check for incomplete container first
    if (isIncomplete()) return "incomplete";
    const hash = props.fileHash;
    if (hash?.verified === true) return "verified";
    if (hash?.verified === false) return "failed";
    if (hash) return "computed";
    // Check for verified matches even without current fileHash
    if (hasVerifiedMatch()) return "verified";
    if (storedHashCount() > 0) return "stored";
    if (historyCount() > 0) return "computed"; // Has history but no stored to verify against
    return "none";
  };

  // Row class based on state
  const rowClass = () => {
    let base = "flex items-center gap-2 px-3 py-2 cursor-pointer transition-colors border-b border-zinc-800/50";
    if (props.isSelected) base += " bg-cyan-500/10";
    if (props.isActive) base += " bg-cyan-500/20 outline outline-1 outline-cyan-500";
    if (props.isFocused && !props.isActive) base += " outline-2 outline-dashed outline-cyan-500 outline-offset-[-2px]";
    if (!props.isSelected && !props.isActive) base += " hover:bg-zinc-800/50";
    return base;
  };

  return (
    <div 
      id={`file-row-${props.index}`}
      class={rowClass()} 
      onMouseEnter={props.onMouseEnter} 
      onMouseLeave={props.onMouseLeave} 
      onClick={props.onSelect}
      onContextMenu={(e) => props.onContextMenu?.(e)}
      data-index={props.index}
      role="option"
      aria-selected={props.isSelected}
      tabIndex={props.isFocused ? 0 : -1}
    >
      <input 
        type="checkbox" 
        class="shrink-0 accent-cyan-500"
        checked={props.isSelected} 
        onChange={(e) => { e.stopPropagation(); props.onToggleSelection(); }} 
        onClick={(e) => e.stopPropagation()} 
      />
      
      <span class={`shrink-0 ${typeClass(props.file.container_type)}`} title={props.file.container_type}>
        {(() => {
          const IconComponent = getContainerTypeIcon(props.file.container_type);
          return <IconComponent class="w-4 h-4" />;
        })()}
      </span>
      
      <div class="flex-1 min-w-0 flex flex-col">
        <span class="text-sm text-zinc-200 truncate" title={props.file.path}>{props.file.filename}</span>
        <span class="text-xs text-zinc-500 flex items-center gap-1">
          <span title={sizeLabel()}>{formatBytes(displaySize())}</span>
          <Show when={props.file.segment_count && props.file.segment_count > 1}>
            <span class="text-zinc-600">• {props.file.segment_count} segs</span>
          </Show>
        </span>
      </div>
      
      <div class="flex items-center gap-1.5 shrink-0">
        <Show when={props.fileInfo?.ad1?.item_count}>
          <span class="flex items-center gap-0.5 px-1.5 py-0.5 text-xs bg-zinc-700 rounded text-zinc-300" title={`${props.fileInfo!.ad1!.item_count.toLocaleString()} items`}>
            <HiOutlineFolderOpen class="w-3.5 h-3.5" />{props.fileInfo!.ad1!.item_count > 999 ? Math.round(props.fileInfo!.ad1!.item_count / 1000) + "k" : props.fileInfo!.ad1!.item_count}
          </span>
        </Show>
        
        {/* Hash Indicators - Icon button with overlay badge */}
        <Show when={isHashing()}>
          <span 
            class="chip chip-amber animate-pulse" 
            title={hasChunkProgress()
              ? `Hashing... ${hashProgress().toFixed(0)}%\nDecompressing: ${formatChunks(chunksProcessed() ?? 0)}/${formatChunks(chunksTotal() ?? 0)} chunks`
              : `Hashing... ${hashProgress().toFixed(0)}%`
            }
          >
            <span>{hashProgress().toFixed(0)}%</span>
            <span class="relative">
              <span class="font-bold">#</span>
              <Show when={totalHashCount() > 0}>
                <span class="count-badge bg-amber-500">{totalHashCount()}</span>
              </Show>
            </span>
          </span>
        </Show>
        
        {/* Completing state - 100% but hash result not yet received */}
        <Show when={isCompleting()}>
          <span class="chip chip-green" title="Finalizing hash...">
            <HiOutlineCheckCircle class="w-3.5 h-3.5" />
            <span class="relative">
              <HiOutlineHashtag class="w-3.5 h-3.5 font-bold" />
              <Show when={totalHashCount() > 0}>
                <span class="count-badge bg-green-500">{totalHashCount()}</span>
              </Show>
            </span>
          </span>
        </Show>
        
        <Show when={!isHashing() && !isCompleting() && hashState() === "verified"}>
          <button 
            class="chip chip-green-hover disabled:opacity-50" 
            onClick={(e) => { e.stopPropagation(); props.onHash(); }} 
            disabled={props.busy} 
            title={`VERIFIED: Hash matches ${props.fileHash ? "stored hash" : "in history"}\n${totalHashCount()} hash(es) • Click to re-hash`}
          >
            <span class="relative"><span class="absolute -left-0.5">✓</span><span class="ml-1">✓</span></span>
            <span class="relative">
              <span class="font-bold">#</span>
              <span class="count-badge bg-green-500">{totalHashCount()}</span>
            </span>
          </button>
        </Show>
        
        <Show when={!isHashing() && !isCompleting() && hashState() === "failed"}>
          <button 
            class="chip chip-red disabled:opacity-50" 
            onClick={(e) => { e.stopPropagation(); props.onHash(); }} 
            disabled={props.busy} 
            title={`✗ MISMATCH: ${props.fileHash?.algorithm ?? "hash"} does NOT match stored hash\n${totalHashCount()} hash(es) • Click to re-hash`}
          >
            <span class="relative">
              <span class="font-bold">#</span>
              <span class="count-badge bg-red-500">{totalHashCount()}</span>
            </span>
          </button>
        </Show>
        
        <Show when={!isHashing() && !isCompleting() && hashState() === "computed"}>
          <button 
            class="chip chip-cyan disabled:opacity-50" 
            onClick={(e) => { e.stopPropagation(); props.onHash(); }} 
            disabled={props.busy} 
            title={`✓ Computed: ${props.fileHash?.algorithm ?? "hash"} (no stored hash to verify against)\n${totalHashCount()} hash(es) • Click to re-hash`}
          >
            <span class="relative">
              <span class="font-bold">#</span>
              <span class="count-badge bg-cyan-500">{totalHashCount()}</span>
            </span>
          </button>
        </Show>
        
        <Show when={!isHashing() && !isCompleting() && hashState() === "incomplete"}>
          <span 
            class="chip chip-orange" 
            title={`Incomplete: Missing ${props.fileInfo?.ad1?.missing_segments?.length ?? 0} segment(s)\nCannot hash - segments are missing`}
          >
            <HiOutlineExclamationTriangle class="w-3.5 h-3.5" />
            <HiOutlineHashtag class="w-3.5 h-3.5" />
          </span>
        </Show>
        
        <Show when={!isHashing() && !isCompleting() && hashState() === "stored"}>
          <button 
            class="chip chip-purple disabled:opacity-50" 
            onClick={(e) => { e.stopPropagation(); props.onHash(); }} 
            disabled={props.busy} 
            title={`${storedHashCount()} stored hash(es) • Click to verify`}
          >
            <span class="relative">
              <span class="font-bold">#</span>
              <span class="count-badge bg-purple-500">{storedHashCount()}</span>
            </span>
          </button>
        </Show>
        
        <Show when={!isHashing() && !isCompleting() && hashState() === "none"}>
          <button 
            class="chip chip-neutral disabled:opacity-50" 
            onClick={(e) => { e.stopPropagation(); props.onHash(); }} 
            disabled={props.busy} 
            title="Click to hash this file"
          >
            <span class="font-bold">#</span>
          </button>
        </Show>
      </div>
      
      {/* Tooltip on hover */}
      <Show when={props.isHovered && !props.isActive}>
        <FileTooltip 
          file={props.file} 
          fileInfo={props.fileInfo} 
          fileHash={props.fileHash} 
        />
      </Show>
    </div>
  );
}

interface FileTooltipProps {
  file: DiscoveredFile;
  fileInfo: ContainerInfo | undefined;
  fileHash: FileHashInfo | undefined;
}

function FileTooltip(props: FileTooltipProps) {
  return (
    <div class="absolute left-full top-0 ml-2 z-50 min-w-[280px] max-w-[360px] p-3 bg-zinc-800 border border-zinc-700 rounded-lg shadow-xl text-sm">
      <div class="font-semibold text-cyan-400 mb-1">{props.file.container_type}</div>
      <div class="text-xs text-zinc-400 break-all mb-2">{props.file.path}</div>
      <div class="flex justify-between text-xs"><span class="text-zinc-500">Size:</span><span class="text-zinc-300">{formatBytes(props.file.size)}</span></div>
      
      <Show when={props.file.segment_count}>
        <div class="flex justify-between text-xs"><span class="text-zinc-500">Segments:</span><span class="text-zinc-300">{props.file.segment_count}</span></div>
      </Show>
      
      <Show when={props.fileInfo}>
        <div class="my-2 border-t border-zinc-700" />
        
        <Show when={props.fileInfo!.ad1}>
          <div class="flex justify-between text-xs"><span class="text-zinc-500">Items:</span><span class="text-zinc-300">{props.fileInfo!.ad1!.item_count}</span></div>
          <Show when={props.fileInfo!.ad1!.companion_log?.case_number}>
            <div class="flex justify-between text-xs"><span class="text-zinc-500">Case:</span><span class="text-zinc-300">{props.fileInfo!.ad1!.companion_log!.case_number}</span></div>
          </Show>
          <Show when={props.fileInfo!.ad1!.companion_log?.evidence_number}>
            <div class="flex justify-between text-xs"><span class="text-zinc-500">Evidence:</span><span class="text-zinc-300">{props.fileInfo!.ad1!.companion_log!.evidence_number}</span></div>
          </Show>
          <Show when={props.fileInfo!.ad1!.volume?.filesystem}>
            <div class="flex justify-between text-xs"><span class="text-zinc-500">FS:</span><span class="text-zinc-300">{props.fileInfo!.ad1!.volume!.filesystem}</span></div>
          </Show>
          <div class="flex justify-between text-xs"><span class="text-zinc-500">Source:</span><span class="text-zinc-300 truncate ml-2">{props.fileInfo!.ad1!.logical.data_source_name}</span></div>
        </Show>
        
        <Show when={props.fileInfo!.e01}>
          <div class="flex justify-between text-xs"><span class="text-zinc-500">Format:</span><span class="text-zinc-300">{props.fileInfo!.e01!.format_version}</span></div>
          <div class="flex justify-between text-xs"><span class="text-zinc-500">Compression:</span><span class="text-zinc-300">{props.fileInfo!.e01!.compression}</span></div>
          <Show when={props.fileInfo!.e01!.case_number}>
            <div class="flex justify-between text-xs"><span class="text-zinc-500">Case:</span><span class="text-zinc-300">{props.fileInfo!.e01!.case_number}</span></div>
          </Show>
        </Show>
        
        <Show when={props.fileInfo!.raw}>
          <div class="flex justify-between text-xs"><span class="text-zinc-500">Segments:</span><span class="text-zinc-300">{props.fileInfo!.raw!.segment_count}</span></div>
        </Show>
        
        <Show when={(props.fileInfo?.e01?.stored_hashes?.length ?? 0) > 0 || (props.fileInfo?.companion_log?.stored_hashes?.length ?? 0) > 0}>
          <div class="my-2 border-t border-zinc-700" />
          <div class="flex items-center gap-1 text-xs font-semibold text-zinc-400 mb-1">
            <HiOutlineDocumentText class="w-3.5 h-3.5" /> Stored Hashes
          </div>
          
          <Show when={(props.fileInfo?.e01?.stored_hashes?.length ?? 0) > 0}>
            {props.fileInfo!.e01!.stored_hashes!.map((sh) => (
              <div class="flex items-center gap-2 text-xs py-0.5">
                <span class="text-cyan-400 font-mono">{sh.algorithm}</span>
                <code class="text-zinc-400 font-mono truncate">{sh.hash.substring(0, 16)}...</code>
                <Show when={sh.verified === true}><HiOutlineCheckCircle class="w-3.5 h-3.5 text-green-400" /></Show>
                <Show when={sh.timestamp}><span class="text-zinc-500 text-[10px]">{sh.timestamp}</span></Show>
              </div>
            ))}
          </Show>
          
          <Show when={(props.fileInfo?.companion_log?.stored_hashes?.length ?? 0) > 0}>
            {props.fileInfo!.companion_log!.stored_hashes.map((sh) => (
              <div class="flex items-center gap-2 text-xs py-0.5">
                <span class="text-cyan-400 font-mono">{sh.algorithm}</span>
                <code class="text-zinc-400 font-mono truncate">{sh.hash.substring(0, 16)}...</code>
                <Show when={sh.verified === true}><HiOutlineCheckCircle class="w-3.5 h-3.5 text-green-400" /></Show>
                <Show when={sh.timestamp}><span class="text-zinc-500 text-[10px]">{sh.timestamp}</span></Show>
              </div>
            ))}
          </Show>
        </Show>
      </Show>
      
      <Show when={props.fileHash}>
        <div class="my-2 border-t border-zinc-700" />
        <div class="flex flex-col gap-1">
          <span class="flex items-center gap-1 text-xs text-cyan-400"><HiOutlineLockClosed class="w-3.5 h-3.5" /> {props.fileHash!.algorithm}</span>
          <code class="text-xs text-zinc-300 font-mono break-all">{props.fileHash!.hash}</code>
        </div>
      </Show>
    </div>
  );
}
