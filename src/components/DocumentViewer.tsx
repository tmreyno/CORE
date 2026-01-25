// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Document Viewer Component - Unified viewer for PDF, DOCX, HTML, Markdown
 * 
 * Uses Rust backend to render documents to HTML, eliminating the need
 * for JavaScript libraries like PDF.js for basic viewing.
 * 
 * Features:
 * - Unified interface for multiple document formats
 * - Metadata display
 * - Search within document
 * - Print support via HTML rendering
 * - Zoom controls
 */

import { createSignal, createEffect, createMemo, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { formatBytes, getBasename } from "../utils";
import {
  HiOutlineMagnifyingGlassPlus,
  HiOutlineMagnifyingGlassMinus,
  HiOutlineExclamationTriangle,
  HiOutlineMagnifyingGlass,
  HiOutlineInformationCircle,
  HiOutlinePrinter,
  HiOutlineArrowDownTray,
} from "./icons";
import { getPreference } from "./preferences";
import { formatDate } from "../utils/metadata";

// ============================================================================
// Types
// ============================================================================

interface DocumentViewerProps {
  /** Path to the document file */
  path: string;
  /** Optional class name */
  class?: string;
  /** Whether to show metadata panel */
  showMetadata?: boolean;
}

interface DocumentMetadata {
  format: string;
  title: string | null;
  author: string | null;
  subject: string | null;
  keywords: string[];
  page_count: number;
  file_size: number;
  created: string | null;
  modified: string | null;
  producer: string | null;
  creator: string | null;
  encrypted: boolean;
  word_count: number | null;
}

interface DocumentContentDto {
  format: string;
  title: string | null;
  author: string | null;
  page_count: number;
  file_size: number;
  text: string;
  html: string;
}

interface DocumentResponse {
  success: boolean;
  content: DocumentContentDto | null;
  error: string | null;
}

interface MetadataResponse {
  success: boolean;
  metadata: DocumentMetadata | null;
  error: string | null;
}

// ============================================================================
// Utility Functions
// ============================================================================

/**
 * Get document format icon
 */
function getFormatIcon(format: string): string {
  switch (format?.toLowerCase()) {
    case "pdf": return "📄";
    case "docx": return "📝";
    case "html": return "🌐";
    case "markdown": return "📋";
    default: return "📃";
  }
}

// ============================================================================
// Component
// ============================================================================

export function DocumentViewer(props: DocumentViewerProps) {
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [content, setContent] = createSignal<DocumentContentDto | null>(null);
  const [metadata, setMetadata] = createSignal<DocumentMetadata | null>(null);
  const [scale, setScale] = createSignal(1.0);
  const [showMetadataPanel, setShowMetadataPanel] = createSignal(false);
  const [searchQuery, setSearchQuery] = createSignal("");
  const [searchHighlights, setSearchHighlights] = createSignal(0);

  let contentRef: HTMLDivElement | undefined;

  // Memoized computed values
  const filename = createMemo(() => getBasename(props.path) || props.path);
  const documentFormat = createMemo(() => content()?.format?.toLowerCase() || 'unknown');
  const formatIcon = createMemo(() => getFormatIcon(documentFormat()));
  const documentTitle = createMemo(() => content()?.title || filename());
  const pageCount = createMemo(() => content()?.page_count ?? metadata()?.page_count ?? 0);
  const fileSize = createMemo(() => content()?.file_size ?? metadata()?.file_size ?? 0);
  const htmlContent = createMemo(() => content()?.html || '');
  const hasMetadata = createMemo(() => metadata() !== null);
  const zoomPercent = createMemo(() => Math.round(scale() * 100));

  // Memoized metadata display values
  const metadataDisplay = createMemo(() => {
    const meta = metadata();
    if (!meta) return null;
    return {
      title: meta.title,
      author: meta.author,
      subject: meta.subject,
      format: meta.format,
      creator: meta.creator,
      producer: meta.producer,
      keywords: meta.keywords || [],
      wordCount: meta.word_count?.toLocaleString() ?? null,
      encrypted: meta.encrypted,
      createdDate: meta.created ? formatDate(meta.created) : null,
      modifiedDate: meta.modified ? formatDate(meta.modified) : null,
    };
  });

  // Memoized file size display
  const fileSizeDisplay = createMemo(() => formatBytes(fileSize()));

  // Load document
  const loadDocument = async () => {
    console.log("[DocumentViewer] Loading document:", props.path);
    setLoading(true);
    setError(null);

    try {
      // Load content and metadata in parallel
      const [contentResult, metadataResult] = await Promise.all([
        invoke<DocumentResponse>("document_read", { path: props.path }),
        invoke<MetadataResponse>("document_get_metadata", { path: props.path }),
      ]);

      console.log("[DocumentViewer] Content result:", contentResult);
      
      if (!contentResult.success || !contentResult.content) {
        throw new Error(contentResult.error || "Failed to load document");
      }

      setContent(contentResult.content);
      console.log("[DocumentViewer] HTML length:", contentResult.content.html?.length);
      
      if (metadataResult.success && metadataResult.metadata) {
        setMetadata(metadataResult.metadata);
      }
    } catch (e) {
      console.error("[DocumentViewer] Failed to load document:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  // Effect to load document when path changes
  createEffect(() => {
    if (props.path) {
      loadDocument();
    }
  });

  // Zoom controls
  const zoomIn = () => {
    setScale((s) => Math.min(s + 0.25, 3.0));
  };

  const zoomOut = () => {
    setScale((s) => Math.max(s - 0.25, 0.5));
  };

  const resetZoom = () => {
    setScale(1.0);
  };

  // Search within rendered HTML
  const performSearch = () => {
    const query = searchQuery();
    if (!query || !contentRef) {
      setSearchHighlights(0);
      return;
    }

    // Clear previous highlights
    const highlighted = contentRef.querySelectorAll(".search-highlight");
    highlighted.forEach((el) => {
      const parent = el.parentNode;
      if (parent) {
        parent.replaceChild(document.createTextNode(el.textContent || ""), el);
        parent.normalize();
      }
    });

    // Simple text highlight (for demonstration)
    // In production, use a proper text search/highlight library
    let count = 0;
    const walk = document.createTreeWalker(contentRef, NodeFilter.SHOW_TEXT, null);
    const matches: { node: Text; start: number; length: number }[] = [];
    
    const caseSensitive = getPreference("caseSensitiveSearch");
    let node: Text | null;
    while ((node = walk.nextNode() as Text)) {
      const text = node.textContent || "";
      const searchText = caseSensitive ? text : text.toLowerCase();
      const searchTerm = caseSensitive ? query : query.toLowerCase();
      let start = 0;
      let idx: number;
      while ((idx = searchText.indexOf(searchTerm, start)) !== -1) {
        matches.push({ node, start: idx, length: query.length });
        count++;
        start = idx + 1;
      }
    }

    setSearchHighlights(count);

    // Highlight first match (scroll into view)
    if (matches.length > 0 && contentRef) {
      // Simplified - just report the count
    }
  };

  // Print document
  const printDocument = () => {
    const html = content()?.html;
    if (!html) return;

    const printWindow = window.open("", "_blank");
    if (printWindow) {
      printWindow.document.write(html);
      printWindow.document.close();
      printWindow.onload = () => {
        printWindow.print();
      };
    }
  };

  // Download as HTML
  const downloadHtml = () => {
    const html = content()?.html;
    const title = content()?.title || "document";
    if (!html) return;

    const blob = new Blob([html], { type: "text/html" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `${title}.html`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <div class={`document-viewer flex flex-col h-full ${props.class || ""}`}>
      {/* Toolbar */}
      <div class="document-toolbar flex items-center gap-2 p-2 border-b border-border bg-bg-secondary">
        {/* Format indicator */}
        <div class="flex items-center gap-1 px-2 py-1 bg-bg-hover rounded text-sm">
          <span>{formatIcon()}</span>
          <span class="font-medium">{documentFormat() || "Document"}</span>
        </div>

        <div class="flex-1" />

        {/* Search */}
        <div class="flex items-center gap-1">
          <div class="relative">
            <input
              type="text"
              value={searchQuery()}
              onInput={(e) => setSearchQuery(e.currentTarget.value)}
              onKeyPress={(e) => e.key === "Enter" && performSearch()}
              placeholder="Search..."
              class="w-40 px-2 py-1 pl-7 text-sm rounded border border-border bg-bg-panel"
            />
            <HiOutlineMagnifyingGlass class="absolute left-2 top-1/2 -translate-y-1/2 w-4 h-4 text-txt-muted" />
          </div>
          <Show when={searchHighlights() > 0}>
            <span class="text-xs text-txt-muted">{searchHighlights()} found</span>
          </Show>
        </div>

        {/* Zoom controls */}
        <div class="flex items-center gap-1 border-l border-border pl-2 ml-2">
          <button
            onClick={zoomOut}
            class="p-1 rounded hover:bg-bg-hover"
            title="Zoom out"
          >
            <HiOutlineMagnifyingGlassMinus class="w-5 h-5" />
          </button>
          <span class="text-sm w-12 text-center">{zoomPercent()}%</span>
          <button
            onClick={zoomIn}
            class="p-1 rounded hover:bg-bg-hover"
            title="Zoom in"
          >
            <HiOutlineMagnifyingGlassPlus class="w-5 h-5" />
          </button>
          <button
            onClick={resetZoom}
            class="text-xs px-2 py-1 rounded hover:bg-bg-hover"
          >
            Reset
          </button>
        </div>

        {/* Actions */}
        <div class="flex items-center gap-1 border-l border-border pl-2 ml-2">
          <button
            onClick={() => setShowMetadataPanel(!showMetadataPanel())}
            class={`p-1 rounded hover:bg-bg-hover ${showMetadataPanel() ? "bg-bg-active" : ""}`}
            title="Show metadata"
          >
            <HiOutlineInformationCircle class="w-5 h-5" />
          </button>
          <button
            onClick={printDocument}
            class="p-1 rounded hover:bg-bg-hover"
            title="Print"
          >
            <HiOutlinePrinter class="w-5 h-5" />
          </button>
          <button
            onClick={downloadHtml}
            class="p-1 rounded hover:bg-bg-hover"
            title="Download as HTML"
          >
            <HiOutlineArrowDownTray class="w-5 h-5" />
          </button>
        </div>
      </div>

      {/* Main content area */}
      <div class="flex-1 flex overflow-hidden">
        {/* Document content */}
        <div class="flex-1 overflow-auto bg-bg">
          <Show
            when={!loading()}
            fallback={
              <div class="flex items-center justify-center h-full">
                <div class="flex flex-col items-center gap-2">
                  <div class="animate-spin w-8 h-8 border-2 border-accent border-t-transparent rounded-full" />
                  <span class="text-txt-muted">Loading document...</span>
                </div>
              </div>
            }
          >
            <Show
              when={!error()}
              fallback={
                <div class="flex items-center justify-center h-full">
                  <div class="flex flex-col items-center gap-2 text-error">
                    <HiOutlineExclamationTriangle class="w-12 h-12" />
                    <span class="font-medium">Failed to load document</span>
                    <span class="text-sm text-txt-muted">{error()}</span>
                    <button
                      onClick={loadDocument}
                      class="btn btn-primary mt-2"
                    >
                      Retry
                    </button>
                  </div>
                </div>
              }
            >
              <div
                ref={contentRef}
                class="document-content h-full"
                style={{
                  transform: `scale(${scale()})`,
                  "transform-origin": "top left",
                }}
                innerHTML={htmlContent() || "<p>No content</p>"}
              />
            </Show>
          </Show>
        </div>

        {/* Metadata panel */}
        <Show when={showMetadataPanel() && hasMetadata()}>
          <div class="w-72 border-l border-border bg-bg-panel overflow-y-auto p-4">
            <h3 class="font-semibold mb-4 flex items-center gap-2">
              <HiOutlineInformationCircle class="w-5 h-5" />
              Document Metadata
            </h3>
            
            <div class="space-y-3 text-sm">
              <Show when={metadataDisplay()?.title}>
                <div>
                  <div class="text-txt-muted text-xs uppercase">Title</div>
                  <div class="font-medium">{documentTitle()}</div>
                </div>
              </Show>
              
              <Show when={metadataDisplay()?.author}>
                <div>
                  <div class="text-txt-muted text-xs uppercase">Author</div>
                  <div>{metadataDisplay()?.author}</div>
                </div>
              </Show>
              
              <Show when={metadataDisplay()?.subject}>
                <div>
                  <div class="text-txt-muted text-xs uppercase">Subject</div>
                  <div>{metadataDisplay()?.subject}</div>
                </div>
              </Show>
              
              <div>
                <div class="text-txt-muted text-xs uppercase">Format</div>
                <div>{metadataDisplay()?.format}</div>
              </div>
              
              <div>
                <div class="text-txt-muted text-xs uppercase">File Size</div>
                <div>{fileSizeDisplay()}</div>
              </div>
              
              <Show when={pageCount() > 0}>
                <div>
                  <div class="text-txt-muted text-xs uppercase">Pages</div>
                  <div>{pageCount()}</div>
                </div>
              </Show>
              
              <Show when={metadataDisplay()?.wordCount}>
                <div>
                  <div class="text-txt-muted text-xs uppercase">Word Count</div>
                  <div>{metadataDisplay()?.wordCount}</div>
                </div>
              </Show>
              
              <Show when={metadataDisplay()?.createdDate}>
                <div>
                  <div class="text-txt-muted text-xs uppercase">Created</div>
                  <div>{metadataDisplay()?.createdDate}</div>
                </div>
              </Show>
              
              <Show when={metadataDisplay()?.modifiedDate}>
                <div>
                  <div class="text-txt-muted text-xs uppercase">Modified</div>
                  <div>{metadataDisplay()?.modifiedDate}</div>
                </div>
              </Show>
              
              <Show when={metadataDisplay()?.creator}>
                <div>
                  <div class="text-txt-muted text-xs uppercase">Creator</div>
                  <div>{metadataDisplay()?.creator}</div>
                </div>
              </Show>
              
              <Show when={metadataDisplay()?.producer}>
                <div>
                  <div class="text-txt-muted text-xs uppercase">Producer</div>
                  <div>{metadataDisplay()?.producer}</div>
                </div>
              </Show>
              
              <Show when={(metadataDisplay()?.keywords?.length ?? 0) > 0}>
                <div>
                  <div class="text-txt-muted text-xs uppercase">Keywords</div>
                  <div class="flex flex-wrap gap-1 mt-1">
                    <For each={metadataDisplay()?.keywords}>
                      {(keyword) => (
                        <span class="px-2 py-0.5 bg-bg-hover rounded text-xs">
                          {keyword}
                        </span>
                      )}
                    </For>
                  </div>
                </div>
              </Show>
              
              <Show when={metadataDisplay()?.encrypted}>
                <div class="flex items-center gap-2 text-warning">
                  <HiOutlineExclamationTriangle class="w-4 h-4" />
                  <span>Encrypted document</span>
                </div>
              </Show>
            </div>
          </div>
        </Show>
      </div>
    </div>
  );
}
