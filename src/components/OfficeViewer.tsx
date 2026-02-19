// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * OfficeViewer - Native viewer for Office documents
 *
 * Renders extracted text and metadata from DOCX, DOC, PPTX, PPT, ODT, ODP,
 * and RTF files using the Rust backend `office_read_document` command.
 */

import { createSignal, createEffect, createMemo, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineDocument,
  HiOutlineExclamationTriangle,
} from "./icons";
import { logger } from "../utils/logger";
import type { OfficeMetadataSection } from "../types/viewerMetadata";

const log = logger.scope("OfficeViewer");

// =============================================================================
// Types (matching Rust OfficeDocumentInfo with camelCase serde)
// =============================================================================

interface OfficeMetadata {
  title: string | null;
  creator: string | null;
  lastModifiedBy: string | null;
  subject: string | null;
  description: string | null;
  created: string | null;
  modified: string | null;
  application: string | null;
  pageCount: number | null;
  wordCount: number | null;
  charCount: number | null;
}

type OfficeFormat =
  | "docx" | "doc" | "pptx" | "ppt"
  | "odt" | "odp" | "rtf" | "unknown";

interface OfficeTextSection {
  label: string | null;
  paragraphs: string[];
}

interface OfficeDocumentInfo {
  path: string;
  format: OfficeFormat;
  formatDescription: string;
  metadata: OfficeMetadata;
  sections: OfficeTextSection[];
  totalChars: number;
  totalWords: number;
  extractionComplete: boolean;
  warnings: string[];
}

// =============================================================================
// Props
// =============================================================================

interface OfficeViewerProps {
  /** Path to the office document file */
  path: string;
  /** Optional class name */
  class?: string;
  /** Callback to emit metadata section for right panel */
  onMetadata?: (section: OfficeMetadataSection) => void;
}

// =============================================================================
// Component
// =============================================================================

export function OfficeViewer(props: OfficeViewerProps) {
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [info, setInfo] = createSignal<OfficeDocumentInfo | null>(null);

  // Derived state
  const formatLabel = createMemo(() => {
    const doc = info();
    if (!doc) return "Office";
    switch (doc.format) {
      case "docx": return "DOCX";
      case "doc": return "DOC";
      case "pptx": return "PPTX";
      case "ppt": return "PPT";
      case "odt": return "ODT";
      case "odp": return "ODP";
      case "rtf": return "RTF";
      default: return "Office";
    }
  });

  const sectionCount = createMemo(() => info()?.sections?.length ?? 0);
  const hasWarnings = createMemo(() => (info()?.warnings?.length ?? 0) > 0);
  const hasMetadata = createMemo(() => {
    const meta = info()?.metadata;
    if (!meta) return false;
    return !!(meta.title || meta.creator || meta.created || meta.application);
  });

  // Load document
  const loadDocument = async () => {
    setLoading(true);
    setError(null);

    try {
      const result = await invoke<OfficeDocumentInfo>("office_read_document", {
        path: props.path,
      });
      setInfo(result);

      // Emit metadata for right panel
      if (props.onMetadata) {
        const meta = result.metadata;
        props.onMetadata({
          kind: "office",
          format: result.formatDescription,
          title: meta.title ?? undefined,
          creator: meta.creator ?? undefined,
          subject: meta.subject ?? undefined,
          description: meta.description ?? undefined,
          created: meta.created ?? undefined,
          modified: meta.modified ?? undefined,
          application: meta.application ?? undefined,
          pageCount: meta.pageCount ?? undefined,
          wordCount: meta.wordCount ?? undefined,
          charCount: meta.charCount ?? undefined,
          sectionCount: result.sections.length,
          totalWords: result.totalWords,
          totalChars: result.totalChars,
          extractionComplete: result.extractionComplete,
          warnings: result.warnings,
        });
      }
    } catch (e) {
      log.error("Failed to load office document:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  // Load on mount and when path changes
  createEffect(() => {
    void props.path;
    loadDocument();
  });

  return (
    <div class={`flex flex-col h-full bg-bg ${props.class ?? ""}`}>
      {/* Header bar */}
      <div class="flex items-center gap-2 px-3 py-2 border-b border-border bg-bg-secondary">
        <HiOutlineDocument class="w-icon-sm h-icon-sm text-accent" />
        <span class="text-sm font-medium text-txt">{formatLabel()}</span>
        <Show when={info()}>
          <span class="text-xs text-txt-muted">
            {info()!.totalWords.toLocaleString()} words · {sectionCount()} section{sectionCount() !== 1 ? "s" : ""}
          </span>
        </Show>
        <Show when={info() && !info()!.extractionComplete}>
          <span class="badge badge-warning text-xs">Partial</span>
        </Show>
      </div>

      {/* Loading */}
      <Show when={loading()}>
        <div class="flex items-center justify-center h-full text-txt-muted text-sm gap-2">
          <div class="animate-pulse-slow">Loading document...</div>
        </div>
      </Show>

      {/* Error */}
      <Show when={error()}>
        <div class="flex flex-col items-center justify-center h-full gap-3 p-6">
          <HiOutlineExclamationTriangle class="w-8 h-8 text-error" />
          <p class="text-sm text-error text-center max-w-md">{error()}</p>
          <button class="btn btn-secondary" onClick={loadDocument}>
            Retry
          </button>
        </div>
      </Show>

      {/* Content */}
      <Show when={!loading() && !error() && info()}>
        <div class="flex-1 overflow-y-auto">
          {/* Warnings */}
          <Show when={hasWarnings()}>
            <div class="mx-3 mt-3 p-2 rounded-lg bg-warning/10 border border-warning/30">
              <div class="flex items-center gap-1.5 text-xs text-warning font-medium mb-1">
                <HiOutlineExclamationTriangle class="w-3.5 h-3.5" />
                Extraction Warnings
              </div>
              <For each={info()!.warnings}>
                {(warning) => (
                  <p class="text-xs text-txt-muted ml-5">{warning}</p>
                )}
              </For>
            </div>
          </Show>

          {/* Metadata summary (inline, compact) */}
          <Show when={hasMetadata()}>
            <div class="mx-3 mt-3 p-3 rounded-lg bg-bg-secondary border border-border">
              <div class="grid grid-cols-2 gap-x-4 gap-y-1 text-xs">
                <Show when={info()!.metadata.title}>
                  <span class="text-txt-muted">Title</span>
                  <span class="text-txt truncate">{info()!.metadata.title}</span>
                </Show>
                <Show when={info()!.metadata.creator}>
                  <span class="text-txt-muted">Author</span>
                  <span class="text-txt truncate">{info()!.metadata.creator}</span>
                </Show>
                <Show when={info()!.metadata.application}>
                  <span class="text-txt-muted">Application</span>
                  <span class="text-txt truncate">{info()!.metadata.application}</span>
                </Show>
                <Show when={info()!.metadata.created}>
                  <span class="text-txt-muted">Created</span>
                  <span class="text-txt truncate">{info()!.metadata.created}</span>
                </Show>
                <Show when={info()!.metadata.modified}>
                  <span class="text-txt-muted">Modified</span>
                  <span class="text-txt truncate">{info()!.metadata.modified}</span>
                </Show>
                <Show when={info()!.metadata.pageCount != null}>
                  <span class="text-txt-muted">Pages</span>
                  <span class="text-txt">{info()!.metadata.pageCount}</span>
                </Show>
              </div>
            </div>
          </Show>

          {/* Text sections */}
          <div class="p-3 space-y-4">
            <For each={info()!.sections}>
              {(section) => (
                <div>
                  {/* Section label (e.g., "Slide 1") */}
                  <Show when={section.label}>
                    <div class="text-xs font-semibold text-accent mb-1.5 uppercase tracking-wide">
                      {section.label}
                    </div>
                  </Show>

                  {/* Paragraphs */}
                  <Show
                    when={section.paragraphs.length > 0}
                    fallback={
                      <p class="text-xs text-txt-muted italic">No text content</p>
                    }
                  >
                    <div class="space-y-1.5">
                      <For each={section.paragraphs}>
                        {(paragraph) => (
                          <p class="text-sm text-txt leading-relaxed whitespace-pre-wrap select-text">
                            {paragraph}
                          </p>
                        )}
                      </For>
                    </div>
                  </Show>
                </div>
              )}
            </For>

            {/* Empty state */}
            <Show when={sectionCount() === 0}>
              <div class="flex flex-col items-center justify-center py-12 text-txt-muted">
                <HiOutlineDocument class="w-10 h-10 mb-2 opacity-40" />
                <p class="text-sm">No text content extracted</p>
                <p class="text-xs mt-1">This document may contain only images or non-text content</p>
              </div>
            </Show>
          </div>
        </div>
      </Show>
    </div>
  );
}
