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

import {
  createSignal,
  createEffect,
  createMemo,
  createResource,
  on,
} from "solid-js";
import type { HeaderRegion, ParsedMetadata, FileTypeInfo } from "../../types";
import type { SelectedEntry } from "../EvidenceTree/types";
import type { DiscoveredFile } from "../../types";
import {
  commands,
  type ProjectDbEvidenceFile,
  type SourceAnalysis,
  type SourceAnalysisOptions,
} from "../../api/commands";
import { logger } from "../../utils/logger";
import { readBytesFromSource, getSourceKey } from "../../hooks";
import { buildEvidenceSourceInput } from "../evidenceSourceInput";
import {
  BYTES_PER_LINE,
  INITIAL_LOAD_SIZE,
  LOAD_MORE_SIZE,
  SCROLL_THRESHOLD,
  getMaxLoadedBytes,
  getRegionColor,
} from "./constants";
import { buildHexAnalysisAnnotations } from "./hexAnalysisAnnotations";
import { isTauri } from "../../utils/platform";

const log = logger.scope("HexViewer");
const ANALYSIS_SAMPLE_BYTES = 64 * 1024;
const ANALYSIS_ENTROPY_WINDOW_BYTES = 4096;

export interface UseHexDataOptions {
  file: () => DiscoveredFile | null | undefined;
  entry: () => SelectedEntry | undefined;
  onMetadataLoaded?: (metadata: ParsedMetadata | null) => void;
  onNavigatorReady?: (
    navigateTo: (offset: number, size?: number) => void,
  ) => void;
}

function evidenceFileFromDiscoveredFile(
  file: DiscoveredFile | null | undefined,
): ProjectDbEvidenceFile | undefined {
  if (!file) return undefined;

  return {
    id: file.path,
    path: file.path,
    filename: file.filename,
    containerType: file.container_type,
    totalSize: file.size,
    segmentCount: file.segment_count ?? 1,
    discoveredAt: new Date().toISOString(),
    created: file.created ?? null,
    modified: file.modified ?? null,
  };
}

async function analyzeSourceWithPersistence(
  source: NonNullable<ReturnType<typeof buildEvidenceSourceInput>>,
  file: DiscoveredFile | null | undefined,
  options: SourceAnalysisOptions,
): Promise<SourceAnalysis> {
  const analyzeTransient = () => {
    if (source.containerType === "disk" && source.path) {
      return commands.viewer.analyzePath(source.path, options);
    }

    return commands.viewer.analyzeSource(source, options);
  };

  try {
    if (await commands.projectDb.isOpen()) {
      const result = await commands.sourceAnalysis.analyzeSourceAndInsert({
        source,
        options,
        evidenceFile: evidenceFileFromDiscoveredFile(file),
        analyzer: "hex-viewer",
      });
      await persistHexAnalysisAnnotations(result.analysis);
      return result.analysis;
    }
  } catch (e) {
    log.warn("Persisted source analysis failed; using transient analysis:", e);
  }

  return analyzeTransient();
}

async function persistHexAnalysisAnnotations(
  analysis: SourceAnalysis,
): Promise<void> {
  const annotations = buildHexAnalysisAnnotations(analysis);
  if (annotations.length === 0) return;

  try {
    const existing = await commands.projectDb.annotations.getForPath(
      analysis.sourceId,
    );
    const existingIds = new Set(existing.map((annotation) => annotation.id));
    for (const annotation of annotations) {
      if (existingIds.has(annotation.id)) continue;
      await commands.projectDb.annotations.insert(annotation);
    }
  } catch (e) {
    log.warn("Hex analysis annotation persistence failed:", e);
  }
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
  const [selectedRegion, setSelectedRegion] = createSignal<HeaderRegion | null>(
    null,
  );
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
      const result = await readBytesFromSource(
        file ?? null,
        entry,
        0,
        INITIAL_LOAD_SIZE,
      );
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
      const sizeToLoad = Math.min(
        LOAD_MORE_SIZE,
        total - currentLoaded,
        maxBytes - currentLoaded,
      );
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
        const result = await readBytesFromSource(
          opts.file() ?? null,
          opts.entry(),
          0,
          targetOffset,
        );
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
        const result = await readBytesFromSource(
          opts.file() ?? null,
          opts.entry(),
          0,
          targetOffset,
        );
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
    setNavigatedRange({
      offset: region.start,
      size: region.end - region.start,
    });
    if (region.start >= loadedUpTo()) {
      const targetOffset = Math.min(
        region.end + LOAD_MORE_SIZE,
        totalFileSize(),
      );
      setLoadingMore(true);
      try {
        const result = await readBytesFromSource(
          opts.file() ?? null,
          opts.entry(),
          0,
          targetOffset,
        );
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
      bytes: {
        value: number;
        color: string | null;
        region: HeaderRegion | null;
      }[];
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
      scrollContainerRef?.scrollTo({
        top: scrollContainerRef.scrollHeight,
        behavior: "smooth",
      });
    }
  };

  // ── Resources: detect file type, parse headers, and run source analysis ──
  const [sourceAnalysisResource] = createResource(sourceKey, async (key) => {
    if (!key) return null;
    if (!isTauri) return null;

    try {
      const source = buildEvidenceSourceInput(
        opts.file() ?? null,
        opts.entry(),
      );
      if (!source) return null;

      const options = {
        offset: 0,
        length: ANALYSIS_SAMPLE_BYTES,
        entropyWindowBytes: ANALYSIS_ENTROPY_WINDOW_BYTES,
      };

      return await analyzeSourceWithPersistence(
        source,
        opts.file() ?? null,
        options,
      );
    } catch (e) {
      log.warn("Failed to analyze source:", e);
      return null;
    }
  });

  const [fileTypeResource] = createResource(sourceKey, async (key) => {
    if (!key) return null;
    if (!isTauri) return null;
    try {
      const source = buildEvidenceSourceInput(
        opts.file() ?? null,
        opts.entry(),
      );
      if (!source) return null;
      if (source.containerType === "disk" && source.path) {
        return await commands.viewer.detectType(source.path);
      }
      return await commands.viewer.detectTypeSource(source);
    } catch (e) {
      log.warn("Failed to detect file type:", e);
      return null;
    }
  });

  const [metadataResource] = createResource(sourceKey, async (key) => {
    if (!key) return null;
    if (!isTauri) return null;
    try {
      const source = buildEvidenceSourceInput(
        opts.file() ?? null,
        opts.entry(),
      );
      if (!source) return null;
      if (source.containerType === "disk" && source.path) {
        return await commands.viewer.parseHeader(source.path);
      }
      return await commands.viewer.parseHeaderSource(source);
    } catch {
      return null;
    }
  });

  createEffect(() => {
    const type = fileTypeResource();
    if (type !== undefined) setFileType(type);
  });

  createEffect(() => {
    const analysis = sourceAnalysisResource();
    if (analysis === undefined) return;

    if (!analysis) {
      if (opts.entry()) {
        setFileType(null);
        setMetadata(null);
        opts.onMetadataLoaded?.(null);
      }
      return;
    }

    const analysisMetadata = sourceAnalysisToMetadata(analysis);
    setMetadata((current) => mergeMetadata(analysisMetadata, current));
    setFileType((current) => current ?? sourceAnalysisToFileType(analysis));
    opts.onMetadataLoaded?.(mergeMetadata(analysisMetadata, metadata()));
  });

  createEffect(() => {
    const meta = metadataResource();
    if (meta !== undefined) {
      const analysisMetadata = sourceAnalysisResource()
        ? sourceAnalysisToMetadata(sourceAnalysisResource()!)
        : null;
      const merged = analysisMetadata
        ? mergeMetadata(analysisMetadata, meta)
        : meta;
      setMetadata(merged);
      opts.onMetadataLoaded?.(merged);
    }
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
        setSelectedRegion(null);
        setMetadata(null);
        setFileType(null);
        opts.onMetadataLoaded?.(null);

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

function sourceAnalysisToFileType(analysis: SourceAnalysis): FileTypeInfo {
  const signature = analysis.signatures?.[0];
  return {
    mime_type: signature?.mimeType ?? null,
    description:
      signature?.description ??
      (analysis.isLikelyText ? "Text Data" : "Binary Data"),
    extension: signature?.extensions?.[0] ?? "",
    is_text: analysis.isLikelyText,
    is_forensic_format: signature?.category === "forensic",
    magic_hex: analysis.magicHex,
  };
}

function sourceAnalysisToMetadata(analysis: SourceAnalysis): ParsedMetadata {
  const signature = analysis.signatures?.[0];
  const totalSize = analysis.totalSize ?? 0;
  const offset = analysis.offset ?? 0;
  const bytesAnalyzed = analysis.bytesAnalyzed ?? 0;
  const printableRatio = analysis.printableRatio ?? 0;
  const fields = [
    {
      key: "Source",
      value: analysis.sourceId ?? "Unknown source",
      category: "Analysis",
    },
    {
      key: "Total Size",
      value: `${totalSize.toLocaleString()} bytes`,
      category: "Analysis",
    },
    {
      key: "Analyzed Range",
      value: `0x${offset.toString(16).toUpperCase()} - 0x${(offset + bytesAnalyzed).toString(16).toUpperCase()} (${bytesAnalyzed.toLocaleString()} bytes)`,
      category: "Analysis",
      source_offset: offset,
    },
    {
      key: "Magic Bytes",
      value: analysis.magicHex || "None",
      category: "Signature",
      linked_region: "Magic Bytes",
      source_offset: 0,
    },
    {
      key: "Entropy",
      value: `${(analysis.entropy ?? 0).toFixed(3)} bits/byte`,
      category: "Byte Statistics",
    },
    {
      key: "Printable Bytes",
      value: `${(analysis.printableBytes ?? 0).toLocaleString()} (${(printableRatio * 100).toFixed(1)}%)`,
      category: "Byte Statistics",
    },
    {
      key: "NUL Bytes",
      value: (analysis.nulBytes ?? 0).toLocaleString(),
      category: "Byte Statistics",
    },
    {
      key: "High-bit Bytes",
      value: (analysis.highBitBytes ?? 0).toLocaleString(),
      category: "Byte Statistics",
    },
    {
      key: "Text Likelihood",
      value: analysis.isLikelyText ? "Likely text" : "Likely binary",
      category: "Byte Statistics",
    },
  ];

  if (signature) {
    fields.push(
      {
        key: "Detected Type",
        value: signature.description,
        category: "Signature",
      },
      {
        key: "MIME Type",
        value: signature.mimeType,
        category: "Signature",
      },
      {
        key: "Category",
        value: signature.category,
        category: "Signature",
      },
      {
        key: "Confidence",
        value: signature.confidence,
        category: "Signature",
      },
    );
  }

  const embeddedSignatures = (analysis.signatures ?? []).filter(
    (item) => item.offset > 0,
  );
  if (embeddedSignatures.length > 0) {
    fields.push({
      key: "Embedded Signatures",
      value: embeddedSignatures
        .slice(0, 12)
        .map(
          (item) =>
            `${item.description} @ 0x${item.offset.toString(16).toUpperCase()}`,
        )
        .join(", "),
      category: "Signature",
    });
  }

  const indicators = analysis.indicators ?? [];
  if (indicators.length > 0) {
    fields.push({
      key: "Extracted Indicators",
      value: indicators
        .slice(0, 16)
        .map(
          (item) =>
            `${item.indicatorType}: ${item.value} @ 0x${item.offset.toString(16).toUpperCase()}`,
        )
        .join(", "),
      category: "Indicators",
    });
  }

  if (analysis.asciiPreview) {
    fields.push({
      key: "ASCII Preview",
      value: analysis.asciiPreview,
      category: "Preview",
    });
  }

  return {
    format:
      signature?.description ??
      (analysis.isLikelyText ? "Text Data" : "Binary Data"),
    version: null,
    fields,
    regions: sourceAnalysisToRegions(analysis),
  };
}

function sourceAnalysisToRegions(analysis: SourceAnalysis): HeaderRegion[] {
  const regions: HeaderRegion[] = [];
  const totalSize = analysis.totalSize ?? 0;
  const offset = analysis.offset ?? 0;
  const bytesAnalyzed = analysis.bytesAnalyzed ?? 0;

  if (analysis.magicHex && totalSize > 0) {
    regions.push({
      start: 0,
      end: Math.min(16, totalSize),
      name: "Magic Bytes",
      color_class: "region-magic",
      description: "Initial bytes used for file signature detection",
    });
  }

  (analysis.signatures ?? [])
    .filter((signature) => signature.offset > 0)
    .slice(0, 24)
    .forEach((signature, index) => {
      const signatureLength = Math.max(
        1,
        signature.magicHex.split(/\s+/).filter(Boolean).length,
      );
      regions.push({
        start: signature.offset,
        end: Math.min(totalSize, signature.offset + signatureLength),
        name: `Embedded Signature ${index + 1}`,
        color_class: "region-metadata",
        description: `${signature.description} (${signature.mimeType}) at 0x${signature.offset.toString(16).toUpperCase()}`,
      });
    });

  if (offset > 0 && bytesAnalyzed > 0) {
    regions.push({
      start: offset,
      end: offset + Math.min(bytesAnalyzed, 256),
      name: "Analysis Window",
      color_class: "region-header",
      description: "Sampled byte range used for source analysis",
    });
  }

  (analysis.entropyWindows ?? [])
    .filter((window) => window.entropy >= 7.5 || window.entropy <= 1.0)
    .slice(0, 8)
    .forEach((window, index) => {
      const high = window.entropy >= 7.5;
      regions.push({
        start: window.offset,
        end: window.offset + window.length,
        name: `${high ? "High" : "Low"} Entropy ${index + 1}`,
        color_class: high ? "region-data" : "region-metadata",
        description: `${high ? "High" : "Low"} entropy window (${window.entropy.toFixed(3)} bits/byte)`,
      });
    });

  return regions;
}

function mergeMetadata(
  primary: ParsedMetadata,
  secondary: ParsedMetadata | null,
): ParsedMetadata {
  if (!secondary) return primary;

  return {
    format: secondary.format || primary.format,
    version: secondary.version ?? primary.version,
    fields: [...primary.fields, ...secondary.fields],
    regions: mergeRegions(primary.regions, secondary.regions),
  };
}

function mergeRegions(
  primary: HeaderRegion[],
  secondary: HeaderRegion[],
): HeaderRegion[] {
  const seen = new Set<string>();
  const merged: HeaderRegion[] = [];
  for (const region of [...primary, ...secondary]) {
    const key = `${region.start}:${region.end}:${region.name}`;
    if (seen.has(key)) continue;
    seen.add(key);
    merged.push(region);
  }
  return merged;
}
