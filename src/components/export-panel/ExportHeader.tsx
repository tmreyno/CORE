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
  HiOutlineCpuChip,
  HiOutlineShieldCheck,
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
        <h2 class="text-sm font-semibold text-txt">Acquire & Export</h2>
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
        <div class="flex gap-2 items-center justify-between flex-wrap">
          <div class="flex gap-2 flex-wrap" role="tablist" aria-label="Export mode">
            <button
              class={props.mode() === "triage" ? "btn-sm-primary" : "btn-sm"}
              onClick={() => props.setMode("triage")}
              title="Step 1: Quick triage — collect system artifacts, credentials, and security data"
              role="tab" aria-selected={props.mode() === "triage"}
            >
              <HiOutlineShieldCheck class="w-4 h-4" />
              Triage
            </button>

            <button
              class={props.mode() === "memory" ? "btn-sm-primary" : "btn-sm"}
              onClick={() => props.setMode("memory")}
              title="Step 2: Capture live system memory (RAM) — volatile data must be collected before imaging"
              role="tab" aria-selected={props.mode() === "memory"}
            >
              <HiOutlineCpuChip class="w-4 h-4" />
              Memory
            </button>

            <button
              class={props.mode() === "physical" ? "btn-sm-primary" : "btn-sm"}
              onClick={() => props.setMode("physical")}
              title="Step 3: Create a forensic disk image (E01 or Raw) from a physical drive"
              role="tab" aria-selected={props.mode() === "physical"}
            >
              <HiOutlineCircleStack class="w-4 h-4" />
              Physical Image
            </button>

            <button
              class={props.mode() === "logical" ? "btn-sm-primary" : "btn-sm"}
              onClick={() => props.setMode("logical")}
              title="Step 4: Acquire files and folders into an L01 logical evidence container"
              role="tab" aria-selected={props.mode() === "logical"}
            >
              <HiOutlineDocumentDuplicate class="w-4 h-4" />
              Logical Image
            </button>

            <button
              class={props.mode() === "aff4" ? "btn-sm-primary" : "btn-sm"}
              onClick={() => props.setMode("aff4")}
              title="Step 5: Acquire files and folders into an AFF4 forensic container with selectable compression and hashes"
              role="tab" aria-selected={props.mode() === "aff4"}
            >
              <HiOutlineDocumentDuplicate class="w-4 h-4" />
              AFF4 Image
            </button>

            <button
              class={props.mode() === "native" ? "btn-sm-primary" : "btn-sm"}
              onClick={() => props.setMode("native")}
              title="Step 6: Export files or create 7z archive with hash manifests"
              role="tab" aria-selected={props.mode() === "native"}
            >
              <HiOutlineArrowUpTray class="w-4 h-4" />
              Export
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
            Acquire physical drives and evidence files into E01 forensic containers or Raw (.dd) disk images with case metadata and hashes
          </Show>
          <Show when={props.mode() === "logical"}>
            Acquire files and folders into L01 logical evidence containers with per-file hashes
          </Show>
          <Show when={props.mode() === "aff4"}>
            Acquire files and folders into AFF4 forensic containers with selectable compression and hash algorithms
          </Show>
          <Show when={props.mode() === "native"}>
            Copy files with forensic manifests, or create compressed 7z archives
          </Show>
          <Show when={props.mode() === "memory"}>
            Capture live system memory (RAM) to a raw dump file for analysis with Volatility, Rekall, or other frameworks
          </Show>
          <Show when={props.mode() === "triage"}>
            Collect system artifacts, credentials, SSH keys, browser data, and security files from a live system
          </Show>
          <Show when={props.mode() === "tools"}>
            Test, repair, validate, or extract split archives
          </Show>
        </div>
      </div>
    </>
  );
}
