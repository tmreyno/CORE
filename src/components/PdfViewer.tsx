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

    setPageRendering(true);

    try {
      const page = await doc.getPage(pageNum);
      
      // Calculate scale to fit container width
      const containerWidth = containerRef?.clientWidth || 800;
      const viewport = page.getViewport({ scale: 1 });
      const fitScale = (containerWidth - 40) / viewport.width;
      const actualScale = scale() * fitScale;
      
      const scaledViewport = page.getViewport({ scale: actualScale });

      // Set canvas dimensions
      canvasRef.height = scaledViewport.height;
      canvasRef.width = scaledViewport.width;

      const context = canvasRef.getContext("2d");
      if (!context) return;

      // Render PDF page
      const renderContext = {
        canvasContext: context,
        viewport: scaledViewport,
      };

      await page.render(renderContext).promise;
    } catch (e) {
      console.error("Failed to render page:", e);
    } finally {
      setPageRendering(false);
    }
  };

  // Generate thumbnails for all pages
  const generateThumbnails = async (pdf: PDFDocumentProxy) => {
    const thumbs: string[] = [];
    const thumbScale = 0.15;

    for (let i = 1; i <= pdf.numPages; i++) {
      try {
        const page = await pdf.getPage(i);
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
          
          thumbs.push(canvas.toDataURL("image/png"));
        }
      } catch (e) {
        console.error(`Failed to generate thumbnail for page ${i}:`, e);
        thumbs.push("");
      }
    }

    setThumbnails(thumbs);
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

  // Set up keyboard listener
  onMount(() => {
    window.addEventListener("keydown", handleKeyDown);
  });

  onCleanup(() => {
    window.removeEventListener("keydown", handleKeyDown);
    // Clean up PDF document
    const doc = pdfDoc();
    if (doc) {
      doc.destroy();
    }
  });

  return (
    <div
      class={`flex flex-col h-full bg-zinc-900 ${props.class || ""}`}
      ref={containerRef}
    >
      {/* Toolbar */}
      <div class="flex items-center justify-between px-3 py-2 border-b border-zinc-800 bg-zinc-900/95 shrink-0">
        <div class="flex items-center gap-2">
          <HiOutlineDocumentText class="w-4 h-4 text-red-400" />
          <span class="text-sm font-medium text-zinc-200">PDF Viewer</span>
          <Show when={numPages() > 0}>
            <span class="text-xs text-zinc-500">
              Page {currentPage()} of {numPages()}
            </span>
          </Show>
        </div>

        <div class="flex items-center gap-1">
          {/* Page navigation */}
          <div class="flex items-center gap-1 mr-2">
            <button
              class="p-1 rounded hover:bg-zinc-700 disabled:opacity-40 disabled:cursor-not-allowed"
              onClick={goToPrevPage}
              disabled={currentPage() <= 1 || loading()}
              title="Previous page (←)"
            >
              <HiOutlineChevronLeft class="w-4 h-4 text-zinc-400" />
            </button>
            <input
              type="number"
              min={1}
              max={numPages()}
              value={currentPage()}
              onChange={(e) => goToPage(parseInt(e.currentTarget.value) || 1)}
              class="w-12 px-1 py-0.5 text-xs text-center bg-zinc-800 border border-zinc-700 rounded text-zinc-200"
              disabled={loading()}
            />
            <button
              class="p-1 rounded hover:bg-zinc-700 disabled:opacity-40 disabled:cursor-not-allowed"
              onClick={goToNextPage}
              disabled={currentPage() >= numPages() || loading()}
              title="Next page (→)"
            >
              <HiOutlineChevronRight class="w-4 h-4 text-zinc-400" />
            </button>
          </div>

          {/* Zoom controls */}
          <div class="flex items-center gap-1 border-l border-zinc-700 pl-2">
            <button
              class="p-1 rounded hover:bg-zinc-700 disabled:opacity-40"
              onClick={zoomOut}
              disabled={scale() <= 0.25 || loading()}
              title="Zoom out (Ctrl+-)"
            >
              <HiOutlineMagnifyingGlassMinus class="w-4 h-4 text-zinc-400" />
            </button>
            <span class="text-xs text-zinc-400 w-12 text-center">
              {Math.round(scale() * 100)}%
            </span>
            <button
              class="p-1 rounded hover:bg-zinc-700 disabled:opacity-40"
              onClick={zoomIn}
              disabled={scale() >= 3.0 || loading()}
              title="Zoom in (Ctrl++)"
            >
              <HiOutlineMagnifyingGlassPlus class="w-4 h-4 text-zinc-400" />
            </button>
            <button
              class="p-1 rounded hover:bg-zinc-700"
              onClick={fitToWidth}
              title="Fit to width"
            >
              <HiOutlineArrowsPointingOut class="w-4 h-4 text-zinc-400" />
            </button>
          </div>

          {/* Thumbnails toggle */}
          <button
            class={`ml-2 px-2 py-0.5 text-xs rounded transition-colors ${
              showThumbnails()
                ? "bg-cyan-600 text-white"
                : "bg-zinc-800 text-zinc-400 hover:bg-zinc-700"
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
          <div class="w-32 shrink-0 border-r border-zinc-800 overflow-y-auto bg-zinc-950">
            <div class="p-2 space-y-2">
              <For each={thumbnails()}>
                {(thumb, index) => (
                  <button
                    class={`w-full p-1 rounded border transition-colors ${
                      currentPage() === index() + 1
                        ? "border-cyan-500 bg-cyan-500/10"
                        : "border-zinc-700 hover:border-zinc-600 hover:bg-zinc-800/50"
                    }`}
                    onClick={() => goToPage(index() + 1)}
                  >
                    <Show
                      when={thumb}
                      fallback={
                        <div class="aspect-[3/4] bg-zinc-800 flex items-center justify-center">
                          <span class="text-xs text-zinc-500">{index() + 1}</span>
                        </div>
                      }
                    >
                      <img
                        src={thumb}
                        alt={`Page ${index() + 1}`}
                        class="w-full"
                      />
                    </Show>
                    <span class="text-[10px] text-zinc-400 mt-1 block">
                      {index() + 1}
                    </span>
                  </button>
                )}
              </For>
            </div>
          </div>
        </Show>

        {/* PDF canvas container */}
        <div class="flex-1 overflow-auto flex items-start justify-center p-4 bg-zinc-800/30">
          <Show when={loading()}>
            <div class="flex flex-col items-center justify-center h-full gap-2">
              <div class="w-8 h-8 border-2 border-cyan-500 border-t-transparent rounded-full animate-spin" />
              <span class="text-sm text-zinc-400">Loading PDF...</span>
            </div>
          </Show>

          <Show when={error()}>
            <div class="flex flex-col items-center justify-center h-full gap-3 text-center max-w-md">
              <HiOutlineExclamationTriangle class="w-12 h-12 text-amber-400" />
              <div class="text-zinc-200 font-medium">Failed to load PDF</div>
              <div class="text-sm text-zinc-400">{error()}</div>
              <button
                class="px-3 py-1.5 text-sm bg-cyan-600 hover:bg-cyan-500 rounded text-white"
                onClick={loadPdf}
              >
                Try Again
              </button>
            </div>
          </Show>

          <Show when={!loading() && !error() && pdfDoc()}>
            <div class="relative">
              <Show when={pageRendering()}>
                <div class="absolute inset-0 bg-zinc-900/50 flex items-center justify-center">
                  <div class="w-6 h-6 border-2 border-cyan-500 border-t-transparent rounded-full animate-spin" />
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
      <div class="px-3 py-1 border-t border-zinc-800 bg-zinc-900/95 text-xs text-zinc-500 shrink-0">
        <span class="truncate">{props.path}</span>
      </div>
    </div>
  );
}

export default PdfViewer;
