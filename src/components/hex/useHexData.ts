// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useHexData — state management hook for the hex viewer.
 *
 * Manages byte loading (initial + incremental), file type detection,
 * metadata parsing, goto-offset, scroll-driven loading, and hex-line memoization.
 */

import { createSignal, createEffect, createMemo, createResource, on } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { HeaderRegion, ParsedMetadata, FileTypeInfo } from "../../types";
import type { SelectedEntry } from "../EvidenceTree/types";
import type { DiscoveredFile } from "../../types";
import { logger } from "../../utils/logger";
import { readBytesFromSource, getSourceKey } from "../../hooks";
import {
  BYTES_PER_LINE,
  INITIAL_LOAD_SIZE,
  LOAD_MORE_SIZE,
  SCROLL_THRESHOLD,
  getMaxLoadedBytes,
  getRegionColor,
} from "./constants";

const log = logger.scope("HexViewer");

export interface UseHexDataOptions {
  file: () => DiscoveredFile | null | undefined;
  entry: () => SelectedEntry | undefined;
  onMetadataLoaded?: (metadata: ParsedMetadata | null) => void;
  onNavigatorReady?: (navigateTo: (offset: number, size?: number) => void) => void;
}

export function useHexData(opts: UseHexDataOptions) {
  let scrollContainerRef: HTMLDivElement | undefined;

  // Max loaded bytes from preference (memoized for reactivity)
  const maxLoadedBytes = createMemo(() => getMaxLoadedBytes());

  // ── State signals ──
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
  const [navigatedRange, setNavigatedRange] = createSignal<{
    offset: number;
    size: number;
  } | null>(null);

  // ── Derived state ──
  const sourceKey = createMemo(() => getSourceKey(opts.file(), opts.entry()));
  const metadataRegions = createMemo(() => metadata()?.regions ?? []);
  const hasRegions = createMemo(() => metadataRegions().length > 0);
  const loadProgress = createMemo(() => {
    const total = totalFileSize();
    return total === 0 ? 0 : Math.round((loadedUpTo() / total) * 100);
  });
  const canLoadMore = createMemo(
    () => loadedUpTo() < totalFileSize() && loadedUpTo() < maxLoadedBytes(),
  );

  // ── Data loading ──
  const loadInitialData = async () => {
    setLoading(true);
    setError(null);
    setLoadedBytes([]);
    setLoadedUpTo(0);

    const file = opts.file();
    const entry = opts.entry();

    log.debug(" loadInitialData called", {
      hasFile: !!file,
      hasEntry: !!entry,
      entry: entry
        ? {
            containerPath: entry.containerPath,
            entryPath: entry.entryPath,
            isArchiveEntry: entry.isArchiveEntry,
            isVfsEntry: entry.isVfsEntry,
            isDiskFile: entry.isDiskFile,
            size: entry.size,
          }
        : null,
    });

    try {
      const result = await readBytesFromSource(file ?? null, entry, 0, INITIAL_LOAD_SIZE);
      log.debug(
        " loadInitialData success, bytes:",
        result.bytes.length,
        "totalSize:",
        result.totalSize,
      );
      setLoadedBytes(result.bytes);
      setLoadedUpTo(result.bytes.length);
      setTotalFileSize(result.totalSize);
    } catch (e) {
      log.error("loadInitialData error:", e);
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
      const result = await readBytesFromSource(
        opts.file() ?? null,
        opts.entry(),
        currentLoaded,
        sizeToLoad,
      );
      setLoadedBytes((prev) => [...prev, ...result.bytes]);
      setLoadedUpTo(currentLoaded + result.bytes.length);
    } catch (e) {
      log.error("Failed to load more data:", e);
    } finally {
      setLoadingMore(false);
    }
  };

  // ── Scroll & navigation ──
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
    const scrollPosition = lineNumber * lineHeight + headerHeight;
    scrollContainerRef.scrollTo({
      top: Math.max(0, scrollPosition - 100),
      behavior: "smooth",
    });
  };

  const navigateToOffset = async (offset: number, size?: number) => {
    if (typeof offset !== "number" || isNaN(offset) || offset < 0) return;
    setNavigatedRange({ offset, size: size ?? 4 });
    if (offset >= loadedUpTo()) {
      const targetOffset = Math.min(offset + LOAD_MORE_SIZE, totalFileSize());
      setLoadingMore(true);
      try {
        const result = await readBytesFromSource(opts.file() ?? null, opts.entry(), 0, targetOffset);
        setLoadedBytes(result.bytes);
        setLoadedUpTo(result.bytes.length);
      } catch (e) {
        log.error("Failed to navigate to offset:", e);
      } finally {
        setLoadingMore(false);
      }
    }
    setTimeout(() => scrollToOffset(offset), 100);
  };

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
        const result = await readBytesFromSource(opts.file() ?? null, opts.entry(), 0, targetOffset);
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

  const handleSelectRegion = async (idx: number) => {
    const regions = metadataRegions();
    if (!regions[idx]) return;
    const region = regions[idx];
    setSelectedRegion(region);
    setNavigatedRange({ offset: region.start, size: region.end - region.start });
    if (region.start >= loadedUpTo()) {
      const targetOffset = Math.min(region.end + LOAD_MORE_SIZE, totalFileSize());
      setLoadingMore(true);
      try {
        const result = await readBytesFromSource(opts.file() ?? null, opts.entry(), 0, targetOffset);
        setLoadedBytes(result.bytes);
        setLoadedUpTo(result.bytes.length);
      } catch (err) {
        log.error("Failed to load region:", err);
      } finally {
        setLoadingMore(false);
      }
    }
    setTimeout(() => scrollToOffset(region.start), 100);
  };

  // ── Hex lines memo ──
  const hexLines = createMemo(() => {
    const bytes = loadedBytes();
    const meta = metadata();
    const doHighlight = highlightRegions();
    if (!bytes.length) return [];

    const lines: {
      offset: number;
      bytes: { value: number; color: string | null; region: HeaderRegion | null }[];
    }[] = [];
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
        }),
      });
    }
    return lines;
  });

  // ── Keyboard handler ──
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Home" && e.ctrlKey) {
      e.preventDefault();
      scrollContainerRef?.scrollTo({ top: 0, behavior: "smooth" });
    } else if (e.key === "End" && e.ctrlKey) {
      e.preventDefault();
      scrollContainerRef?.scrollTo({ top: scrollContainerRef.scrollHeight, behavior: "smooth" });
    }
  };

  // ── Resources: detect file type & parse metadata (disk files only) ──
  const [fileTypeResource] = createResource(
    () => {
      const f = opts.file();
      return f?.path;
    },
    async (path) => {
      if (!path) return null;
      try {
        return await invoke<FileTypeInfo>("viewer_detect_type", { path });
      } catch (e) {
        log.warn("Failed to detect file type:", e);
        return null;
      }
    },
  );

  const [metadataResource] = createResource(
    () => {
      const f = opts.file();
      return f?.path;
    },
    async (path) => {
      if (!path) return null;
      try {
        const meta = await invoke<ParsedMetadata>("viewer_parse_header", { path });
        opts.onMetadataLoaded?.(meta);
        return meta;
      } catch {
        opts.onMetadataLoaded?.(null);
        return null;
      }
    },
  );

  createEffect(() => {
    const type = fileTypeResource();
    if (type !== undefined) setFileType(type);
  });

  createEffect(() => {
    const meta = metadataResource();
    if (meta !== undefined) setMetadata(meta);
  });

  // ── Effect: Load byte data when source changes ──
  createEffect(
    on(
      sourceKey,
      (key) => {
        if (!key) return;
        setLoadedBytes([]);
        setError(null);
        setLoadedUpTo(0);
        setTotalFileSize(0);
        setNavigatedRange(null);

        if (!opts.file() && opts.entry()) {
          opts.onMetadataLoaded?.(null);
        }

        loadInitialData();
      },
      { defer: false },
    ),
  );

  // ── Navigator callback ──
  if (opts.onNavigatorReady) {
    opts.onNavigatorReady(navigateToOffset);
  }

  return {
    // Refs
    get scrollContainerRef() {
      return scrollContainerRef;
    },
    setScrollContainerRef: (el: HTMLDivElement | undefined) => {
      scrollContainerRef = el;
    },

    // State accessors
    loadedBytes,
    totalFileSize,
    loadedUpTo,
    loading,
    loadingMore,
    error,
    gotoOffset,
    showAscii,
    showAddress,
    highlightRegions,
    selectedRegion,
    hoveredOffset,
    navigatedRange,
    fileType,

    // Derived
    maxLoadedBytes,
    metadataRegions,
    hasRegions,
    loadProgress,
    canLoadMore,
    hexLines,

    // Actions
    setGotoOffset,
    setShowAscii,
    setHighlightRegions,
    setHoveredOffset,
    setNavigatedRange,
    loadMoreData,
    handleScroll,
    handleGotoOffset,
    handleSelectRegion,
    handleKeyDown,
  };
}
