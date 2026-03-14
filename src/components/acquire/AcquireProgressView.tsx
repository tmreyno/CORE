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

import { Component, Show, For, createSignal, createEffect, onCleanup } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineArrowLeft,
  HiOutlineCheck,
  HiOutlineCircleStack,
  HiOutlineExclamationTriangle,
  HiOutlineFingerPrint,
  HiOutlineXMark,
} from "../icons";
import type { ImagingConfig } from "./AcquireImageWizard";
import type { EwfExportProgress } from "../../api/ewfExport";
import type { L01ExportProgress } from "../../api/l01Export";
import type { SegmentVerifyResult, SegmentHashProgress } from "../../api/segmentHash";
import { APP_NAME } from "../../utils/edition";

// =============================================================================
// Types
// =============================================================================

type ImagingState = "running" | "verifying" | "completed" | "failed" | "cancelled";

interface ImagingResult {
  outputPath: string;
  md5Hash: string | null;
  sha1Hash: string | null;
  bytesWritten: number;
  filesIncluded: number;
  durationMs: number;
  format: string;
  segmentCount?: number;
  compressionRatio?: number;
  totalCompressedBytes?: number;
  outputPaths?: string[];
  segmentVerify?: SegmentVerifyResult;
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
// Acquisition Report
// =============================================================================

/**
 * Generate a companion .txt report file documenting the acquisition.
 * Written alongside the output image (e.g., MyImage.L01 → MyImage_acquisition_report.txt).
 */
async function writeAcquisitionReport(
  outputPath: string,
  result: ImagingResult,
  config: ImagingConfig,
): Promise<void> {
  try {
    // Derive report path from the first output file
    const basePath = outputPath.replace(/\.[A-Za-z0-9]+$/, "");
    const reportPath = `${basePath}_acquisition_report.txt`;

    const now = new Date();
    const lines: string[] = [];

    lines.push("=".repeat(72));
    lines.push(`${APP_NAME} ACQUISITION REPORT`);
    lines.push("=".repeat(72));
    lines.push("");
    lines.push(`Report Generated: ${now.toISOString()}`);
    lines.push(`Report File:      ${reportPath}`);
    lines.push("");

    // Case Information
    lines.push("-".repeat(72));
    lines.push("CASE INFORMATION");
    lines.push("-".repeat(72));
    if (config.caseNumber) lines.push(`Case Number:      ${config.caseNumber}`);
    if (config.evidenceNumber) lines.push(`Evidence Number:  ${config.evidenceNumber}`);
    if (config.examinerName) lines.push(`Examiner:         ${config.examinerName}`);
    if (config.description) lines.push(`Description:      ${config.description}`);
    if (config.notes) lines.push(`Notes:            ${config.notes}`);
    if (!config.caseNumber && !config.evidenceNumber && !config.examinerName) {
      lines.push("(No case information provided)");
    }
    lines.push("");

    // Acquisition Details
    lines.push("-".repeat(72));
    lines.push("ACQUISITION DETAILS");
    lines.push("-".repeat(72));
    lines.push(`Image Format:     ${result.format}`);
    lines.push(`Mode:             ${config.mode === "physical" ? "Physical (disk/device)" : "Logical (files/folders)"}`);
    lines.push(`Compression:      ${config.compression || "none"}`);
    if (config.segmentSizeMb > 0) {
      lines.push(`Segment Size:     ${config.segmentSizeMb} MB`);
    } else {
      lines.push(`Segment Size:     No splitting`);
    }
    lines.push("");

    // Source Files
    lines.push("-".repeat(72));
    lines.push("SOURCE FILES / DIRECTORIES");
    lines.push("-".repeat(72));
    for (const src of config.sources) {
      lines.push(`  ${src}`);
    }
    lines.push("");

    // Output
    lines.push("-".repeat(72));
    lines.push("OUTPUT");
    lines.push("-".repeat(72));
    lines.push(`Destination:      ${config.destination}`);
    lines.push(`Image Name:       ${config.imageName}`);
    lines.push(`Primary Output:   ${result.outputPath}`);
    if (result.outputPaths && result.outputPaths.length > 1) {
      lines.push(`Segment Files:`);
      for (const p of result.outputPaths) {
        lines.push(`  ${p}`);
      }
    }
    if (result.segmentCount && result.segmentCount > 1) {
      lines.push(`Total Segments:   ${result.segmentCount}`);
    }
    lines.push("");

    // Results
    lines.push("-".repeat(72));
    lines.push("RESULTS");
    lines.push("-".repeat(72));
    lines.push(`Files Included:   ${result.filesIncluded}`);
    lines.push(`Data Size:        ${formatBytes(result.bytesWritten)} (${result.bytesWritten} bytes)`);
    if (result.totalCompressedBytes !== undefined) {
      lines.push(`Compressed Size:  ${formatBytes(result.totalCompressedBytes)} (${result.totalCompressedBytes} bytes)`);
    }
    if (result.compressionRatio !== undefined && result.compressionRatio > 0) {
      lines.push(`Compression Ratio: ${(result.compressionRatio * 100).toFixed(1)}%`);
    }
    lines.push(`Duration:         ${formatDuration(result.durationMs)} (${result.durationMs} ms)`);
    lines.push("");

    // Hash Values
    lines.push("-".repeat(72));
    lines.push("IMAGE HASH VALUES");
    lines.push("-".repeat(72));
    if (result.md5Hash) {
      lines.push(`MD5:              ${result.md5Hash}`);
    }
    if (result.sha1Hash) {
      lines.push(`SHA-1:            ${result.sha1Hash}`);
    }
    if (!result.md5Hash && !result.sha1Hash) {
      lines.push("(No hash computed)");
    }
    lines.push("");

    // Segment Verification Results (if applicable)
    if (result.segmentVerify) {
      const sv = result.segmentVerify;
      lines.push("-".repeat(72));
      lines.push("SEGMENT VERIFICATION");
      lines.push("-".repeat(72));
      lines.push(`Algorithm:        ${sv.combinedAlgorithm || config.segmentHashAlgorithm}`);
      lines.push(`Segments Found:   ${sv.segmentCount}`);
      lines.push(`Total Bytes:      ${formatBytes(sv.totalBytes)} (${sv.totalBytes} bytes)`);
      lines.push(`Duration:         ${formatDuration(sv.durationMs)} (${sv.durationMs} ms)`);
      lines.push("");

      if (sv.combinedHash) {
        lines.push(`Combined Hash:    ${sv.combinedHash}`);
        lines.push("  (All segments hashed as a single continuous stream)");
        lines.push("");
      }

      if (sv.segmentHashes.length > 0) {
        lines.push("Individual Segment Hashes:");
        for (const sh of sv.segmentHashes) {
          lines.push(`  ${sh.segmentName.padEnd(24)} ${sh.hash}  (${formatBytes(sh.size)})`);
        }
        lines.push("");
      }
    }

    lines.push("=".repeat(72));
    lines.push("END OF REPORT");
    lines.push("=".repeat(72));
    lines.push("");

    const content = lines.join("\n");
    await invoke("write_text_file", { path: reportPath, content });
  } catch (err) {
    // Report generation is best-effort — don't fail the acquisition
    console.warn("Failed to write acquisition report:", err);
  }
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
  const [filesProcessed, setFilesProcessed] = createSignal(0);
  const [totalFiles, setTotalFiles] = createSignal(0);
  const [phase, setPhase] = createSignal("preparing");
  const [result, setResult] = createSignal<ImagingResult | null>(null);
  const [errorMessage, setErrorMessage] = createSignal("");
  const [startTime] = createSignal(Date.now());
  const [elapsed, setElapsed] = createSignal(0);
  const [verifyPercent, setVerifyPercent] = createSignal(0);
  const [verifyPhase, setVerifyPhase] = createSignal<"combined" | "individual">("combined");
  // Track whether we've received the first progress event from the backend
  const [initialized, setInitialized] = createSignal(false);

  const isPhysical = () => props.config.mode === "physical";
  const outputPath = () =>
    `${props.config.destination}/${props.config.imageName}`;

  // Elapsed time ticker
  const timer = setInterval(() => {
    const s = state();
    if (s === "running" || s === "verifying") {
      setElapsed(Date.now() - startTime());
    }
  }, 1000);
  onCleanup(() => clearInterval(timer));

  // Start the imaging operation
  createEffect(() => {
    let cancelled = false;

    const run = async () => {
      try {
        let imagingResult: ImagingResult;

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
              if (!initialized()) setInitialized(true);
              setPercent(p.percent);
              setCurrentFile(p.currentFile);
              setBytesWritten(p.bytesWritten);
              setTotalBytes(p.totalBytes);
              setPhase(p.phase);
              setFilesProcessed(p.fileIndex);
              setTotalFiles(p.totalFiles);
            },
          );
          if (cancelled) return;
          imagingResult = {
            outputPath: ewfResult.outputPath,
            md5Hash: ewfResult.md5Hash,
            sha1Hash: ewfResult.sha1Hash,
            bytesWritten: ewfResult.bytesWritten,
            filesIncluded: ewfResult.filesIncluded,
            durationMs: ewfResult.durationMs,
            format: ewfResult.format,
          };
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
              if (!initialized()) setInitialized(true);
              setPercent(p.percent);
              setCurrentFile(p.currentFile);
              setBytesWritten(p.bytesWritten);
              setTotalBytes(p.totalBytes);
              setFilesProcessed(p.filesProcessed);
              setTotalFiles(p.totalFiles);
              setPhase(p.phase);
            },
          );
          if (cancelled) return;
          imagingResult = {
            outputPath: l01Result.outputPaths[0],
            md5Hash: l01Result.md5Hash,
            sha1Hash: l01Result.sha1Hash,
            bytesWritten: l01Result.totalDataBytes,
            filesIncluded: l01Result.totalFiles,
            durationMs: l01Result.durationMs,
            format: "L01",
            segmentCount: l01Result.segmentCount,
            compressionRatio: l01Result.compressionRatio,
            totalCompressedBytes: l01Result.totalCompressedBytes,
            outputPaths: l01Result.outputPaths,
          };
        }

        // Post-acquisition segment verification
        const wantHash = props.config.hashSegments || props.config.hashSegmentsIndividually;
        if (wantHash && !cancelled) {
          setState("verifying");
          setVerifyPercent(0);
          try {
            const { hashContainerSegments, listenSegmentHashProgress } = await import("../../api/segmentHash");
            const unlisten = await listenSegmentHashProgress((p: SegmentHashProgress) => {
              setVerifyPercent(p.percent);
              setVerifyPhase(p.phase);
            });
            try {
              const verifyResult = await hashContainerSegments(
                imagingResult.outputPath,
                props.config.segmentHashAlgorithm,
                props.config.hashSegments,
                props.config.hashSegmentsIndividually,
              );
              imagingResult.segmentVerify = verifyResult;
            } finally {
              unlisten();
            }
          } catch (err) {
            console.warn("Segment verification failed:", err);
            // Don't fail the whole acquisition — verification is supplementary
          }
        }

        if (cancelled) return;
        setResult(imagingResult);
        setState("completed");
        console.log("[AcquireProgress] Image created successfully:", imagingResult.outputPath);
        await writeAcquisitionReport(imagingResult.outputPath, imagingResult, props.config);
      } catch (err) {
        if (cancelled) return;
        const msg = err instanceof Error ? err.message : String(err);
        console.error("[AcquireProgress] Imaging error:", msg);
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
          when={state() !== "running" && state() !== "verifying"}
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
              ? "Creating E01 Image\u2026"
              : "Creating L01 Image\u2026"
            : state() === "verifying"
              ? "Verifying Segments\u2026"
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
                <HiOutlineCircleStack class="w-8 h-8 text-accent" />
              </div>
              <p class="text-base font-medium text-txt">
                {initialized() ? phaseLabel(phase()) : "Scanning source files…"}
              </p>
            </div>

            {/* Progress Bar — only show after first progress event */}
            <Show when={initialized()}>
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

              {/* File counter — only show during data-writing phase */}
              <Show when={totalFiles() > 0 && phase() === "writingData"}>
                <div class="text-center text-xs text-txt-muted">
                  File {Math.min(filesProcessed() + 1, totalFiles())} of {totalFiles()}
                </div>
              </Show>

              {/* Current file */}
              <Show when={currentFile()}>
                <div class="text-xs text-txt-muted font-mono truncate px-1">
                  {currentFile()}
                </div>
              </Show>
            </Show>

            {/* Initializing indicator — show before first progress event */}
            <Show when={!initialized()}>
              <div class="text-center text-xs text-txt-muted">
                Building file list and preparing writer…
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

          {/* ---- Verifying State ---- */}
          <Show when={state() === "verifying"}>
            {/* Icon + Phase */}
            <div class="text-center">
              <div class="inline-flex items-center justify-center w-16 h-16 rounded-2xl bg-accent/10 mb-4 animate-pulse-slow">
                <HiOutlineFingerPrint class="w-8 h-8 text-accent" />
              </div>
              <p class="text-base font-medium text-txt">Verifying segments…</p>
              <p class="text-xs text-txt-muted mt-1">
                {verifyPhase() === "combined" ? "Computing combined hash" : "Hashing individual segments"}
              </p>
            </div>

            {/* Progress Bar */}
            <div class="space-y-2">
              <div class="h-3 bg-bg-secondary rounded-full overflow-hidden border border-border">
                <div
                  class="h-full bg-accent rounded-full transition-all duration-300"
                  style={{ width: `${Math.min(verifyPercent(), 100)}%` }}
                />
              </div>
              <div class="flex items-center justify-between text-xs text-txt-muted">
                <span>{verifyPercent().toFixed(1)}%</span>
                <span>Elapsed: {formatDuration(elapsed())}</span>
              </div>
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
                  <Show when={res().segmentCount && res().segmentCount! > 1}>
                    <div class="acquire-summary-row">
                      <span class="text-txt-muted">Segments</span>
                      <span class="text-txt">{res().segmentCount}</span>
                    </div>
                  </Show>
                  <div class="acquire-summary-row">
                    <span class="text-txt-muted">Size</span>
                    <span class="text-txt">{formatBytes(res().bytesWritten)}</span>
                  </div>
                  <Show when={res().totalCompressedBytes !== undefined && res().totalCompressedBytes! > 0}>
                    <div class="acquire-summary-row">
                      <span class="text-txt-muted">Compressed</span>
                      <span class="text-txt">{formatBytes(res().totalCompressedBytes!)} ({((res().compressionRatio ?? 0) * 100).toFixed(1)}%)</span>
                    </div>
                  </Show>
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
                  {/* Segment verification results */}
                  <Show when={res().segmentVerify}>
                    <div class="border-t border-border/30 pt-2 mt-2">
                      <div class="text-2xs uppercase tracking-wider text-txt-muted font-medium mb-1.5">
                        Segment Verification ({res().segmentVerify!.combinedAlgorithm || ""})
                      </div>
                      <Show when={res().segmentVerify!.combinedHash}>
                        <div class="acquire-summary-row">
                          <span class="text-txt-muted">Combined</span>
                          <span class="text-txt font-mono text-compact">{res().segmentVerify!.combinedHash}</span>
                        </div>
                      </Show>
                      <Show when={res().segmentVerify!.segmentHashes.length > 0}>
                        <For each={res().segmentVerify!.segmentHashes}>
                          {(sh) => (
                            <div class="acquire-summary-row">
                              <span class="text-txt-muted">{sh.segmentName}</span>
                              <span class="text-txt font-mono text-compact truncate">{sh.hash}</span>
                            </div>
                          )}
                        </For>
                      </Show>
                      <div class="acquire-summary-row">
                        <span class="text-txt-muted">Verify Time</span>
                        <span class="text-txt">{formatDuration(res().segmentVerify!.durationMs)}</span>
                      </div>
                    </div>
                  </Show>
                </div>

                {/* Report note */}
                <p class="text-xs text-txt-muted text-center">
                  Acquisition report saved alongside the image file.
                </p>

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
