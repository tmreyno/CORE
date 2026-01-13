// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import type { DiscoveredFile } from "../types";
import { HiOutlineExclamationTriangle } from "./icons";

// --- Constants ---
const DEFAULT_MAX_CHARS = 100000; // 100KB of text

interface TextViewerProps {
  file: DiscoveredFile;
}

export function TextViewer(props: TextViewerProps) {
  const [content, setContent] = createSignal<string>("");
  const [loading, setLoading] = createSignal(false);
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
  
  // Load file content
  const loadContent = async () => {
    setLoading(true);
    setError(null);
    
    try {
      const text = await invoke<string>("viewer_read_text", {
        path: props.file.path,
        offset: 0,
        maxChars: DEFAULT_MAX_CHARS
      });
      setContent(text);
      setLoadedChars(text.length);
      setTotalSize(props.file.size);
    } catch (e) {
      setError(`Failed to load file: ${e}`);
      setContent("");
    } finally {
      setLoading(false);
    }
  };
  
  // Load on file change
  createEffect(() => {
    const file = props.file;
    if (!file) return;
    
    setContent("");
    setError(null);
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
  
  // Ref for content container
  let contentRef: HTMLDivElement | undefined;
  
  // Split content into lines
  const lines = () => content().split('\n');
  
  // Detect language for syntax highlighting (basic detection)
  const detectLanguage = (): string => {
    const ext = props.file.filename.split('.').pop()?.toLowerCase() || '';
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
    <div class="flex flex-col h-full bg-zinc-900 text-sm">
      {/* Toolbar */}
      <div class="panel-header gap-4">
        <div class="row text-xs">
          <span class="text-cyan-400">{detectLanguage()}</span>
          <span class="text-zinc-500">
            {formatFileSize(loadedChars())}
            <Show when={isTruncated()}>
              {" / " + formatFileSize(totalSize()) + " (truncated)"}
            </Show>
          </span>
          <span class="text-zinc-500">{lines().length} lines</span>
        </div>
        
        <div class="flex-1 flex justify-center">
          {/* Search */}
          <div class="flex items-center gap-1 bg-zinc-800 rounded border border-zinc-700">
            <input
              type="text"
              class="w-40 px-2 py-1 text-xs bg-transparent text-zinc-200 outline-none"
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
              <span class="text-[10px] text-zinc-500 px-1">
                {searchResults().length > 0 
                  ? `${currentResult() + 1}/${searchResults().length}`
                  : "No results"
                }
              </span>
              <button class="px-1 text-xs text-zinc-400 hover:text-zinc-200" onClick={prevResult} title="Previous (Shift+Enter)">
                ▲
              </button>
              <button class="px-1 text-xs text-zinc-400 hover:text-zinc-200" onClick={nextResult} title="Next (Enter)">
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
              class="w-3.5 h-3.5 accent-cyan-500"
              checked={showLineNumbers()}
              onChange={e => setShowLineNumbers(e.currentTarget.checked)}
            />
            Lines
          </label>
          <label class="label-with-icon">
            <input
              type="checkbox"
              class="w-3.5 h-3.5 accent-cyan-500"
              checked={wordWrap()}
              onChange={e => setWordWrap(e.currentTarget.checked)}
            />
            Wrap
          </label>
          
          {/* Font size */}
          <div class="row gap-1 text-zinc-400">
            <button 
              class="px-1.5 py-0.5 rounded hover:bg-zinc-700 hover:text-zinc-200" 
              onClick={() => setFontSize(s => Math.max(10, s - 1))}
            >
              −
            </button>
            <span class="w-8 text-center">{fontSize()}px</span>
            <button 
              class="px-1.5 py-0.5 rounded hover:bg-zinc-700 hover:text-zinc-200" 
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
        <div class="flex-1 flex items-center justify-center text-zinc-500">Loading...</div>
      </Show>
      
      {/* Content */}
      <Show when={!loading() && content()}>
        <div 
          ref={contentRef}
          class={`flex-1 overflow-auto font-mono ${wordWrap() ? 'whitespace-pre-wrap' : 'whitespace-pre'}`}
          style={{ 'font-size': `${fontSize()}px` }}
        >
          <div class="flex">
            <Show when={showLineNumbers()}>
              <div class="flex flex-col text-right pr-3 mr-3 border-r border-zinc-700 bg-zinc-900 sticky left-0 select-none">
                {lines().map((_, i) => (
                  <div class="text-zinc-600 leading-relaxed">{i + 1}</div>
                ))}
              </div>
            </Show>
            <pre class="flex-1 text-zinc-300">
              <code class={`language-${detectLanguage()}`}>
                {content()}
              </code>
            </pre>
          </div>
        </div>
      </Show>
      
      {/* Truncation warning */}
      <Show when={!loading() && isTruncated()}>
        <div class="flex items-center gap-1.5 px-3 py-2 text-xs text-amber-400 bg-amber-900/20 border-t border-zinc-700">
          <HiOutlineExclamationTriangle class="w-4 h-4" /> File is truncated. Showing first {formatFileSize(loadedChars())} of {formatFileSize(totalSize())}.
        </div>
      </Show>
      
      {/* Empty state */}
      <Show when={!loading() && !content() && !error()}>
        <div class="flex-1 flex items-center justify-center text-zinc-500">
          Select a text file to view its contents
        </div>
      </Show>
    </div>
  );
}
