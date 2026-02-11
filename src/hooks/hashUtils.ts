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
import type {
  StoredHashEntry,
  HashProgressEvent,
  HashAlgorithmName,
} from "../types/hash";
import { normalizeAlgorithm, algorithmsMatch } from "../types/hash";

// =============================================================================
// Stored Hash Extraction
// =============================================================================

/**
 * Extract stored hashes from E01/Ex01 containers.
 * Uses ewf_info command to retrieve MD5/SHA1/SHA256 hashes from segment metadata.
 * 
 * @param path - Path to E01/Ex01 container
 * @returns Array of stored hash entries
 */
export async function extractE01StoredHashes(
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
    logger.error("[extractE01StoredHashes] Error:", error);
    return [];
  }
}

/**
 * Extract stored hashes from L01/Lx01 containers.
 * Uses ewf_info command to retrieve MD5/SHA1/SHA256 hashes from logical evidence metadata.
 * 
 * @param path - Path to L01/Lx01 container
 * @returns Array of stored hash entries
 */
export async function extractL01StoredHashes(
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
    logger.error("[extractL01StoredHashes] Error:", error);
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
    logger.error("[extractAd1StoredHashes] Error:", error);
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
    logger.error("[extractUfedStoredHashes] Error:", error);
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
      return extractE01StoredHashes(path);

    case "l01":
    case "lx01":
      return extractL01StoredHashes(path);

    case "ad1":
      return extractAd1StoredHashes(path);

    case "ufed":
    case "ufd":
      return extractUfedStoredHashes(path);

    default:
      logger.debug(`[extractStoredHashes] No stored hashes for extension: ${ext}`);
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
 * Hash E01/Ex01 container using EWF-aware hashing.
 * 
 * @param path - Path to E01/Ex01 file
 * @param algorithm - Hash algorithm to use
 * @returns Computed hash string (uppercase hex)
 */
export async function hashE01Container(
  path: string,
  algorithm: HashAlgorithmName
): Promise<string> {
  const result = await invoke<string>("hash_ewf", {
    path,
    algorithm,
  });
  return result.toUpperCase();
}

/**
 * Hash L01/Lx01 container using EWF-aware hashing.
 * 
 * @param path - Path to L01/Lx01 file
 * @param algorithm - Hash algorithm to use
 * @returns Computed hash string (uppercase hex)
 */
export async function hashL01Container(
  path: string,
  algorithm: HashAlgorithmName
): Promise<string> {
  const result = await invoke<string>("hash_ewf", {
    path,
    algorithm,
  });
  return result.toUpperCase();
}

/**
 * Hash AD1 container using AccessData-aware hashing.
 * 
 * @param path - Path to AD1 file
 * @param algorithm - Hash algorithm to use
 * @returns Computed hash string (uppercase hex)
 */
export async function hashAd1Container(
  path: string,
  algorithm: HashAlgorithmName
): Promise<string> {
  const result = await invoke<string>("hash_ad1", {
    path,
    algorithm,
  });
  return result.toUpperCase();
}

/**
 * Hash UFED container using Cellebrite-aware hashing.
 * 
 * @param path - Path to UFED file
 * @param algorithm - Hash algorithm to use
 * @returns Computed hash string (uppercase hex)
 */
export async function hashUfedContainer(
  path: string,
  algorithm: HashAlgorithmName
): Promise<string> {
  const result = await invoke<string>("hash_ufed", {
    path,
    algorithm,
  });
  return result.toUpperCase();
}

/**
 * Hash raw/DD container using standard file hashing.
 * 
 * @param path - Path to raw file
 * @param algorithm - Hash algorithm to use
 * @returns Computed hash string (uppercase hex)
 */
export async function hashRawContainer(
  path: string,
  algorithm: HashAlgorithmName
): Promise<string> {
  const result = await invoke<string>("hash_file", {
    path,
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
      return hashE01Container(path, algorithm);

    case "l01":
    case "lx01":
      return hashL01Container(path, algorithm);

    case "ad1":
      return hashAd1Container(path, algorithm);

    case "ufed":
    case "ufd":
      return hashUfedContainer(path, algorithm);

    case "raw":
    case "dd":
    case "img":
    case "dmg":
    default:
      return hashRawContainer(path, algorithm);
  }
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
