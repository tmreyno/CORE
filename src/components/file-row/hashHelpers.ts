// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { ContainerInfo, HashHistoryEntry } from "../../types";
import type { FileStatus, FileHashInfo } from "../../hooks";
import { formatBytes } from "../../utils";
import type { HashState } from "./types";

/** Check if container is incomplete (missing segments) */
export function isContainerIncomplete(fileInfo: ContainerInfo | undefined): boolean {
  return (fileInfo?.ad1?.missing_segments?.length ?? 0) > 0;
}

/** Get total container size (all segments combined) when available */
export function getTotalContainerSize(fileInfo: ContainerInfo | undefined): number | null {
  if (fileInfo?.ad1?.total_size) return fileInfo.ad1.total_size;
  if (fileInfo?.e01?.total_size) return fileInfo.e01.total_size;
  if (fileInfo?.l01?.total_size) return fileInfo.l01.total_size;
  if (fileInfo?.raw?.total_size) return fileInfo.raw.total_size;
  if (fileInfo?.archive?.total_size) return fileInfo.archive.total_size;
  return null;
}

/** Build display size label */
export function buildSizeLabel(
  fileSize: number,
  totalSize: number | null,
  segmentCount: number | undefined
): string {
  const hasMulti = (segmentCount ?? 1) > 1;
  if (totalSize && hasMulti) {
    return `Total: ${formatBytes(totalSize)} (${segmentCount} segments, first segment: ${formatBytes(fileSize)})`;
  }
  return formatBytes(totalSize ?? fileSize);
}

/** Get stored hash count from all sources */
export function getStoredHashCount(fileInfo: ContainerInfo | undefined): number {
  return (fileInfo?.e01?.stored_hashes?.length ?? 0) + (fileInfo?.companion_log?.stored_hashes?.length ?? 0);
}

/** Total hash count (stored + computed + history) */
export function getTotalHashCount(
  fileInfo: ContainerInfo | undefined,
  fileHash: FileHashInfo | undefined,
  hashHistory: HashHistoryEntry[]
): number {
  return getStoredHashCount(fileInfo) + (fileHash ? 1 : 0) + (hashHistory?.length ?? 0);
}

/** Check if any hash matches exist (stored vs history, or history vs history) */
export function hasVerifiedMatch(
  fileInfo: ContainerInfo | undefined,
  hashHistory: HashHistoryEntry[]
): boolean {
  const storedHashes = [
    ...(fileInfo?.e01?.stored_hashes ?? []),
    ...(fileInfo?.ufed?.stored_hashes ?? []),
    ...(fileInfo?.companion_log?.stored_hashes ?? []),
  ];
  const history = hashHistory ?? [];

  for (const stored of storedHashes) {
    const match = history.find(
      (h) =>
        h.algorithm.toLowerCase() === stored.algorithm.toLowerCase() &&
        h.hash.toLowerCase() === stored.hash.toLowerCase()
    );
    if (match) return true;
  }

  for (let i = 0; i < history.length; i++) {
    for (let j = i + 1; j < history.length; j++) {
      if (
        history[i].algorithm.toLowerCase() === history[j].algorithm.toLowerCase() &&
        history[i].hash.toLowerCase() === history[j].hash.toLowerCase()
      ) {
        return true;
      }
    }
  }

  return false;
}

/** Determine hash state */
export function getHashState(
  fileInfo: ContainerInfo | undefined,
  fileHash: FileHashInfo | undefined,
  hashHistory: HashHistoryEntry[]
): HashState {
  if (isContainerIncomplete(fileInfo)) return "incomplete";
  if (fileHash?.verified === true) return "verified";
  if (fileHash?.verified === false) return "failed";
  if (fileHash) return "computed";
  if (hasVerifiedMatch(fileInfo, hashHistory)) return "verified";
  if (getStoredHashCount(fileInfo) > 0) return "stored";
  if ((hashHistory?.length ?? 0) > 0) return "computed";
  return "none";
}

/** Check if currently hashing */
export function isCurrentlyHashing(fileStatus: FileStatus | undefined, fileHash: FileHashInfo | undefined): boolean {
  return fileStatus?.status === "hashing" && !fileHash && (fileStatus?.progress ?? 0) < 95;
}

/** Check if completing (>= 95% but no result yet) */
export function isCurrentlyCompleting(fileStatus: FileStatus | undefined, fileHash: FileHashInfo | undefined): boolean {
  return fileStatus?.status === "hashing" && (fileStatus?.progress ?? 0) >= 95 && !fileHash;
}

/** Format chunk count for display */
export function formatChunks(count: number): string {
  if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M`;
  if (count >= 1000) return `${(count / 1000).toFixed(0)}k`;
  return count.toString();
}
