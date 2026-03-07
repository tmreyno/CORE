// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import {
  HiOutlineCircleStack,
  HiOutlineDocumentDuplicate,
  HiOutlineArrowUpTray,
  HiOutlineWrench,
  HiOutlineXMark,
} from "../icons";
import type { ExportMode } from "../../hooks/useExportState";
import type { Accessor, Setter } from "solid-js";

interface ExportHeaderProps {
  mode: Accessor<ExportMode>;
  setMode: Setter<ExportMode>;
  onReset: () => void;
  onClose?: () => void;
}

export function ExportHeader(props: ExportHeaderProps) {
  return (
    <>
      {/* Header */}
      <div class="panel-header">
        <h2 class="text-sm font-semibold text-txt">Export & Archive</h2>
        <div class="flex items-center gap-2">
          <Show when={props.onClose}>
            <button class="icon-btn-sm" onClick={props.onClose} title="Close">
              <HiOutlineXMark class="w-4 h-4" />
            </button>
          </Show>
        </div>
      </div>

      {/* Mode Selector */}
      <div class="p-3 border-b border-border">
        <div class="flex gap-2 items-center justify-between">
          <div class="flex gap-2">
            <button
              class={props.mode() === "physical" ? "btn-sm-primary" : "btn-sm"}
              onClick={() => props.setMode("physical")}
              title="Create E01 disk image (physical/raw byte stream)"
            >
              <HiOutlineCircleStack class="w-4 h-4" />
              Physical
            </button>

            <button
              class={props.mode() === "logical" ? "btn-sm-primary" : "btn-sm"}
              onClick={() => props.setMode("logical")}
              title="Create L01 logical evidence container (file-based)"
            >
              <HiOutlineDocumentDuplicate class="w-4 h-4" />
              Logical
            </button>

            <button
              class={props.mode() === "native" ? "btn-sm-primary" : "btn-sm"}
              onClick={() => props.setMode("native")}
              title="Export files or create 7z archive"
            >
              <HiOutlineArrowUpTray class="w-4 h-4" />
              Native
            </button>

            <button
              class={props.mode() === "tools" ? "btn-sm-primary" : "btn-sm"}
              onClick={() => props.setMode("tools")}
              title="Archive Tools (Test, Repair, Validate, Extract)"
            >
              <HiOutlineWrench class="w-4 h-4" />
              Tools
            </button>
          </div>

          <button class="btn-sm" onClick={props.onReset} title="Clear all form fields">
            <HiOutlineXMark class="w-4 h-4" />
            Clear
          </button>
        </div>

        {/* Mode Description */}
        <div class="mt-2 text-xs text-txt-secondary">
          <Show when={props.mode() === "physical"}>
            Wrap raw images and evidence files into E01 containers with case metadata and hashes
          </Show>
          <Show when={props.mode() === "logical"}>
            Package files and folders into L01 logical evidence containers with per-file hashes
          </Show>
          <Show when={props.mode() === "native"}>
            Copy files with forensic manifests, or create compressed 7z archives
          </Show>
          <Show when={props.mode() === "tools"}>
            Test, repair, validate, or extract split archives
          </Show>
        </div>
      </div>
    </>
  );
}
