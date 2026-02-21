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

import { createSignal, createEffect, Show, Switch, Match, untrack, createMemo, lazy, Suspense } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { HiOutlineDocument, HiOutlineArrowLeft, HiOutlineEye } from "./icons";
import { CompactErrorBoundary } from "./ErrorBoundary";
import { logger } from '../utils/logger';
import type { SelectedEntry } from "./EvidenceTree";

const log = logger.scope('ContainerEntryViewer');
import { HexViewer } from "./HexViewer";
import { TextViewer } from "./TextViewer";

// Lazy-loaded viewers — only loaded when the user previews that file type
const DocumentViewer = lazy(() => import("./DocumentViewer").then(m => ({ default: m.DocumentViewer })));
const PdfViewer = lazy(() => import("./PdfViewer").then(m => ({ default: m.PdfViewer })));
const SpreadsheetViewer = lazy(() => import("./SpreadsheetViewer").then(m => ({ default: m.SpreadsheetViewer })));
const ImageViewer = lazy(() => import("./ImageViewer").then(m => ({ default: m.ImageViewer })));
const EmailViewer = lazy(() => import("./EmailViewer").then(m => ({ default: m.EmailViewer })));
const PstViewer = lazy(() => import("./PstViewer").then(m => ({ default: m.PstViewer })));
const PlistViewer = lazy(() => import("./PlistViewer").then(m => ({ default: m.PlistViewer })));
const ExifPanel = lazy(() => import("./ExifPanel").then(m => ({ default: m.ExifPanel })));
const BinaryViewer = lazy(() => import("./BinaryViewer").then(m => ({ default: m.BinaryViewer })));
const RegistryViewer = lazy(() => import("./RegistryViewer").then(m => ({ default: m.RegistryViewer })));
const DatabaseViewer = lazy(() => import("./DatabaseViewer").then(m => ({ default: m.DatabaseViewer })));
const OfficeViewer = lazy(() => import("./OfficeViewer").then(m => ({ default: m.OfficeViewer })));
import { formatBytes } from "../utils";
import { 
  getExtension, 
  isImage, 
  isSpreadsheet, 
  isPdf, 
  isTextDocument,
  isCode,
  isEmail,
  isPst,
  isPlist,
  isBinaryExecutable,
  isRegistryHive,
  isDatabase,
  isConfig,
  isOffice,
} from "../utils/fileTypeUtils";
import type { ViewerMetadata, ViewerMetadataSection, ArchiveMetadataSection } from "../types/viewerMetadata";

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
  // Use centralized type guards — single source of truth
  return (
    isPdf(name) ||
    isImage(name) ||
    isSpreadsheet(name) ||
    isOffice(name) ||
    isTextDocument(name) ||
    isCode(name) ||
    isConfig(name) ||
    isEmail(name) ||
    isPst(name) ||
    isPlist(name) ||
    isBinaryExecutable(name) ||
    isDatabase(name) ||
    isRegistryHive(name)
  );
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
  const fileCanPreview = createMemo(() => canPreview(props.entry.name));
  const fileIsPdf = createMemo(() => isPdf(props.entry.name) || detectedFormat()?.viewerType === "Pdf");
  const fileIsImage = createMemo(() => isImage(props.entry.name) || detectedFormat()?.viewerType === "Image" || detectedFormat()?.viewerType === "Svg");
  const fileIsSpreadsheet = createMemo(() => isSpreadsheet(props.entry.name) || detectedFormat()?.viewerType === "Spreadsheet");
  const fileIsOffice = createMemo(() => isOffice(props.entry.name) || detectedFormat()?.viewerType === "Office");
  // For Document viewer: only use extension-based checks (DocumentViewer backend requires known extension).
  // Content-detected text files with unknown extensions go through the separate fileIsDetectedText path.
  const fileIsDocument = createMemo(() => isTextDocument(props.entry.name) || isCode(props.entry.name) || isConfig(props.entry.name) || detectedFormat()?.viewerType === "Html");
  // Content-detected text: file has unknown extension but magic bytes say it's text/JSON
  const fileIsDetectedText = createMemo(() => !fileIsDocument() && (detectedFormat()?.viewerType === "Text"));
  const fileIsEmail = createMemo(() => isEmail(props.entry.name) || detectedFormat()?.viewerType === "Email");
  const fileIsPst = createMemo(() => isPst(props.entry.name) || detectedFormat()?.viewerType === "Pst");
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
    if (isCode(props.entry.name) || isTextDocument(props.entry.name) || isConfig(props.entry.name)) {
      return "text";
    }
    
    // Default to hex for binary files
    return "hex";
  };
  
  // Extract file to temp and set preview path
  // For disk files, we can use the path directly without extraction
  const handlePreview = async () => {
    if (props.entry.isDir) return;
    if (previewPath()) return; // Already have a preview
    
    // Capture the entry key NOW (before await) so we stamp the correct key
    // even if props.entry changes during async extraction.
    const capturedKey = `${props.entry.containerPath}::${props.entry.entryPath}`;
    
    setPreviewLoading(true);
    setPreviewError(null);
    
    try {
      // Detect if this is a disk file (a real file on the filesystem, not inside a container).
      // VFS entries have paths like "/Partition1_NTFS/file.txt" which start with "/" but are NOT
      // real filesystem paths. We must never treat them as disk files.
      const isDiskFile = props.entry.isDiskFile === true || 
        (props.entry.containerPath === props.entry.entryPath && !props.entry.isVfsEntry && !props.entry.isArchiveEntry);
      
      let filePath: string;
      
      if (isDiskFile) {
        log.debug('Using disk file path directly:', props.entry.entryPath);
        filePath = props.entry.entryPath;
      } else {
        // For container entries, extract to temp
        log.debug('Extracting for preview:', props.entry.entryPath, {
          containerPath: props.entry.containerPath,
          isVfsEntry: props.entry.isVfsEntry,
          isArchiveEntry: props.entry.isArchiveEntry,
          isDiskFile: props.entry.isDiskFile,
          size: props.entry.size,
        });
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
      
      // If the entry changed during extraction, discard the result
      if (capturedKey !== entryKey()) {
        log.debug('Entry changed during extraction, discarding preview:', capturedKey);
        return;
      }
      
      // Stamp the entry key so guardedPreviewPath() knows this path
      // belongs to the current entry (prevents stale path leaking on switch).
      previewPathEntryKey = capturedKey;
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
      // Only set error if we're still on the same entry
      if (capturedKey === entryKey()) {
        log.error("Preview extraction failed:", e);
        setPreviewError(e instanceof Error ? e.message : String(e));
      }
    } finally {
      // Only clear loading if we're still on the same entry that started this extraction.
      // If the user clicked a different entry during extraction, the new entry's
      // handlePreview() has its own loading state — don't clear it.
      if (capturedKey === entryKey()) {
        setPreviewLoading(false);
      }
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
  
  // Track which entry key the current previewPath belongs to.
  // This prevents stale preview paths from rendering when the entry changes.
  let previewPathEntryKey = "";
  
  // Reactive entry key — changes whenever the selected entry changes.
  const entryKey = createMemo(() => `${props.entry.containerPath}::${props.entry.entryPath}`);
  
  // Guarded preview path: returns the preview path only if it belongs to the
  // currently selected entry. When the user clicks a new file, the entry key
  // changes immediately but the old previewPath signal hasn't been cleared yet
  // (the reset effect runs after render). This accessor prevents the old path
  // from leaking into the render for the new entry.
  const guardedPreviewPath = () => {
    const key = entryKey();
    const path = previewPath();
    // If the path was set for a different entry, treat it as null
    if (path && previewPathEntryKey !== key) return null;
    return path;
  };

  // Determine effective mode for display
  // Uses guardedPreviewPath() to ensure stale paths from previous entries
  // are never read during render when the entry has changed.
  // Note: createMemo ensures consistent tracking — all <Show> blocks read
  // the same memoized value and re-evaluate together when dependencies change.
  const effectiveMode = createMemo((): "hex" | "text" | "preview" => {
    const path = guardedPreviewPath();
    const viewMode = props.viewMode;
    const loading = previewLoading();
    const error = previewError();
    const auto = autoMode();
    
    // If preview extraction failed, fall back to hex (user still sees error + content)
    if (error && !path) {
      return "hex";
    }
    // If we have a preview path and mode requests preview, show preview
    if ((viewMode === "preview" || viewMode === "document") && path) {
      return "preview";
    }
    
    // For explicit modes
    switch (viewMode) {
      case "hex": return "hex";
      case "text": return "text";
      case "preview": return path ? "preview" : "hex";
      case "document": return path ? "preview" : auto;
      case "auto": {
        // For auto mode: show preview if we have a path, otherwise use autoMode
        if (path) return "preview";
        const mode = auto;
        // If autoMode determined "preview" but path isn't ready yet,
        // show loading spinner (previewLoading handles this) or hex fallback
        if (mode === "preview" && !path) {
          // During extraction, show "preview" so the spinner renders
          // Once loading completes (success or failure), this recalculates
          if (loading) return "preview";
          // Not loading and no path = extraction not started or completed without path
          return "hex";
        }
        return mode;
      }
      default: return "hex";
    }
  });

  // Auto-extract for preview when entry changes or viewMode requests preview
  createEffect(() => {
    const currentKey = entryKey();
    const mode = props.viewMode;
    const entryChanged = currentKey !== lastEntryKey;
    
    // If entry changed, reset everything
    if (entryChanged) {
      lastEntryKey = currentKey;
      isHandlingPreview = false;
      previewPathEntryKey = "";
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
  
  // Viewer-specific metadata section captured from child viewers
  const [viewerSection, setViewerSection] = createSignal<ViewerMetadataSection | null>(null);
  
  // Clear viewer section when entry changes
  createEffect(() => {
    // Track entry key to detect changes (void to suppress unused-var lint)
    void `${props.entry.containerPath}::${props.entry.entryPath}`;
    setViewerSection(null);
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
      else if (fileIsOffice()) viewerType = "Office";
      else if (fileIsEmail()) viewerType = "Email";
      else if (fileIsPst()) viewerType = "PST";
      else if (fileIsPlist()) viewerType = "Plist";
      else if (fileIsBinary()) viewerType = "Binary";
      else if (fileIsRegistry()) viewerType = "Registry";
      else if (fileIsDatabase()) viewerType = "Database";
      else if (fileIsDetectedText()) viewerType = "Text";
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
        // Case document attributes (from SelectedEntry.metadata set by createDocumentEntry)
        modified: entry.metadata?.modified as string | null | undefined,
        caseNumber: entry.metadata?.case_number as string | null | undefined,
        evidenceId: entry.metadata?.evidence_id as string | null | undefined,
        documentType: entry.metadata?.document_type as string | null | undefined,
        format: entry.metadata?.format as string | null | undefined,
      },
      viewerType,
      sections: viewerSection() ? [viewerSection()!] : [],
    };

    // Add archive metadata section for archive entries
    if (entry.isArchiveEntry && entry.metadata?.archiveFormat) {
      const archiveSection: ArchiveMetadataSection = {
        kind: "archive",
        archiveFormat: (entry.metadata.archiveFormat as string) || "Unknown",
        totalEntries: (entry.metadata.totalEntries as number) || 0,
        totalFiles: (entry.metadata.totalFiles as number) || 0,
        totalFolders: (entry.metadata.totalFolders as number) || 0,
        archiveSize: (entry.metadata.archiveSize as number) || 0,
        encrypted: (entry.metadata.encrypted as boolean) || false,
        entryPath: entry.entryPath,
        entryCompressedSize: entry.metadata.entryCompressedSize as number | undefined,
        entryCrc32: entry.metadata.entryCrc32 as number | undefined,
        entryModified: entry.metadata.entryModified as string | undefined,
      };
      metadata.sections = [archiveSection, ...metadata.sections];
    }

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
      <div class="flex-1 overflow-hidden flex flex-col">
        {/* Preview Error - compact banner that doesn't block content */}
        <Show when={previewError()}>
          <div class="mx-4 mt-2 p-3 rounded-lg bg-error/10 border border-error/30 shrink-0">
            <div class="flex items-center gap-2">
              <svg class="w-4 h-4 text-error shrink-0" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
              </svg>
              <p class="text-error text-xs font-medium">Preview unavailable</p>
              <p class="text-txt-muted text-xs truncate flex-1">{previewError()}</p>
            </div>
          </div>
        </Show>
        
        {/* Viewer content area */}
        <div class="flex-1 overflow-hidden">
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
        <Show when={effectiveMode() === "preview" && guardedPreviewPath() && !previewLoading()}>
          {(() => {
            const path = guardedPreviewPath()!;
            const detected = detectedFormat();
            log.debug('Rendering preview:', { 
              name: props.entry.name, 
              path,
              isPdf: fileIsPdf(),
              isImage: fileIsImage(),
              isSpreadsheet: fileIsSpreadsheet(),
              isOffice: fileIsOffice(),
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
          <Suspense fallback={<div class="flex items-center justify-center h-full text-txt-muted text-sm">Loading viewer...</div>}>
          <CompactErrorBoundary name="ViewerSwitch">
          <Switch fallback={<DocumentViewer path={guardedPreviewPath()!} onMetadata={setViewerSection} />}>
            <Match when={fileIsPdf()}>
              <PdfViewer path={guardedPreviewPath()!} />
            </Match>
            <Match when={fileIsImage()}>
              {/* Image viewer full-width — EXIF metadata shown in right panel */}
              <ImageViewer path={guardedPreviewPath()!} />
              {/* Hidden EXIF data emitter — loads EXIF and sends metadata to right panel */}
              <ExifPanel path={guardedPreviewPath()!} onMetadata={setViewerSection} class="hidden" />
            </Match>
            <Match when={fileIsSpreadsheet()}>
              <SpreadsheetViewer path={guardedPreviewPath()!} onMetadata={setViewerSection} />
            </Match>
            <Match when={fileIsOffice()}>
              <OfficeViewer path={guardedPreviewPath()!} onMetadata={setViewerSection} />
            </Match>
            <Match when={fileIsEmail()}>
              <EmailViewer path={guardedPreviewPath()!} onMetadata={setViewerSection} />
            </Match>
            <Match when={fileIsPst()}>
              <PstViewer path={guardedPreviewPath()!} onMetadata={setViewerSection} />
            </Match>
            <Match when={fileIsPlist()}>
              <PlistViewer path={guardedPreviewPath()!} onMetadata={setViewerSection} />
            </Match>
            <Match when={fileIsBinary()}>
              <BinaryViewer path={guardedPreviewPath()!} onMetadata={setViewerSection} />
            </Match>
            <Match when={fileIsRegistry()}>
              <RegistryViewer path={guardedPreviewPath()!} onMetadata={setViewerSection} />
            </Match>
            <Match when={fileIsDatabase()}>
              <DatabaseViewer path={guardedPreviewPath()!} onMetadata={setViewerSection} />
            </Match>
            <Match when={fileIsDetectedText()}>
              {/* Content-detected text file with unknown extension — use TextViewer with extracted temp file */}
              <TextViewer file={{ path: guardedPreviewPath()!, filename: props.entry.name, container_type: "", size: props.entry.size, segment_count: 1 }} />
            </Match>
            <Match when={detectedFormat()?.viewerType === "Hex"}>
              {/* Hex viewer for detected unknown formats */}
              <HexViewer entry={props.entry} />
            </Match>
            <Match when={detectedFormat()?.viewerType === "Archive"}>
              {/* Archives inside containers can't be browsed as trees — show hex */}
              <HexViewer entry={props.entry} />
            </Match>
          </Switch>
          </CompactErrorBoundary>
          </Suspense>
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
    </div>
  );
}
