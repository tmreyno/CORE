// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// MemoryMode - Live RAM capture UI

import { Show, onMount, createMemo } from "solid-js";
import type { Accessor, Setter } from "solid-js";
import {
  HiOutlineCpuChip,
  HiOutlineFingerPrint,
  HiOutlineExclamationTriangle,
  HiOutlineCheckCircle,
  HiOutlineInformationCircle,
  HiOutlineShieldExclamation,
} from "../icons";
import type { MemoryCaptureInfo, MemoryCaptureProgress, MemoryCaptureResult } from "../../api/memory";

// --- Props ---

export interface MemoryModeProps {
  memoryInfo: Accessor<MemoryCaptureInfo | null>;
  memoryInfoLoading: Accessor<boolean>;
  memoryComputeHashes: Accessor<boolean>;
  setMemoryComputeHashes: Setter<boolean>;
  memoryOutputName: Accessor<string>;
  setMemoryOutputName: Setter<string>;
  memoryProgress: Accessor<MemoryCaptureProgress | null>;
  memoryResult: Accessor<MemoryCaptureResult | null>;
  onLoadInfo: () => void;
}

// --- Helpers ---

function formatSize(bytes: number): string {
  if (bytes >= 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }
  if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
  return `${(bytes / 1024).toFixed(1)} KB`;
}

// --- Component ---

export function MemoryMode(props: MemoryModeProps) {
  onMount(() => {
    if (!props.memoryInfo()) {
      props.onLoadInfo();
    }
  });

  const supported = createMemo(() => props.memoryInfo()?.captureSupported ?? false);
  const progress = createMemo(() => props.memoryProgress());
  const result = createMemo(() => props.memoryResult());

  return (
    <div class="space-y-3">
      {/* System Memory Info */}
      <div class="card">
        <div class="flex items-center gap-2 mb-3">
          <HiOutlineCpuChip class="w-icon-sm h-icon-sm text-accent" />
          <span class="text-sm font-medium text-txt">System Memory</span>
        </div>

        <Show when={props.memoryInfoLoading()}>
          <div class="text-xs text-txt-muted animate-pulse-slow">Detecting system memory...</div>
        </Show>

        <Show when={!props.memoryInfoLoading() && props.memoryInfo()}>
          {(_info) => {
            const info = () => props.memoryInfo()!;
            return (
              <div class="space-y-2">
                <div class="grid grid-cols-2 gap-2">
                  <div class="stat-box">
                    <div class="text-txt-muted text-xs">Total RAM</div>
                    <div class="text-lg font-semibold text-txt">{formatSize(info().totalMemoryBytes)}</div>
                  </div>
                  <div class="stat-box">
                    <div class="text-txt-muted text-xs">Available</div>
                    <div class="text-lg font-semibold text-txt">{formatSize(info().availableMemoryBytes)}</div>
                  </div>
                </div>

                <div class="flex items-center gap-2 text-xs">
                  <span class="text-txt-muted">Platform:</span>
                  <span class="text-txt">{info().platform}</span>
                  <span class="text-txt-muted">•</span>
                  <span class="text-txt-muted">Method:</span>
                  <span class="text-txt">{info().captureMethod}</span>
                </div>

                {/* Support Status */}
                <Show when={info().captureSupported}>
                  <div class="flex items-center gap-2 p-2 rounded-lg bg-green-500/10 text-green-400">
                    <HiOutlineCheckCircle class="w-4 h-4 shrink-0" />
                    <span class="text-xs">Memory capture is supported on this system</span>
                  </div>
                </Show>

                <Show when={!info().captureSupported}>
                  <div class="flex items-center gap-2 p-2 rounded-lg bg-red-500/10 text-red-400">
                    <HiOutlineExclamationTriangle class="w-4 h-4 shrink-0" />
                    <span class="text-xs">{info().unsupportedReason || "Memory capture is not supported"}</span>
                  </div>
                </Show>

                {/* Elevation Warning */}
                <Show when={info().captureSupported && info().requiresElevation}>
                  <div class="flex items-start gap-2 p-2 rounded-lg bg-amber-500/10 text-amber-400">
                    <HiOutlineShieldExclamation class="w-4 h-4 shrink-0 mt-0.5" />
                    <div class="text-xs space-y-1">
                      <div class="font-medium">Elevated privileges required</div>
                      <div class="text-amber-400/80">{info().elevationInstructions}</div>
                    </div>
                  </div>
                </Show>
              </div>
            );
          }}
        </Show>
      </div>

      {/* Capture Options */}
      <Show when={supported()}>
        <div class="card">
          <div class="flex items-center gap-2 mb-3">
            <HiOutlineInformationCircle class="w-icon-sm h-icon-sm text-txt-muted" />
            <span class="text-sm font-medium text-txt">Capture Options</span>
          </div>

          <div class="space-y-3">
            {/* Output filename */}
            <div class="form-group">
              <label class="label">Output Filename</label>
              <div class="flex items-center gap-2">
                <input
                  class="input-sm flex-1"
                  value={props.memoryOutputName()}
                  onInput={(e) => props.setMemoryOutputName(e.currentTarget.value)}
                  placeholder="memory_dump"
                />
                <span class="text-xs text-txt-muted">.mem</span>
              </div>
            </div>

            {/* Compute Hashes */}
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={props.memoryComputeHashes()}
                onChange={(e) => props.setMemoryComputeHashes(e.currentTarget.checked)}
                class="rounded border-border"
              />
              <HiOutlineFingerPrint class="w-4 h-4 text-txt-muted" />
              <span class="text-sm text-txt">Compute MD5 + SHA-256 hashes</span>
            </label>

            {/* Info note */}
            <div class="text-xs text-txt-muted p-2 bg-bg-secondary rounded">
              Output format is raw memory dump (.mem), compatible with Volatility, Rekall, and other memory analysis frameworks.
              The destination folder must have at least {formatSize(props.memoryInfo()?.totalMemoryBytes ?? 0)} of free space.
            </div>
          </div>
        </div>
      </Show>

      {/* Progress */}
      <Show when={progress()}>
        {(_p) => {
          const p = () => props.memoryProgress()!;
          return (
            <div class="card">
              <div class="flex items-center justify-between mb-2">
                <span class="text-xs font-medium text-txt">{p().phase === "capturing" ? "Capturing memory..." : p().phase === "hashing" ? "Computing hashes..." : p().phase}</span>
                <span class="text-xs text-txt-muted">{p().percent.toFixed(1)}%</span>
              </div>
              <div class="w-full h-2 bg-bg-secondary rounded-full overflow-hidden">
                <div
                  class="h-full bg-accent rounded-full transition-all duration-200"
                  style={{ width: `${Math.min(p().percent, 100)}%` }}
                />
              </div>
              <div class="flex justify-between mt-1 text-2xs text-txt-muted">
                <span>{formatSize(p().bytesCaptured)}</span>
                <span>{formatSize(p().totalBytes)}</span>
              </div>
            </div>
          );
        }}
      </Show>

      {/* Result */}
      <Show when={result()}>
        {(_r) => {
          const r = () => props.memoryResult()!;
          return (
            <div class="card border border-green-500/30">
              <div class="flex items-center gap-2 mb-2">
                <HiOutlineCheckCircle class="w-icon-sm h-icon-sm text-success" />
                <span class="text-sm font-medium text-success">Capture Complete</span>
              </div>
              <div class="space-y-1 text-xs">
                <div class="flex items-baseline gap-2">
                  <span class="text-txt-muted w-20">Output</span>
                  <span class="text-txt font-mono text-compact truncate">{r().outputPath}</span>
                </div>
                <div class="flex items-baseline gap-2">
                  <span class="text-txt-muted w-20">Size</span>
                  <span class="text-txt">{formatSize(r().bytesCaptured)}</span>
                </div>
                <div class="flex items-baseline gap-2">
                  <span class="text-txt-muted w-20">Duration</span>
                  <span class="text-txt">
                    {r().durationSecs < 60
                      ? `${r().durationSecs.toFixed(1)}s`
                      : `${Math.floor(r().durationSecs / 60)}m ${Math.floor(r().durationSecs % 60)}s`}
                  </span>
                </div>
                <Show when={r().hashMd5}>
                  <div class="flex items-baseline gap-2">
                    <span class="text-txt-muted w-20">MD5</span>
                    <span class="text-txt font-mono text-compact truncate">{r().hashMd5}</span>
                  </div>
                </Show>
                <Show when={r().hashSha256}>
                  <div class="flex items-baseline gap-2">
                    <span class="text-txt-muted w-20">SHA-256</span>
                    <span class="text-txt font-mono text-compact truncate">{r().hashSha256}</span>
                  </div>
                </Show>
              </div>
            </div>
          );
        }}
      </Show>
    </div>
  );
}
