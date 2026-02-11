// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * PDF Viewer Component - Renders PDF files using PDF.js
 * 
 * Features:
 * - Page-by-page navigation
 * - Zoom controls
 * - Full page view
 * - Page thumbnails sidebar
 */

import { createSignal, createEffect, Show, onCleanup, onMount } from "solid-js";
import { createResizeObserver } from "@solid-primitives/resize-observer";
import { debounce } from "@solid-primitives/scheduled";
import { makeEventListener } from "@solid-primitives/event-listener";
import { GlobalWorkerOptions, type PDFDocumentProxy } from "pdfjs-dist";
import {
  HiOutlineExclamationTriangle,
} from "./icons";
import { PdfToolbar } from "./pdf/PdfToolbar";
import { PdfThumbnails } from "./pdf/PdfThumbnails";
import { loadPdfDocument, renderPdfPage, generateThumbnailsBatch } from "./pdf/pdfHelpers";

// Set up PDF.js worker - using CDN for compatibility
GlobalWorkerOptions.workerSrc = "https://cdnjs.cloudflare.com/ajax/libs/pdf.js/3.11.174/pdf.worker.min.js";

// ============================================================================
// Types
// ============================================================================

interface PdfViewerProps {
  /** Path to the PDF file */
  path: string;
  /** Optional class name */
  class?: string;
}

// ============================================================================
// Component
// ============================================================================

export function PdfViewer(props: PdfViewerProps) {
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [numPages, setNumPages] = createSignal(0);
  const [currentPage, setCurrentPage] = createSignal(1);
  const [scale, setScale] = createSignal(1.0);
  const [pageRendering, setPageRendering] = createSignal(false);
  const [pdfDoc, setPdfDoc] = createSignal<PDFDocumentProxy | null>(null);
  const [showThumbnails, setShowThumbnails] = createSignal(true);
  const [thumbnails, setThumbnails] = createSignal<string[]>([]);

  let canvasRef: HTMLCanvasElement | undefined;
  let containerRef: HTMLDivElement | undefined;
  let renderTaskRef: { cancel: () => void } | null = null;
  let pendingRenderPage: number | null = null;

  // Load PDF document
  const loadPdf = async () => {
    setLoading(true);
    setError(null);
    setThumbnails([]);

    try {
      const pdf = await loadPdfDocument(props.path);
      
      setPdfDoc(pdf);
      setNumPages(pdf.numPages);
      setCurrentPage(1);
      
      // Render first page
      await renderPage(1, pdf);
      
      // Generate thumbnails in background
      generateThumbnails(pdf);
    } catch (e) {
      console.error("Failed to load PDF:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  // Render a specific page
  const renderPage = async (pageNum: number, pdf?: PDFDocumentProxy) => {
    const doc = pdf || pdfDoc();
    if (!doc || !canvasRef) return;

    // Cancel any ongoing render
    if (renderTaskRef) {
      try {
        renderTaskRef.cancel();
      } catch {
        // Ignore cancel errors
      }
      renderTaskRef = null;
    }

    // If already rendering, queue this page
    if (pageRendering()) {
      pendingRenderPage = pageNum;
      return;
    }

    setPageRendering(true);

    try {
      const page = await doc.getPage(pageNum);
      
      // Wait for container to have dimensions
      await new Promise<void>((resolve) => {
        if (containerRef && containerRef.clientWidth > 0) {
          resolve();
        } else {
          // Use requestAnimationFrame to wait for layout
          requestAnimationFrame(() => resolve());
        }
      });
      
      // Use helper to render page
      const containerWidth = containerRef?.clientWidth || 800;
      await renderPdfPage(page, canvasRef, containerWidth, scale());
      
    } catch (e) {
      // Ignore cancel errors
      if (e instanceof Error && e.message.includes("Rendering cancelled")) {
        // This is expected when navigating quickly
      } else {
        console.error("Failed to render page:", e);
      }
    } finally {
      setPageRendering(false);
      
      // Check if there's a pending render
      if (pendingRenderPage !== null) {
        const nextPage = pendingRenderPage;
        pendingRenderPage = null;
        // Use setTimeout to avoid stack overflow on rapid navigation
        setTimeout(() => renderPage(nextPage), 0);
      }
    }
  };

  // Generate thumbnails for all pages (non-blocking, batched)
  const generateThumbnails = async (pdf: PDFDocumentProxy) => {
    // Initialize with empty placeholders
    const initialThumbs = new Array(pdf.numPages).fill("");
    setThumbnails(initialThumbs);

    // Use helper to generate thumbnails in batches
    await generateThumbnailsBatch(pdf, 3, (startIndex, batchThumbs) => {
      // Update thumbnails incrementally as batches complete
      setThumbnails((prev) => {
        const updated = [...prev];
        batchThumbs.forEach((thumb, i) => {
          updated[startIndex + i] = thumb;
        });
        return updated;
      });
    });
  };

  // Navigation handlers
  const goToPrevPage = () => {
    if (currentPage() > 1) {
      const newPage = currentPage() - 1;
      setCurrentPage(newPage);
      renderPage(newPage);
    }
  };

  const goToNextPage = () => {
    if (currentPage() < numPages()) {
      const newPage = currentPage() + 1;
      setCurrentPage(newPage);
      renderPage(newPage);
    }
  };

  const goToPage = (pageNum: number) => {
    if (pageNum >= 1 && pageNum <= numPages()) {
      setCurrentPage(pageNum);
      renderPage(pageNum);
    }
  };

  // Zoom handlers
  const zoomIn = () => {
    const newScale = Math.min(scale() + 0.25, 3.0);
    setScale(newScale);
    renderPage(currentPage());
  };

  const zoomOut = () => {
    const newScale = Math.max(scale() - 0.25, 0.25);
    setScale(newScale);
    renderPage(currentPage());
  };

  const fitToWidth = () => {
    setScale(1.0);
    renderPage(currentPage());
  };

  // Keyboard navigation
  const handleKeyDown = (e: KeyboardEvent) => {
    switch (e.key) {
      case "ArrowLeft":
      case "PageUp":
        e.preventDefault();
        goToPrevPage();
        break;
      case "ArrowRight":
      case "PageDown":
        e.preventDefault();
        goToNextPage();
        break;
      case "+":
      case "=":
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault();
          zoomIn();
        }
        break;
      case "-":
        if (e.ctrlKey || e.metaKey) {
          e.preventDefault();
          zoomOut();
        }
        break;
    }
  };

  // Load PDF when path changes
  createEffect(() => {
    const path = props.path;
    if (path) {
      loadPdf();
    }
  });

  // Set up keyboard listener and resize observer
  const debouncedRerender = debounce(() => {
    if (pdfDoc() && !loading()) {
      renderPage(currentPage());
    }
  }, 150);
  
  onMount(() => {
    // makeEventListener auto-cleans up on component unmount
    makeEventListener(window, "keydown", handleKeyDown);
    
    // Set up resize observer for container using solid-primitives
    if (containerRef) {
      createResizeObserver(containerRef, () => {
        debouncedRerender();
      });
    }
  });

  onCleanup(() => {
    // Cancel any pending render
    if (renderTaskRef) {
      try {
        renderTaskRef.cancel();
      } catch {
        // Ignore
      }
    }
    
    // Clean up PDF document
    const doc = pdfDoc();
    if (doc) {
      doc.destroy();
    }
  });

  return (
    <div
      class={`flex flex-col h-full bg-bg ${props.class || ""}`}
      ref={containerRef}
    >
      {/* Toolbar */}
      <PdfToolbar
        currentPage={currentPage()}
        numPages={numPages()}
        scale={scale()}
        loading={loading()}
        showThumbnails={showThumbnails()}
        onPrevPage={goToPrevPage}
        onNextPage={goToNextPage}
        onGoToPage={goToPage}
        onZoomIn={zoomIn}
        onZoomOut={zoomOut}
        onFitToWidth={fitToWidth}
        onToggleThumbnails={() => setShowThumbnails(!showThumbnails())}
      />

      {/* Main content area */}
      <div class="flex flex-1 min-h-0 overflow-hidden">
        {/* Thumbnails sidebar */}
        <PdfThumbnails
          thumbnails={thumbnails()}
          currentPage={currentPage()}
          show={showThumbnails()}
          onSelectPage={goToPage}
        />

        {/* PDF canvas container */}
        <div class="flex-1 overflow-auto flex items-start justify-center p-4 bg-bg-panel/30">
          <Show when={loading()}>
            <div class="flex flex-col items-center justify-center h-full gap-2">
              <div class="w-8 h-8 border-2 border-accent border-t-transparent rounded-full animate-spin" />
              <span class="text-sm text-txt-secondary">Loading PDF...</span>
            </div>
          </Show>

          <Show when={error()}>
            <div class="flex flex-col items-center justify-center h-full gap-3 text-center max-w-md">
              <HiOutlineExclamationTriangle class="w-12 h-12 text-amber-400" />
              <div class="text-txt font-medium">Failed to load PDF</div>
              <div class="text-sm text-txt-secondary">{error()}</div>
              <button
                class="px-3 py-1.5 text-sm bg-accent hover:bg-accent rounded text-white"
                onClick={loadPdf}
              >
                Try Again
              </button>
            </div>
          </Show>

          <Show when={!loading() && !error() && pdfDoc()}>
            <div class="relative">
              <Show when={pageRendering()}>
                <div class="absolute inset-0 bg-bg/50 flex items-center justify-center">
                  <div class="w-6 h-6 border-2 border-accent border-t-transparent rounded-full animate-spin" />
                </div>
              </Show>
              <canvas
                ref={canvasRef}
                class="shadow-lg rounded"
                style={{ "background-color": "white" }}
              />
            </div>
          </Show>
        </div>
      </div>

      {/* Status bar */}
      <div class="px-3 py-1 border-t border-border bg-bg-toolbar text-xs text-txt-muted shrink-0">
        <span class="truncate">{props.path}</span>
      </div>
    </div>
  );
}
