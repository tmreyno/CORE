// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, onCleanup, onMount } from "solid-js";
import { createResizeObserver } from "@solid-primitives/resize-observer";
import { debounce } from "@solid-primitives/scheduled";
import { makeEventListener } from "@solid-primitives/event-listener";
import { GlobalWorkerOptions, type PDFDocumentProxy } from "pdfjs-dist";
import PdfWorkerUrl from "pdfjs-dist/build/pdf.worker.min.mjs?url";
import { loadPdfDocument, renderPdfPage, generateThumbnailsBatch } from "./pdfHelpers";
import { logger } from "../../utils/logger";

const log = logger.scope("PdfViewer");

// Set up PDF.js worker - bundled locally via Vite (no CDN needed)
GlobalWorkerOptions.workerSrc = PdfWorkerUrl;

/**
 * Hook encapsulating all PDF viewer state and logic:
 * loading, navigation, zoom, thumbnails, keyboard handling, and resize.
 */
export function usePdfViewer(getPath: () => string) {
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

  // ── Load PDF ──────────────────────────────────────────────────────────

  const loadPdf = async () => {
    setLoading(true);
    setError(null);
    setThumbnails([]);

    try {
      const pdf = await loadPdfDocument(getPath());
      setPdfDoc(pdf);
      setNumPages(pdf.numPages);
      setCurrentPage(1);
      await renderPage(1, pdf);
      generateThumbnails(pdf);
    } catch (e) {
      log.error("Failed to load PDF:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  // ── Render page ───────────────────────────────────────────────────────

  const renderPage = async (pageNum: number, pdf?: PDFDocumentProxy) => {
    const doc = pdf || pdfDoc();
    if (!doc || !canvasRef) return;

    if (renderTaskRef) {
      try { renderTaskRef.cancel(); } catch { /* ignore */ }
      renderTaskRef = null;
    }

    if (pageRendering()) {
      pendingRenderPage = pageNum;
      return;
    }

    setPageRendering(true);

    try {
      const page = await doc.getPage(pageNum);

      await new Promise<void>((resolve) => {
        if (containerRef && containerRef.clientWidth > 0) {
          resolve();
        } else {
          requestAnimationFrame(() => resolve());
        }
      });

      const containerWidth = containerRef?.clientWidth || 800;
      await renderPdfPage(page, canvasRef, containerWidth, scale());
    } catch (e) {
      if (e instanceof Error && e.message.includes("Rendering cancelled")) {
        // expected during rapid navigation
      } else {
        log.error("Failed to render page:", e);
      }
    } finally {
      setPageRendering(false);
      if (pendingRenderPage !== null) {
        const nextPage = pendingRenderPage;
        pendingRenderPage = null;
        setTimeout(() => renderPage(nextPage), 0);
      }
    }
  };

  // ── Thumbnails ────────────────────────────────────────────────────────

  const generateThumbnails = async (pdf: PDFDocumentProxy) => {
    setThumbnails(new Array(pdf.numPages).fill(""));
    await generateThumbnailsBatch(pdf, 3, (startIndex, batchThumbs) => {
      setThumbnails((prev) => {
        const updated = [...prev];
        batchThumbs.forEach((thumb, i) => {
          updated[startIndex + i] = thumb;
        });
        return updated;
      });
    });
  };

  // ── Navigation ────────────────────────────────────────────────────────

  const goToPrevPage = () => {
    if (currentPage() > 1) {
      const p = currentPage() - 1;
      setCurrentPage(p);
      renderPage(p);
    }
  };

  const goToNextPage = () => {
    if (currentPage() < numPages()) {
      const p = currentPage() + 1;
      setCurrentPage(p);
      renderPage(p);
    }
  };

  const goToPage = (pageNum: number) => {
    if (pageNum >= 1 && pageNum <= numPages()) {
      setCurrentPage(pageNum);
      renderPage(pageNum);
    }
  };

  // ── Zoom ──────────────────────────────────────────────────────────────

  const zoomIn = () => {
    setScale(Math.min(scale() + 0.25, 3.0));
    renderPage(currentPage());
  };

  const zoomOut = () => {
    setScale(Math.max(scale() - 0.25, 0.25));
    renderPage(currentPage());
  };

  const fitToWidth = () => {
    setScale(1.0);
    renderPage(currentPage());
  };

  // ── Keyboard ──────────────────────────────────────────────────────────

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
        if (e.ctrlKey || e.metaKey) { e.preventDefault(); zoomIn(); }
        break;
      case "-":
        if (e.ctrlKey || e.metaKey) { e.preventDefault(); zoomOut(); }
        break;
    }
  };

  // ── Lifecycle ─────────────────────────────────────────────────────────

  createEffect(() => {
    const path = getPath();
    if (path) loadPdf();
  });

  const debouncedRerender = debounce(() => {
    if (pdfDoc() && !loading()) renderPage(currentPage());
  }, 150);

  onMount(() => {
    makeEventListener(window, "keydown", handleKeyDown);
    if (containerRef) {
      createResizeObserver(containerRef, () => debouncedRerender());
    }
  });

  onCleanup(() => {
    if (renderTaskRef) {
      try { renderTaskRef.cancel(); } catch { /* ignore */ }
    }
    const doc = pdfDoc();
    if (doc) doc.destroy();
  });

  return {
    // State
    loading,
    error,
    numPages,
    currentPage,
    scale,
    pageRendering,
    pdfDoc,
    showThumbnails,
    thumbnails,
    // Actions
    loadPdf,
    goToPrevPage,
    goToNextPage,
    goToPage,
    zoomIn,
    zoomOut,
    fitToWidth,
    toggleThumbnails: () => setShowThumbnails(!showThumbnails()),
    // Refs (assign in component)
    setCanvasRef: (el: HTMLCanvasElement) => { canvasRef = el; },
    setContainerRef: (el: HTMLDivElement) => { containerRef = el; },
  };
}
