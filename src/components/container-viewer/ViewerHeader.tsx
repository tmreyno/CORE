// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ViewerHeader — Header bar for ContainerEntryViewer.
 * Shows file name, size, source badges, detected format, and view mode toggle.
 */

import { Show, type Accessor } from "solid-js";
import { HiOutlineDocument, HiOutlineArrowLeft, HiOutlineEye } from "../icons";
import { formatBytes } from "../../utils";
import type { SelectedEntry } from "../EvidenceTree";
import type { ContentDetectResult, EntryViewMode } from "./types";

export interface ViewerHeaderProps {
  entry: SelectedEntry;
  effectiveMode: Accessor<"hex" | "text" | "preview">;
  effectiveCanPreview: Accessor<boolean>;
  previewLoading: Accessor<boolean>;
  detectedFormat: Accessor<ContentDetectResult | null>;
  onBack?: () => void;
  onViewModeChange?: (mode: EntryViewMode) => void;
  onPreview: () => void;
  onClosePreview: () => void;
}

export function ViewerHeader(props: ViewerHeaderProps) {
  return (
    <div class="panel-header gap-3">
      <Show when={props.onBack}>
        <button class="btn-text" onClick={props.onBack} title="Back to file list">
          <HiOutlineArrowLeft class="w-3 h-3" /> Back
        </button>
      </Show>
      <div class="row flex-1 min-w-0">
        <span
          class="text-sm text-txt truncate flex items-center gap-1.5"
          title={props.entry.entryPath}
        >
          <HiOutlineDocument class="w-3.5 h-3.5 shrink-0" /> {props.entry.name}
        </span>
        <span class="text-xs text-txt-muted">{formatBytes(props.entry.size)}</span>
        <Show when={props.entry.isDiskFile}>
          <span class="px-1.5 py-0.5 text-2xs leading-tight bg-bg-hover text-txt-secondary rounded">
            Disk File
          </span>
        </Show>
        <Show when={props.entry.isVfsEntry}>
          <span class="px-1.5 py-0.5 text-2xs leading-tight bg-blue-700/50 text-blue-300 rounded">
            VFS
          </span>
        </Show>
        <Show when={props.entry.isArchiveEntry}>
          <span class="px-1.5 py-0.5 text-2xs leading-tight bg-purple-700/50 text-purple-300 rounded">
            Archive
          </span>
        </Show>
        <Show when={props.detectedFormat()}>
          <span
            class="px-1.5 py-0.5 text-2xs leading-tight bg-cyan-700/50 text-cyan-300 rounded"
            title={`Detected: ${props.detectedFormat()!.description} (${props.detectedFormat()!.mimeType})`}
          >
            {props.detectedFormat()!.description}
          </span>
        </Show>
      </div>

      {/* View mode toggle */}
      <Show when={props.onViewModeChange}>
        <div class="flex items-center gap-1">
          {/* Preview button */}
          <Show when={props.effectiveCanPreview() && !props.entry.isDir}>
            <button
              class={`px-2 py-1 text-xs rounded flex items-center gap-1 border ${
                props.effectiveMode() === "preview"
                  ? "bg-accent text-white border-accent"
                  : "bg-bg-panel border-border text-txt-secondary hover:text-txt hover:border-accent"
              }`}
              onClick={
                props.effectiveMode() === "preview" ? props.onClosePreview : props.onPreview
              }
              disabled={props.previewLoading()}
              title={
                props.effectiveMode() === "preview"
                  ? "Close preview"
                  : "Preview as document"
              }
            >
              <HiOutlineEye class="w-3 h-3" />
              {props.previewLoading()
                ? "Loading..."
                : props.effectiveMode() === "preview"
                  ? "Close"
                  : "Preview"}
            </button>
          </Show>

          {/* Hex/Text toggle */}
          <div class="flex items-center gap-0.5 bg-bg-panel rounded border border-border">
            <button
              class={`px-2 py-1 text-xs rounded ${
                props.effectiveMode() === "hex"
                  ? "bg-accent text-white"
                  : "text-txt-secondary hover:text-txt"
              }`}
              onClick={() => {
                props.onClosePreview();
                props.onViewModeChange?.("hex");
              }}
              title="View as hex"
            >
              Hex
            </button>
            <button
              class={`px-2 py-1 text-xs rounded ${
                props.effectiveMode() === "text"
                  ? "bg-accent text-white"
                  : "text-txt-secondary hover:text-txt"
              }`}
              onClick={() => {
                props.onClosePreview();
                props.onViewModeChange?.("text");
              }}
              title="View as text"
            >
              Text
            </button>
          </div>
        </div>
      </Show>
    </div>
  );
}
