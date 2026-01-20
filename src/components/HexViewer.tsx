// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, createMemo, For, Show, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { DiscoveredFile, FileChunk, HeaderRegion, MetadataField, ParsedMetadata, FileTypeInfo } from "../types";
import type { SelectedEntry } from "./EvidenceTree/types";
import { formatOffset, byteToHex, byteToAscii, formatBytes } from "../utils";

// Re-export viewer types for backward compatibility
export type { FileChunk, HeaderRegion, MetadataField, ParsedMetadata, FileTypeInfo };

// --- Constants ---
const BYTES_PER_LINE = 16;
const INITIAL_LOAD_SIZE = 65536; // 64KB initial load (4096 lines)
const LOAD_MORE_SIZE = 32768; // 32KB per additional load
const MAX_LOADED_BYTES = 2097152; // 2MB max loaded in memory
const SCROLL_THRESHOLD = 200; // pixels from bottom to trigger load

// Map color classes to actual colors (with very light transparency)
const COLOR_MAP: Record<string, string> = {
  "region-signature": "rgba(239, 68, 68, 0.15)",
  "region-header": "rgba(249, 115, 22, 0.15)",
  "region-segment": "rgba(249, 115, 22, 0.15)",
  "region-metadata": "rgba(234, 179, 8, 0.15)",
  "region-data": "rgba(34, 197, 94, 0.15)",
  "region-checksum": "rgba(59, 130, 246, 0.15)",
  "region-reserved": "rgba(139, 92, 246, 0.15)",
  "region-footer": "rgba(236, 72, 153, 0.15)",
};

const NAVIGATED_COLOR = "rgba(34, 197, 94, 0.4)";

function getRegionColor(colorClass: string): string {
  return COLOR_MAP[colorClass] || "#6a6a7a";
}

/**
 * Read bytes from any source: disk file, AD1 container entry, VFS entry, or archive entry
 */
async function readBytesFromSource(
  file: DiscoveredFile | null,
  entry: SelectedEntry | undefined,
  offset: number,
  size: number
): Promise<{ bytes: number[]; totalSize: number }> {
  // Case 1: SelectedEntry provided (container file viewing)
  if (entry) {
    // VFS entries (E01/Raw)
    if (entry.isVfsEntry) {
      const bytes = await invoke<number[]>("vfs_read_file", {
        containerPath: entry.containerPath,
        filePath: entry.entryPath,
        offset,
        length: size
      });
      return { bytes, totalSize: entry.size };
    }
    
    // Archive entries (ZIP, 7z, TAR, etc.)
    if (entry.isArchiveEntry) {
      // Check if this is a nested archive entry (path contains "::")
      // Format: "nestedArchive.zip::file.txt" means file.txt inside nestedArchive.zip
      if (entry.entryPath.includes("::")) {
        const [nestedArchivePath, nestedEntryPath] = entry.entryPath.split("::", 2);
        const bytes = await invoke<number[]>("nested_archive_read_entry_chunk", {
          containerPath: entry.containerPath,
          nestedArchivePath,
          entryPath: nestedEntryPath,
          offset,
          size
        });
        return { bytes, totalSize: entry.size };
      }
      
      // Regular archive entry
      const bytes = await invoke<number[]>("archive_read_entry_chunk", {
        containerPath: entry.containerPath,
        entryPath: entry.entryPath,
        offset,
        size
      });
      return { bytes, totalSize: entry.size };
    }
    
    // Disk file entry (file inside container that's actually on disk)
    if (entry.isDiskFile) {
      const bytes = await invoke<number[]>("read_file_bytes", {
        path: entry.entryPath,
        offset,
        length: size
      });
      return { bytes, totalSize: entry.size };
    }
    
    // AD1 container entry - use chunk-based reading for scroll support
    const bytes = await invoke<number[]>("container_read_entry_chunk", {
      containerPath: entry.containerPath,
      entryPath: entry.entryPath,
      offset,
      size
    });
    return { bytes, totalSize: entry.size };
  }
  
  // Case 2: Regular disk file (DiscoveredFile)
  if (file) {
    const result = await invoke<FileChunk>("viewer_read_chunk", {
      path: file.path,
      offset,
      size
    });
    return { bytes: result.bytes, totalSize: result.total_size };
  }
  
  throw new Error("No file or entry provided");
}

interface HexViewerProps {
  /** Regular disk file */
  file?: DiscoveredFile | null;
  /** Container entry (file inside AD1/E01/etc.) */
  entry?: SelectedEntry;
  onMetadataLoaded?: (metadata: ParsedMetadata | null) => void;
  onNavigatorReady?: (navigateTo: (offset: number, size?: number) => void) => void;
}

export function HexViewer(props: HexViewerProps) {
  let scrollContainerRef: HTMLDivElement | undefined;
  
  const [loadedBytes, setLoadedBytes] = createSignal<number[]>([]);
  const [totalFileSize, setTotalFileSize] = createSignal(0);
  const [loadedUpTo, setLoadedUpTo] = createSignal(0);
  const [metadata, setMetadata] = createSignal<ParsedMetadata | null>(null);
  const [fileType, setFileType] = createSignal<FileTypeInfo | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [loadingMore, setLoadingMore] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [gotoOffset, setGotoOffset] = createSignal("");
  const [showAscii, setShowAscii] = createSignal(true);
  const [highlightRegions, setHighlightRegions] = createSignal(true);
  const [showAddress, _setShowAddress] = createSignal(true);
  const [selectedRegion, setSelectedRegion] = createSignal<HeaderRegion | null>(null);
  const [hoveredOffset, setHoveredOffset] = createSignal<number | null>(null);
  const [navigatedRange, setNavigatedRange] = createSignal<{ offset: number; size: number } | null>(null);
  
  // Get the source identifier for change detection
  const sourceKey = () => {
    if (props.entry) return `entry:${props.entry.containerPath}:${props.entry.entryPath}`;
    if (props.file) return `file:${props.file.path}`;
    return null;
  };
  
  const loadInitialData = async () => {
    setLoading(true);
    setError(null);
    setLoadedBytes([]);
    setLoadedUpTo(0);
    
    // Debug: Log what we're trying to load
    console.log('[HexViewer] loadInitialData called', {
      hasFile: !!props.file,
      hasEntry: !!props.entry,
      entry: props.entry ? {
        containerPath: props.entry.containerPath,
        entryPath: props.entry.entryPath,
        isArchiveEntry: props.entry.isArchiveEntry,
        isVfsEntry: props.entry.isVfsEntry,
        isDiskFile: props.entry.isDiskFile,
        size: props.entry.size
      } : null
    });
    
    try {
      const result = await readBytesFromSource(props.file ?? null, props.entry, 0, INITIAL_LOAD_SIZE);
      console.log('[HexViewer] loadInitialData success, bytes:', result.bytes.length, 'totalSize:', result.totalSize);
      setLoadedBytes(result.bytes);
      setLoadedUpTo(result.bytes.length);
      setTotalFileSize(result.totalSize);
    } catch (e) {
      console.error('[HexViewer] loadInitialData error:', e);
      setError(`Failed to load file: ${e}`);
      setLoadedBytes([]);
    } finally {
      setLoading(false);
    }
  };
  
  const loadMoreData = async () => {
    if (loadingMore() || loading()) return;
    const currentLoaded = loadedUpTo();
    const total = totalFileSize();
    if (currentLoaded >= total || currentLoaded >= MAX_LOADED_BYTES) return;
    
    setLoadingMore(true);
    try {
      const sizeToLoad = Math.min(LOAD_MORE_SIZE, total - currentLoaded, MAX_LOADED_BYTES - currentLoaded);
      const result = await readBytesFromSource(props.file ?? null, props.entry, currentLoaded, sizeToLoad);
      setLoadedBytes(prev => [...prev, ...result.bytes]);
      setLoadedUpTo(currentLoaded + result.bytes.length);
    } catch (e) {
      console.error("Failed to load more data:", e);
    } finally {
      setLoadingMore(false);
    }
  };
  
  const handleScroll = () => {
    if (!scrollContainerRef) return;
    const { scrollTop, scrollHeight, clientHeight } = scrollContainerRef;
    const distanceFromBottom = scrollHeight - scrollTop - clientHeight;
    if (distanceFromBottom < SCROLL_THRESHOLD) {
      loadMoreData();
    }
  };
  
  const scrollToOffset = (offset: number) => {
    if (!scrollContainerRef) return;
    const lineNumber = Math.floor(offset / BYTES_PER_LINE);
    const lineHeight = 20;
    const headerHeight = 28;
    const scrollPosition = (lineNumber * lineHeight) + headerHeight;
    scrollContainerRef.scrollTo({
      top: Math.max(0, scrollPosition - 100),
      behavior: 'smooth'
    });
  };
  
  createEffect(() => {
    const key = sourceKey();
    if (!key) return;
    
    setLoadedBytes([]);
    setMetadata(null);
    setFileType(null);
    setError(null);
    setLoadedUpTo(0);
    setTotalFileSize(0);
    setNavigatedRange(null);
    
    loadInitialData();
    
    // Only load metadata/type for disk files (not container entries)
    if (props.file) {
      invoke<FileTypeInfo>("viewer_detect_type", { path: props.file.path })
        .then(setFileType)
        .catch(e => console.warn("Failed to detect file type:", e));
      
      invoke<ParsedMetadata>("viewer_parse_header", { path: props.file.path })
        .then(meta => {
          setMetadata(meta);
          if (props.onMetadataLoaded) props.onMetadataLoaded(meta);
        })
        .catch(() => {
          setMetadata(null);
          if (props.onMetadataLoaded) props.onMetadataLoaded(null);
        });
    } else {
      // For container entries, no metadata parsing yet
      if (props.onMetadataLoaded) props.onMetadataLoaded(null);
    }
  });
  
  if (props.onNavigatorReady) {
    props.onNavigatorReady(async (offset: number, size?: number) => {
      if (typeof offset === 'number' && !isNaN(offset) && offset >= 0) {
        setNavigatedRange({ offset, size: size ?? 4 });
        if (offset >= loadedUpTo()) {
          const targetOffset = Math.min(offset + LOAD_MORE_SIZE, totalFileSize());
          setLoadingMore(true);
          try {
            const result = await readBytesFromSource(props.file ?? null, props.entry, 0, targetOffset);
            setLoadedBytes(result.bytes);
            setLoadedUpTo(result.bytes.length);
          } catch (e) {
            console.error("Failed to navigate to offset:", e);
          } finally {
            setLoadingMore(false);
          }
        }
        setTimeout(() => scrollToOffset(offset), 100);
      }
    });
  }
  
  const handleGotoOffset = async () => {
    const input = gotoOffset().trim();
    let offset: number;
    if (input.toLowerCase().startsWith("0x")) {
      offset = parseInt(input, 16);
    } else {
      offset = parseInt(input, 10);
    }
    
    if (isNaN(offset) || offset < 0) {
      setError("Invalid offset");
      return;
    }
    if (offset >= totalFileSize()) {
      setError("Offset exceeds file size");
      return;
    }
    
    setNavigatedRange({ offset, size: 4 });
    if (offset >= loadedUpTo()) {
      const targetOffset = Math.min(offset + LOAD_MORE_SIZE, totalFileSize());
      setLoadingMore(true);
      try {
        const result = await readBytesFromSource(props.file ?? null, props.entry, 0, targetOffset);
        setLoadedBytes(result.bytes);
        setLoadedUpTo(result.bytes.length);
      } catch (e) {
        setError(`Failed to navigate: ${e}`);
        return;
      } finally {
        setLoadingMore(false);
      }
    }
    
    setError(null);
    setGotoOffset("");
    setTimeout(() => scrollToOffset(offset), 100);
  };
  
  const hexLines = createMemo(() => {
    const bytes = loadedBytes();
    const meta = metadata();
    const doHighlight = highlightRegions();
    if (!bytes.length) return [];
    
    const lines: { offset: number; bytes: { value: number; color: string | null; region: HeaderRegion | null }[] }[] = [];
    for (let i = 0; i < bytes.length; i += BYTES_PER_LINE) {
      const lineBytes = bytes.slice(i, i + BYTES_PER_LINE);
      const lineOffset = i;
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
  
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Home" && e.ctrlKey) {
      e.preventDefault();
      scrollContainerRef?.scrollTo({ top: 0, behavior: 'smooth' });
    } else if (e.key === "End" && e.ctrlKey) {
      e.preventDefault();
      scrollContainerRef?.scrollTo({ top: scrollContainerRef.scrollHeight, behavior: 'smooth' });
    }
  };
  
  onCleanup(() => {});
  
  const loadProgress = () => {
    const total = totalFileSize();
    if (total === 0) return 0;
    return Math.round((loadedUpTo() / total) * 100);
  };
  
  const canLoadMore = () => loadedUpTo() < totalFileSize() && loadedUpTo() < MAX_LOADED_BYTES;

  return (
    <div class="flex flex-col h-full bg-bg text-txt font-mono text-sm" tabIndex={0} onKeyDown={handleKeyDown}>
      <div class="flex items-center gap-2 px-3 py-2 bg-bg-panel border-b border-border flex-wrap">
        <div class="flex items-center gap-2">
          <Show when={fileType()}>
            {type => (
              <span class="text-xs text-accent px-2 py-0.5 bg-bg rounded" title={type().magic_hex}>
                {type().description}
              </span>
            )}
          </Show>
          <Show when={totalFileSize() > 0}>
            <span class="text-xs text-txt-secondary">{formatBytes(totalFileSize())}</span>
          </Show>
        </div>
        
        <div class="flex items-center gap-2 ml-auto mr-auto">
          <span class="text-xs text-txt-secondary">
            Loaded: {formatBytes(loadedUpTo())}
            {loadedUpTo() < totalFileSize() && ` (${loadProgress()}%)`}
          </span>
          <Show when={canLoadMore()}>
            <button class="px-2 py-0.5 text-xs bg-bg-hover hover:bg-bg-active rounded text-txt-tertiary" onClick={loadMoreData} disabled={loadingMore()}>
              {loadingMore() ? "Loading..." : "Load More"}
            </button>
          </Show>
          <Show when={loadedUpTo() >= MAX_LOADED_BYTES && totalFileSize() > MAX_LOADED_BYTES}>
            <span class="text-xs text-amber-400">Max preview reached</span>
          </Show>
        </div>
        
        <div class="flex items-center gap-2">
          <input type="text" class="w-32 px-2 py-1 text-xs bg-bg border border-border rounded text-txt placeholder-txt-muted focus:border-accent focus:outline-none" placeholder="Go to offset (hex: 0x...)" value={gotoOffset()} onInput={e => setGotoOffset(e.currentTarget.value)} onKeyDown={e => e.key === "Enter" && handleGotoOffset()} />
          <button class="px-2 py-1 text-xs bg-accent hover:bg-accent-hover rounded text-white" onClick={handleGotoOffset}>Go</button>
          
          <label class="label-with-icon">
            <input type="checkbox" class="w-3 h-3 accent-accent" checked={showAscii()} onChange={e => setShowAscii(e.currentTarget.checked)} />
            ASCII
          </label>
          <label class="label-with-icon">
            <input type="checkbox" class="w-3 h-3 accent-accent" checked={highlightRegions()} onChange={e => setHighlightRegions(e.currentTarget.checked)} />
            Highlight
          </label>
          
          <Show when={highlightRegions() && metadata()?.regions.length}>
            <select class="px-2 py-1 text-xs bg-bg border border-border rounded text-txt focus:border-accent focus:outline-none" onChange={async e => {
              const idx = parseInt(e.currentTarget.value);
              const regions = metadata()?.regions;
              if (!isNaN(idx) && regions && regions[idx]) {
                const region = regions[idx];
                setSelectedRegion(region);
                setNavigatedRange({ offset: region.start, size: region.end - region.start });
                if (region.start >= loadedUpTo()) {
                  const targetOffset = Math.min(region.end + LOAD_MORE_SIZE, totalFileSize());
                  setLoadingMore(true);
                  try {
                    const result = await readBytesFromSource(props.file ?? null, props.entry, 0, targetOffset);
                    setLoadedBytes(result.bytes);
                    setLoadedUpTo(result.bytes.length);
                  } catch (err) {
                    console.error("Failed to load region:", err);
                  } finally {
                    setLoadingMore(false);
                  }
                }
                setTimeout(() => scrollToOffset(region.start), 100);
              }
              e.currentTarget.value = "";
            }}>
              <option value="">Jump to region...</option>
              <For each={metadata()?.regions}>
                {(region, idx) => <option value={idx()}>{region.name} (0x{formatOffset(region.start, { width: 4 })})</option>}
              </For>
            </select>
          </Show>
        </div>
      </div>
      
      <Show when={error()}>
        <div class="px-3 py-2 text-sm text-red-400 bg-red-900/20 border-b border-red-500/30">{error()}</div>
      </Show>
      
      <Show when={loading()}>
        <div class="flex items-center justify-center py-8 text-txt-secondary">Loading...</div>
      </Show>
      
      <Show when={!loading() && loadedBytes().length > 0}>
        <div ref={scrollContainerRef} class="flex-1 overflow-auto p-2" onScroll={handleScroll}>
          <div class="flex items-center gap-0 text-[10px] leading-tight text-txt-muted pb-1 border-b border-border/50 mb-1 sticky top-0 bg-bg z-10">
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
          
          <div class="flex flex-col">
            <For each={hexLines()}>
              {line => (
                <div class="flex items-center gap-0 leading-tight hover:bg-bg-panel/30">
                  <Show when={showAddress()}>
                    <span class="w-20 shrink-0 text-[10px] text-accent/80">{formatOffset(line.offset)}</span>
                  </Show>
                  <span class="flex gap-0">
                    <For each={line.bytes}>
                      {(byteData, byteIdx) => {
                        const byteOffset = line.offset + byteIdx();
                        const isInSelectedRegion = () => { const sel = selectedRegion(); return !!(sel && byteOffset >= sel.start && byteOffset <= sel.end); };
                        const isHovered = () => hoveredOffset() === byteOffset;
                        const isNavigated = () => { const nav = navigatedRange(); return nav !== null && byteOffset >= nav.offset && byteOffset < nav.offset + nav.size; };
                        const bgColor = () => { if (isNavigated()) return NAVIGATED_COLOR; return byteData.color || undefined; };
                        return (
                          <span class="w-[22px] text-center text-[10px] cursor-default" classList={{ 'ring-1 ring-accent/50': isInSelectedRegion(), 'ring-1 ring-white/30': isHovered() && !isInSelectedRegion(), 'font-bold': isNavigated() }} style={bgColor() ? { "background-color": bgColor() } : {}} title={byteData.region ? `${byteData.region.name}: ${byteData.region.description}` : undefined} onMouseEnter={() => setHoveredOffset(byteOffset)} onMouseLeave={() => setHoveredOffset(null)} onClick={() => setNavigatedRange(null)}>
                            {byteToHex(byteData.value)}
                          </span>
                        );
                      }}
                    </For>
                    <For each={[...Array(Math.max(0, BYTES_PER_LINE - line.bytes.length)).keys()]}>
                      {() => <span class="w-[22px] text-center text-[10px]">  </span>}
                    </For>
                  </span>
                  <Show when={showAscii()}>
                    <span class="ml-2 flex text-[10px] text-txt-secondary">
                      <For each={line.bytes}>
                        {(byteData, byteIdx) => {
                          const byteOffset = line.offset + byteIdx();
                          const isInSelectedRegion = () => { const sel = selectedRegion(); return !!(sel && byteOffset >= sel.start && byteOffset <= sel.end); };
                          const isHovered = () => hoveredOffset() === byteOffset;
                          const isNavigated = () => { const nav = navigatedRange(); return nav !== null && byteOffset >= nav.offset && byteOffset < nav.offset + nav.size; };
                          const bgColor = () => { if (isNavigated()) return NAVIGATED_COLOR; return byteData.color || undefined; };
                          return (
                            <span class="w-2 text-center cursor-default" classList={{ 'ring-1 ring-accent/50': isInSelectedRegion(), 'ring-1 ring-white/30': isHovered() && !isInSelectedRegion(), 'font-bold': isNavigated() }} style={bgColor() ? { "background-color": bgColor() } : {}} onMouseEnter={() => setHoveredOffset(byteOffset)} onMouseLeave={() => setHoveredOffset(null)}>
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
          
          <Show when={loadingMore()}>
            <div class="flex items-center justify-center py-4 text-txt-muted text-xs">Loading more...</div>
          </Show>
          <Show when={!loadingMore() && loadedUpTo() >= totalFileSize()}>
            <div class="flex items-center justify-center py-4 text-txt-muted text-xs">— End of file —</div>
          </Show>
          <Show when={!loadingMore() && loadedUpTo() >= MAX_LOADED_BYTES && totalFileSize() > MAX_LOADED_BYTES}>
            <div class="flex items-center justify-center py-4 text-amber-500/70 text-xs">— Maximum preview size reached ({formatBytes(MAX_LOADED_BYTES)}) —</div>
          </Show>
        </div>
      </Show>
      
      <Show when={!loading() && loadedBytes().length === 0 && !error()}>
        <div class="flex items-center justify-center py-8 text-txt-muted">Select a file to view its contents</div>
      </Show>
    </div>
  );
}
