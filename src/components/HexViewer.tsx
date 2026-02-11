// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, createMemo, createResource, For, Show, on } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { DiscoveredFile, HeaderRegion, MetadataField, ParsedMetadata, FileTypeInfo, FileChunk } from "../types";
import type { SelectedEntry } from "./EvidenceTree/types";
import { byteToHex, formatBytes } from "../utils";
import { logger } from '../utils/logger';
import { getPreference } from "./preferences";

const log = logger.scope('HexViewer');
import { readBytesFromSource, getSourceKey } from "../hooks";
import { HexToolbar } from "./hex/HexToolbar";
import { HexLine } from "./hex/HexLine";

// Re-export viewer types for backward compatibility
export type { FileChunk, HeaderRegion, MetadataField, ParsedMetadata, FileTypeInfo };

// --- Constants ---
const BYTES_PER_LINE = 16;
const INITIAL_LOAD_SIZE = 65536; // 64KB initial load (4096 lines)
const LOAD_MORE_SIZE = 32768; // 32KB per additional load
const SCROLL_THRESHOLD = 200; // pixels from bottom to trigger load

// Get max loaded bytes from preferences (convert MB to bytes) - memoized at module level
const getMaxLoadedBytes = () => getPreference("maxPreviewSizeMb") * 1024 * 1024;

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
} as const;

const NAVIGATED_COLOR = "rgba(34, 197, 94, 0.4)";

// Memoized color lookup (used in hot path)
function getRegionColor(colorClass: string): string {
  return COLOR_MAP[colorClass as keyof typeof COLOR_MAP] || "#6a6a7a";
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
  
  // Get max loaded bytes from preference (memoized for reactivity in JSX)
  const maxLoadedBytes = createMemo(() => getMaxLoadedBytes());
  
  // ==========================================================================
  // State signals
  // ==========================================================================
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
  
  // ==========================================================================
  // Memoized derived state (computed once when dependencies change)
  // ==========================================================================
  
  // Get the source identifier for change detection (using shared utility)
  const sourceKey = createMemo(() => getSourceKey(props.file, props.entry));
  
  // Memoized regions from metadata (avoid repeated access)
  const metadataRegions = createMemo(() => metadata()?.regions ?? []);
  const hasRegions = createMemo(() => metadataRegions().length > 0);
  
  // Load progress computation (avoid recalculating in JSX)
  const loadProgress = createMemo(() => {
    const total = totalFileSize();
    return total === 0 ? 0 : Math.round((loadedUpTo() / total) * 100);
  });
  
  // Can load more (derived state)
  const canLoadMore = createMemo(() => 
    loadedUpTo() < totalFileSize() && loadedUpTo() < maxLoadedBytes()
  );
  
  const loadInitialData = async () => {
    setLoading(true);
    setError(null);
    setLoadedBytes([]);
    setLoadedUpTo(0);
    
    // Debug: Log what we're trying to load
    log.debug(' loadInitialData called', {
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
      log.debug(' loadInitialData success, bytes:', result.bytes.length, 'totalSize:', result.totalSize);
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
    const maxBytes = getMaxLoadedBytes();
    if (currentLoaded >= total || currentLoaded >= maxBytes) return;
    
    setLoadingMore(true);
    try {
      const sizeToLoad = Math.min(LOAD_MORE_SIZE, total - currentLoaded, maxBytes - currentLoaded);
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
  
  // ==========================================================================
  // Resources: Async data fetching with Suspense support
  // ==========================================================================
  
  // Fetch file type detection (only for disk files)
  const [fileTypeResource] = createResource(
    () => props.file?.path,
    async (path) => {
      if (!path) return null;
      try {
        return await invoke<FileTypeInfo>("viewer_detect_type", { path });
      } catch (e) {
        console.warn("Failed to detect file type:", e);
        return null;
      }
    }
  );
  
  // Fetch parsed metadata (only for disk files)
  const [metadataResource] = createResource(
    () => props.file?.path,
    async (path) => {
      if (!path) return null;
      try {
        const meta = await invoke<ParsedMetadata>("viewer_parse_header", { path });
        props.onMetadataLoaded?.(meta);
        return meta;
      } catch {
        props.onMetadataLoaded?.(null);
        return null;
      }
    }
  );
  
  // Sync resources to local signals for compatibility with existing code
  createEffect(() => {
    const type = fileTypeResource();
    if (type !== undefined) setFileType(type);
  });
  
  createEffect(() => {
    const meta = metadataResource();
    if (meta !== undefined) setMetadata(meta);
  });
  
  // ==========================================================================
  // Effect: Load byte data when source changes (with explicit dependency tracking)
  // ==========================================================================
  createEffect(on(
    sourceKey,
    (key) => {
      if (!key) return;
      
      // Reset state (metadata is handled by resources now)
      setLoadedBytes([]);
      setError(null);
      setLoadedUpTo(0);
      setTotalFileSize(0);
      setNavigatedRange(null);
      
      // For container entries, notify no metadata
      if (!props.file && props.entry) {
        props.onMetadataLoaded?.(null);
      }
      
      loadInitialData();
    },
    { defer: false }
  ));
  
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

  return (
    <div class="flex flex-col h-full bg-bg text-txt font-mono text-sm" tabIndex={0} onKeyDown={handleKeyDown}>
      <HexToolbar
        fileType={fileType()}
        fileSize={totalFileSize()}
        loadedBytes={loadedUpTo()}
        loadProgress={loadProgress()}
        loading={loading()}
        loadingMore={loadingMore()}
        canLoadMore={canLoadMore()}
        maxLoadedBytes={maxLoadedBytes()}
        gotoOffset={gotoOffset()}
        showAscii={showAscii()}
        highlightRegions={highlightRegions()}
        hasRegions={hasRegions()}
        regions={metadataRegions()}
        onLoadMore={loadMoreData}
        onGotoOffset={handleGotoOffset}
        onSetGotoOffset={setGotoOffset}
        onToggleAscii={setShowAscii}
        onToggleHighlight={setHighlightRegions}
        onSelectRegion={async (idx) => {
          const regions = metadataRegions();
          if (regions[idx]) {
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
        }}
      />
      
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
                <HexLine
                  line={line}
                  showAddress={showAddress()}
                  showAscii={showAscii()}
                  hoveredOffset={hoveredOffset()}
                  selectedRegion={selectedRegion()}
                  navigatedRange={navigatedRange()}
                  navigatedColor={NAVIGATED_COLOR}
                  onHoverByte={setHoveredOffset}
                  onClearNavigation={() => setNavigatedRange(null)}
                />
              )}
            </For>
          </div>
          
          <Show when={loadingMore()}>
            <div class="flex items-center justify-center py-4 text-txt-muted text-xs">Loading more...</div>
          </Show>
          <Show when={!loadingMore() && loadedUpTo() >= totalFileSize()}>
            <div class="flex items-center justify-center py-4 text-txt-muted text-xs">— End of file —</div>
          </Show>
          <Show when={!loadingMore() && loadedUpTo() >= maxLoadedBytes() && totalFileSize() > maxLoadedBytes()}>
            <div class="flex items-center justify-center py-4 text-amber-500/70 text-xs">— Maximum preview size reached ({formatBytes(maxLoadedBytes())}) —</div>
          </Show>
        </div>
      </Show>
      
      <Show when={!loading() && loadedBytes().length === 0 && !error()}>
        <div class="flex items-center justify-center py-8 text-txt-muted">Select a file to view its contents</div>
      </Show>
    </div>
  );
}
