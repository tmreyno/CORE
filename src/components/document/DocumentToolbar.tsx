// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * DocumentToolbar Component
 * 
 * Toolbar for document viewer with:
 * - Format indicator (PDF/DOCX/HTML/Markdown with icon)
 * - Search within document
 * - Zoom controls (in/out/reset)
 * - Actions (metadata panel toggle, print, download as HTML)
 */

import { Component, Show } from "solid-js";
import {
  HiOutlineMagnifyingGlass,
  HiOutlineMagnifyingGlassMinus,
  HiOutlineMagnifyingGlassPlus,
  HiOutlineInformationCircle,
  HiOutlinePrinter,
  HiOutlineArrowDownTray,
} from "solid-icons/hi";

interface DocumentToolbarProps {
  // Format display
  formatIcon: string;
  documentFormat: string;
  
  // Search
  searchQuery: string;
  searchHighlights: number;
  onSearchQueryChange: (query: string) => void;
  onSearchSubmit: () => void;
  
  // Zoom
  zoomPercent: number;
  onZoomIn: () => void;
  onZoomOut: () => void;
  onResetZoom: () => void;
  
  // Actions
  showMetadataPanel: boolean;
  onToggleMetadata: () => void;
  onPrint: () => void;
  onDownload: () => void;
}

export const DocumentToolbar: Component<DocumentToolbarProps> = (props) => {
  return (
    <div class="document-toolbar flex items-center gap-2 p-2 border-b border-border bg-bg-secondary">
      {/* Format indicator */}
      <div class="flex items-center gap-1 px-2 py-1 bg-bg-hover rounded text-sm">
        <span>{props.formatIcon}</span>
        <span class="font-medium">{props.documentFormat || "Document"}</span>
      </div>

      <div class="flex-1" />

      {/* Search */}
      <div class="flex items-center gap-1">
        <div class="relative">
          <input
            type="text"
            value={props.searchQuery}
            onInput={(e) => props.onSearchQueryChange(e.currentTarget.value)}
            onKeyPress={(e) => e.key === "Enter" && props.onSearchSubmit()}
            placeholder="Search..."
            class="w-40 px-2 py-1 pl-7 text-sm rounded border border-border bg-bg-panel"
          />
          <HiOutlineMagnifyingGlass class="absolute left-2 top-1/2 -translate-y-1/2 w-4 h-4 text-txt-muted" />
        </div>
        <Show when={props.searchHighlights > 0}>
          <span class="text-xs text-txt-muted">{props.searchHighlights} found</span>
        </Show>
      </div>

      {/* Zoom controls */}
      <div class="flex items-center gap-1 border-l border-border pl-2 ml-2">
        <button
          onClick={props.onZoomOut}
          class="p-1 rounded hover:bg-bg-hover"
          title="Zoom out"
        >
          <HiOutlineMagnifyingGlassMinus class="w-5 h-5" />
        </button>
        <span class="text-sm w-12 text-center">{props.zoomPercent}%</span>
        <button
          onClick={props.onZoomIn}
          class="p-1 rounded hover:bg-bg-hover"
          title="Zoom in"
        >
          <HiOutlineMagnifyingGlassPlus class="w-5 h-5" />
        </button>
        <button
          onClick={props.onResetZoom}
          class="text-xs px-2 py-1 rounded hover:bg-bg-hover"
        >
          Reset
        </button>
      </div>

      {/* Actions */}
      <div class="flex items-center gap-1 border-l border-border pl-2 ml-2">
        <button
          onClick={props.onToggleMetadata}
          class={`p-1 rounded hover:bg-bg-hover ${props.showMetadataPanel ? "bg-bg-active" : ""}`}
          title="Show metadata"
        >
          <HiOutlineInformationCircle class="w-5 h-5" />
        </button>
        <button
          onClick={props.onPrint}
          class="p-1 rounded hover:bg-bg-hover"
          title="Print"
        >
          <HiOutlinePrinter class="w-5 h-5" />
        </button>
        <button
          onClick={props.onDownload}
          class="p-1 rounded hover:bg-bg-hover"
          title="Download as HTML"
        >
          <HiOutlineArrowDownTray class="w-5 h-5" />
        </button>
      </div>
    </div>
  );
};
