// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, createMemo, type Accessor } from "solid-js";
import { getExtension } from "../../utils";
import { getPreference } from "../preferences";
import { readTextFromSource, getSourceKey, getSourceFilename } from "../../hooks";
import { logger } from "../../utils/logger";
import type { TextViewerProps } from "./types";
import {
  INITIAL_LOAD_SIZE,
  LOAD_MORE_SIZE,
  SCROLL_THRESHOLD,
  getMaxLoadedChars,
  LANGUAGE_MAP,
} from "./constants";

const log = logger.scope("TextViewer");

export interface UseTextViewerReturn {
  content: Accessor<string>;
  loading: Accessor<boolean>;
  loadingMore: Accessor<boolean>;
  error: Accessor<string | null>;
  totalSize: Accessor<number>;
  loadedChars: Accessor<number>;
  maxLoadedChars: Accessor<number>;
  showLineNumbers: Accessor<boolean>;
  setShowLineNumbers: (v: boolean) => void;
  wordWrap: Accessor<boolean>;
  setWordWrap: (v: boolean) => void;
  fontSize: Accessor<number>;
  setFontSize: (fn: (prev: number) => number) => void;
  searchQuery: Accessor<string>;
  setSearchQuery: (v: string) => void;
  searchResults: Accessor<number[]>;
  currentResult: Accessor<number>;
  displayFilename: Accessor<string>;
  lines: Accessor<string[]>;
  lineCount: Accessor<number>;
  isTruncated: Accessor<boolean>;
  detectLanguage: () => string;
  nextResult: () => void;
  prevResult: () => void;
  loadMoreContent: () => Promise<void>;
  handleScroll: (contentRef: HTMLDivElement) => void;
  scrollToResult: (contentRef: HTMLDivElement | undefined) => void;
}

export function useTextViewer(props: TextViewerProps): UseTextViewerReturn {
  const maxLoadedCharsMemo = createMemo(() => getMaxLoadedChars());

  const [content, setContent] = createSignal("");
  const [loading, setLoading] = createSignal(false);
  const [loadingMore, setLoadingMore] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [totalSize, setTotalSize] = createSignal(0);
  const [loadedChars, setLoadedChars] = createSignal(0);

  // View options
  const [showLineNumbers, setShowLineNumbers] = createSignal(true);
  const [wordWrap, setWordWrap] = createSignal(true);
  const [fontSize, setFontSize] = createSignal(13);

  // Search
  const [searchQuery, setSearchQuery] = createSignal("");
  const [searchResults, setSearchResults] = createSignal<number[]>([]);
  const [currentResult, setCurrentResult] = createSignal(0);

  const sourceKey = () => getSourceKey(props.file, props.entry);
  const displayFilename = () => getSourceFilename(props.file, props.entry);

  // Load initial file content
  const loadContent = async () => {
    setLoading(true);
    setError(null);

    try {
      const result = await readTextFromSource(props.file ?? null, props.entry, 0, INITIAL_LOAD_SIZE);
      setContent(result.text);
      setLoadedChars(result.text.length);
      setTotalSize(result.totalSize);
    } catch (e) {
      setError(`Failed to load file: ${e}`);
      setContent("");
    } finally {
      setLoading(false);
    }
  };

  // Load more content when scrolling
  const loadMoreContent = async () => {
    if (loadingMore() || loading()) return;

    const currentLoaded = loadedChars();
    const total = totalSize();
    const maxChars = getMaxLoadedChars();

    if (currentLoaded >= total || currentLoaded >= maxChars) return;

    setLoadingMore(true);

    try {
      const result = await readTextFromSource(
        props.file ?? null,
        props.entry,
        currentLoaded,
        Math.min(LOAD_MORE_SIZE, total - currentLoaded, maxChars - currentLoaded)
      );

      setContent((prev) => prev + result.text);
      setLoadedChars(currentLoaded + result.text.length);
    } catch (e) {
      log.error("Failed to load more text:", e);
    } finally {
      setLoadingMore(false);
    }
  };

  // Handle scroll events
  const handleScroll = (contentRef: HTMLDivElement) => {
    const { scrollTop, scrollHeight, clientHeight } = contentRef;
    const distanceFromBottom = scrollHeight - scrollTop - clientHeight;

    if (distanceFromBottom < SCROLL_THRESHOLD) {
      loadMoreContent();
    }
  };

  // Load on file/entry change
  createEffect(() => {
    const key = sourceKey();
    if (!key) return;

    setContent("");
    setError(null);
    setLoadedChars(0);
    setTotalSize(0);
    loadContent();
  });

  // Search functionality
  createEffect(() => {
    const caseSensitive = getPreference("caseSensitiveSearch");
    const query = caseSensitive ? searchQuery() : searchQuery().toLowerCase();
    const text = caseSensitive ? content() : content().toLowerCase();

    if (!query || !text) {
      setSearchResults([]);
      return;
    }

    const results: number[] = [];
    let pos = 0;
    while ((pos = text.indexOf(query, pos)) !== -1) {
      results.push(pos);
      pos += 1;
    }
    setSearchResults(results);
    setCurrentResult(results.length > 0 ? 0 : -1);
  });

  const nextResult = () => {
    const results = searchResults();
    if (results.length === 0) return;
    setCurrentResult((currentResult() + 1) % results.length);
  };

  const prevResult = () => {
    const results = searchResults();
    if (results.length === 0) return;
    setCurrentResult((currentResult() - 1 + results.length) % results.length);
  };

  const scrollToResult = (contentRef: HTMLDivElement | undefined) => {
    if (!contentRef) return;
    const results = searchResults();
    const idx = currentResult();
    if (idx < 0 || idx >= results.length) return;

    const charPos = results[idx];
    const text = content();

    let lineNum = 0;
    let charCount = 0;
    const textLines = text.split("\n");
    for (let i = 0; i < textLines.length; i++) {
      const lineLength = textLines[i].length + 1;
      if (charCount + lineLength > charPos) {
        lineNum = i;
        break;
      }
      charCount += lineLength;
    }

    const lineHeight = fontSize() * 1.5;
    const scrollTop = lineNum * lineHeight;
    const viewportHeight = contentRef.clientHeight;
    const targetScroll = Math.max(0, scrollTop - viewportHeight / 2);

    contentRef.scrollTo({
      top: targetScroll,
      behavior: "smooth",
    });
  };

  const lines = createMemo(() => content().split("\n"));
  const lineCount = createMemo(() => lines().length);

  const detectLanguage = (): string => {
    const filename = displayFilename();
    const ext = getExtension(filename);
    return LANGUAGE_MAP[ext] || "plaintext";
  };

  const isTruncated = createMemo(() => loadedChars() < totalSize());

  return {
    content,
    loading,
    loadingMore,
    error,
    totalSize,
    loadedChars,
    maxLoadedChars: maxLoadedCharsMemo,
    showLineNumbers,
    setShowLineNumbers,
    wordWrap,
    setWordWrap,
    fontSize,
    setFontSize,
    searchQuery,
    setSearchQuery,
    searchResults,
    currentResult,
    displayFilename,
    lines,
    lineCount,
    isTruncated,
    detectLanguage,
    nextResult,
    prevResult,
    loadMoreContent,
    handleScroll,
    scrollToResult,
  };
}
