// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, createMemo, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { HiOutlineDocument, HiOutlineExclamationTriangle } from "../icons";
import { logger } from "../../utils/logger";
import { DocumentParagraph } from "./DocumentParagraph";
import type { OfficeViewerProps, OfficeDocumentInfo } from "./types";

const log = logger.scope("OfficeViewer");

export function OfficeViewer(props: OfficeViewerProps) {
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [info, setInfo] = createSignal<OfficeDocumentInfo | null>(null);
  let loadGeneration = 0;

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
    const gen = ++loadGeneration;
    setLoading(true);
    setError(null);

    try {
      const result = await invoke<OfficeDocumentInfo>("office_read_document", {
        path: props.path,
      });
      if (gen !== loadGeneration) return; // stale response
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
      if (gen !== loadGeneration) return;
      log.error("Failed to load office document:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

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
            <div class="px-12 py-10">
              {/* Metadata header */}
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
                    <Show when={section.label}>
                      <div class="text-xs font-semibold text-accent mt-6 mb-3 uppercase tracking-wide border-b border-accent/20 pb-1">
                        {section.label}
                      </div>
                    </Show>
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
