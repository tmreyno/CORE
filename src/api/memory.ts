// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Live RAM capture API.
 *
 * Wraps the `memory_capture` Tauri commands for:
 * - Querying system memory info and capture support
 * - Performing live memory dumps (Linux/Windows)
 * - Cancelling in-progress captures
 * - Listening for capture progress events
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isTauri } from "../utils/platform";

// =============================================================================
// Types
// =============================================================================

/** Information about system memory and capture capability. */
export interface MemoryCaptureInfo {
  /** Total physical memory in bytes */
  totalMemoryBytes: number;
  /** Available (free) memory in bytes */
  availableMemoryBytes: number;
  /** OS platform: "linux", "windows", "macos" */
  platform: string;
  /** Whether memory capture is supported on this platform */
  captureSupported: boolean;
  /** Description of capture method (e.g. "/proc/kcore", "WinPmem") */
  captureMethod: string;
  /** Whether elevated privileges are required */
  requiresElevation: boolean;
  /** Platform-specific instructions for gaining elevation */
  elevationInstructions: string;
  /** Reason capture is not supported, if any */
  unsupportedReason?: string;
}

/** Progress event emitted during memory capture. */
export interface MemoryCaptureProgress {
  /** Bytes captured so far */
  bytesCaptured: number;
  /** Total bytes to capture */
  totalBytes: number;
  /** Progress percentage (0–100) */
  percent: number;
  /** Current phase: "preparing", "capturing", "hashing", "complete" */
  phase: string;
}

/** Result of a completed memory capture. */
export interface MemoryCaptureResult {
  /** Path to the output .mem file */
  outputPath: string;
  /** Total bytes captured */
  bytesCaptured: number;
  /** Total expected bytes */
  totalBytes: number;
  /** Duration of capture in seconds */
  durationSecs: number;
  /** MD5 hash of the capture file, if computed */
  hashMd5?: string;
  /** SHA-256 hash of the capture file, if computed */
  hashSha256?: string;
}

// =============================================================================
// API Functions
// =============================================================================

/**
 * Query system memory information and capture support.
 *
 * Returns memory sizes, platform, whether capture is supported,
 * and what elevation is required.
 */
export async function getMemoryCaptureInfo(): Promise<MemoryCaptureInfo> {
  if (!isTauri) {
    return {
      totalMemoryBytes: 0,
      availableMemoryBytes: 0,
      platform: "browser",
      captureSupported: false,
      captureMethod: "unavailable",
      requiresElevation: false,
      elevationInstructions: "",
      unsupportedReason: "Memory capture is available in the desktop app.",
    };
  }

  return invoke<MemoryCaptureInfo>("memory_capture_info");
}

/**
 * Perform a live memory capture.
 *
 * On Linux, reads `/proc/kcore` using ELF headers + `/proc/iomem` ranges.
 * On Windows, invokes WinPmem to capture physical memory.
 * Requires elevated privileges on both platforms.
 *
 * @param outputPath - Destination file path for the .mem dump
 * @param computeHashes - Whether to compute MD5 + SHA-256 after capture
 * @returns Capture result with path, size, duration, and optional hashes
 */
export async function captureMemory(
  outputPath: string,
  computeHashes: boolean
): Promise<MemoryCaptureResult> {
  if (!isTauri) {
    void outputPath;
    void computeHashes;
    throw new Error("Memory capture is available in the desktop app.");
  }

  return invoke<MemoryCaptureResult>("memory_capture", {
    outputPath,
    computeHashes,
  });
}

/**
 * Cancel an in-progress memory capture.
 *
 * Sets the cancel flag; the capture loop will stop at the next chunk boundary.
 */
export async function cancelMemoryCapture(): Promise<void> {
  if (!isTauri) {
    return;
  }

  return invoke<void>("memory_capture_cancel");
}

/**
 * Listen for memory capture progress events.
 *
 * @param callback - Called with progress updates during capture
 * @returns Unlisten function to stop receiving events
 */
export async function listenMemoryCaptureProgress(
  callback: (progress: MemoryCaptureProgress) => void
): Promise<UnlistenFn> {
  if (!isTauri) {
    void callback;
    return () => {};
  }

  return listen<MemoryCaptureProgress>(
    "memory-capture-progress",
    (event) => {
      callback(event.payload);
    }
  );
}
