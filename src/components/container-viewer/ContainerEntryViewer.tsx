// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ContainerEntryViewer — View file content from within forensic containers.
 *
 * Supports AD1 (address-based V2 API), E01/Raw (VFS), and archives.
 * Routes to HexViewer, TextViewer, or a lazy-loaded preview viewer
 * (PDF, Image, Spreadsheet, Office, Email, PST, Plist, Binary, Registry, Database).
 */

import { createSignal, createEffect, Show, untrack, createMemo } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { logger } from "../../utils/logger";
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
} from "../../utils/fileTypeUtils";
import { HexViewer } from "../HexViewer";
import { TextViewer } from "../TextViewer";
import type { ArchiveMetadataSection, ViewerMetadataSection } from "../../types/viewerMetadata";
import type { ContainerEntryViewerProps, ContentDetectResult } from "./types";
import { canPreview } from "./canPreview";
import { ViewerSwitch } from "./ViewerSwitch";
import { ViewerHeader } from "./ViewerHeader";

const log = logger.scope("ContainerEntryViewer");

export function ContainerEntryViewer(props: ContainerEntryViewerProps) {
  const [previewPath, setPreviewPath] = createSignal<string | null>(null);
  const [previewLoading, setPreviewLoading] = createSignal(false);
  const [previewError, setPreviewError] = createSignal<string | null>(null);
  const [autoMode, setAutoMode] = createSignal<"hex" | "text" | "preview">("hex");
  const [detectedFormat, setDetectedFormat] = createSignal<ContentDetectResult | null>(null);

  // ── File type memos (avoid recalculating on every render) ───────────────

  const fileCanPreview = createMemo(() => canPreview(props.entry.name));
  const fileIsPdf = createMemo(
    () => isPdf(props.entry.name) || detectedFormat()?.viewerType === "Pdf",
  );
  const fileIsImage = createMemo(
    () =>
      isImage(props.entry.name) ||
      detectedFormat()?.viewerType === "Image" ||
      detectedFormat()?.viewerType === "Svg",
  );
  const fileIsSpreadsheet = createMemo(
    () => isSpreadsheet(props.entry.name) || detectedFormat()?.viewerType === "Spreadsheet",
  );
  const fileIsOffice = createMemo(
    () => isOffice(props.entry.name) || detectedFormat()?.viewerType === "Office",
  );
  // For Document viewer: only use extension-based checks (backend requires known extension).
  // Content-detected text files with unknown extensions go through fileIsDetectedText.
  const fileIsDocument = createMemo(
    () =>
      isTextDocument(props.entry.name) ||
      isCode(props.entry.name) ||
      isConfig(props.entry.name) ||
      detectedFormat()?.viewerType === "Html",
  );
  // Content-detected text: unknown extension but magic bytes say it's text/JSON
  const fileIsDetectedText = createMemo(
    () => !fileIsDocument() && detectedFormat()?.viewerType === "Text",
  );
  const fileIsEmail = createMemo(
    () => isEmail(props.entry.name) || detectedFormat()?.viewerType === "Email",
  );
  const fileIsPst = createMemo(
    () => isPst(props.entry.name) || detectedFormat()?.viewerType === "Pst",
  );
  const fileIsPlist = createMemo(
    () => isPlist(props.entry.name) || detectedFormat()?.viewerType === "Plist",
  );
  const fileIsBinary = createMemo(
    () => isBinaryExecutable(props.entry.name) || detectedFormat()?.viewerType === "Binary",
  );
  const fileIsRegistry = createMemo(
    () => isRegistryHive(props.entry.name) || detectedFormat()?.viewerType === "Registry",
  );
  const fileIsDatabase = createMemo(
    () => isDatabase(props.entry.name) || detectedFormat()?.viewerType === "Database",
  );

  // Extension OR content detection says previewable
  const effectiveCanPreview = createMemo(() => fileCanPreview() || detectedFormat() !== null);

  // ── Auto-mode determination ─────────────────────────────────────────────

  const determineAutoMode = (): "hex" | "text" | "preview" => {
    if (fileCanPreview()) return "preview";
    if (detectedFormat() !== null) return "preview";
    if (
      isCode(props.entry.name) ||
      isTextDocument(props.entry.name) ||
      isConfig(props.entry.name)
    )
      return "text";
    return "hex";
  };

  // ── Preview extraction ──────────────────────────────────────────────────

  let lastEntryKey = "";
  let isHandlingPreview = false;
  let previewPathEntryKey = "";

  const entryKey = createMemo(
    () => `${props.entry.containerPath}::${props.entry.entryPath}`,
  );

  /** Preview path guarded against stale entries */
  const guardedPreviewPath = () => {
    const key = entryKey();
    const path = previewPath();
    if (path && previewPathEntryKey !== key) return null;
    return path;
  };

  const handlePreview = async () => {
    if (props.entry.isDir) return;
    if (previewPath()) return;

    const capturedKey = `${props.entry.containerPath}::${props.entry.entryPath}`;
    setPreviewLoading(true);
    setPreviewError(null);

    try {
      const isDiskFile =
        props.entry.isDiskFile === true ||
        (props.entry.containerPath === props.entry.entryPath &&
          !props.entry.isVfsEntry &&
          !props.entry.isArchiveEntry);

      let filePath: string;

      if (isDiskFile) {
        log.debug("Using disk file path directly:", props.entry.entryPath);
        filePath = props.entry.entryPath;
      } else {
        log.debug("Extracting for preview:", props.entry.entryPath, {
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
        log.debug("Preview extracted to:", filePath);
      }

      if (capturedKey !== entryKey()) {
        log.debug("Entry changed during extraction, discarding preview:", capturedKey);
        return;
      }

      previewPathEntryKey = capturedKey;
      setPreviewPath(filePath);

      // For unknown extensions, run magic-byte content detection
      if (!fileCanPreview()) {
        try {
          log.debug("Running content detection for unknown type:", props.entry.name);
          const detected = await invoke<ContentDetectResult>("detect_content_format", {
            path: filePath,
          });
          log.debug("Content detection result:", detected);
          if (detected.format !== "Binary" || detected.method === "magic") {
            setDetectedFormat(detected);
          }
        } catch (detectErr) {
          log.warn("Content detection failed, falling back to hex:", detectErr);
        }
      }
    } catch (e) {
      if (capturedKey === entryKey()) {
        log.error("Preview extraction failed:", e);
        setPreviewError(e instanceof Error ? e.message : String(e));
      }
    } finally {
      if (capturedKey === entryKey()) {
        setPreviewLoading(false);
      }
    }
  };

  const closePreview = () => {
    setPreviewPath(null);
    setPreviewError(null);
    setDetectedFormat(null);
    props.onViewModeChange?.("hex");
  };

  // ── Effective display mode ──────────────────────────────────────────────

  const effectiveMode = createMemo((): "hex" | "text" | "preview" => {
    const path = guardedPreviewPath();
    const viewMode = props.viewMode;
    const loading = previewLoading();
    const error = previewError();
    const auto = autoMode();

    if (error && !path) return "hex";
    if ((viewMode === "preview" || viewMode === "document") && path) return "preview";

    switch (viewMode) {
      case "hex":
        return "hex";
      case "text":
        return "text";
      case "preview":
        return path ? "preview" : "hex";
      case "document":
        return path ? "preview" : auto;
      case "auto": {
        if (path) return "preview";
        const mode = auto;
        if (mode === "preview" && !path) {
          if (loading) return "preview";
          return "hex";
        }
        return mode;
      }
      default:
        return "hex";
    }
  });

  // ── Auto-extract effect ─────────────────────────────────────────────────

  createEffect(() => {
    const currentKey = entryKey();
    const mode = props.viewMode;
    const entryChanged = currentKey !== lastEntryKey;

    if (entryChanged) {
      lastEntryKey = currentKey;
      isHandlingPreview = false;
      previewPathEntryKey = "";
      setPreviewPath(null);
      setPreviewError(null);
      setDetectedFormat(null);
      setAutoMode(determineAutoMode());
    }

    const shouldPreview =
      (mode === "preview" || mode === "document" || mode === "auto") && !props.entry.isDir;
    const shouldAttempt = shouldPreview && (canPreview(props.entry.name) || mode === "auto");
    const hasPath = untrack(() => previewPath());

    if (shouldAttempt && !isHandlingPreview && !hasPath) {
      isHandlingPreview = true;
      handlePreview();
    }
  });

  // ── Viewer metadata emission ────────────────────────────────────────────

  const [viewerSection, setViewerSection] = createSignal<ViewerMetadataSection | null>(null);

  // Clear viewer section when entry changes
  createEffect(() => {
    void `${props.entry.containerPath}::${props.entry.entryPath}`;
    setViewerSection(null);
  });

  // Emit viewer metadata to parent for right panel display
  createEffect(() => {
    const entry = props.entry;
    const mode = effectiveMode();
    const detected = detectedFormat();
    const ext = getExtension(entry.name);

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

    const metadata = {
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

  // ── JSX ─────────────────────────────────────────────────────────────────

  return (
    <div class="flex flex-col h-full bg-bg">
      <ViewerHeader
        entry={props.entry}
        effectiveMode={effectiveMode}
        effectiveCanPreview={effectiveCanPreview}
        previewLoading={previewLoading}
        detectedFormat={detectedFormat}
        onBack={props.onBack}
        onViewModeChange={props.onViewModeChange}
        onPreview={handlePreview}
        onClosePreview={closePreview}
      />

      {/* Content */}
      <div class="flex-1 overflow-hidden flex flex-col">
        {/* Preview Error banner */}
        <Show when={previewError()}>
          <div class="mx-4 mt-2 p-3 rounded-lg bg-error/10 border border-error/30 shrink-0">
            <div class="flex items-center gap-2">
              <svg
                class="w-4 h-4 text-error shrink-0"
                fill="none"
                viewBox="0 0 24 24"
                stroke="currentColor"
              >
                <path
                  stroke-linecap="round"
                  stroke-linejoin="round"
                  stroke-width="2"
                  d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z"
                />
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
              <div class="relative w-12 h-12">
                <div class="absolute inset-0 rounded-full border-2 border-border opacity-30" />
                <div class="absolute inset-0 rounded-full border-2 border-accent border-t-transparent animate-spin" />
                <div
                  class="absolute inset-2 rounded-full border-2 border-accent/50 border-b-transparent animate-spin"
                  style="animation-direction: reverse; animation-duration: 0.8s"
                />
              </div>
              <div class="text-center">
                <p class="text-txt-secondary font-medium">Extracting file...</p>
                <p class="text-txt-muted text-xs mt-1">{props.entry.name}</p>
              </div>
            </div>
          </Show>

          {/* Preview Mode */}
          <Show when={effectiveMode() === "preview" && guardedPreviewPath() && !previewLoading()}>
            {(() => {
              const path = guardedPreviewPath()!;
              const detected = detectedFormat();
              log.debug("Rendering preview:", {
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
            <ViewerSwitch
              entry={props.entry}
              previewPath={guardedPreviewPath()!}
              detectedFormat={detectedFormat}
              onMetadata={setViewerSection}
              fileIsPdf={fileIsPdf}
              fileIsImage={fileIsImage}
              fileIsSpreadsheet={fileIsSpreadsheet}
              fileIsOffice={fileIsOffice}
              fileIsEmail={fileIsEmail}
              fileIsPst={fileIsPst}
              fileIsPlist={fileIsPlist}
              fileIsBinary={fileIsBinary}
              fileIsRegistry={fileIsRegistry}
              fileIsDatabase={fileIsDatabase}
              fileIsDetectedText={fileIsDetectedText}
              fileIsDocument={fileIsDocument}
            />
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
