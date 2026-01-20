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

import { createSignal, createEffect, Show, For, onCleanup, onMount } from "solid-js";
import { getDocument, GlobalWorkerOptions, type PDFDocumentProxy } from "pdfjs-dist";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineChevronLeft,
  HiOutlineChevronRight,
  HiOutlineMagnifyingGlassPlus,
  HiOutlineMagnifyingGlassMinus,
  HiOutlineArrowsPointingOut,
  HiOutlineDocumentText,
  HiOutlineExclamationTriangle,
} from "./icons";

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
      // Read file as base64 via Tauri command (avoids file:// URL security restrictions)
      const base64Data = await invoke<string>("viewer_read_binary_base64", { path: props.path });
      
      // Convert base64 to Uint8Array
      const binaryString = atob(base64Data);
      const bytes = new Uint8Array(binaryString.length);
      for (let i = 0; i < binaryString.length; i++) {
        bytes[i] = binaryString.charCodeAt(i);
      }

      const loadingTask = getDocument({ data: bytes });
      const pdf = await loadingTask.promise;
      
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
      
      // Calculate scale to fit container width
      const containerWidth = containerRef?.clientWidth || 800;
      const viewport = page.getViewport({ scale: 1 });
      const fitScale = Math.max(0.5, (containerWidth - 80) / viewport.width);
      const actualScale = scale() * fitScale;
      
      const scaledViewport = page.getViewport({ scale: actualScale });

      // Set canvas dimensions with device pixel ratio for crisp rendering
      const pixelRatio = window.devicePixelRatio || 1;
      const canvasWidth = Math.floor(scaledViewport.width * pixelRatio);
      const canvasHeight = Math.floor(scaledViewport.height * pixelRatio);
      
      canvasRef.width = canvasWidth;
      canvasRef.height = canvasHeight;
      canvasRef.style.width = `${Math.floor(scaledViewport.width)}px`;
      canvasRef.style.height = `${Math.floor(scaledViewport.height)}px`;

      const context = canvasRef.getContext("2d");
      if (!context) {
        setPageRendering(false);
        return;
      }

      // Scale context for high DPI displays
      context.setTransform(pixelRatio, 0, 0, pixelRatio, 0, 0);

      // Clear canvas before render
      context.fillStyle = "white";
      context.fillRect(0, 0, scaledViewport.width, scaledViewport.height);

      // Render PDF page
      const renderContext = {
        canvasContext: context,
        viewport: scaledViewport,
      };

      const renderTask = page.render(renderContext);
      renderTaskRef = renderTask;
      
      await renderTask.promise;
      renderTaskRef = null;
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
    const thumbScale = 0.15;
    const batchSize = 3; // Generate 3 thumbnails at a time
    
    // Initialize with empty placeholders
    const initialThumbs = new Array(pdf.numPages).fill("");
    setThumbnails(initialThumbs);

    for (let batch = 0; batch < pdf.numPages; batch += batchSize) {
      const batchEnd = Math.min(batch + batchSize, pdf.numPages);
      const batchPromises: Promise<{ index: number; dataUrl: string }>[] = [];
      
      for (let i = batch; i < batchEnd; i++) {
        batchPromises.push(
          (async () => {
            try {
              const page = await pdf.getPage(i + 1);
              const viewport = page.getViewport({ scale: thumbScale });
              
              const canvas = document.createElement("canvas");
              canvas.width = viewport.width;
              canvas.height = viewport.height;
              
              const context = canvas.getContext("2d");
              if (context) {
                await page.render({
                  canvasContext: context,
                  viewport: viewport,
                }).promise;
                
                return { index: i, dataUrl: canvas.toDataURL("image/jpeg", 0.6) };
              }
            } catch (e) {
              console.error(`Failed to generate thumbnail for page ${i + 1}:`, e);
            }
            return { index: i, dataUrl: "" };
          })()
        );
      }
      
      // Wait for batch to complete
      const results = await Promise.all(batchPromises);
      
      // Update thumbnails incrementally
      setThumbnails((prev) => {
        const updated = [...prev];
        for (const result of results) {
          updated[result.index] = result.dataUrl;
        }
        return updated;
      });
      
      // Yield to main thread between batches
      await new Promise((resolve) => setTimeout(resolve, 10));
    }
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
  let resizeObserver: ResizeObserver | null = null;
  let resizeTimeout: ReturnType<typeof setTimeout> | null = null;
  
  onMount(() => {
    window.addEventListener("keydown", handleKeyDown);
    
    // Set up resize observer for container
    resizeObserver = new ResizeObserver(() => {
      // Debounce resize events
      if (resizeTimeout) {
        clearTimeout(resizeTimeout);
      }
      resizeTimeout = setTimeout(() => {
        if (pdfDoc() && !loading()) {
          renderPage(currentPage());
        }
      }, 150);
    });
    
    if (containerRef) {
      resizeObserver.observe(containerRef);
    }
  });

  onCleanup(() => {
    window.removeEventListener("keydown", handleKeyDown);
    
    // Clean up resize observer
    if (resizeObserver) {
      resizeObserver.disconnect();
    }
    if (resizeTimeout) {
      clearTimeout(resizeTimeout);
    }
    
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
      <div class="flex items-center justify-between px-3 py-2 border-b border-border bg-bg-toolbar shrink-0">
        <div class="flex items-center gap-2">
          <HiOutlineDocumentText class="w-4 h-4 text-red-400" />
          <span class="text-sm font-medium text-txt">PDF Viewer</span>
          <Show when={numPages() > 0}>
            <span class="text-xs text-txt-muted">
              Page {currentPage()} of {numPages()}
            </span>
          </Show>
        </div>

        <div class="flex items-center gap-1">
          {/* Page navigation */}
          <div class="flex items-center gap-1 mr-2">
            <button
              class="p-1 rounded hover:bg-bg-hover disabled:opacity-40 disabled:cursor-not-allowed"
              onClick={goToPrevPage}
              disabled={currentPage() <= 1 || loading()}
              title="Previous page (←)"
            >
              <HiOutlineChevronLeft class="w-4 h-4 text-txt-secondary" />
            </button>
            <input
              type="number"
              min={1}
              max={numPages()}
              value={currentPage()}
              onChange={(e) => goToPage(parseInt(e.currentTarget.value) || 1)}
              class="w-12 px-1 py-0.5 text-xs text-center bg-bg-panel border border-border rounded text-txt"
              disabled={loading()}
            />
            <button
              class="p-1 rounded hover:bg-bg-hover disabled:opacity-40 disabled:cursor-not-allowed"
              onClick={goToNextPage}
              disabled={currentPage() >= numPages() || loading()}
              title="Next page (→)"
            >
              <HiOutlineChevronRight class="w-4 h-4 text-txt-secondary" />
            </button>
          </div>

          {/* Zoom controls */}
          <div class="flex items-center gap-1 border-l border-border pl-2">
            <button
              class="p-1 rounded hover:bg-bg-hover disabled:opacity-40"
              onClick={zoomOut}
              disabled={scale() <= 0.25 || loading()}
              title="Zoom out (Ctrl+-)"
            >
              <HiOutlineMagnifyingGlassMinus class="w-4 h-4 text-txt-secondary" />
            </button>
            <span class="text-xs text-txt-secondary w-12 text-center">
              {Math.round(scale() * 100)}%
            </span>
            <button
              class="p-1 rounded hover:bg-bg-hover disabled:opacity-40"
              onClick={zoomIn}
              disabled={scale() >= 3.0 || loading()}
              title="Zoom in (Ctrl++)"
            >
              <HiOutlineMagnifyingGlassPlus class="w-4 h-4 text-txt-secondary" />
            </button>
            <button
              class="p-1 rounded hover:bg-bg-hover"
              onClick={fitToWidth}
              title="Fit to width"
            >
              <HiOutlineArrowsPointingOut class="w-4 h-4 text-txt-secondary" />
            </button>
          </div>

          {/* Thumbnails toggle */}
          <button
            class={`ml-2 px-2 py-0.5 text-xs rounded transition-colors ${
              showThumbnails()
                ? "bg-accent text-white"
                : "bg-bg-panel text-txt-secondary hover:bg-bg-hover"
            }`}
            onClick={() => setShowThumbnails(!showThumbnails())}
          >
            Pages
          </button>
        </div>
      </div>

      {/* Main content area */}
      <div class="flex flex-1 min-h-0 overflow-hidden">
        {/* Thumbnails sidebar */}
        <Show when={showThumbnails() && thumbnails().length > 0}>
          <div class="w-32 shrink-0 border-r border-border overflow-y-auto bg-bg">
            <div class="p-2 space-y-2">
              <For each={thumbnails()}>
                {(thumb, index) => (
                  <button
                    class={`w-full p-1 rounded border transition-colors ${
                      currentPage() === index() + 1
                        ? "border-accent bg-accent/10"
                        : "border-border hover:border-border hover:bg-bg-panel/50"
                    }`}
                    onClick={() => goToPage(index() + 1)}
                  >
                    <Show
                      when={thumb}
                      fallback={
                        <div class="aspect-[3/4] bg-bg-panel flex items-center justify-center">
                          <span class="text-xs text-txt-muted">{index() + 1}</span>
                        </div>
                      }
                    >
                      <img
                        src={thumb}
                        alt={`Page ${index() + 1}`}
                        class="w-full"
                      />
                    </Show>
                    <span class="text-[10px] leading-tight text-txt-secondary mt-1 block">
                      {index() + 1}
                    </span>
                  </button>
                )}
              </For>
            </div>
          </div>
        </Show>

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

export default PdfViewer;
