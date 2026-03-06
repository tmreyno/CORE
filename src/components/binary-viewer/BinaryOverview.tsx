// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show } from "solid-js";
import { WarningIcon } from "../icons";
import { formatBytes } from "../../utils";
import type { BinaryInfo } from "./types";
import { formatHex, formatTimestamp } from "./helpers";

interface BinaryOverviewProps {
  data: BinaryInfo;
}

export function BinaryOverview(props: BinaryOverviewProps) {
  const data = () => props.data;

  return (
    <>
      {/* Overview Grid */}
      <div class="grid grid-cols-2 gap-3">
        <div class="stat-box">
          <div class="text-txt-muted text-xs">Format</div>
          <div class="text-sm font-semibold text-txt">{data().format}</div>
        </div>
        <div class="stat-box">
          <div class="text-txt-muted text-xs">Architecture</div>
          <div class="text-sm font-semibold text-txt">
            {data().architecture} ({data().is_64bit ? "64-bit" : "32-bit"})
          </div>
        </div>
        <div class="stat-box">
          <div class="text-txt-muted text-xs">Entry Point</div>
          <div class="text-sm font-mono text-txt">{formatHex(data().entry_point)}</div>
        </div>
        <div class="stat-box">
          <div class="text-txt-muted text-xs">File Size</div>
          <div class="text-sm font-semibold text-txt">{formatBytes(data().file_size)}</div>
        </div>
      </div>

      {/* PE-specific info */}
      <Show when={data().pe_timestamp || data().pe_subsystem}>
        <div class="card">
          <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-2">PE Information</h3>
          <Show when={data().pe_timestamp}>
            <div class="flex gap-2 text-xs py-0.5">
              <span class="text-txt-muted w-24">Compile Time</span>
              <span class="text-accent font-mono">{formatTimestamp(data().pe_timestamp)}</span>
            </div>
          </Show>
          <Show when={data().pe_subsystem}>
            <div class="flex gap-2 text-xs py-0.5">
              <span class="text-txt-muted w-24">Subsystem</span>
              <span class="text-txt">{data().pe_subsystem}</span>
            </div>
          </Show>
          <Show when={data().pe_checksum}>
            <div class="flex gap-2 text-xs py-0.5">
              <span class="text-txt-muted w-24">Checksum</span>
              <span class="text-txt font-mono">{formatHex(data().pe_checksum)}</span>
            </div>
          </Show>
        </div>
      </Show>

      {/* Mach-O specific */}
      <Show when={data().macho_cpu_type || data().macho_filetype}>
        <div class="card">
          <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-2">Mach-O Information</h3>
          <Show when={data().macho_cpu_type}>
            <div class="flex gap-2 text-xs py-0.5">
              <span class="text-txt-muted w-24">CPU Type</span>
              <span class="text-txt">{data().macho_cpu_type}</span>
            </div>
          </Show>
          <Show when={data().macho_filetype}>
            <div class="flex gap-2 text-xs py-0.5">
              <span class="text-txt-muted w-24">File Type</span>
              <span class="text-txt">{data().macho_filetype}</span>
            </div>
          </Show>
        </div>
      </Show>

      {/* Security Indicators */}
      <div class="card">
        <h3 class="text-xs font-semibold text-txt-secondary uppercase tracking-wider mb-2 flex items-center gap-1">
          <WarningIcon class="w-3 h-3" /> Security Indicators
        </h3>
        <div class="grid grid-cols-3 gap-2 text-xs">
          <div class="flex items-center gap-1.5">
            <span class={`w-2 h-2 rounded-full ${data().has_debug_info ? "bg-warning" : "bg-bg-hover"}`} />
            <span class="text-txt-secondary">Debug Info</span>
          </div>
          <div class="flex items-center gap-1.5">
            <span class={`w-2 h-2 rounded-full ${data().is_stripped ? "bg-error" : "bg-bg-hover"}`} />
            <span class="text-txt-secondary">Stripped</span>
          </div>
          <div class="flex items-center gap-1.5">
            <span class={`w-2 h-2 rounded-full ${data().has_code_signing ? "bg-success" : "bg-bg-hover"}`} />
            <span class="text-txt-secondary">Code Signed</span>
          </div>
        </div>
      </div>
    </>
  );
}
