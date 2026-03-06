// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import { HiOutlineExclamationTriangle } from "../icons";
import { PdfToolbar } from "./PdfToolbar";
import { PdfThumbnails } from "./PdfThumbnails";
import { usePdfViewer } from "./usePdfViewer";
import type { PdfViewerProps } from "./types";

export function PdfViewer(props: PdfViewerProps) {
  const viewer = usePdfViewer(() => props.path);

  return (
    <div
      class={`flex flex-col h-full bg-bg ${props.class || ""}`}
      ref={viewer.setContainerRef}
    >
      {/* Toolbar */}
      <PdfToolbar
        currentPage={viewer.currentPage()}
        numPages={viewer.numPages()}
        scale={viewer.scale()}
        loading={viewer.loading()}
        showThumbnails={viewer.showThumbnails()}
        onPrevPage={viewer.goToPrevPage}
        onNextPage={viewer.goToNextPage}
        onGoToPage={viewer.goToPage}
        onZoomIn={viewer.zoomIn}
        onZoomOut={viewer.zoomOut}
        onFitToWidth={viewer.fitToWidth}
        onToggleThumbnails={viewer.toggleThumbnails}
      />

      {/* Main content area */}
      <div class="flex flex-1 min-h-0 overflow-hidden">
        {/* Thumbnails sidebar */}
        <PdfThumbnails
          thumbnails={viewer.thumbnails()}
          currentPage={viewer.currentPage()}
          show={viewer.showThumbnails()}
          onSelectPage={viewer.goToPage}
        />

        {/* PDF canvas container */}
        <div class="flex-1 overflow-auto flex items-start justify-center p-4 bg-bg-panel/30">
          <Show when={viewer.loading()}>
            <div class="flex flex-col items-center justify-center h-full gap-2">
              <div class="w-8 h-8 border-2 border-accent border-t-transparent rounded-full animate-spin" />
              <span class="text-sm text-txt-secondary">Loading PDF...</span>
            </div>
          </Show>

          <Show when={viewer.error()}>
            <div class="flex flex-col items-center justify-center h-full gap-3 text-center max-w-md">
              <HiOutlineExclamationTriangle class="w-12 h-12 text-amber-400" />
              <div class="text-txt font-medium">Failed to load PDF</div>
              <div class="text-sm text-txt-secondary">{viewer.error()}</div>
              <button
                class="px-3 py-1.5 text-sm bg-accent hover:bg-accent rounded text-white"
                onClick={viewer.loadPdf}
              >
                Try Again
              </button>
            </div>
          </Show>

          <Show when={!viewer.loading() && !viewer.error() && viewer.pdfDoc()}>
            <div class="relative">
              <Show when={viewer.pageRendering()}>
                <div class="absolute inset-0 bg-bg/50 flex items-center justify-center">
                  <div class="w-6 h-6 border-2 border-accent border-t-transparent rounded-full animate-spin" />
                </div>
              </Show>
              <canvas
                ref={viewer.setCanvasRef}
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
