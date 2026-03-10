// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Hash Utilities
 * 
 * Shared utility functions for hash management across different container types.
 * Consolidates duplicate logic for stored hash extraction, progress monitoring,
 * and type-specific hashing operations.
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import { logger } from "../utils/logger";
const log = logger.scope('hashUtils');
import type {
  StoredHashEntry,
  HashProgressEvent,
  HashAlgorithmName,
} from "../types/hash";
import { normalizeAlgorithm, algorithmsMatch } from "../types/hash";
import type { ContainerInfo } from "../types";
import { getBasename } from "../utils";

// =============================================================================
// Stored Hash Extraction
// =============================================================================

/**
 * Extract stored hashes from EWF containers (E01/Ex01/L01/Lx01).
 * Uses ewf_info command to retrieve MD5/SHA1/SHA256 hashes from segment metadata.
 *
 * @param path - Path to EWF container
 * @returns Array of stored hash entries
 */
export async function extractEwfStoredHashes(
  path: string
): Promise<StoredHashEntry[]> {
  try {
    const info = await invoke<{
      md5?: string;
      sha1?: string;
      sha256?: string;
    }>("ewf_info", { path });

    const hashes: StoredHashEntry[] = [];

    if (info.md5) {
      hashes.push({
        algorithm: "MD5",
        hash: info.md5.toUpperCase(),
        source: "container",
      });
    }
    if (info.sha1) {
      hashes.push({
        algorithm: "SHA-1",
        hash: info.sha1.toUpperCase(),
        source: "container",
      });
    }
    if (info.sha256) {
      hashes.push({
        algorithm: "SHA-256",
        hash: info.sha256.toUpperCase(),
        source: "container",
      });
    }

    return hashes;
  } catch (error) {
    log.error("Error:", error);
    return [];
  }
}

/**
 * Extract stored hashes from AD1 containers.
 * Uses logical_info command to retrieve stored hash metadata.
 * 
 * @param path - Path to AD1 container
 * @returns Array of stored hash entries
 */
export async function extractAd1StoredHashes(
  path: string
): Promise<StoredHashEntry[]> {
  try {
    const info = await invoke<{
      stored_hash?: {
        algorithm: string;
        hash: string;
      };
    }>("logical_info", { path });

    const hashes: StoredHashEntry[] = [];

    if (info.stored_hash?.algorithm && info.stored_hash?.hash) {
      hashes.push({
        algorithm: normalizeAlgorithm(info.stored_hash.algorithm),
        hash: info.stored_hash.hash.toUpperCase(),
        source: "container",
      });
    }

    return hashes;
  } catch (error) {
    log.error("Error:", error);
    return [];
  }
}

/**
 * Extract stored hashes from UFED containers.
 * Uses ufed_info command to retrieve metadata hashes.
 * 
 * @param path - Path to UFED container
 * @returns Array of stored hash entries
 */
export async function extractUfedStoredHashes(
  path: string
): Promise<StoredHashEntry[]> {
  try {
    const info = await invoke<{
      stored_hash?: {
        algorithm: string;
        hash: string;
      };
    }>("ufed_info", { path });

    const hashes: StoredHashEntry[] = [];

    if (info.stored_hash?.algorithm && info.stored_hash?.hash) {
      hashes.push({
        algorithm: normalizeAlgorithm(info.stored_hash.algorithm),
        hash: info.stored_hash.hash.toUpperCase(),
        source: "container",
      });
    }

    return hashes;
  } catch (error) {
    log.error("Error:", error);
    return [];
  }
}

/**
 * Unified stored hash extractor for all container types.
 * Routes to appropriate extraction function based on file extension.
 * 
 * @param path - Path to container file
 * @param extension - File extension (lowercase)
 * @returns Array of stored hash entries
 */
export async function extractStoredHashes(
  path: string,
  extension: string
): Promise<StoredHashEntry[]> {
  const ext = extension.toLowerCase();

  switch (ext) {
    case "e01":
    case "ex01":
    case "l01":
    case "lx01":
      return extractEwfStoredHashes(path);

    case "ad1":
      return extractAd1StoredHashes(path);

    case "ufed":
    case "ufd":
      return extractUfedStoredHashes(path);

    default:
      log.debug(`No stored hashes for extension: ${ext}`);
      return [];
  }
}

// =============================================================================
// Progress Event Handling
// =============================================================================

/**
 * Set up progress event listener for hash operations.
 * Handles progress updates and provides cleanup function.
 * 
 * @param eventName - Name of the progress event (e.g., "hash-progress")
 * @param onProgress - Callback for progress updates
 * @returns Cleanup function to unlisten
 */
export async function setupProgressListener(
  eventName: string,
  onProgress: (event: HashProgressEvent) => void
): Promise<UnlistenFn> {
  const unlisten = await listen<HashProgressEvent>(eventName, (event) => {
    onProgress(event.payload);
  });

  return unlisten;
}

// =============================================================================
// Hash Computation by Container Type
// =============================================================================

/**
 * Hash EWF container (E01/Ex01/L01/Lx01) using EWF-aware hashing.
 *
 * @param path - Path to EWF file
 * @param algorithm - Hash algorithm to use
 * @returns Computed hash string (uppercase hex)
 */
async function hashEwfContainer(
  path: string,
  algorithm: HashAlgorithmName
): Promise<string> {
  const result = await invoke<string>("e01_v3_verify", {
    inputPath: path,
    algorithm,
  });
  return result.toUpperCase();
}

/**
 * Hash AD1 container using AccessData-aware segment hashing.
 *
 * @param path - Path to AD1 file
 * @param algorithm - Hash algorithm to use
 * @returns Computed hash string (uppercase hex)
 */
async function hashAd1Container(
  path: string,
  algorithm: HashAlgorithmName
): Promise<string> {
  const result = await invoke<string>("ad1_hash_segments", {
    inputPath: path,
    algorithm,
  });
  return result.toUpperCase();
}

/**
 * Hash any file directly (raw bytes). Used for Raw/DD/UFED/archives.
 *
 * @param path - Path to file
 * @param algorithm - Hash algorithm to use
 * @returns Computed hash string (uppercase hex)
 */
async function hashFileDirectly(
  path: string,
  algorithm: HashAlgorithmName
): Promise<string> {
  const result = await invoke<string>("raw_verify", {
    inputPath: path,
    algorithm,
  });
  return result.toUpperCase();
}

/**
 * Unified hash computation for all container types.
 * Routes to appropriate hashing function based on file extension.
 * 
 * @param path - Path to container file
 * @param extension - File extension (lowercase)
 * @param algorithm - Hash algorithm to use
 * @returns Computed hash string (uppercase hex)
 */
export async function hashContainer(
  path: string,
  extension: string,
  algorithm: HashAlgorithmName
): Promise<string> {
  const ext = extension.toLowerCase();

  switch (ext) {
    case "e01":
    case "ex01":
    case "l01":
    case "lx01":
      return hashEwfContainer(path, algorithm);

    case "ad1":
      return hashAd1Container(path, algorithm);

    default:
      return hashFileDirectly(path, algorithm);
  }
}

// =============================================================================
// Stored Hash Collection from ContainerInfo
// =============================================================================

/**
 * Collect ALL stored hashes from a ContainerInfo for a given file path.
 *
 * Gathers hashes from every source:
 *   - E01/L01 segment metadata (stored_hashes array)
 *   - Companion log hashes
 *   - AD1 companion log (md5_hash, sha1_hash, sha256_hash)
 *   - UFED stored hashes (matched by filename)
 *
 * This replaces the ~40-line stored hash collection block that was
 * previously duplicated in both hashSingleFile and hashSelectedFiles.
 *
 * @param filePath - Full path to the evidence file
 * @param info - ContainerInfo for the file (may be undefined)
 * @returns Unified array of StoredHashEntry
 */
export function collectStoredHashes(
  filePath: string,
  info: ContainerInfo | undefined,
): StoredHashEntry[] {
  const stored: StoredHashEntry[] = [];

  // E01 stored hashes
  if (info?.e01?.stored_hashes) {
    for (const sh of info.e01.stored_hashes) {
      stored.push({ algorithm: sh.algorithm, hash: sh.hash, source: "container" });
    }
  }

  // L01 stored hashes
  if (info?.l01?.stored_hashes) {
    for (const sh of info.l01.stored_hashes) {
      stored.push({ algorithm: sh.algorithm, hash: sh.hash, source: "container" });
    }
  }

  // Companion log stored hashes
  if (info?.companion_log?.stored_hashes) {
    for (const sh of info.companion_log.stored_hashes) {
      stored.push({ algorithm: sh.algorithm, hash: sh.hash, source: "companion" });
    }
  }

  // AD1 companion log
  if (info?.ad1?.companion_log) {
    const adLog = info.ad1.companion_log;
    if (adLog.md5_hash) stored.push({ algorithm: "MD5", hash: adLog.md5_hash, source: "companion" });
    if (adLog.sha1_hash) stored.push({ algorithm: "SHA-1", hash: adLog.sha1_hash, source: "companion" });
    if (adLog.sha256_hash) stored.push({ algorithm: "SHA-256", hash: adLog.sha256_hash, source: "companion" });
  }

  // UFED stored hashes (match by filename)
  if (info?.ufed?.stored_hashes) {
    const fileName = getBasename(filePath);
    const ufedMatch = info.ufed.stored_hashes.find(
      (sh) => sh.filename.toLowerCase() === fileName.toLowerCase(),
    );
    if (ufedMatch) {
      stored.push({
        algorithm: ufedMatch.algorithm,
        hash: ufedMatch.hash,
        source: "container",
        filename: ufedMatch.filename,
      });
    }
  }

  return stored;
}

// =============================================================================
// Verification Helpers
// =============================================================================

/**
 * Determine verification status by comparing a computed hash against
 * stored hashes and hash history.
 *
 * @param computedHash - Newly computed hash value
 * @param algorithm - Algorithm used for computation
 * @param storedHashes - Stored hashes extracted with collectStoredHashes()
 * @param history - Previous hash history entries for the file
 * @returns Object with verified status, verifiedAgainst hash, and matching entry
 */
export function determineVerification(
  computedHash: string,
  algorithm: string,
  storedHashes: StoredHashEntry[],
  history: { source: string; hash: string; algorithm: string }[],
): { verified: boolean | null; verifiedAgainst: string | undefined } {
  // Check against stored hashes first (container/companion)
  const matchingStored = findMatchingStoredHash(
    computedHash,
    algorithm as HashAlgorithmName,
    storedHashes,
  );

  if (matchingStored) {
    return {
      verified: compareHashes(computedHash, matchingStored.hash, algorithm, matchingStored.algorithm),
      verifiedAgainst: matchingStored.hash,
    };
  }

  // Check hash history for self-verification (previously computed matches)
  const matchingHistory = history.find(
    (h) => algorithmsMatch(h.algorithm, algorithm) && h.hash.toLowerCase() === computedHash.toLowerCase(),
  );

  if (matchingHistory) {
    return { verified: true, verifiedAgainst: matchingHistory.hash };
  }

  return { verified: null, verifiedAgainst: undefined };
}

// =============================================================================
// Hash Comparison Utilities
// =============================================================================

/**
 * Compare two hash values accounting for algorithm naming variations.
 * Case-insensitive comparison with algorithm normalization.
 * 
 * @param hash1 - First hash value
 * @param hash2 - Second hash value
 * @param algorithm1 - Algorithm for first hash
 * @param algorithm2 - Algorithm for second hash
 * @returns true if hashes match (same algorithm and value)
 */
export function compareHashes(
  hash1: string,
  hash2: string,
  algorithm1: string,
  algorithm2: string
): boolean {
  // Normalize algorithms for comparison
  if (!algorithmsMatch(algorithm1, algorithm2)) {
    return false;
  }

  // Case-insensitive hash comparison
  return hash1.toLowerCase() === hash2.toLowerCase();
}

/**
 * Find matching stored hash for a computed hash.
 * Returns the first stored hash that matches both algorithm and value.
 * 
 * @param computedHash - Computed hash value
 * @param computedAlgorithm - Algorithm used for computed hash
 * @param storedHashes - Array of stored hashes to check
 * @returns Matching stored hash entry or null
 */
export function findMatchingStoredHash(
  computedHash: string,
  computedAlgorithm: HashAlgorithmName,
  storedHashes: StoredHashEntry[]
): StoredHashEntry | null {
  return (
    storedHashes.find((stored) =>
      compareHashes(
        computedHash,
        stored.hash,
        computedAlgorithm,
        stored.algorithm
      )
    ) || null
  );
}

/**
 * Check if stored hashes contain a specific algorithm.
 * 
 * @param storedHashes - Array of stored hashes
 * @param algorithm - Algorithm to check for
 * @returns true if algorithm is present in stored hashes
 */
export function hasStoredHashForAlgorithm(
  storedHashes: StoredHashEntry[],
  algorithm: HashAlgorithmName
): boolean {
  return storedHashes.some((stored) =>
    algorithmsMatch(stored.algorithm, algorithm)
  );
}

// =============================================================================
// Deduplication Utilities
// =============================================================================

/**
 * Deduplicate stored hashes by algorithm.
 * If multiple hashes exist for the same algorithm, keeps the first one.
 * 
 * @param hashes - Array of stored hash entries (may contain duplicates)
 * @returns Deduplicated array
 */
export function deduplicateStoredHashes(
  hashes: StoredHashEntry[]
): StoredHashEntry[] {
  const seen = new Set<string>();
  const deduplicated: StoredHashEntry[] = [];

  for (const hash of hashes) {
    const normalized = normalizeAlgorithm(hash.algorithm);
    if (!seen.has(normalized)) {
      seen.add(normalized);
      deduplicated.push({
        ...hash,
        algorithm: normalized, // Use normalized algorithm name
      });
    }
  }

  return deduplicated;
}

/**
 * Group stored hashes by algorithm.
 * Returns a map of normalized algorithm names to their hash entries.
 * 
 * @param hashes - Array of stored hash entries
 * @returns Record mapping algorithm names to hash entries
 */
export function groupHashesByAlgorithm(
  hashes: StoredHashEntry[]
): Record<string, StoredHashEntry[]> {
  const groups: Record<string, StoredHashEntry[]> = {};

  for (const hash of hashes) {
    const normalized = normalizeAlgorithm(hash.algorithm);
    if (!groups[normalized]) {
      groups[normalized] = [];
    }
    groups[normalized].push({
      ...hash,
      algorithm: normalized,
    });
  }

  return groups;
}
