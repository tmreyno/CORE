// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, createMemo, For, Show, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { DiscoveredFile, FileChunk, HeaderRegion, MetadataField, ParsedMetadata, FileTypeInfo } from "../types";
import { formatOffset, byteToHex, byteToAscii, formatBytes } from "../utils";

// Re-export viewer types for backward compatibility
export type { FileChunk, HeaderRegion, MetadataField, ParsedMetadata, FileTypeInfo };

// --- Constants ---
const BYTES_PER_LINE = 16;
const DEFAULT_CHUNK_SIZE = 4096; // 256 lines
const MIN_CHUNK_SIZE = 256;
const MAX_CHUNK_SIZE = 65536;

// Map color classes to actual colors (with very light transparency)
const COLOR_MAP: Record<string, string> = {
  "region-signature": "rgba(239, 68, 68, 0.15)",    // Red - file signatures
  "region-header": "rgba(249, 115, 22, 0.15)",      // Orange - headers
  "region-segment": "rgba(249, 115, 22, 0.15)",     // Orange - segments
  "region-metadata": "rgba(234, 179, 8, 0.15)",     // Yellow - metadata
  "region-data": "rgba(34, 197, 94, 0.15)",         // Green - data sections
  "region-checksum": "rgba(59, 130, 246, 0.15)",    // Blue - checksums
  "region-reserved": "rgba(139, 92, 246, 0.15)",    // Purple - reserved
  "region-footer": "rgba(236, 72, 153, 0.15)",      // Pink - footers
};

// Selected/navigated location color (darker transparent green)
const NAVIGATED_COLOR = "rgba(34, 197, 94, 0.4)";  // Darker green for navigated location

// Helper to get actual color from color_class
function getRegionColor(colorClass: string): string {
  return COLOR_MAP[colorClass] || "#6a6a7a";
}

// --- Component Props ---

interface HexViewerProps {
  file: DiscoveredFile;
  /** Callback when metadata is loaded (for right panel) */
  onMetadataLoaded?: (metadata: ParsedMetadata | null) => void;
  /** Callback to expose navigation function to parent (offset, optional size in bytes) */
  onNavigatorReady?: (navigateTo: (offset: number, size?: number) => void) => void;
}

export function HexViewer(props: HexViewerProps) {
  // Viewer state
  const [chunk, setChunk] = createSignal<FileChunk | null>(null);
  const [metadata, setMetadata] = createSignal<ParsedMetadata | null>(null);
  const [fileType, setFileType] = createSignal<FileTypeInfo | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  
  // Navigation state
  const [currentOffset, setCurrentOffset] = createSignal(0);
  const [chunkSize, setChunkSize] = createSignal(DEFAULT_CHUNK_SIZE);
  const [gotoOffset, setGotoOffset] = createSignal("");
  
  // View options
  const [showAscii, setShowAscii] = createSignal(true);
  const [highlightRegions, setHighlightRegions] = createSignal(true);
  const [showAddress, _setShowAddress] = createSignal(true);
  
  // Selection/focus state
  const [selectedRegion, setSelectedRegion] = createSignal<HeaderRegion | null>(null);
  const [hoveredOffset, setHoveredOffset] = createSignal<number | null>(null);
  // Track navigated location - offset and size (default to 1 byte if size not specified)
  const [navigatedRange, setNavigatedRange] = createSignal<{ offset: number; size: number } | null>(null);
  
  // Load file data
  const loadChunk = async (chunkOffset: number) => {
    // Validate offset before making Tauri call
    const validOffset = typeof chunkOffset === 'number' && !isNaN(chunkOffset) && chunkOffset >= 0
      ? Math.floor(chunkOffset)
      : 0;
    
    setLoading(true);
    setError(null);
    
    try {
      const result = await invoke<FileChunk>("viewer_read_chunk", {
        path: props.file.path,
        offset: validOffset,
        size: chunkSize()
      });
      setChunk(result);
      setCurrentOffset(result.offset);
    } catch (e) {
      setError(`Failed to load file: ${e}`);
      setChunk(null);
    } finally {
      setLoading(false);
    }
  };
  
  // Load metadata and type on file change
  createEffect(() => {
    const file = props.file;
    if (!file) return;
    
    // Reset state
    setChunk(null);
    setMetadata(null);
    setFileType(null);
    setCurrentOffset(0);
    setError(null);
    
    // Load initial data
    loadChunk(0);
    
    // Load file type
    invoke<FileTypeInfo>("viewer_detect_type", { path: file.path })
      .then(setFileType)
      .catch(e => console.warn("Failed to detect file type:", e));
    
    // Load metadata (for forensic formats)
    invoke<ParsedMetadata>("viewer_parse_header", { path: file.path })
      .then(meta => {
        setMetadata(meta);
        if (props.onMetadataLoaded) {
          props.onMetadataLoaded(meta);
        }
      })
      .catch(() => {
        // Not all files have parseable metadata
        setMetadata(null);
        if (props.onMetadataLoaded) {
          props.onMetadataLoaded(null);
        }
      });
  });
  
  // Expose navigation function to parent when ready (only once when mounted)
  if (props.onNavigatorReady) {
    props.onNavigatorReady((offset: number, size?: number) => {
      // Ensure offset is valid before loading
      if (typeof offset === 'number' && !isNaN(offset) && offset >= 0) {
        // Track the navigated location with size (default to 4 bytes for visibility)
        setNavigatedRange({ offset, size: size ?? 4 });
        loadChunk(offset);
      }
    });
  }
  
  // Navigation handlers
  const goToStart = () => loadChunk(0);
  const goToEnd = () => {
    const c = chunk();
    if (c) {
      const lastOffset = Math.max(0, c.total_size - chunkSize());
      loadChunk(lastOffset);
    }
  };
  const goPrev = () => {
    const newOffset = Math.max(0, currentOffset() - chunkSize());
    loadChunk(newOffset);
  };
  const goNext = () => {
    const c = chunk();
    if (c && c.has_more) {
      loadChunk(currentOffset() + chunkSize());
    }
  };
  
  const handleGotoOffset = () => {
    const input = gotoOffset().trim();
    let offset: number;
    
    // Support hex (0x...) or decimal
    if (input.toLowerCase().startsWith("0x")) {
      offset = parseInt(input, 16);
    } else {
      offset = parseInt(input, 10);
    }
    
    if (isNaN(offset) || offset < 0) {
      setError("Invalid offset");
      return;
    }
    
    const c = chunk();
    if (c && offset >= c.total_size) {
      setError("Offset exceeds file size");
      return;
    }
    
    loadChunk(offset);
    setGotoOffset("");
  };
  
  // Render hex lines with highlight data (memo to track metadata changes)
  const hexLines = createMemo(() => {
    const c = chunk();
    const meta = metadata();  // Track metadata changes
    const doHighlight = highlightRegions();
    
    if (!c) return [];
    
    const lines: { 
      offset: number; 
      bytes: { value: number; color: string | null; region: HeaderRegion | null }[] 
    }[] = [];
    
    for (let i = 0; i < c.bytes.length; i += BYTES_PER_LINE) {
      const lineBytes = c.bytes.slice(i, i + BYTES_PER_LINE);
      const lineOffset = c.offset + i;
      
      lines.push({
        offset: lineOffset,
        bytes: lineBytes.map((byte, j) => {
          const byteOffset = lineOffset + j;
          let color: string | null = null;
          let region: HeaderRegion | null = null;
          
          if (doHighlight && meta) {
            for (const r of meta.regions) {
              if (byteOffset >= r.start && byteOffset < r.end) {
                color = getRegionColor(r.color_class);
                region = r;
                break;
              }
            }
          }
          
          return { value: byte, color, region };
        })
      });
    }
    return lines;
  });
  
  // Keyboard navigation
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "PageDown" || (e.key === "ArrowDown" && e.ctrlKey)) {
      e.preventDefault();
      goNext();
    } else if (e.key === "PageUp" || (e.key === "ArrowUp" && e.ctrlKey)) {
      e.preventDefault();
      goPrev();
    } else if (e.key === "Home" && e.ctrlKey) {
      e.preventDefault();
      goToStart();
    } else if (e.key === "End" && e.ctrlKey) {
      e.preventDefault();
      goToEnd();
    }
  };
  
  // Cleanup
  onCleanup(() => {
    // Any cleanup needed
  });
  
  return (
    <div class="flex flex-col h-full bg-zinc-900 text-zinc-200 font-mono text-sm" tabIndex={0} onKeyDown={handleKeyDown}>
      {/* Toolbar */}
      <div class="flex items-center gap-2 px-3 py-2 bg-zinc-800 border-b border-zinc-700 flex-wrap">
        <div class="flex items-center gap-2">
          {/* File info */}
          <Show when={fileType()}>
            {type => (
              <span class="text-xs text-cyan-400 px-2 py-0.5 bg-zinc-900 rounded" title={type().magic_hex}>
                {type().description}
              </span>
            )}
          </Show>
          <Show when={chunk()}>
            {c => (
              <span class="text-xs text-zinc-400">
                {formatBytes(c().total_size)}
              </span>
            )}
          </Show>
        </div>
        
        <div class="flex items-center gap-1 ml-auto mr-auto">
          {/* Navigation */}
          <button 
            class="btn-nav" 
            onClick={goToStart} 
            disabled={loading() || !chunk()?.has_prev}
            title="Go to start (Ctrl+Home)"
          >
            ⏮
          </button>
          <button 
            class="btn-nav" 
            onClick={goPrev} 
            disabled={loading() || !chunk()?.has_prev}
            title="Previous page (PageUp)"
          >
            ◀
          </button>
          
          {/* Offset display */}
          <Show when={chunk()}>
            {c => (
              <span class="text-xs text-zinc-300 px-2 font-mono">
                0x{formatOffset(c().offset)} - 0x{formatOffset(Math.min(c().offset + c().bytes.length, c().total_size))}
              </span>
            )}
          </Show>
          
          <button 
            class="btn-nav" 
            onClick={goNext} 
            disabled={loading() || !chunk()?.has_more}
            title="Next page (PageDown)"
          >
            ▶
          </button>
          <button 
            class="btn-nav" 
            onClick={goToEnd} 
            disabled={loading() || !chunk()?.has_more}
            title="Go to end (Ctrl+End)"
          >
            ⏭
          </button>
        </div>
        
        <div class="flex items-center gap-2">
          {/* Go to offset */}
          <input
            type="text"
            class="w-32 px-2 py-1 text-xs bg-zinc-900 border border-zinc-700 rounded text-zinc-200 placeholder-zinc-500 focus:border-cyan-500 focus:outline-none"
            placeholder="Go to offset (hex: 0x...)"
            value={gotoOffset()}
            onInput={e => setGotoOffset(e.currentTarget.value)}
            onKeyDown={e => e.key === "Enter" && handleGotoOffset()}
          />
          <button class="px-2 py-1 text-xs bg-cyan-600 hover:bg-cyan-500 rounded text-white" onClick={handleGotoOffset}>
            Go
          </button>
          
          {/* View options */}
          <label class="label-with-icon">
            <input
              type="checkbox"
              class="w-3 h-3 accent-cyan-500"
              checked={showAscii()}
              onChange={e => setShowAscii(e.currentTarget.checked)}
            />
            ASCII
          </label>
          <label class="label-with-icon">
            <input
              type="checkbox"
              class="w-3 h-3 accent-cyan-500"
              checked={highlightRegions()}
              onChange={e => setHighlightRegions(e.currentTarget.checked)}
            />
            Highlight
          </label>
          
          {/* Chunk size */}
          <select
            class="px-2 py-1 text-xs bg-zinc-900 border border-zinc-700 rounded text-zinc-200 focus:border-cyan-500 focus:outline-none"
            value={chunkSize()}
            onChange={e => {
              setChunkSize(parseInt(e.currentTarget.value));
              loadChunk(currentOffset());
            }}
          >
            <option value={MIN_CHUNK_SIZE}>256 B</option>
            <option value={1024}>1 KB</option>
            <option value={DEFAULT_CHUNK_SIZE}>4 KB</option>
            <option value={16384}>16 KB</option>
            <option value={MAX_CHUNK_SIZE}>64 KB</option>
          </select>
          
          {/* Region jump dropdown */}
          <Show when={highlightRegions() && metadata()?.regions.length}>
            <select
              class="px-2 py-1 text-xs bg-zinc-900 border border-zinc-700 rounded text-zinc-200 focus:border-cyan-500 focus:outline-none"
              onChange={e => {
                const idx = parseInt(e.currentTarget.value);
                const regions = metadata()?.regions;
                if (!isNaN(idx) && regions && regions[idx]) {
                  const region = regions[idx];
                  setSelectedRegion(region);
                  loadChunk(region.start);
                }
                e.currentTarget.value = ""; // Reset selection
              }}
            >
              <option value="">Jump to region...</option>
              <For each={metadata()?.regions}>
                {(region, idx) => (
                  <option value={idx()}>
                    {region.name} (0x{formatOffset(region.start, { width: 4 })})
                  </option>
                )}
              </For>
            </select>
          </Show>
        </div>
      </div>
      
      {/* Error display */}
      <Show when={error()}>
        <div class="px-3 py-2 text-sm text-red-400 bg-red-900/20 border-b border-red-500/30">{error()}</div>
      </Show>
      
      {/* Loading indicator */}
      <Show when={loading()}>
        <div class="flex items-center justify-center py-8 text-zinc-400">Loading...</div>
      </Show>
      
      {/* Hex content */}
      <Show when={!loading() && chunk()}>
        <div class="flex-1 overflow-auto p-2">
          {/* Header */}
          <div class="flex items-center gap-0 text-[10px] text-zinc-500 pb-1 border-b border-zinc-700/50 mb-1 sticky top-0 bg-zinc-900">
            <Show when={showAddress()}>
              <span class="w-20 shrink-0">Offset</span>
            </Show>
            <span class="flex gap-0">
              <For each={[...Array(BYTES_PER_LINE).keys()]}>
                {i => <span class="w-[22px] text-center">{byteToHex(i)}</span>}
              </For>
            </span>
            <Show when={showAscii()}>
              <span class="ml-2 w-32">ASCII</span>
            </Show>
          </div>
          
          {/* Lines */}
          <div class="flex flex-col">
            <For each={hexLines()}>
              {line => (
                <div class="flex items-center gap-0 leading-tight hover:bg-zinc-800/30">
                  <Show when={showAddress()}>
                    <span class="w-20 shrink-0 text-[11px] text-cyan-500/80">
                      {formatOffset(line.offset)}
                    </span>
                  </Show>
                  
                  <span class="flex gap-0">
                    <For each={line.bytes}>
                      {(byteData, byteIdx) => {
                        const byteOffset = line.offset + byteIdx();
                        const isInSelectedRegion = () => {
                          const sel = selectedRegion();
                          return !!(sel && byteOffset >= sel.start && byteOffset <= sel.end);
                        };
                        const isHovered = () => hoveredOffset() === byteOffset;
                        const isNavigated = () => {
                          const nav = navigatedRange();
                          // Highlight only the specific bytes in the navigated range
                          return nav !== null && 
                            byteOffset >= nav.offset && 
                            byteOffset < nav.offset + nav.size;
                        };
                        
                        // Get background color - navigated takes precedence
                        const bgColor = () => {
                          if (isNavigated()) return NAVIGATED_COLOR;
                          return byteData.color || undefined;
                        };
                        
                        return (
                          <span 
                            class="w-[22px] text-center text-[11px] cursor-default"
                            classList={{ 
                              'ring-1 ring-cyan-400/50': isInSelectedRegion(),
                              'ring-1 ring-white/30': isHovered() && !isInSelectedRegion(),
                              'font-bold': isNavigated()
                            }}
                            style={bgColor() ? { 
                              "background-color": bgColor()
                            } : {}}
                            title={byteData.region ? `${byteData.region.name}: ${byteData.region.description}` : undefined}
                            onMouseEnter={() => setHoveredOffset(byteOffset)}
                            onMouseLeave={() => setHoveredOffset(null)}
                            onClick={() => setNavigatedRange(null)}  // Clear navigation highlight on click
                          >
                            {byteToHex(byteData.value)}
                          </span>
                        );
                      }}
                    </For>
                    {/* Pad incomplete lines */}
                    <For each={[...Array(Math.max(0, BYTES_PER_LINE - line.bytes.length)).keys()]}>
                      {() => <span class="w-[22px] text-center text-[11px]">  </span>}
                    </For>
                  </span>
                  
                  <Show when={showAscii()}>
                    <span class="ml-2 flex text-[11px] text-zinc-400">
                      <For each={line.bytes}>
                        {(byteData, byteIdx) => {
                          const byteOffset = line.offset + byteIdx();
                          const isInSelectedRegion = () => {
                            const sel = selectedRegion();
                            return !!(sel && byteOffset >= sel.start && byteOffset <= sel.end);
                          };
                          const isHovered = () => hoveredOffset() === byteOffset;
                          const isNavigated = () => {
                            const nav = navigatedRange();
                            return nav !== null && 
                              byteOffset >= nav.offset && 
                              byteOffset < nav.offset + nav.size;
                          };
                          
                          const bgColor = () => {
                            if (isNavigated()) return NAVIGATED_COLOR;
                            return byteData.color || undefined;
                          };
                          
                          return (
                            <span 
                              class="w-2 text-center cursor-default"
                              classList={{ 
                                'ring-1 ring-cyan-400/50': isInSelectedRegion(),
                                'ring-1 ring-white/30': isHovered() && !isInSelectedRegion(),
                                'font-bold': isNavigated()
                              }}
                              style={bgColor() ? { "background-color": bgColor() } : {}}
                              onMouseEnter={() => setHoveredOffset(byteOffset)}
                              onMouseLeave={() => setHoveredOffset(null)}
                            >
                              {byteToAscii(byteData.value)}
                            </span>
                          );
                        }}
                      </For>
                    </span>
                  </Show>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>
      
      {/* Empty state */}
      <Show when={!loading() && !chunk() && !error()}>
        <div class="flex items-center justify-center py-8 text-zinc-500">
          Select a file to view its contents
        </div>
      </Show>
    </div>
  );
}
