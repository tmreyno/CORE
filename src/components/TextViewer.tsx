// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { DiscoveredFile } from "../types";
import type { SelectedEntry } from "./EvidenceTree/types";
import { HiOutlineExclamationTriangle } from "./icons";
import { getExtension } from "../utils";

// --- Constants ---
const INITIAL_LOAD_SIZE = 100000; // 100KB initial text load
const LOAD_MORE_SIZE = 50000; // 50KB per additional load
const MAX_LOADED_CHARS = 2000000; // 2MB max loaded in memory
const SCROLL_THRESHOLD = 300; // pixels from bottom to trigger load

/**
 * Read text from any source: disk file, AD1 container entry, VFS entry, or archive entry
 */
async function readTextFromSource(
  file: DiscoveredFile | null,
  entry: SelectedEntry | undefined,
  offset: number,
  maxChars: number
): Promise<{ text: string; totalSize: number }> {
  // Case 1: SelectedEntry provided (container file viewing)
  if (entry) {
    // VFS entries (E01/Raw)
    if (entry.isVfsEntry) {
      const bytes = await invoke<number[]>("vfs_read_file", {
        containerPath: entry.containerPath,
        filePath: entry.entryPath,
        offset,
        length: maxChars
      });
      const text = new TextDecoder("utf-8", { fatal: false }).decode(new Uint8Array(bytes));
      return { text, totalSize: entry.size };
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
          size: maxChars
        });
        const text = new TextDecoder("utf-8", { fatal: false }).decode(new Uint8Array(bytes));
        return { text, totalSize: entry.size };
      }
      
      // Regular archive entry
      const bytes = await invoke<number[]>("archive_read_entry_chunk", {
        containerPath: entry.containerPath,
        entryPath: entry.entryPath,
        offset,
        size: maxChars
      });
      const text = new TextDecoder("utf-8", { fatal: false }).decode(new Uint8Array(bytes));
      return { text, totalSize: entry.size };
    }
    
    // Disk file entry (file inside container that's actually on disk)
    if (entry.isDiskFile) {
      const bytes = await invoke<number[]>("read_file_bytes", {
        path: entry.entryPath,
        offset,
        length: maxChars
      });
      const text = new TextDecoder("utf-8", { fatal: false }).decode(new Uint8Array(bytes));
      return { text, totalSize: entry.size };
    }
    
    // AD1 container entry - use chunk-based reading for scroll support
    const bytes = await invoke<number[]>("container_read_entry_chunk", {
      containerPath: entry.containerPath,
      entryPath: entry.entryPath,
      offset,
      size: maxChars
    });
    const text = new TextDecoder("utf-8", { fatal: false }).decode(new Uint8Array(bytes));
    return { text, totalSize: entry.size };
  }
  
  // Case 2: Regular disk file (DiscoveredFile)
  if (file) {
    const text = await invoke<string>("viewer_read_text", {
      path: file.path,
      offset,
      maxChars
    });
    return { text, totalSize: file.size };
  }
  
  throw new Error("No file or entry provided");
}

interface TextViewerProps {
  /** Regular disk file */
  file?: DiscoveredFile | null;
  /** Container entry (file inside AD1/E01/etc.) */
  entry?: SelectedEntry;
}

export function TextViewer(props: TextViewerProps) {
  // Content ref for scroll handling
  let contentRef: HTMLDivElement | undefined;
  
  const [content, setContent] = createSignal<string>("");
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
  
  // Get the source identifier for change detection
  const sourceKey = () => {
    if (props.entry) return `entry:${props.entry.containerPath}:${props.entry.entryPath}`;
    if (props.file) return `file:${props.file.path}`;
    return null;
  };
  
  // Get the filename for display purposes
  const displayFilename = () => {
    if (props.entry) return props.entry.name;
    if (props.file) return props.file.filename;
    return "";
  };
  
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
    
    // Don't load if we've already loaded everything or hit the max
    if (currentLoaded >= total || currentLoaded >= MAX_LOADED_CHARS) return;
    
    setLoadingMore(true);
    
    try {
      const result = await readTextFromSource(
        props.file ?? null, 
        props.entry, 
        currentLoaded, 
        Math.min(LOAD_MORE_SIZE, total - currentLoaded, MAX_LOADED_CHARS - currentLoaded)
      );
      
      // Append new text to existing
      setContent(prev => prev + result.text);
      setLoadedChars(currentLoaded + result.text.length);
    } catch (e) {
      console.error("Failed to load more text:", e);
    } finally {
      setLoadingMore(false);
    }
  };
  
  // Handle scroll events
  const handleScroll = () => {
    if (!contentRef) return;
    
    const { scrollTop, scrollHeight, clientHeight } = contentRef;
    const distanceFromBottom = scrollHeight - scrollTop - clientHeight;
    
    // Load more when user scrolls near the bottom
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
    const query = searchQuery().toLowerCase();
    const text = content().toLowerCase();
    
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
  
  // Navigate search results
  const nextResult = () => {
    const results = searchResults();
    if (results.length === 0) return;
    setCurrentResult((currentResult() + 1) % results.length);
    scrollToResult();
  };
  
  const prevResult = () => {
    const results = searchResults();
    if (results.length === 0) return;
    setCurrentResult((currentResult() - 1 + results.length) % results.length);
    scrollToResult();
  };
  
  const scrollToResult = () => {
    if (!contentRef) return;
    const results = searchResults();
    const idx = currentResult();
    if (idx < 0 || idx >= results.length) return;
    
    const charPos = results[idx];
    const text = content();
    
    // Find which line this character position is on
    let lineNum = 0;
    let charCount = 0;
    const textLines = text.split('\n');
    for (let i = 0; i < textLines.length; i++) {
      const lineLength = textLines[i].length + 1; // +1 for newline
      if (charCount + lineLength > charPos) {
        lineNum = i;
        break;
      }
      charCount += lineLength;
    }
    
    // Calculate scroll position based on line number
    // Approximate line height from font size (1.5 line-height typical for code)
    const lineHeight = fontSize() * 1.5;
    const scrollTop = lineNum * lineHeight;
    
    // Scroll to position, centering the result in the viewport
    const viewportHeight = contentRef.clientHeight;
    const targetScroll = Math.max(0, scrollTop - viewportHeight / 2);
    
    contentRef.scrollTo({
      top: targetScroll,
      behavior: 'smooth'
    });
  };
  
  // Split content into lines
  const lines = () => content().split('\n');
  
  // Detect language for syntax highlighting (basic detection)
  const detectLanguage = (): string => {
    const filename = displayFilename();
    const ext = getExtension(filename);
    const langMap: Record<string, string> = {
      'js': 'javascript',
      'ts': 'typescript',
      'jsx': 'javascript',
      'tsx': 'typescript',
      'py': 'python',
      'rs': 'rust',
      'go': 'go',
      'java': 'java',
      'c': 'c',
      'cpp': 'cpp',
      'h': 'c',
      'hpp': 'cpp',
      'cs': 'csharp',
      'rb': 'ruby',
      'php': 'php',
      'html': 'html',
      'htm': 'html',
      'css': 'css',
      'scss': 'scss',
      'sass': 'sass',
      'less': 'less',
      'json': 'json',
      'xml': 'xml',
      'yaml': 'yaml',
      'yml': 'yaml',
      'toml': 'toml',
      'md': 'markdown',
      'sql': 'sql',
      'sh': 'bash',
      'bash': 'bash',
      'zsh': 'bash',
      'ps1': 'powershell',
      'bat': 'batch',
      'cmd': 'batch',
    };
    return langMap[ext] || 'plaintext';
  };
  
  const formatFileSize = (bytes: number): string => {
    if (bytes < 1024) return `${bytes} B`;
    if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  };
  
  const isTruncated = () => loadedChars() < totalSize();

  return (
    <div class="flex flex-col h-full bg-bg text-sm">
      {/* Toolbar */}
      <div class="panel-header gap-4">
        <div class="row text-xs">
          <span class="text-accent">{detectLanguage()}</span>
          <span class="text-txt-muted">
            {formatFileSize(loadedChars())}
            <Show when={isTruncated()}>
              {" / " + formatFileSize(totalSize()) + " (truncated)"}
            </Show>
          </span>
          <span class="text-txt-muted">{lines().length} lines</span>
        </div>
        
        <div class="flex-1 flex justify-center">
          {/* Search */}
          <div class="flex items-center gap-1 bg-bg-panel rounded border border-border">
            <input
              type="text"
              class="w-40 px-2 py-1 text-xs bg-transparent text-txt outline-none"
              placeholder="Search..."
              value={searchQuery()}
              onInput={e => setSearchQuery(e.currentTarget.value)}
              onKeyDown={e => {
                if (e.key === "Enter") {
                  e.shiftKey ? prevResult() : nextResult();
                }
              }}
            />
            <Show when={searchQuery()}>
              <span class={`text-[10px] leading-tight text-txt-muted px-1`}>
                {searchResults().length > 0 
                  ? `${currentResult() + 1}/${searchResults().length}`
                  : "No results"
                }
              </span>
              <button class="px-1 text-xs text-txt-secondary hover:text-txt" onClick={prevResult} title="Previous (Shift+Enter)">
                ▲
              </button>
              <button class="px-1 text-xs text-txt-secondary hover:text-txt" onClick={nextResult} title="Next (Enter)">
                ▼
              </button>
            </Show>
          </div>
        </div>
        
        <div class="row gap-3 text-xs">
          {/* View options */}
          <label class="label-with-icon">
            <input
              type="checkbox"
              class="w-3.5 h-3.5 accent-accent"
              checked={showLineNumbers()}
              onChange={e => setShowLineNumbers(e.currentTarget.checked)}
            />
            Lines
          </label>
          <label class="label-with-icon">
            <input
              type="checkbox"
              class="w-3.5 h-3.5 accent-accent"
              checked={wordWrap()}
              onChange={e => setWordWrap(e.currentTarget.checked)}
            />
            Wrap
          </label>
          
          {/* Font size */}
          <div class="row gap-1 text-txt-secondary">
            <button 
              class="px-1.5 py-0.5 rounded hover:bg-bg-hover hover:text-txt" 
              onClick={() => setFontSize(s => Math.max(10, s - 1))}
            >
              −
            </button>
            <span class="w-8 text-center">{fontSize()}px</span>
            <button 
              class="px-1.5 py-0.5 rounded hover:bg-bg-hover hover:text-txt" 
              onClick={() => setFontSize(s => Math.min(24, s + 1))}
            >
              +
            </button>
          </div>
        </div>
      </div>
      
      {/* Error display */}
      <Show when={error()}>
        <div class="p-4 text-red-400 bg-red-900/20">{error()}</div>
      </Show>
      
      {/* Loading indicator */}
      <Show when={loading()}>
        <div class="flex-1 flex items-center justify-center text-txt-muted">Loading...</div>
      </Show>
      
      {/* Content */}
      <Show when={!loading() && content()}>
        <div 
          ref={contentRef}
          class={`flex-1 overflow-auto font-mono ${wordWrap() ? 'whitespace-pre-wrap' : 'whitespace-pre'}`}
          style={{ 'font-size': `${fontSize()}px` }}
          onScroll={handleScroll}
        >
          <div class="flex">
            <Show when={showLineNumbers()}>
              <div class="flex flex-col text-right pr-3 mr-3 border-r border-border bg-bg sticky left-0 select-none">
                {lines().map((_, i) => (
                  <div class="text-txt-muted leading-relaxed">{i + 1}</div>
                ))}
              </div>
            </Show>
            <pre class="flex-1 text-txt-tertiary">
              <code class={`language-${detectLanguage()}`}>
                {content()}
              </code>
            </pre>
          </div>
          
          {/* Loading more indicator */}
          <Show when={loadingMore()}>
            <div class="flex items-center justify-center py-4 text-txt-muted text-xs">
              Loading more...
            </div>
          </Show>
          
          {/* End of file indicator */}
          <Show when={!loadingMore() && loadedChars() >= totalSize()}>
            <div class="flex items-center justify-center py-4 text-txt-muted text-xs">
              — End of file —
            </div>
          </Show>
          
          {/* Max loaded indicator */}
          <Show when={!loadingMore() && loadedChars() >= MAX_LOADED_CHARS && totalSize() > MAX_LOADED_CHARS}>
            <div class="flex items-center justify-center py-4 text-amber-500/70 text-xs">
              — Maximum preview size reached ({formatFileSize(MAX_LOADED_CHARS)}) —
            </div>
          </Show>
        </div>
      </Show>
      
      {/* Progress info bar */}
      <Show when={!loading() && isTruncated()}>
        <div class="flex items-center gap-2 px-3 py-2 text-xs text-amber-400 bg-amber-900/20 border-t border-border">
          <HiOutlineExclamationTriangle class="w-4 h-4" />
          <span>Loaded {formatFileSize(loadedChars())} of {formatFileSize(totalSize())} ({Math.round((loadedChars() / totalSize()) * 100)}%)</span>
          <Show when={loadedChars() < MAX_LOADED_CHARS}>
            <button 
              class="px-2 py-0.5 bg-bg-hover hover:bg-bg-active rounded text-txt-tertiary"
              onClick={loadMoreContent}
              disabled={loadingMore()}
            >
              {loadingMore() ? "Loading..." : "Load More"}
            </button>
          </Show>
          <Show when={loadedChars() >= MAX_LOADED_CHARS}>
            <span class="text-txt-muted">(max preview reached)</span>
          </Show>
        </div>
      </Show>
      
      {/* Empty state */}
      <Show when={!loading() && !content() && !error()}>
        <div class="flex-1 flex items-center justify-center text-txt-muted">
          Select a text file to view its contents
        </div>
      </Show>
    </div>
  );
}
