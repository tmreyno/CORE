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

import { createSignal, createEffect, createMemo, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { formatBytes, getBasename } from "../utils";
import { HiOutlineExclamationTriangle } from "solid-icons/hi";
import { formatDate } from "../utils/metadata";
import { logger } from '../utils/logger';
import { DocumentToolbar } from "./document/DocumentToolbar";
import { DocumentMetadataPanel } from "./document/DocumentMetadataPanel";
import { getFormatIcon, performSearch, printDocument, downloadHtml } from "./document/documentHelpers";

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
    logger.debug("[DocumentViewer] Loading document:", props.path);
    setLoading(true);
    setError(null);

    try {
      // Load content and metadata in parallel
      const [contentResult, metadataResult] = await Promise.all([
        invoke<DocumentResponse>("document_read", { path: props.path }),
        invoke<MetadataResponse>("document_get_metadata", { path: props.path }),
      ]);

      logger.debug("[DocumentViewer] Content result:", contentResult);
      
      if (!contentResult.success || !contentResult.content) {
        throw new Error(contentResult.error || "Failed to load document");
      }

      setContent(contentResult.content);
      logger.debug("[DocumentViewer] HTML length:", contentResult.content.html?.length);
      
      if (metadataResult.success && metadataResult.metadata) {
        setMetadata(metadataResult.metadata);
      }
    } catch (e) {
      logger.error("[DocumentViewer] Failed to load document:", e);
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
  const zoomIn = () => setScale((s) => Math.min(s + 0.25, 3.0));
  const zoomOut = () => setScale((s) => Math.max(s - 0.25, 0.5));
  const resetZoom = () => setScale(1.0);

  // Search within rendered HTML
  const handleSearch = () => {
    const count = performSearch(contentRef, searchQuery());
    setSearchHighlights(count);
  };

  // Print and download handlers
  const handlePrint = () => printDocument(content()?.html);
  const handleDownload = () => downloadHtml(content()?.html, content()?.title || "document");

  return (
    <div class={`document-viewer flex flex-col h-full ${props.class || ""}`}>
      {/* Toolbar */}
      <DocumentToolbar
        formatIcon={formatIcon()}
        documentFormat={documentFormat()}
        searchQuery={searchQuery()}
        searchHighlights={searchHighlights()}
        onSearchQueryChange={setSearchQuery}
        onSearchSubmit={handleSearch}
        zoomPercent={zoomPercent()}
        onZoomIn={zoomIn}
        onZoomOut={zoomOut}
        onResetZoom={resetZoom}
        showMetadataPanel={showMetadataPanel()}
        onToggleMetadata={() => setShowMetadataPanel(!showMetadataPanel())}
        onPrint={handlePrint}
        onDownload={handleDownload}
      />

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
                      class="btn-sm-primary mt-2"
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
          <DocumentMetadataPanel
            metadataDisplay={metadataDisplay()}
            documentTitle={documentTitle()}
            fileSizeDisplay={fileSizeDisplay()}
            pageCount={pageCount()}
          />
        </Show>
      </div>
    </div>
  );
}
