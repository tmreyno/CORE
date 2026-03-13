// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * AcquireProgressView — Shows real-time progress during E01/L01 imaging.
 *
 * Displays: progress bar, current file, bytes written, elapsed time,
 * phase indicator, and a cancel button. On completion shows a summary
 * card with hash values and output path.
 */

import { Component, Show, createSignal, createEffect, onCleanup } from "solid-js";
import {
  HiOutlineArrowLeft,
  HiOutlineCheck,
  HiOutlineExclamationTriangle,
  HiOutlineFingerPrint,
  HiOutlineXMark,
} from "../icons";
import type { ImagingConfig } from "./AcquireImageWizard";
import type { EwfExportProgress } from "../../api/ewfExport";
import type { L01ExportProgress } from "../../api/l01Export";

// =============================================================================
// Types
// =============================================================================

type ImagingState = "running" | "completed" | "failed" | "cancelled";

interface ImagingResult {
  outputPath: string;
  md5Hash: string | null;
  sha1Hash: string | null;
  bytesWritten: number;
  filesIncluded: number;
  durationMs: number;
  format: string;
}

export interface AcquireProgressViewProps {
  config: ImagingConfig;
  onDone: () => void; // Return to dashboard
  onNewImage: () => void; // Start another image
}

// =============================================================================
// Helpers
// =============================================================================

function formatBytes(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  return `${(bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}

function formatDuration(ms: number): string {
  const secs = Math.floor(ms / 1000);
  if (secs < 60) return `${secs}s`;
  const mins = Math.floor(secs / 60);
  const remainSecs = secs % 60;
  if (mins < 60) return `${mins}m ${remainSecs}s`;
  const hrs = Math.floor(mins / 60);
  const remainMins = mins % 60;
  return `${hrs}h ${remainMins}m`;
}

function phaseLabel(phase: string): string {
  const map: Record<string, string> = {
    preparing: "Preparing…",
    writingData: "Writing data…",
    buildingTables: "Building tables…",
    writingLtree: "Building file tree…",
    computingHash: "Computing hash…",
    finalizing: "Finalizing…",
    writing: "Writing image…",
  };
  return map[phase] || phase;
}

// =============================================================================
// Component
// =============================================================================

const AcquireProgressView: Component<AcquireProgressViewProps> = (props) => {
  const [state, setState] = createSignal<ImagingState>("running");
  const [percent, setPercent] = createSignal(0);
  const [currentFile, setCurrentFile] = createSignal("");
  const [bytesWritten, setBytesWritten] = createSignal(0);
  const [totalBytes, setTotalBytes] = createSignal(0);
  const [phase, setPhase] = createSignal("preparing");
  const [result, setResult] = createSignal<ImagingResult | null>(null);
  const [errorMessage, setErrorMessage] = createSignal("");
  const [startTime] = createSignal(Date.now());
  const [elapsed, setElapsed] = createSignal(0);

  const isPhysical = () => props.config.mode === "physical";
  const outputPath = () =>
    `${props.config.destination}/${props.config.imageName}`;

  // Elapsed time ticker
  const timer = setInterval(() => {
    if (state() === "running") {
      setElapsed(Date.now() - startTime());
    }
  }, 1000);
  onCleanup(() => clearInterval(timer));

  // Start the imaging operation
  createEffect(() => {
    let cancelled = false;

    const run = async () => {
      try {
        if (isPhysical()) {
          const { createE01Image } = await import("../../api/ewfExport");
          const ewfResult = await createE01Image(
            {
              sourcePaths: props.config.sources,
              outputPath: outputPath(),
              format: props.config.ewfFormat ?? "e01",
              compression: props.config.compression,
              compressionMethod: props.config.compressionMethod,
              segmentSize:
                props.config.segmentSizeMb > 0
                  ? props.config.segmentSizeMb * 1024 * 1024
                  : 0,
              caseNumber: props.config.caseNumber || undefined,
              evidenceNumber: props.config.evidenceNumber || undefined,
              examinerName: props.config.examinerName || undefined,
              description: props.config.description || undefined,
              notes: props.config.notes || undefined,
              computeMd5: props.config.computeMd5,
              computeSha1: props.config.computeSha1,
            },
            (p: EwfExportProgress) => {
              if (cancelled) return;
              setPercent(p.percent);
              setCurrentFile(p.currentFile);
              setBytesWritten(p.bytesWritten);
              setTotalBytes(p.totalBytes);
              setPhase(p.phase);
            },
          );
          if (cancelled) return;
          setResult({
            outputPath: ewfResult.outputPath,
            md5Hash: ewfResult.md5Hash,
            sha1Hash: ewfResult.sha1Hash,
            bytesWritten: ewfResult.bytesWritten,
            filesIncluded: ewfResult.filesIncluded,
            durationMs: ewfResult.durationMs,
            format: ewfResult.format,
          });
          setState("completed");
        } else {
          const { createL01Image } = await import("../../api/l01Export");
          const l01Result = await createL01Image(
            {
              sourcePaths: props.config.sources,
              outputPath: outputPath(),
              compression: props.config.compression,
              segmentSize:
                props.config.segmentSizeMb > 0
                  ? props.config.segmentSizeMb * 1024 * 1024
                  : undefined,
              caseNumber: props.config.caseNumber || undefined,
              evidenceNumber: props.config.evidenceNumber || undefined,
              examinerName: props.config.examinerName || undefined,
              description: props.config.description || undefined,
              notes: props.config.notes || undefined,
            },
            (p: L01ExportProgress) => {
              if (cancelled) return;
              setPercent(p.percent);
              setCurrentFile(p.currentFile);
              setBytesWritten(p.bytesWritten);
              setTotalBytes(p.totalBytes);
              setPhase(p.phase);
            },
          );
          if (cancelled) return;
          setResult({
            outputPath: l01Result.outputPaths[0],
            md5Hash: l01Result.md5Hash,
            sha1Hash: l01Result.sha1Hash,
            bytesWritten: l01Result.totalDataBytes,
            filesIncluded: l01Result.totalFiles,
            durationMs: l01Result.durationMs,
            format: "L01",
          });
          setState("completed");
        }
      } catch (err) {
        if (cancelled) return;
        const msg = err instanceof Error ? err.message : String(err);
        if (msg.toLowerCase().includes("cancel")) {
          setState("cancelled");
        } else {
          setErrorMessage(msg);
          setState("failed");
        }
      }
    };

    run();

    onCleanup(() => {
      cancelled = true;
    });
  });

  // Cancel handler
  const handleCancel = async () => {
    try {
      if (isPhysical()) {
        const { cancelE01Export } = await import("../../api/ewfExport");
        await cancelE01Export(outputPath());
      } else {
        const { cancelL01Export } = await import("../../api/l01Export");
        await cancelL01Export(outputPath());
      }
    } catch {
      // Best-effort cancel
    }
    setState("cancelled");
  };

  return (
    <div class="acquire-panel">
      {/* Header */}
      <div class="acquire-panel-header">
        <Show
          when={state() !== "running"}
          fallback={<div class="w-20" />}
        >
          <button class="btn btn-ghost gap-1.5" onClick={props.onDone}>
            <HiOutlineArrowLeft class="w-4 h-4" />
            Back
          </button>
        </Show>
        <h2 class="text-lg font-medium text-txt">
          {state() === "running"
            ? isPhysical()
              ? "Creating E01 Image…"
              : "Creating L01 Image…"
            : state() === "completed"
              ? "Image Created"
              : state() === "cancelled"
                ? "Imaging Cancelled"
                : "Imaging Failed"}
        </h2>
        <div class="w-20" />
      </div>

      {/* Body */}
      <div class="acquire-panel-body overflow-y-auto">
        <div class="p-6 max-w-2xl mx-auto space-y-6">

          {/* ---- Running State ---- */}
          <Show when={state() === "running"}>
            {/* Icon + Phase */}
            <div class="text-center">
              <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-accent/10 mb-4 animate-pulse-slow">
                <HiOutlineFingerPrint class="w-8 h-8 text-accent" />
              </div>
              <p class="text-base font-medium text-txt">{phaseLabel(phase())}</p>
            </div>

            {/* Progress Bar */}
            <div class="space-y-2">
              <div class="h-3 bg-bg-secondary rounded-full overflow-hidden border border-border">
                <div
                  class="h-full bg-accent rounded-full transition-all duration-300"
                  style={{ width: `${Math.min(percent(), 100)}%` }}
                />
              </div>
              <div class="flex items-center justify-between text-xs text-txt-muted">
                <span>{percent().toFixed(1)}%</span>
                <span>{formatBytes(bytesWritten())} / {formatBytes(totalBytes())}</span>
              </div>
            </div>

            {/* Current file */}
            <Show when={currentFile()}>
              <div class="text-xs text-txt-muted font-mono truncate px-1">
                {currentFile()}
              </div>
            </Show>

            {/* Elapsed time */}
            <div class="text-center text-xs text-txt-muted">
              Elapsed: {formatDuration(elapsed())}
            </div>

            {/* Cancel button */}
            <div class="text-center">
              <button
                class="btn btn-secondary gap-1.5"
                onClick={handleCancel}
              >
                <HiOutlineXMark class="w-4 h-4" />
                Cancel
              </button>
            </div>
          </Show>

          {/* ---- Completed State ---- */}
          <Show when={state() === "completed" && result()}>
            {(res) => (
              <div class="space-y-5">
                {/* Success icon */}
                <div class="text-center">
                  <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-success/10 mb-3">
                    <HiOutlineCheck class="w-8 h-8 text-success" />
                  </div>
                  <p class="text-base font-medium text-txt">Image created successfully</p>
                  <p class="text-xs text-txt-muted mt-1">
                    Completed in {formatDuration(res().durationMs)}
                  </p>
                </div>

                {/* Summary card */}
                <div class="card p-4 space-y-2">
                  <div class="acquire-summary-row">
                    <span class="text-txt-muted">Format</span>
                    <span class="text-txt font-medium">{res().format}</span>
                  </div>
                  <div class="acquire-summary-row">
                    <span class="text-txt-muted">Output</span>
                    <span class="text-txt font-mono text-compact truncate">{res().outputPath}</span>
                  </div>
                  <div class="acquire-summary-row">
                    <span class="text-txt-muted">Size</span>
                    <span class="text-txt">{formatBytes(res().bytesWritten)}</span>
                  </div>
                  <div class="acquire-summary-row">
                    <span class="text-txt-muted">Files</span>
                    <span class="text-txt">{res().filesIncluded}</span>
                  </div>
                  <Show when={res().md5Hash}>
                    <div class="acquire-summary-row">
                      <span class="text-txt-muted">MD5</span>
                      <span class="text-txt font-mono text-compact">{res().md5Hash}</span>
                    </div>
                  </Show>
                  <Show when={res().sha1Hash}>
                    <div class="acquire-summary-row">
                      <span class="text-txt-muted">SHA-1</span>
                      <span class="text-txt font-mono text-compact">{res().sha1Hash}</span>
                    </div>
                  </Show>
                </div>

                {/* Action buttons */}
                <div class="flex items-center justify-center gap-3">
                  <button class="btn btn-secondary gap-1.5" onClick={props.onDone}>
                    <HiOutlineArrowLeft class="w-4 h-4" />
                    Back to Dashboard
                  </button>
                  <button class="btn btn-primary gap-1.5" onClick={props.onNewImage}>
                    Create Another Image
                  </button>
                </div>
              </div>
            )}
          </Show>

          {/* ---- Failed State ---- */}
          <Show when={state() === "failed"}>
            <div class="space-y-5">
              <div class="text-center">
                <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-error/10 mb-3">
                  <HiOutlineExclamationTriangle class="w-8 h-8 text-error" />
                </div>
                <p class="text-base font-medium text-txt">Imaging failed</p>
                <p class="text-sm text-error mt-2 max-w-md mx-auto">{errorMessage()}</p>
              </div>
              <div class="flex items-center justify-center gap-3">
                <button class="btn btn-secondary gap-1.5" onClick={props.onDone}>
                  <HiOutlineArrowLeft class="w-4 h-4" />
                  Back to Dashboard
                </button>
                <button class="btn btn-primary gap-1.5" onClick={props.onNewImage}>
                  Try Again
                </button>
              </div>
            </div>
          </Show>

          {/* ---- Cancelled State ---- */}
          <Show when={state() === "cancelled"}>
            <div class="space-y-5">
              <div class="text-center">
                <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-warning/10 mb-3">
                  <HiOutlineXMark class="w-8 h-8 text-warning" />
                </div>
                <p class="text-base font-medium text-txt">Imaging cancelled</p>
                <p class="text-xs text-txt-muted mt-1">
                  The operation was stopped after {formatDuration(elapsed())}
                </p>
              </div>
              <div class="flex items-center justify-center gap-3">
                <button class="btn btn-secondary gap-1.5" onClick={props.onDone}>
                  <HiOutlineArrowLeft class="w-4 h-4" />
                  Back to Dashboard
                </button>
                <button class="btn btn-primary gap-1.5" onClick={props.onNewImage}>
                  Start Over
                </button>
              </div>
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
};

export default AcquireProgressView;
