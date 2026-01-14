// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, Show, createSignal, createMemo } from "solid-js";
import type { ParsedMetadata, MetadataField } from "./HexViewer";
import type { ContainerInfo } from "../types";
import type { SelectedEntry } from "./EvidenceTree";
import { formatBytes, formatOffsetLabel } from "../utils";
import {
  HiOutlineClipboardDocument,
  HiOutlineMapPin,
  HiOutlineCircleStack,
  HiOutlineArchiveBox,
  HiOutlineArrowPath,
  HiOutlineFolder,
  HiOutlineDocument,
  HiOutlineArrowRightOnRectangle,
  HiOutlineUserGroup,
} from "./icons";

// File info passed from parent
interface FileInfo {
  path: string;
  filename: string;
  size: number;
  created?: string;
  modified?: string;
  container_type?: string;
  segment_count?: number;
}

interface MetadataPanelProps {
  metadata: ParsedMetadata | null;
  /** File information from discovery */
  fileInfo?: FileInfo | null;
  /** Full container info with details */
  containerInfo?: ContainerInfo | null;
  /** Currently selected/navigated offset (to highlight in list) */
  selectedOffset?: number | null;
  /** Callback when a region is clicked (to jump to offset in hex viewer) */
  onRegionClick?: (offset: number, size?: number) => void;
  /** Currently selected entry from evidence tree (for hex location display) */
  selectedEntry?: SelectedEntry | null;
}

export function MetadataPanel(props: MetadataPanelProps) {
  // Track expanded categories
  const [expandedCategories, setExpandedCategories] = createSignal<Set<string>>(
    new Set(["Format", "Case Info", "Hashes", "Container", "_container", "_hexLocations"]) // Start with key sections expanded
  );
  
  const toggleCategory = (category: string) => {
    setExpandedCategories(prev => {
      const next = new Set(prev);
      if (next.has(category)) {
        next.delete(category);
      } else {
        next.add(category);
      }
      return next;
    });
  };
  
  const isExpanded = (category: string) => expandedCategories().has(category);
  
  // Preferred category order
  const CATEGORY_ORDER = [
    "Format",
    "Case Info", 
    "Acquisition",
    "Device",
    "Volume",
    "Hashes",
    "Errors",
    "Sections",
    "General"
  ];
  
  // Group fields by category
  const groupedFields = () => {
    const meta = props.metadata;
    if (!meta?.fields.length) return new Map<string, MetadataField[]>();
    
    const groups = new Map<string, MetadataField[]>();
    for (const field of meta.fields) {
      const category = field.category || "General";
      if (!groups.has(category)) {
        groups.set(category, []);
      }
      groups.get(category)!.push(field);
    }
    
    // Sort by preferred order
    const sortedGroups = new Map<string, MetadataField[]>();
    for (const cat of CATEGORY_ORDER) {
      if (groups.has(cat)) {
        sortedGroups.set(cat, groups.get(cat)!);
        groups.delete(cat);
      }
    }
    // Add remaining
    for (const [cat, fields] of groups) {
      sortedGroups.set(cat, fields);
    }
    
    return sortedGroups;
  };
  
  // Get EWF info from container (either E01 or L01) - use memo for reactivity
  const ewfInfo = createMemo(() => {
    const info = props.containerInfo;
    if (!info) return null;
    return info.e01 || info.l01 || null;
  });
  
  const handleRowClick = (offset: number | undefined | null, size?: number) => {
    if (offset !== undefined && offset !== null && props.onRegionClick) {
      props.onRegionClick(offset, size);
    }
  };
  
  // Reusable style constants
  const rowBase = "grid gap-2 py-1 px-2 text-[10px] items-baseline transition-colors hover:bg-zinc-800/50";
  const rowGrid = "grid-cols-[minmax(80px,1fr)_minmax(100px,2fr)_auto]";
  const rowClickable = "cursor-pointer hover:bg-cyan-900/30";
  const keyStyle = "text-zinc-500 truncate";
  const valueStyle = "font-mono text-zinc-300 truncate";
  const offsetStyle = "font-mono text-[9px] text-zinc-500 whitespace-nowrap";
  const offsetClickable = "text-cyan-400";
  const categoryHeader = "flex items-center gap-1.5 py-1.5 px-2 bg-zinc-800/50 cursor-pointer select-none hover:bg-zinc-800 transition-colors";

  return (
    <div class="flex flex-col h-full bg-zinc-900 text-xs overflow-auto">
      <Show when={!props.metadata && !props.fileInfo}>
        <div class="flex flex-col items-center justify-center py-8 text-zinc-500">
          <HiOutlineClipboardDocument class="w-8 h-8 mb-2 opacity-40" />
          <span class="text-xs">No metadata</span>
        </div>
      </Show>
      
      {/* Format header - prominent display */}
      <Show when={props.metadata}>
        {meta => (
          <div class="panel-header">
            <span class="text-zinc-500 text-[10px]">Format</span>
            <span class="font-semibold text-cyan-400">{meta().format}</span>
            <Show when={meta().version}>
              <span class="text-[10px] text-zinc-500">v{meta().version}</span>
            </Show>
          </div>
        )}
      </Show>
      
      {/* File info row */}
      <Show when={props.fileInfo}>
        {info => (
          <div class="border-b border-zinc-700">
            <div class={`${rowBase} ${rowGrid}`}>
              <span class={keyStyle}>File</span>
              <span class={valueStyle} title={info().path}>{info().filename}</span>
              <span class={offsetStyle}></span>
            </div>
            <div class={`${rowBase} ${rowGrid}`}>
              <span class={keyStyle}>Size</span>
              <span class={valueStyle}>{formatBytes(info().size)}</span>
              <span class={offsetStyle}></span>
            </div>
            <Show when={info().segment_count && info().segment_count! > 1}>
              <div class={`${rowBase} ${rowGrid}`}>
                <span class={keyStyle}>Segments</span>
                <span class={valueStyle}>{info().segment_count}</span>
                <span class={offsetStyle}></span>
              </div>
            </Show>
          </div>
        )}
      </Show>
      
      {/* 📍 SELECTED ENTRY HEX LOCATIONS Section */}
      <Show when={props.selectedEntry}>
        {entry => (
          <div class="bg-zinc-800/80 border-2 border-cyan-900/50">
            <div 
              class={`${categoryHeader} bg-cyan-900/30`}
              onClick={() => toggleCategory("_hexLocations")}
            >
              <span class="text-[10px] text-zinc-500 w-3">
                {isExpanded("_hexLocations") ? "▾" : "▸"}
              </span>
              <span class="flex items-center gap-1 text-[10px] font-medium text-zinc-300 flex-1">
                <HiOutlineMapPin class="w-3.5 h-3.5" /> HEX LOCATIONS - {entry().isDir ? <HiOutlineFolder class="w-3.5 h-3.5 inline" /> : <HiOutlineDocument class="w-3.5 h-3.5 inline" />} {entry().name}
              </span>
            </div>
            
            <Show when={isExpanded("_hexLocations")}>
              <div class="py-1">
                {/* Entry Type */}
                <div class={`${rowBase} ${rowGrid}`}>
                  <span class={keyStyle}>TYPE</span>
                  <span class={valueStyle}>{entry().isDir ? "Folder" : "File"}</span>
                  <span class={offsetStyle}></span>
                </div>
                
                {/* Entry Path */}
                <div class={`${rowBase} ${rowGrid}`}>
                  <span class={keyStyle}>PATH</span>
                  <span class={`${valueStyle} text-[10px] break-all`} title={entry().entryPath}>{entry().entryPath}</span>
                  <span class={offsetStyle}></span>
                </div>
                
                {/* Decompressed Size */}
                <div class={`${rowBase} ${rowGrid}`}>
                  <span class={keyStyle}>SIZE</span>
                  <span class={valueStyle}>{formatBytes(entry().size)}</span>
                  <span class={offsetStyle}></span>
                </div>
                
                {/* Item Header Address - clickable to jump */}
                <Show when={entry().itemAddr != null}>
                  <div 
                    class={`${rowBase} ${rowGrid} ${rowClickable} bg-green-900/20 border-l-2 border-green-500`}
                    onClick={() => handleRowClick(entry().itemAddr)}
                    title="Click to view item header in hex"
                  >
                    <span class="flex items-center gap-0.5">{keyStyle && <HiOutlineMapPin class="w-3 h-3 text-green-400" />} ITEM HEADER</span>
                    <span class={`${valueStyle} text-cyan-400 font-bold`}>0x{entry().itemAddr!.toString(16).toUpperCase().padStart(8, '0')}</span>
                    <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(entry().itemAddr!)}</span>
                  </div>
                </Show>
                
                {/* Metadata Address */}
                <Show when={entry().metadataAddr != null}>
                  <div 
                    class={`${rowBase} ${rowGrid} ${rowClickable}`}
                    onClick={() => handleRowClick(entry().metadataAddr)}
                    title="Click to view metadata in hex"
                  >
                    <span class="flex items-center gap-0.5">{keyStyle && <HiOutlineClipboardDocument class="w-3 h-3 text-zinc-400" />} METADATA</span>
                    <span class={`${valueStyle} text-cyan-400 font-bold`}>0x{entry().metadataAddr!.toString(16).toUpperCase().padStart(8, '0')}</span>
                    <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(entry().metadataAddr!)}</span>
                  </div>
                </Show>
                
                {/* Data Start Address (for files) */}
                <Show when={entry().dataAddr != null && !entry().isDir}>
                  <div 
                    class={`${rowBase} ${rowGrid} ${rowClickable} bg-blue-900/20 border-l-2 border-blue-500`}
                    onClick={() => handleRowClick(entry().dataAddr)}
                    title="Click to view compressed data start in hex"
                  >
                    <span class="flex items-center gap-0.5">{keyStyle && <HiOutlineCircleStack class="w-3 h-3 text-blue-400" />} DATA START</span>
                    <span class={`${valueStyle} text-cyan-400 font-bold`}>0x{entry().dataAddr!.toString(16).toUpperCase().padStart(8, '0')}</span>
                    <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(entry().dataAddr!)}</span>
                  </div>
                </Show>
                
                {/* Compressed Size (for files) */}
                <Show when={entry().compressedSize != null && !entry().isDir}>
                  <div class={`${rowBase} ${rowGrid}`}>
                    <span class="flex items-center gap-0.5">{keyStyle && <HiOutlineArchiveBox class="w-3 h-3 text-zinc-400" />} COMPRESSED</span>
                    <span class={valueStyle}>{formatBytes(entry().compressedSize!)}</span>
                    <span class={offsetStyle}></span>
                  </div>
                </Show>
                
                {/* Data End Address (for files) */}
                <Show when={entry().dataEndAddr != null && !entry().isDir}>
                  <div 
                    class={`${rowBase} ${rowGrid} ${rowClickable} bg-red-900/20 border-l-2 border-red-500`}
                    onClick={() => handleRowClick(entry().dataEndAddr)}
                    title="Click to view compressed data end in hex"
                  >
                    <span class="flex items-center gap-0.5">{keyStyle && <HiOutlineArrowRightOnRectangle class="w-3 h-3 text-red-400" />} DATA END</span>
                    <span class={`${valueStyle} text-cyan-400 font-bold`}>0x{entry().dataEndAddr!.toString(16).toUpperCase().padStart(8, '0')}</span>
                    <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(entry().dataEndAddr!)}</span>
                  </div>
                </Show>
                
                {/* First Child Address (for folders) */}
                <Show when={entry().firstChildAddr != null && entry().isDir}>
                  <div 
                    class={`${rowBase} ${rowGrid} ${rowClickable}`}
                    onClick={() => handleRowClick(entry().firstChildAddr)}
                    title="Click to view first child item in hex"
                  >
                    <span class="flex items-center gap-0.5">{keyStyle && <HiOutlineUserGroup class="w-3 h-3 text-zinc-400" />} FIRST CHILD</span>
                    <span class={`${valueStyle} text-cyan-400 font-bold`}>0x{entry().firstChildAddr!.toString(16).toUpperCase().padStart(8, '0')}</span>
                    <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(entry().firstChildAddr!)}</span>
                  </div>
                </Show>
                
                {/* Container Path */}
                <div class={`${rowBase} ${rowGrid} border-t border-zinc-700 mt-1 pt-1.5`}>
                  <span class={keyStyle}>CONTAINER</span>
                  <span class={valueStyle} title={entry().containerPath}>
                    {entry().containerPath.split('/').pop() || entry().containerPath}
                  </span>
                  <span class={offsetStyle}></span>
                </div>
              </div>
            </Show>
          </div>
        )}
      </Show>
      
      {/* Debug: Show if containerInfo is missing */}
      <Show when={!ewfInfo() && props.fileInfo?.container_type?.toLowerCase().includes('e01')}>
        <div class="border-b border-zinc-700">
          <div class={categoryHeader}>
            <span class="flex items-center gap-1 text-[10px] font-medium text-zinc-300">
              <HiOutlineArrowPath class="w-3.5 h-3.5 animate-spin" /> Loading container info...
            </span>
          </div>
        </div>
      </Show>
      
      {/* 📋 CONTAINER DETAILS Section */}
      <Show when={ewfInfo()}>
        {ewf => {
          // Use actual section offsets from backend, fall back to typical defaults
          // EWF structure: signature(0x0) + segment(0x9) + section_header(0xD) + section_data
          const headerOffset = ewf().header_section_offset ?? 0xD;
          const volumeOffset = ewf().volume_section_offset ?? 0x59;
          // hashOffset and digestOffset available for future hash navigation
          // const hashOffset = ewf().hash_section_offset;
          // const digestOffset = ewf().digest_section_offset;
          
          // Section data starts 76 bytes after section header
          // Field offsets within volume data: chunk_count +4, sectors_per_chunk +8, bytes_per_sector +12, compression +56
          const volumeDataStart = volumeOffset + 76;
          // Header section data is zlib-compressed - show where the compressed blob starts
          const headerDataStart = headerOffset + 76;
          
          return (
          <div class="border-b border-zinc-700">
            <div 
              class={categoryHeader}
              onClick={() => toggleCategory("_container")}
            >
              <span class="text-[10px] text-zinc-500 w-3">
                {isExpanded("_container") ? "▾" : "▸"}
              </span>
              <span class="flex items-center gap-1 text-[10px] font-medium text-zinc-300 flex-1">
                <HiOutlineClipboardDocument class="w-3.5 h-3.5" /> CONTAINER DETAILS
              </span>
            </div>
            
            <Show when={isExpanded("_container")}>
              <div class="py-0.5">
                {/* Format Info - links to signature */}
                <div 
                  class={`${rowBase} ${rowGrid} ${rowClickable}`}
                  onClick={() => handleRowClick(0x0, 8)}
                  title="Click to view EVF signature at 0x0"
                >
                  <span class={keyStyle}>FORMAT</span>
                  <span class={valueStyle}>{ewf().format_version}</span>
                  <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(0x0)}</span>
                </div>
                
                {/* Segments - links to segment number field */}
                <div 
                  class={`${rowBase} ${rowGrid} ${rowClickable}`}
                  onClick={() => handleRowClick(0x9, 2)}
                  title="Click to view segment number at 0x9"
                >
                  <span class={keyStyle}>SEGMENTS</span>
                  <span class={valueStyle}>{ewf().segment_count}</span>
                  <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(0x9)}</span>
                </div>
                
                {/* Total Size */}
                <div class={`${rowBase} ${rowGrid}`}>
                  <span class={keyStyle}>TOTAL SIZE</span>
                  <span class={valueStyle}>{formatBytes(ewf().total_size)}</span>
                  <span class={offsetStyle}></span>
                </div>
                
                {/* Compression - in volume section at offset +56 (0x38) */}
                <div 
                  class={`${rowBase} ${rowGrid} ${rowClickable}`}
                  onClick={() => handleRowClick(volumeDataStart + 0x38, 1)}
                  title="Click to view compression level in volume section"
                >
                  <span class={keyStyle}>COMPRESSION</span>
                  <span class={valueStyle}>{ewf().compression || "Unknown"}</span>
                  <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(volumeDataStart + 0x38)}</span>
                </div>
                
                {/* Bytes per sector - in volume section at offset +12 (0x0C) */}
                <div 
                  class={`${rowBase} ${rowGrid} ${rowClickable}`}
                  onClick={() => handleRowClick(volumeDataStart + 0x0C, 4)}
                  title="Click to view bytes/sector in volume section"
                >
                  <span class={keyStyle}>BYTES/SECTOR</span>
                  <span class={valueStyle}>{ewf().bytes_per_sector}</span>
                  <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(volumeDataStart + 0x0C)}</span>
                </div>
                
                {/* Sectors per chunk - in volume section at offset +8 */}
                <div 
                  class={`${rowBase} ${rowGrid} ${rowClickable}`}
                  onClick={() => handleRowClick(volumeDataStart + 0x08, 4)}
                  title="Click to view sectors/chunk in volume section"
                >
                  <span class={keyStyle}>SECTORS/CHUNK</span>
                  <span class={valueStyle}>{ewf().sectors_per_chunk}</span>
                  <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(volumeDataStart + 0x08)}</span>
                </div>
                
                {/* Case Info - stored as zlib-compressed data in header section */}
                <Show when={ewf().case_number}>
                  <div 
                    class={`${rowBase} ${rowGrid} ${rowClickable}`}
                    onClick={() => handleRowClick(headerDataStart)}
                    title="Click to view compressed header data (zlib stream)"
                  >
                    <span class={keyStyle}>CASE #</span>
                    <span class={valueStyle}>{ewf().case_number}</span>
                    <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(headerDataStart)} <HiOutlineArchiveBox class="w-3 h-3 inline" /></span>
                  </div>
                </Show>
                <Show when={ewf().evidence_number}>
                  <div 
                    class={`${rowBase} ${rowGrid} ${rowClickable}`}
                    onClick={() => handleRowClick(headerDataStart)}
                    title="Click to view compressed header data (zlib stream)"
                  >
                    <span class={keyStyle}>EVIDENCE #</span>
                    <span class={valueStyle}>{ewf().evidence_number}</span>
                    <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(headerDataStart)} <HiOutlineArchiveBox class="w-3 h-3 inline" /></span>
                  </div>
                </Show>
                <Show when={ewf().examiner_name}>
                  <div 
                    class={`${rowBase} ${rowGrid} ${rowClickable}`}
                    onClick={() => handleRowClick(headerDataStart)}
                    title="Click to view compressed header data (zlib stream)"
                  >
                    <span class={keyStyle}>EXAMINER</span>
                    <span class={valueStyle}>{ewf().examiner_name}</span>
                    <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(headerDataStart)} <HiOutlineArchiveBox class="w-3 h-3 inline" /></span>
                  </div>
                </Show>
                <Show when={ewf().acquiry_date}>
                  <div 
                    class={`${rowBase} ${rowGrid} ${rowClickable}`}
                    onClick={() => handleRowClick(headerDataStart)}
                    title="Click to view compressed header data (zlib stream)"
                  >
                    <span class={keyStyle}>ACQUIRED</span>
                    <span class={valueStyle}>{ewf().acquiry_date}</span>
                    <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(headerDataStart)} <HiOutlineArchiveBox class="w-3 h-3 inline" /></span>
                  </div>
                </Show>
                <Show when={ewf().system_date}>
                  <div 
                    class={`${rowBase} ${rowGrid} ${rowClickable}`}
                    onClick={() => handleRowClick(headerDataStart)}
                    title="Click to view compressed header data (zlib stream)"
                  >
                    <span class={keyStyle}>SYSTEM DATE</span>
                    <span class={valueStyle}>{ewf().system_date}</span>
                    <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(headerDataStart)} <HiOutlineArchiveBox class="w-3 h-3 inline" /></span>
                  </div>
                </Show>
                <Show when={ewf().description}>
                  <div 
                    class={`${rowBase} ${rowGrid} ${rowClickable}`}
                    onClick={() => handleRowClick(headerDataStart)}
                    title="Click to view compressed header data (zlib stream)"
                  >
                    <span class={keyStyle}>DESCRIPTION</span>
                    <span class={valueStyle}>{ewf().description}</span>
                    <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(headerDataStart)} <HiOutlineArchiveBox class="w-3 h-3 inline" /></span>
                  </div>
                </Show>
                <Show when={ewf().notes}>
                  <div 
                    class={`${rowBase} ${rowGrid} ${rowClickable}`}
                    onClick={() => handleRowClick(headerDataStart)}
                    title="Click to view compressed header data (zlib stream)"
                  >
                    <span class={keyStyle}>NOTES</span>
                    <span class={valueStyle}>{ewf().notes}</span>
                    <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(headerDataStart)} <HiOutlineArchiveBox class="w-3 h-3 inline" /></span>
                  </div>
                </Show>
                
                {/* Device Info */}
                <Show when={ewf().model}>
                  <div 
                    class={`${rowBase} ${rowGrid} ${rowClickable}`}
                    onClick={() => handleRowClick(headerDataStart)}
                    title="Click to view compressed header data (zlib stream)"
                  >
                    <span class={keyStyle}>MODEL</span>
                    <span class={valueStyle}>{ewf().model}</span>
                    <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(headerDataStart)} <HiOutlineArchiveBox class="w-3 h-3 inline" /></span>
                  </div>
                </Show>
                <Show when={ewf().serial_number}>
                  <div 
                    class={`${rowBase} ${rowGrid} ${rowClickable}`}
                    onClick={() => handleRowClick(headerDataStart)}
                    title="Click to view compressed header data (zlib stream)"
                  >
                    <span class={keyStyle}>SERIAL #</span>
                    <span class={valueStyle}>{ewf().serial_number}</span>
                    <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(headerDataStart)} <HiOutlineArchiveBox class="w-3 h-3 inline" /></span>
                  </div>
                </Show>
                
                {/* Stored Hashes - raw bytes in hash/digest section (NOT compressed) */}
                <Show when={ewf().stored_hashes && ewf().stored_hashes!.length > 0}>
                  <For each={ewf().stored_hashes}>
                    {hash => {
                      const hasOffset = hash.offset != null && hash.size != null;
                      return (
                        <div 
                          class={`${rowBase} ${rowGrid} ${hasOffset ? rowClickable : ''}`}
                          onClick={() => {
                            if (hasOffset) {
                              handleRowClick(hash.offset!, hash.size!);
                            }
                          }}
                        >
                          <span class={keyStyle}>🔒 {hash.algorithm.toUpperCase()}</span>
                          <span class={`${valueStyle} text-[10px] text-zinc-500 select-all`}>{hash.hash}</span>
                          <span class={`${offsetStyle} ${hasOffset ? offsetClickable : ''}`}>
                            {hasOffset ? formatOffsetLabel(hash.offset!) : ''}
                          </span>
                        </div>
                      );
                    }}
                  </For>
                </Show>
              </div>
            </Show>
          </div>
          );
        }}
      </Show>
      
      {/* Categories */}
      <Show when={props.metadata}>
        <div class="flex-1">
          <For each={[...groupedFields().entries()]}>
            {([category, fields]) => (
              <div class="border-b border-zinc-700 last:border-b-0">
                <div 
                  class={categoryHeader}
                  onClick={() => toggleCategory(category)}
                >
                  <span class="text-[10px] text-zinc-500 w-3">
                    {isExpanded(category) ? "▾" : "▸"}
                  </span>
                  <span class="text-[10px] font-medium text-zinc-300 flex-1">{category}</span>
                  <span class="text-[9px] text-zinc-500">{fields.length}</span>
                </div>
                
                <Show when={isExpanded(category)}>
                  <div class="py-0.5">
                    <For each={fields}>
                      {field => {
                        const hasOffset = field.source_offset !== undefined && field.source_offset !== null;
                        return (
                          <div 
                            class={`${rowBase} ${rowGrid} ${hasOffset ? rowClickable : ''}`}
                            onClick={(e) => {
                              e.stopPropagation();
                              handleRowClick(field.source_offset);
                            }}
                            title={hasOffset ? `Click to view hex at ${formatOffsetLabel(field.source_offset)}` : field.value}
                          >
                            <span class={keyStyle}>{field.key}</span>
                            <span class={valueStyle}>{field.value}</span>
                            <span class={`${offsetStyle} ${hasOffset ? offsetClickable : ''}`}>{formatOffsetLabel(field.source_offset)}</span>
                          </div>
                        );
                      }}
                    </For>
                  </div>
                </Show>
              </div>
            )}
          </For>
        </div>
      </Show>
      
      {/* Header Regions - compact list */}
      <Show when={props.metadata?.regions.length}>
        <div class="border-b border-zinc-700">
          <div 
            class={categoryHeader}
            onClick={() => toggleCategory("_regions")}
          >
            <span class="text-[10px] text-zinc-500 w-3">
              {isExpanded("_regions") ? "▾" : "▸"}
            </span>
            <span class="text-[10px] font-medium text-zinc-300 flex-1">Hex Regions</span>
            <span class="text-[9px] text-zinc-500">{props.metadata!.regions.length}</span>
          </div>
          
          <Show when={isExpanded("_regions")}>
            <div class="py-0.5">
              <For each={props.metadata!.regions}>
                {region => (
                  <div 
                    class={`${rowBase} ${rowGrid} ${rowClickable}`}
                    onClick={(e) => {
                      e.stopPropagation();
                      handleRowClick(region.start);
                    }}
                    title={region.description}
                  >
                    <span class={keyStyle}>{region.name}</span>
                    <span class={valueStyle}>{region.end - region.start} bytes</span>
                    <span class={`${offsetStyle} ${offsetClickable}`}>{formatOffsetLabel(region.start)}</span>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </div>
      </Show>
    </div>
  );
}
