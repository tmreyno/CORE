// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { ContainerInfo, HashHistoryEntry } from "../../types";
import type { FileStatus, FileHashInfo } from "../../hooks";
import { compareHashes } from "../../hooks/hashUtils";
import type { HashState } from "./types";

/** Calculate hash state from props */
export function getHashState(props: {
  fileHash?: FileHashInfo | null;
  containerInfo?: ContainerInfo | null;
  hashHistory?: HashHistoryEntry[];
}): HashState {
  const { fileHash, containerInfo, hashHistory = [] } = props;

  // Check for incomplete container first
  if ((containerInfo?.ad1?.missing_segments?.length ?? 0) > 0) {
    return "incomplete";
  }

  const hash = fileHash;
  if (hash?.verified === true) return "verified";
  if (hash?.verified === false) return "failed";
  if (hash) return "computed";

  // Check for verified matches even without current fileHash
  if (hasVerifiedMatch(containerInfo, hashHistory)) return "verified";

  const storedCount = getStoredHashCount(containerInfo);
  if (storedCount > 0) return "stored";
  if (hashHistory.length > 0) return "computed"; // Has history but no stored to verify against

  return "none";
}

/** Check if any stored hash matches history */
export function hasVerifiedMatch(
  containerInfo?: ContainerInfo | null,
  hashHistory?: HashHistoryEntry[],
): boolean {
  const storedHashes = [
    ...(containerInfo?.e01?.stored_hashes ?? []),
    ...(containerInfo?.l01?.stored_hashes ?? []),
    ...(containerInfo?.ufed?.stored_hashes ?? []),
    ...(containerInfo?.companion_log?.stored_hashes ?? []),
    ...(containerInfo?.ad1?.companion_log?.md5_hash ? [{ algorithm: "MD5", hash: containerInfo.ad1.companion_log.md5_hash }] : []),
    ...(containerInfo?.ad1?.companion_log?.sha1_hash ? [{ algorithm: "SHA-1", hash: containerInfo.ad1.companion_log.sha1_hash }] : []),
    ...(containerInfo?.ad1?.companion_log?.sha256_hash ? [{ algorithm: "SHA-256", hash: containerInfo.ad1.companion_log.sha256_hash }] : []),
  ];
  const history = hashHistory ?? [];

  // Check if any stored hash matches any history hash using algorithm-aware comparison
  for (const stored of storedHashes) {
    const match = history.find((h) =>
      compareHashes(h.hash, stored.hash, h.algorithm, stored.algorithm),
    );
    if (match) return true;
  }

  return false;
}

/** Count stored hashes */
export function getStoredHashCount(containerInfo?: ContainerInfo | null): number {
  return (
    (containerInfo?.e01?.stored_hashes?.length ?? 0) +
    (containerInfo?.l01?.stored_hashes?.length ?? 0) +
    (containerInfo?.ufed?.stored_hashes?.length ?? 0) +
    (containerInfo?.companion_log?.stored_hashes?.length ?? 0) +
    (containerInfo?.ad1?.companion_log?.md5_hash ? 1 : 0) +
    (containerInfo?.ad1?.companion_log?.sha1_hash ? 1 : 0) +
    (containerInfo?.ad1?.companion_log?.sha256_hash ? 1 : 0)
  );
}

/** Get total hash count */
export function getTotalHashCount(props: {
  fileHash?: FileHashInfo | null;
  containerInfo?: ContainerInfo | null;
  hashHistory?: HashHistoryEntry[];
}): number {
  const storedCount = getStoredHashCount(props.containerInfo);
  const historyCount = props.hashHistory?.length ?? 0;
  return storedCount + (props.fileHash ? 1 : 0) + historyCount;
}

/** Check if currently hashing */
export function isHashing(fileStatus?: FileStatus | null, fileHash?: FileHashInfo | null): boolean {
  return fileStatus?.status === "hashing" && !fileHash && (fileStatus?.progress ?? 0) < 95;
}

/** Check if hash is completing (95%+ but not done) */
export function isCompleting(fileStatus?: FileStatus | null, fileHash?: FileHashInfo | null): boolean {
  return fileStatus?.status === "hashing" && (fileStatus?.progress ?? 0) >= 95 && !fileHash;
}

/** Format chunk count for display */
export function formatChunks(count: number): string {
  if (count >= 1000000) return `${(count / 1000000).toFixed(1)}M`;
  if (count >= 1000) return `${(count / 1000).toFixed(0)}k`;
  return count.toString();
}
