// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import {
  HiOutlineDocumentText,
  HiOutlineChevronLeft,
  HiOutlineChevronRight,
  HiOutlineMagnifyingGlassPlus,
  HiOutlineMagnifyingGlassMinus,
  HiOutlineArrowsPointingOut,
} from "../icons";

interface PdfToolbarProps {
  currentPage: number;
  numPages: number;
  scale: number;
  loading: boolean;
  showThumbnails: boolean;
  onPrevPage: () => void;
  onNextPage: () => void;
  onGoToPage: (page: number) => void;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onFitToWidth: () => void;
  onToggleThumbnails: () => void;
}

export const PdfToolbar: Component<PdfToolbarProps> = (props) => {
  return (
    <div class="flex items-center justify-between px-3 py-2 border-b border-border bg-bg-toolbar shrink-0">
      <div class="flex items-center gap-2">
        <HiOutlineDocumentText class="w-4 h-4 text-red-400" />
        <span class="text-sm font-medium text-txt">PDF Viewer</span>
        <Show when={props.numPages > 0}>
          <span class="text-xs text-txt-muted">
            Page {props.currentPage} of {props.numPages}
          </span>
        </Show>
      </div>

      <div class="flex items-center gap-1">
        {/* Page navigation */}
        <div class="flex items-center gap-1 mr-2">
          <button
            class="p-1 rounded hover:bg-bg-hover disabled:opacity-40 disabled:cursor-not-allowed"
            onClick={props.onPrevPage}
            disabled={props.currentPage <= 1 || props.loading}
            title="Previous page (←)"
          >
            <HiOutlineChevronLeft class="w-4 h-4 text-txt-secondary" />
          </button>
          <input
            type="number"
            min={1}
            max={props.numPages}
            value={props.currentPage}
            onChange={(e) => props.onGoToPage(parseInt(e.currentTarget.value) || 1)}
            class="input-xs w-12"
            disabled={props.loading}
          />
          <button
            class="p-1 rounded hover:bg-bg-hover disabled:opacity-40 disabled:cursor-not-allowed"
            onClick={props.onNextPage}
            disabled={props.currentPage >= props.numPages || props.loading}
            title="Next page (→)"
          >
            <HiOutlineChevronRight class="w-4 h-4 text-txt-secondary" />
          </button>
        </div>

        {/* Zoom controls */}
        <div class="flex items-center gap-1 border-l border-border pl-2">
          <button
            class="p-1 rounded hover:bg-bg-hover disabled:opacity-40"
            onClick={props.onZoomOut}
            disabled={props.scale <= 0.25 || props.loading}
            title="Zoom out (Ctrl+-)"
          >
            <HiOutlineMagnifyingGlassMinus class="w-4 h-4 text-txt-secondary" />
          </button>
          <span class="text-xs text-txt-secondary w-12 text-center">
            {Math.round(props.scale * 100)}%
          </span>
          <button
            class="p-1 rounded hover:bg-bg-hover disabled:opacity-40"
            onClick={props.onZoomIn}
            disabled={props.scale >= 3.0 || props.loading}
            title="Zoom in (Ctrl++)"
          >
            <HiOutlineMagnifyingGlassPlus class="w-4 h-4 text-txt-secondary" />
          </button>
          <button
            class="p-1 rounded hover:bg-bg-hover"
            onClick={props.onFitToWidth}
            title="Fit to width"
          >
            <HiOutlineArrowsPointingOut class="w-4 h-4 text-txt-secondary" />
          </button>
        </div>

        {/* Thumbnails toggle */}
        <button
          class={`ml-2 px-2 py-0.5 text-xs rounded transition-colors ${
            props.showThumbnails
              ? "bg-accent text-white"
              : "bg-bg-panel text-txt-secondary hover:bg-bg-hover"
          }`}
          onClick={props.onToggleThumbnails}
        >
          Pages
        </button>
      </div>
    </div>
  );
};
