// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * TransferJobCard Component
 * 
 * Displays a single transfer job with progress, status, and controls.
 */

import { Show } from "solid-js";
import { getBasename } from "../../utils";
import {
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
  HiOutlineArchiveBox,
  HiOutlineStop,
} from "../icons";
import { formatBytes, formatSpeed, formatEta } from "../../transfer";
import { formatShortDate, formatFullDateTime } from "./utils";
import type { TransferJobCardProps } from "./types";

export function TransferJobCard(props: TransferJobCardProps) {
  const job = () => props.job;
  
  return (
    <div class={`bg-bg-secondary rounded border border-border/50 p-2`}>
      {/* Job Header */}
      <div class="flex items-center justify-between mb-1">
        <div class="flex items-center gap-1.5 min-w-0">
          <Show when={job().status === "running"}>
            <div class="w-2 h-2 rounded-full bg-accent animate-pulse" />
          </Show>
          <Show when={job().status === "completed"}>
            <HiOutlineCheckCircle class={`w-3 h-3 text-green-400`} />
          </Show>
          <Show when={job().status === "failed" || job().status === "cancelled"}>
            <HiOutlineExclamationTriangle class={`w-3 h-3 text-red-400`} />
          </Show>
          <Show when={job().containerAware}>
            <HiOutlineArchiveBox class={`w-3 h-3 text-accent`} title="Forensic container transfer" />
          </Show>
          <span class={`text-[11px] leading-tight text-txt-tertiary truncate`}>
            {getBasename(job().destination)}
          </span>
        </div>
        <Show when={job().status === "running" || job().status === "pending"}>
          <button
            onClick={() => props.onCancel(job().id)}
            class={`flex items-center gap-1 px-2 py-0.5 rounded text-red-400 hover:text-red-300 hover:bg-red-500/20 border border-red-500/30 hover:border-red-500/50 transition-colors`}
            title="Stop transfer"
          >
            <HiOutlineStop class="w-3 h-3" />
            <span class="text-[11px] leading-tight">Stop</span>
          </button>
        </Show>
      </div>

      {/* Progress Bar */}
      <Show when={job().progress && job().status === "running"}>
        <div class="space-y-1">
          <div class="h-1 bg-bg-hover rounded-full overflow-hidden">
            <div
              class="h-full bg-accent transition-all duration-150"
              style={{ width: `${job().progress!.overall_percent}%` }}
            />
          </div>
          <div class="flex items-center justify-between">
            <span class={`text-[11px] leading-tight text-txt-muted`}>
              {getBasename(job().progress!.current_file || "") || "Starting..."}
            </span>
            <span class={`text-[11px] leading-tight text-txt-secondary`}>
              {formatSpeed(job().progress!.bytes_per_second)} · {formatEta(job().progress!.eta_seconds)}
            </span>
          </div>
        </div>
      </Show>

      {/* Completed Summary */}
      <Show when={job().result && job().status === "completed"}>
        <div class="space-y-1">
          <div class={`text-[11px] leading-tight text-txt-secondary`}>
            {job().result!.successful_files} files · {formatBytes(job().result!.bytes_transferred)}
            <Show when={job().endTime}>
              <span 
                class="ml-2 text-txt-muted"
                title={formatFullDateTime(job().endTime!)}
              >
                · {formatShortDate(job().endTime!)}
              </span>
            </Show>
          </div>
        </div>
      </Show>

      {/* Error Message */}
      <Show when={job().result && !job().result!.success}>
        <div class={`text-[11px] leading-tight text-red-400`}>
          {job().result!.error || "Transfer failed"}
        </div>
      </Show>
    </div>
  );
}
