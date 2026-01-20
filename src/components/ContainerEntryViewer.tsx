// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ContainerEntryViewer - View file content from within forensic containers
 * 
 * This component wraps HexViewer and TextViewer to display content of files
 * stored inside forensic containers:
 * - AD1: Uses container_read_entry_chunk (with address-based V2 API support)
 * - E01/Raw (VFS): Uses vfs_read_file
 * - Archives: Uses archive_read_entry_chunk
 * 
 * The HexViewer and TextViewer components now support both disk files and
 * container entries through their optional `entry` prop.
 * 
 * Preview feature: Extracts the file to a temp location and opens it with
 * the native document viewer (PDF, images, Office docs, etc.)
 */

import { createSignal, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { HiOutlineDocument, HiOutlineArrowLeft, HiOutlineEye } from "./icons";
import type { SelectedEntry } from "./EvidenceTree";
import { HexViewer } from "./HexViewer";
import { TextViewer } from "./TextViewer";
import { DocumentViewer } from "./DocumentViewer";
import { PdfViewer } from "./PdfViewer";
import { SpreadsheetViewer } from "./SpreadsheetViewer";
import { formatBytes } from "../utils";

// View mode types - hex and text are guaranteed to work, preview uses native viewers
export type EntryViewMode = "auto" | "hex" | "text" | "document" | "preview";

interface ContainerEntryViewerProps {
  /** The selected entry to display */
  entry: SelectedEntry;
  /** View mode: hex, text, auto, document, or preview */
  viewMode: EntryViewMode;
  /** Callback when user wants to go back/close this view */
  onBack?: () => void;
  /** Callback when user toggles view mode */
  onViewModeChange?: (mode: EntryViewMode) => void;
}

/** Get file extension from name */
function getExtension(name: string): string {
  const parts = name.split(".");
  return parts.length > 1 ? parts[parts.length - 1].toLowerCase() : "";
}

/** Check if file type can be previewed */
function canPreview(name: string): boolean {
  const ext = getExtension(name);
  const previewable = [
    // Documents
    "pdf", "docx", "doc", "xlsx", "xls", "pptx", "ppt",
    // Text
    "txt", "md", "json", "xml", "csv", "html", "htm",
    // Images
    "jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico",
    // Other
    "rtf", "odt", "ods", "odp"
  ];
  return previewable.includes(ext);
}

/** Check if file is a PDF */
function isPdf(name: string): boolean {
  return getExtension(name) === "pdf";
}

/** Check if file is an image */
function isImage(name: string): boolean {
  const ext = getExtension(name);
  return ["jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico"].includes(ext);
}

/** Check if file is a spreadsheet */
function isSpreadsheet(name: string): boolean {
  const ext = getExtension(name);
  return ["xlsx", "xls", "ods", "csv", "tsv"].includes(ext);
}

export function ContainerEntryViewer(props: ContainerEntryViewerProps) {
  const [previewPath, setPreviewPath] = createSignal<string | null>(null);
  const [previewLoading, setPreviewLoading] = createSignal(false);
  const [previewError, setPreviewError] = createSignal<string | null>(null);
  
  // Determine effective mode for display
  const effectiveMode = () => {
    if (props.viewMode === "preview" && previewPath()) {
      return "preview";
    }
    switch (props.viewMode) {
      case "hex": return "hex";
      case "text": return "text";
      default: return "hex";
    }
  };
  
  // Extract file to temp and set preview path
  const handlePreview = async () => {
    if (props.entry.isDir) return;
    
    setPreviewLoading(true);
    setPreviewError(null);
    
    try {
      const tempPath = await invoke<string>("container_extract_entry_to_temp", {
        containerPath: props.entry.containerPath,
        entryPath: props.entry.entryPath,
        entrySize: props.entry.size,
        isVfsEntry: props.entry.isVfsEntry || false,
        isArchiveEntry: props.entry.isArchiveEntry || false,
        dataAddr: props.entry.dataAddr ?? null,
      });
      
      setPreviewPath(tempPath);
      props.onViewModeChange?.("preview");
    } catch (e) {
      console.error("Preview extraction failed:", e);
      setPreviewError(e instanceof Error ? e.message : String(e));
    } finally {
      setPreviewLoading(false);
    }
  };
  
  // Close preview and return to hex/text
  const closePreview = () => {
    setPreviewPath(null);
    setPreviewError(null);
    props.onViewModeChange?.("hex");
  };
  
  return (
    <div class="flex flex-col h-full bg-bg">
      {/* Header */}
      <div class="panel-header gap-3">
        <Show when={props.onBack}>
          <button class="btn-text flex items-center gap-1" onClick={props.onBack} title="Back to file list">
            <HiOutlineArrowLeft class="w-3 h-3" /> Back
          </button>
        </Show>
        <div class="row flex-1 min-w-0">
          <span class="text-sm text-txt truncate flex items-center gap-1.5" title={props.entry.entryPath}>
            <HiOutlineDocument class="w-3.5 h-3.5 shrink-0" /> {props.entry.name}
          </span>
          <span class="text-xs text-txt-muted">{formatBytes(props.entry.size)}</span>
          <Show when={props.entry.isDiskFile}>
            <span class="px-1.5 py-0.5 text-[10px] leading-tight bg-bg-hover text-txt-secondary rounded">Disk File</span>
          </Show>
          <Show when={props.entry.isVfsEntry}>
            <span class="px-1.5 py-0.5 text-[10px] leading-tight bg-blue-700/50 text-blue-300 rounded">VFS</span>
          </Show>
          <Show when={props.entry.isArchiveEntry}>
            <span class="px-1.5 py-0.5 text-[10px] leading-tight bg-purple-700/50 text-purple-300 rounded">Archive</span>
          </Show>
        </div>
        
        {/* View mode toggle */}
        <Show when={props.onViewModeChange}>
          <div class="flex items-center gap-1">
            {/* Preview button - separate from toggle */}
            <Show when={canPreview(props.entry.name) && !props.entry.isDir}>
              <button
                class={`px-2 py-1 text-xs rounded flex items-center gap-1 border ${
                  effectiveMode() === "preview" 
                    ? 'bg-accent text-white border-accent' 
                    : 'bg-bg-panel border-border text-txt-secondary hover:text-txt hover:border-accent'
                }`}
                onClick={effectiveMode() === "preview" ? closePreview : handlePreview}
                disabled={previewLoading()}
                title={effectiveMode() === "preview" ? "Close preview" : "Preview as document"}
              >
                <HiOutlineEye class="w-3 h-3" />
                {previewLoading() ? "Loading..." : effectiveMode() === "preview" ? "Close" : "Preview"}
              </button>
            </Show>
            
            {/* Hex/Text toggle */}
            <div class="flex items-center gap-0.5 bg-bg-panel rounded border border-border">
              <button 
                class={`px-2 py-1 text-xs rounded ${props.viewMode === "hex" || (props.viewMode !== "text" && props.viewMode !== "preview") ? 'bg-accent text-white' : 'text-txt-secondary hover:text-txt'}`}
                onClick={() => { closePreview(); props.onViewModeChange?.("hex"); }}
                title="View as hex"
              >
                Hex
              </button>
              <button 
                class={`px-2 py-1 text-xs rounded ${props.viewMode === "text" ? 'bg-accent text-white' : 'text-txt-secondary hover:text-txt'}`}
                onClick={() => { closePreview(); props.onViewModeChange?.("text"); }}
                title="View as text"
              >
                Text
              </button>
            </div>
          </div>
        </Show>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-hidden">
        {/* Preview Error */}
        <Show when={previewError()}>
          <div class="p-4 bg-error/10 text-error text-sm">
            <strong>Preview Error:</strong> {previewError()}
          </div>
        </Show>
        
        {/* Preview Loading */}
        <Show when={previewLoading()}>
          <div class="flex items-center justify-center h-full">
            <div class="text-txt-muted">Extracting file for preview...</div>
          </div>
        </Show>
        
        {/* Preview Mode - use appropriate viewer */}
        <Show when={effectiveMode() === "preview" && previewPath() && !previewLoading()}>
          <Show when={isPdf(props.entry.name)} fallback={
            <Show when={isImage(props.entry.name)} fallback={
              <Show when={isSpreadsheet(props.entry.name)} fallback={
                <DocumentViewer path={previewPath()!} />
              }>
                {/* Native spreadsheet viewer */}
                <SpreadsheetViewer path={previewPath()!} />
              </Show>
            }>
              {/* Image preview */}
              <div class="flex items-center justify-center h-full p-4 overflow-auto">
                <img 
                  src={`file://${previewPath()}`} 
                  alt={props.entry.name}
                  class="max-w-full max-h-full object-contain"
                />
              </div>
            </Show>
          }>
            <PdfViewer path={previewPath()!} />
          </Show>
        </Show>
        
        {/* Hex View */}
        <Show when={effectiveMode() === "hex" && !previewLoading()}>
          <HexViewer entry={props.entry} />
        </Show>
        
        {/* Text View */}
        <Show when={effectiveMode() === "text" && !previewLoading()}>
          <TextViewer entry={props.entry} />
        </Show>
      </div>
    </div>
  );
}

export default ContainerEntryViewer;
