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

import { createSignal, createEffect, Show, untrack, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { HiOutlineDocument, HiOutlineArrowLeft, HiOutlineEye } from "./icons";
import { logger } from '../utils/logger';
import type { SelectedEntry } from "./EvidenceTree";

const log = logger.scope('ContainerEntryViewer');
import { HexViewer } from "./HexViewer";
import { TextViewer } from "./TextViewer";
import { DocumentViewer } from "./DocumentViewer";
import { PdfViewer } from "./PdfViewer";
import { SpreadsheetViewer } from "./SpreadsheetViewer";
import { ImageViewer } from "./ImageViewer";
import { EmailViewer } from "./EmailViewer";
import { PlistViewer } from "./PlistViewer";
import { ExifPanel } from "./ExifPanel";
import { BinaryViewer } from "./BinaryViewer";
import { RegistryViewer } from "./RegistryViewer";
import { DatabaseViewer } from "./DatabaseViewer";
import { formatBytes } from "../utils";
import { 
  getExtension, 
  isImage, 
  isSpreadsheet, 
  isPdf, 
  isTextDocument,
  isCode,
  isEmail,
  isPlist,
  isBinaryExecutable,
  isRegistryHive,
  isDatabase,
} from "../utils/fileTypeUtils";
import type { ViewerMetadata } from "../types/viewerMetadata";

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
  /** Callback when viewer metadata is available (for right panel) */
  onMetadata?: (metadata: ViewerMetadata | null) => void;
}

/** Check if file type can be previewed with a native viewer */
function canPreview(name: string): boolean {
  const ext = getExtension(name);
  const previewable = [
    // Documents (backend-supported)
    "pdf", "docx", "doc", "html", "htm", "md", "markdown", "txt",
    // Spreadsheets (backend-supported)
    "xlsx", "xls", "csv", "ods",
    // Images
    "jpg", "jpeg", "png", "gif", "bmp", "webp", "svg", "ico",
    // Email
    "eml", "mbox",
    // Property lists
    "plist", "mobileprovision",
    // Binary executables
    "exe", "dll", "so", "dylib", "sys", "drv",
    "elf", "bin", "com", "scr", "ocx", "cpl",
    // Databases
    "db", "sqlite", "sqlite3",
  ];
  return previewable.includes(ext) || isRegistryHive(name);
}

export function ContainerEntryViewer(props: ContainerEntryViewerProps) {
  const [previewPath, setPreviewPath] = createSignal<string | null>(null);
  const [previewLoading, setPreviewLoading] = createSignal(false);
  const [previewError, setPreviewError] = createSignal<string | null>(null);
  const [autoMode, setAutoMode] = createSignal<"hex" | "text" | "preview">("hex");
  
  // Content detection for unknown file types (magic-byte based)
  interface ContentDetectResult {
    format: string;
    viewerType: string;
    description: string;
    mimeType: string;
    method: string;
  }
  const [detectedFormat, setDetectedFormat] = createSignal<ContentDetectResult | null>(null);
  
  // Memoized file type checks (avoid recalculating on every render)
  const fileExtension = createMemo(() => getExtension(props.entry.name));
  const fileCanPreview = createMemo(() => canPreview(props.entry.name));
  const fileIsPdf = createMemo(() => isPdf(props.entry.name) || detectedFormat()?.viewerType === "Pdf");
  const fileIsImage = createMemo(() => isImage(props.entry.name) || detectedFormat()?.viewerType === "Image");
  const fileIsSpreadsheet = createMemo(() => isSpreadsheet(props.entry.name) || detectedFormat()?.viewerType === "Spreadsheet");
  const fileIsDocument = createMemo(() => isTextDocument(props.entry.name) || detectedFormat()?.viewerType === "Text" || detectedFormat()?.viewerType === "Html");
  const fileIsEmail = createMemo(() => isEmail(props.entry.name) || detectedFormat()?.viewerType === "Email");
  const fileIsPlist = createMemo(() => isPlist(props.entry.name) || detectedFormat()?.viewerType === "Plist");
  const fileIsBinary = createMemo(() => isBinaryExecutable(props.entry.name) || detectedFormat()?.viewerType === "Binary");
  const fileIsRegistry = createMemo(() => isRegistryHive(props.entry.name) || detectedFormat()?.viewerType === "Registry");
  const fileIsDatabase = createMemo(() => isDatabase(props.entry.name) || detectedFormat()?.viewerType === "Database");
  
  // Whether this file can be previewed (by extension OR by detected content)
  const effectiveCanPreview = createMemo(() => fileCanPreview() || detectedFormat() !== null);
  
  // Determine the best mode for "auto" based on file type
  const determineAutoMode = (): "hex" | "text" | "preview" => {
    // Previewable files -> preview mode
    if (fileCanPreview()) {
      return "preview";
    }
    
    // If content detection found a viewable format, use preview
    if (detectedFormat() !== null) {
      return "preview";
    }
    
    // Code, config, and text files -> text mode
    if (isCode(props.entry.name) || isTextDocument(props.entry.name)) {
      return "text";
    }
    
    // Additional text-like extensions not in the centralized lists
    const extraTextExtensions = [
      "log", "ini", "cfg", "conf", "properties", "env",
      "gitignore", "editorconfig", "eslintrc", "prettierrc",
    ];
    if (extraTextExtensions.includes(fileExtension())) {
      return "text";
    }
    
    // Default to hex for binary files
    return "hex";
  };
  
  // Determine effective mode for display
  const effectiveMode = (): "hex" | "text" | "preview" => {
    // If we have a preview path and mode requests preview, show preview
    if ((props.viewMode === "preview" || props.viewMode === "document") && previewPath()) {
      return "preview";
    }
    
    // For explicit modes
    switch (props.viewMode) {
      case "hex": return "hex";
      case "text": return "text";
      case "preview": return previewPath() ? "preview" : "hex"; // Fallback to hex if no path yet
      case "document": return previewPath() ? "preview" : autoMode(); // Use auto mode until preview loads
      case "auto": {
        // For auto mode: show preview if we have a path, otherwise use autoMode
        // but only show "preview" effective mode when we actually have the path ready
        if (previewPath()) return "preview";
        const mode = autoMode();
        // If autoMode determined "preview" but path isn't ready yet, 
        // show hex/text fallback while extracting (previewLoading handles the spinner)
        if (mode === "preview" && !previewPath()) {
          // During extraction, fall back to showing the loading state
          // The previewLoading guard in the template will show the spinner
          return "preview";
        }
        return mode;
      }
      default: return "hex";
    }
  };
  
  // Extract file to temp and set preview path
  // For disk files, we can use the path directly without extraction
  const handlePreview = async () => {
    if (props.entry.isDir) return;
    if (previewPath()) return; // Already have a preview
    
    setPreviewLoading(true);
    setPreviewError(null);
    
    try {
      // Detect if this is a disk file:
      // 1. Explicitly marked as disk file
      // 2. containerPath equals entryPath (self-referencing = disk file)
      // 3. No container flags set and path is absolute
      const isDiskFile = props.entry.isDiskFile || 
        (props.entry.containerPath === props.entry.entryPath) ||
        (!props.entry.isVfsEntry && !props.entry.isArchiveEntry && props.entry.entryPath.startsWith('/'));
      
      let filePath: string;
      
      if (isDiskFile) {
        log.debug('Using disk file path directly:', props.entry.entryPath);
        filePath = props.entry.entryPath;
      } else {
        // For container entries, extract to temp
        log.debug('Extracting for preview:', props.entry.entryPath);
        filePath = await invoke<string>("container_extract_entry_to_temp", {
          containerPath: props.entry.containerPath,
          entryPath: props.entry.entryPath,
          entrySize: props.entry.size,
          isVfsEntry: props.entry.isVfsEntry || false,
          isArchiveEntry: props.entry.isArchiveEntry || false,
          dataAddr: props.entry.dataAddr ?? null,
        });
        log.debug('Preview extracted to:', filePath);
      }
      
      setPreviewPath(filePath);
      
      // For files with unknown extensions, run magic-byte content detection
      if (!fileCanPreview()) {
        try {
          log.debug('Running content detection for unknown type:', props.entry.name);
          const detected = await invoke<ContentDetectResult>("detect_content_format", { path: filePath });
          log.debug('Content detection result:', detected);
          // Only use detection if it found something useful (not generic Binary/fallback)
          if (detected.format !== "Binary" || detected.method === "magic") {
            setDetectedFormat(detected);
          }
        } catch (detectErr) {
          log.warn('Content detection failed, falling back to hex:', detectErr);
        }
      }
      
    } catch (e) {
      log.error("Preview extraction failed:", e);
      setPreviewError(e instanceof Error ? e.message : String(e));
    } finally {
      setPreviewLoading(false);
    }
  };
  
  // Close preview and return to hex/text
  const closePreview = () => {
    setPreviewPath(null);
    setPreviewError(null);
    setDetectedFormat(null);
    props.onViewModeChange?.("hex");
  };
  
  // Track state to prevent infinite loops (using plain JS variables, not signals)
  let lastEntryKey = "";
  let isHandlingPreview = false;
  
  // Auto-extract for preview when entry changes or viewMode requests preview
  createEffect(() => {
    // Create a unique key for this entry - this is what we WANT to react to
    const entryKey = `${props.entry.containerPath}::${props.entry.entryPath}`;
    const mode = props.viewMode;
    const entryChanged = entryKey !== lastEntryKey;
    
    // If entry changed, reset everything
    if (entryChanged) {
      lastEntryKey = entryKey;
      isHandlingPreview = false;
      setPreviewPath(null);
      setPreviewError(null);
      setDetectedFormat(null);
      setAutoMode(determineAutoMode());
    }
    
    // Determine if we should auto-preview
    const shouldPreview = (mode === "preview" || mode === "document" || mode === "auto") && 
                          !props.entry.isDir;
    
    // For auto mode: preview known types immediately, detect unknown types
    const shouldAttempt = shouldPreview && (canPreview(props.entry.name) || mode === "auto");
    
    // Check previewPath without tracking it (to avoid re-triggering when it's set)
    const hasPath = untrack(() => previewPath());
    
    // Only trigger preview if:
    // 1. We should preview/detect
    // 2. We're not already handling a preview
    // 3. We don't already have a preview path
    if (shouldAttempt && !isHandlingPreview && !hasPath) {
      isHandlingPreview = true;
      handlePreview();
    }
  });
  
  // Emit viewer metadata to parent for right panel display
  createEffect(() => {
    // React to changes in: entry info, effective mode, detected format
    const entry = props.entry;
    const mode = effectiveMode();
    const detected = detectedFormat();
    const ext = getExtension(entry.name);
    
    // Determine the active viewer type
    let viewerType = "Hex";
    if (mode === "preview") {
      if (fileIsPdf()) viewerType = "PDF";
      else if (fileIsImage()) viewerType = "Image";
      else if (fileIsSpreadsheet()) viewerType = "Spreadsheet";
      else if (fileIsEmail()) viewerType = "Email";
      else if (fileIsPlist()) viewerType = "Plist";
      else if (fileIsBinary()) viewerType = "Binary";
      else if (fileIsRegistry()) viewerType = "Registry";
      else if (fileIsDatabase()) viewerType = "Database";
      else if (fileIsDocument()) viewerType = "Document";
      else if (detected?.viewerType) viewerType = detected.viewerType;
    } else if (mode === "text") {
      viewerType = "Text";
    }

    const metadata: ViewerMetadata = {
      fileInfo: {
        name: entry.name,
        path: entry.entryPath,
        size: entry.size,
        extension: ext || undefined,
        containerPath: entry.containerPath !== entry.entryPath ? entry.containerPath : undefined,
        containerType: entry.containerType,
        isDiskFile: entry.isDiskFile,
        isVfsEntry: entry.isVfsEntry,
        isArchiveEntry: entry.isArchiveEntry,
      },
      viewerType,
      sections: [],
    };

    props.onMetadata?.(metadata);
  });
  
  // Clear metadata when component unmounts (entry changes away)
  // Note: Solid's reactive system handles cleanup naturally when
  // the component is removed from the DOM
  
  return (
    <div class="flex flex-col h-full bg-bg">
      {/* Header */}
      <div class="panel-header gap-3">
        <Show when={props.onBack}>
          <button class="btn-text" onClick={props.onBack} title="Back to file list">
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
          <Show when={detectedFormat()}>
            <span class="px-1.5 py-0.5 text-[10px] leading-tight bg-cyan-700/50 text-cyan-300 rounded" title={`Detected: ${detectedFormat()!.description} (${detectedFormat()!.mimeType})`}>
              {detectedFormat()!.description}
            </span>
          </Show>
        </div>
        
        {/* View mode toggle */}
        <Show when={props.onViewModeChange}>
          <div class="flex items-center gap-1">
            {/* Preview button - separate from toggle */}
            <Show when={effectiveCanPreview() && !props.entry.isDir}>
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
                class={`px-2 py-1 text-xs rounded ${effectiveMode() === "hex" ? 'bg-accent text-white' : 'text-txt-secondary hover:text-txt'}`}
                onClick={() => { closePreview(); props.onViewModeChange?.("hex"); }}
                title="View as hex"
              >
                Hex
              </button>
              <button 
                class={`px-2 py-1 text-xs rounded ${effectiveMode() === "text" ? 'bg-accent text-white' : 'text-txt-secondary hover:text-txt'}`}
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
          <div class="m-4 p-4 rounded-lg bg-error/10 border border-error/30">
            <div class="flex items-start gap-3">
              <div class="shrink-0 w-8 h-8 rounded-full bg-error/20 flex items-center justify-center">
                <svg class="w-4 h-4 text-error" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
                </svg>
              </div>
              <div class="flex-1 min-w-0">
                <p class="text-error font-medium text-sm">Preview Error</p>
                <p class="text-txt-muted text-xs mt-1">{previewError()}</p>
              </div>
            </div>
          </div>
        </Show>
        
        {/* Preview Loading */}
        <Show when={previewLoading()}>
          <div class="flex flex-col items-center justify-center h-full gap-4">
            {/* Animated spinner */}
            <div class="relative w-12 h-12">
              <div class="absolute inset-0 rounded-full border-2 border-border opacity-30" />
              <div class="absolute inset-0 rounded-full border-2 border-accent border-t-transparent animate-spin" />
              <div class="absolute inset-2 rounded-full border-2 border-accent/50 border-b-transparent animate-spin" style="animation-direction: reverse; animation-duration: 0.8s" />
            </div>
            {/* Loading text */}
            <div class="text-center">
              <p class="text-txt-secondary font-medium">Extracting file...</p>
              <p class="text-txt-muted text-xs mt-1">{props.entry.name}</p>
            </div>
          </div>
        </Show>
        
        {/* Preview Mode - use appropriate viewer */}
        <Show when={effectiveMode() === "preview" && previewPath() && !previewLoading()}>
          {(() => {
            const path = previewPath()!;
            const detected = detectedFormat();
            log.debug('Rendering preview:', { 
              name: props.entry.name, 
              path,
              isPdf: fileIsPdf(),
              isImage: fileIsImage(),
              isSpreadsheet: fileIsSpreadsheet(),
              isDocumentViewerFile: fileIsDocument(),
              isEmail: fileIsEmail(),
              isPlist: fileIsPlist(),
              isBinary: fileIsBinary(),
              isRegistry: fileIsRegistry(),
              isDatabase: fileIsDatabase(),
              detectedFormat: detected?.format,
              detectedViewer: detected?.viewerType,
            });
            return null;
          })()}
          <Show when={fileIsPdf()} fallback={
            <Show when={fileIsImage()} fallback={
              <Show when={fileIsSpreadsheet()} fallback={
                <Show when={fileIsEmail()} fallback={
                  <Show when={fileIsPlist()} fallback={
                    <Show when={fileIsBinary()} fallback={
                      <Show when={fileIsRegistry()} fallback={
                        <Show when={fileIsDatabase()} fallback={
                          <Show when={detectedFormat()?.viewerType === "Hex"} fallback={
                            <DocumentViewer path={previewPath()!} />
                          }>
                            {/* Hex viewer for detected unknown formats */}
                            <HexViewer entry={props.entry} />
                          </Show>
                        }>
                          {/* SQLite database viewer */}
                          <DatabaseViewer path={previewPath()!} />
                        </Show>
                      }>
                        {/* Windows Registry hive viewer */}
                        <RegistryViewer path={previewPath()!} />
                      </Show>
                    }>
                      {/* Binary executable analyzer (PE/ELF/Mach-O) */}
                      <BinaryViewer path={previewPath()!} />
                    </Show>
                  }>
                    {/* Apple property list viewer */}
                    <PlistViewer path={previewPath()!} />
                  </Show>
                }>
                  {/* Email viewer (EML/MBOX) */}
                  <EmailViewer path={previewPath()!} />
                </Show>
              }>
                {/* Native spreadsheet viewer */}
                <SpreadsheetViewer path={previewPath()!} />
              </Show>
            }>
              {/* Image viewer with EXIF metadata panel */}
              <div class="flex h-full">
                <div class="flex-1 overflow-hidden">
                  <ImageViewer path={previewPath()!} />
                </div>
                <ExifPanel path={previewPath()!} />
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
