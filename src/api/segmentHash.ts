// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { isTauri } from "../utils/platform";

// =============================================================================
// Types
// =============================================================================

export interface SegmentHashProgress {
  currentSegment: number;
  totalSegments: number;
  bytesHashed: number;
  totalBytes: number;
  percent: number;
  phase: "combined" | "individual";
}

export interface SegmentHashResult {
  segmentName: string;
  segmentPath: string;
  segmentNumber: number;
  algorithm: string;
  hash: string;
  size: number;
}

export interface SegmentVerifyResult {
  combinedHash: string | null;
  combinedAlgorithm: string | null;
  segmentHashes: SegmentHashResult[];
  segmentCount: number;
  totalBytes: number;
  durationMs: number;
}

// =============================================================================
// API
// =============================================================================

export async function hashContainerSegments(
  path: string,
  algorithm: string,
  hashCombined: boolean,
  hashIndividual: boolean,
): Promise<SegmentVerifyResult> {
  if (!isTauri) {
    void path;
    void algorithm;
    void hashCombined;
    void hashIndividual;
    throw new Error("Segment hashing is available in the desktop app.");
  }

  return invoke<SegmentVerifyResult>("hash_container_segments", {
    path,
    algorithm,
    hashCombined,
    hashIndividual,
  });
}

export async function listenSegmentHashProgress(
  callback: (progress: SegmentHashProgress) => void,
): Promise<UnlistenFn> {
  if (!isTauri) {
    void callback;
    return () => {};
  }

  return listen<SegmentHashProgress>("segment-hash-progress", (event) => {
    callback(event.payload);
  });
}
