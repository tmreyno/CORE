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

type ParagraphHint =
  | "normal" | "heading1" | "heading2" | "heading3" | "heading4"
  | "title" | "subtitle" | "listItem" | "quote";

interface OfficeParagraph {
  text: string;
  hint: ParagraphHint;
}

interface OfficeTextSection {
  label: string | null;
  paragraphs: OfficeParagraph[];
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
        <div class="flex-1 overflow-y-auto bg-bg">
          {/* Warnings */}
          <Show when={hasWarnings()}>
            <div class="max-w-[816px] mx-auto mt-3 px-3">
              <div class="p-2 rounded-lg bg-warning/10 border border-warning/30">
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
            </div>
          </Show>

          {/* Document page */}
          <div class="max-w-[816px] mx-auto my-4 bg-bg-card shadow-lg rounded-sm border border-border">
            {/* Page content with document-like padding */}
            <div class="px-12 py-10">
              {/* Metadata header (if title/author exist) */}
              <Show when={hasMetadata() && info()!.metadata.title}>
                <div class="mb-6 pb-4 border-b border-border">
                  <Show when={info()!.metadata.title}>
                    <h1 class="text-2xl font-bold text-txt mb-1">
                      {info()!.metadata.title}
                    </h1>
                  </Show>
                  <div class="flex items-center gap-3 text-xs text-txt-muted mt-2">
                    <Show when={info()!.metadata.creator}>
                      <span>{info()!.metadata.creator}</span>
                    </Show>
                    <Show when={info()!.metadata.modified}>
                      <span>Modified: {info()!.metadata.modified}</span>
                    </Show>
                    <Show when={info()!.metadata.application}>
                      <span>{info()!.metadata.application}</span>
                    </Show>
                  </div>
                </div>
              </Show>

              {/* Document sections */}
              <For each={info()!.sections}>
                {(section) => (
                  <div>
                    {/* Section label (e.g., "Slide 1") */}
                    <Show when={section.label}>
                      <div class="text-xs font-semibold text-accent mt-6 mb-3 uppercase tracking-wide border-b border-accent/20 pb-1">
                        {section.label}
                      </div>
                    </Show>

                    {/* Paragraphs with style hints */}
                    <Show
                      when={section.paragraphs.length > 0}
                      fallback={
                        <p class="text-sm text-txt-faint italic my-4">
                          No text content
                        </p>
                      }
                    >
                      <For each={section.paragraphs}>
                        {(para) => (
                          <DocumentParagraph text={para.text} hint={para.hint} />
                        )}
                      </For>
                    </Show>
                  </div>
                )}
              </For>

              {/* Empty state */}
              <Show when={sectionCount() === 0}>
                <div class="flex flex-col items-center justify-center py-16 text-txt-muted">
                  <HiOutlineDocument class="w-12 h-12 mb-3 opacity-40" />
                  <p class="text-base">No text content extracted</p>
                  <p class="text-xs mt-1.5">
                    This document may contain only images or non-text content
                  </p>
                </div>
              </Show>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
}

// =============================================================================
// Document Paragraph — renders text based on its ParagraphHint
// =============================================================================

function DocumentParagraph(props: { text: string; hint: ParagraphHint }) {
  switch (props.hint) {
    case "title":
      return (
        <h1 class="text-2xl font-bold text-txt mt-6 mb-3 leading-tight select-text">
          {props.text}
        </h1>
      );
    case "subtitle":
      return (
        <h2 class="text-lg font-medium text-txt-secondary mt-1 mb-4 leading-snug select-text">
          {props.text}
        </h2>
      );
    case "heading1":
      return (
        <h2 class="text-xl font-bold text-txt mt-8 mb-2 leading-tight select-text border-b border-border pb-1">
          {props.text}
        </h2>
      );
    case "heading2":
      return (
        <h3 class="text-lg font-semibold text-txt mt-6 mb-2 leading-snug select-text">
          {props.text}
        </h3>
      );
    case "heading3":
      return (
        <h4 class="text-base font-semibold text-txt mt-5 mb-1.5 leading-snug select-text">
          {props.text}
        </h4>
      );
    case "heading4":
      return (
        <h5 class="text-sm font-semibold text-txt-secondary mt-4 mb-1 leading-snug select-text">
          {props.text}
        </h5>
      );
    case "listItem":
      return (
        <div class="flex gap-2 ml-6 my-1 select-text">
          <span class="text-txt-faint select-none mt-0.5">•</span>
          <p class="text-[15px] text-txt leading-relaxed">
            {props.text}
          </p>
        </div>
      );
    case "quote":
      return (
        <blockquote class="ml-4 pl-4 border-l-2 border-border my-3 select-text">
          <p class="text-[15px] text-txt-secondary italic leading-relaxed">
            {props.text}
          </p>
        </blockquote>
      );
    default:
      return (
        <p class="text-[15px] text-txt leading-relaxed my-2 whitespace-pre-wrap select-text">
          {props.text}
        </p>
      );
  }
}
