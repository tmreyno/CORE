// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Archive Creation API using sevenzip-ffi
 * 
 * Provides forensic-grade 7z archive creation with:
 * - AES-256 encryption
 * - Multi-threading
 * - Split archives for large files
 * - Progress tracking
 * - SHA-256 verification
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

/**
 * Archive creation options
 */
export interface CreateArchiveOptions {
  /** Compression level (0-9, default: 5) */
  compressionLevel?: number;
  /** Optional password for AES-256 encryption */
  password?: string;
  /** Number of threads (0 = auto, default: 2) */
  numThreads?: number;
  /** Dictionary size in MB (0 = auto) */
  dictSizeMb?: number;
  /** Enable solid compression (default: true) */
  solid?: boolean;
  /** Split archive size in MB (0 = no split) */
  splitSizeMb?: number;
  /** Chunk size for streaming in MB (default: 64) */
  chunkSizeMb?: number;
}

/**
 * Progress event during archive creation
 */
export interface ArchiveCreateProgress {
  /** Archive path being created */
  archivePath: string;
  /** Current file being processed */
  currentFile: string;
  /** Bytes processed so far */
  bytesProcessed: number;
  /** Total bytes to process (0 if unknown) */
  bytesTotal: number;
  /** Current file bytes processed */
  currentFileBytes: number;
  /** Current file total bytes */
  currentFileTotal: number;
  /** Progress percentage (0-100) */
  percent: number;
  /** Current operation status */
  status: string;
}

/**
 * Compression level presets
 * 
 * FORENSIC RECOMMENDATION: Use Store (Level 0) for evidence containers
 * 
 * Forensic containers (E01, AD1, L01, AFF4) are already compressed internally.
 * Attempting to compress them again wastes CPU time with zero size benefit.
 * Store mode provides maximum throughput (~500+ MB/s) limited only by disk I/O.
 * 
 * Speed benchmarks:
 * - Store:   ~500+ MB/s, 100% ratio - RECOMMENDED for E01/AD1/compressed data
 * - Fastest: ~180 MB/s, ~17% ratio - Good for uncompressed files (logs, text)
 * - Fast:    ~80 MB/s, ~16% ratio
 * - Normal:  ~22 MB/s, ~16% ratio
 * - Maximum: ~12 MB/s, ~15% ratio
 * - Ultra:   ~9 MB/s, ~15% ratio
 */
export const CompressionLevel = {
  /** Store mode - no compression, maximum speed. Best for E01/AD1/compressed data */
  Store: 0,
  /** Fastest compression (~180 MB/s, ~17% ratio) - Good for text/logs */
  Fastest: 1,
  /** Fast compression (~80 MB/s, ~16% ratio) */
  Fast: 3,
  /** Normal compression (~22 MB/s, ~16% ratio) */
  Normal: 5,
  /** Maximum compression (~12 MB/s, ~15% ratio) */
  Maximum: 7,
  /** Ultra compression (~9 MB/s, ~15% ratio) */
  Ultra: 9,
} as const;

/**
 * Create a 7z archive with the given files
 * 
 * @param archivePath - Output archive path (e.g., "evidence.7z")
 * @param inputPaths - Array of file/directory paths to compress
 * @param options - Compression options
 * @param onProgress - Optional progress callback
 * @returns Promise resolving to the archive path
 * 
 * @example
 * ```ts
 * // Basic usage
 * const archivePath = await createArchive(
 *   "/path/to/output.7z",
 *   ["/path/to/file1.txt", "/path/to/directory"],
 *   { compressionLevel: CompressionLevel.Normal }
 * );
 * 
 * // With encryption and progress tracking
 * const unlisten = await listenToProgress((progress) => {
 *   console.log(`Progress: ${progress.percent.toFixed(1)}% - ${progress.status}`);
 * });
 * 
 * try {
 *   const archivePath = await createArchive(
 *     "/path/to/encrypted.7z",
 *     ["/path/to/sensitive"],
 *     {
 *       compressionLevel: CompressionLevel.Maximum,
 *       password: "strong_password",
 *       numThreads: 8,
 *     }
 *   );
 *   console.log("Archive created:", archivePath);
 * } finally {
 *   unlisten();
 * }
 * 
 * // Split archive (multi-volume)
 * const archivePath = await createArchive(
 *   "/path/to/large-archive.7z",
 *   ["/path/to/large-file.img"],
 *   {
 *     compressionLevel: CompressionLevel.Normal,
 *     splitSizeMb: 4096, // 4GB segments
 *     chunkSizeMb: 64,
 *   }
 * );
 * ```
 */
export async function createArchive(
  archivePath: string,
  inputPaths: string[],
  options?: CreateArchiveOptions,
  onProgress?: (progress: ArchiveCreateProgress) => void
): Promise<string> {
  let unlisten: UnlistenFn | null = null;

  try {
    // Set up progress listener if callback provided
    if (onProgress) {
      unlisten = await listen<ArchiveCreateProgress>(
        "archive-create-progress",
        (event) => onProgress(event.payload)
      );
    }

    // Invoke backend command
    const result = await invoke<string>("create_7z_archive", {
      archivePath,
      inputPaths,
      options: options || null,
    });

    return result;
  } finally {
    // Clean up listener
    if (unlisten) {
      unlisten();
    }
  }
}

/**
 * Listen to archive creation progress events
 * 
 * @param callback - Function to call with progress updates
 * @returns Unlisten function to stop listening
 * 
 * @example
 * ```ts
 * const unlisten = await listenToProgress((progress) => {
 *   console.log(`${progress.percent.toFixed(1)}% - ${progress.currentFile}`);
 * });
 * 
 * // Later, stop listening
 * unlisten();
 * ```
 */
export async function listenToProgress(
  callback: (progress: ArchiveCreateProgress) => void
): Promise<UnlistenFn> {
  return await listen<ArchiveCreateProgress>("archive-create-progress", (event) =>
    callback(event.payload)
  );
}

/**
 * Test archive integrity after creation
 * 
 * @param archivePath - Path to archive to test
 * @param password - Optional password if encrypted
 * @returns Promise resolving to true if valid
 * 
 * @example
 * ```ts
 * const isValid = await testArchive("/path/to/archive.7z");
 * if (isValid) {
 *   console.log("Archive is valid!");
 * }
 * ```
 */
export async function testArchive(
  archivePath: string,
  password?: string
): Promise<boolean> {
  return await invoke<boolean>("test_7z_archive", {
    archivePath,
    password: password || null,
  });
}

/**
 * Estimate archive size before creating it
 * 
 * @param inputPaths - Array of file/directory paths to compress
 * @param compressionLevel - Compression level (0-9)
 * @returns Promise resolving to [uncompressedSize, estimatedCompressedSize]
 * 
 * @example
 * ```ts
 * const [uncompressed, compressed] = await estimateSize(
 *   ["/path/to/files"],
 *   CompressionLevel.Normal
 * );
 * console.log(`Will compress ${formatBytes(uncompressed)} to ~${formatBytes(compressed)}`);
 * ```
 */
export async function estimateSize(
  inputPaths: string[],
  compressionLevel: number = CompressionLevel.Normal
): Promise<[number, number]> {
  return await invoke<[number, number]>("estimate_archive_size", {
    inputPaths,
    compressionLevel,
  });
}

/**
 * Cancel an in-progress archive creation
 * 
 * @param archivePath - Path to the archive being created
 * @returns Promise resolving when cancelled
 * 
 * @note Currently not implemented - will return an error
 */
export async function cancelCreation(archivePath: string): Promise<void> {
  return await invoke<void>("cancel_archive_creation", {
    archivePath,
  });
}

/**
 * Format bytes to human-readable size
 * 
 * @param bytes - Number of bytes
 * @returns Formatted string (e.g., "1.5 GB")
 */
export function formatBytes(bytes: number): string {
  const units = ["B", "KB", "MB", "GB", "TB"];
  let size = bytes;
  let unitIndex = 0;

  while (size >= 1024 && unitIndex < units.length - 1) {
    size /= 1024;
    unitIndex++;
  }

  return `${size.toFixed(2)} ${units[unitIndex]}`;
}

/**
 * Format progress percentage
 * 
 * @param progress - Progress object
 * @returns Formatted percentage string
 */
export function formatProgress(progress: ArchiveCreateProgress): string {
  return `${progress.percent.toFixed(1)}%`;
}

/**
 * Get compression ratio from sizes
 * 
 * @param uncompressed - Uncompressed size in bytes
 * @param compressed - Compressed size in bytes
 * @returns Compression ratio as percentage
 */
export function getCompressionRatio(
  uncompressed: number,
  compressed: number
): number {
  if (uncompressed === 0) return 0;
  return ((1 - compressed / uncompressed) * 100);
}

/**
 * Archive creation error types
 */
export class ArchiveCreationError extends Error {
  constructor(message: string) {
    super(message);
    this.name = "ArchiveCreationError";
  }
}

// =============================================================================
// Advanced Archive Features
// =============================================================================

/**
 * Archive test progress event
 */
export interface ArchiveTestProgress {
  archivePath: string;
  status: string;
  percent: number;
}

/**
 * Archive repair progress event
 */
export interface ArchiveRepairProgress {
  percent: number;
  status: string;
}

/**
 * Split extract progress event
 */
export interface SplitExtractProgress {
  status: string;
  percent: number;
}

/**
 * Archive validation result
 */
export interface ArchiveValidationResult {
  isValid: boolean;
  errorMessage?: string;
  fileContext?: string;
  suggestion?: string;
}

/**
 * Detailed archive error information
 */
export interface DetailedArchiveError {
  code: number;
  message: string;
  fileContext: string;
  position: number;
  suggestion: string;
}

/**
 * Repair a corrupted archive
 * 
 * @param corruptedPath - Path to corrupted archive
 * @param repairedPath - Output path for repaired archive
 * @param onProgress - Optional progress callback
 * @returns Promise resolving to the repaired archive path
 * 
 * @example
 * ```ts
 * const unlisten = await listenToRepairProgress((progress) => {
 *   console.log(`Repairing: ${progress.percent}% - ${progress.status}`);
 * });
 * 
 * try {
 *   const repaired = await repairArchive(
 *     "/evidence/corrupted.7z",
 *     "/evidence/repaired.7z"
 *   );
 *   console.log(`Repaired archive saved to: ${repaired}`);
 * } finally {
 *   unlisten();
 * }
 * ```
 */
export async function repairArchive(
  corruptedPath: string,
  repairedPath: string,
  onProgress?: (progress: ArchiveRepairProgress) => void
): Promise<string> {
  let unlisten: UnlistenFn | null = null;

  try {
    // Set up progress listener if callback provided
    if (onProgress) {
      unlisten = await listen<ArchiveRepairProgress>(
        "archive-repair-progress",
        (event) => onProgress(event.payload)
      );
    }

    const result = await invoke<string>("repair_7z_archive", {
      corruptedPath,
      repairedPath,
    });

    return result;
  } finally {
    if (unlisten) {
      unlisten();
    }
  }
}

/**
 * Listen to archive repair progress events
 * 
 * @param callback - Function to call with progress updates
 * @returns Unlisten function to stop listening
 */
export async function listenToRepairProgress(
  callback: (progress: ArchiveRepairProgress) => void
): Promise<UnlistenFn> {
  return await listen<ArchiveRepairProgress>("archive-repair-progress", (event) =>
    callback(event.payload)
  );
}

/**
 * Validate an archive with detailed error information
 * 
 * @param archivePath - Path to archive to validate
 * @returns Promise resolving to validation result
 * 
 * @example
 * ```ts
 * const validation = await validateArchive("/evidence/suspect.7z");
 * if (!validation.isValid) {
 *   console.error(`Error: ${validation.errorMessage}`);
 *   console.log(`Suggestion: ${validation.suggestion}`);
 * }
 * ```
 */
export async function validateArchive(
  archivePath: string
): Promise<ArchiveValidationResult> {
  return await invoke<ArchiveValidationResult>("validate_7z_archive", {
    archivePath,
  });
}

/**
 * Extract a split/multi-volume archive
 * 
 * @param firstVolume - Path to first volume (e.g., "archive.7z.001")
 * @param outputDir - Directory to extract files to
 * @param password - Optional password if encrypted
 * @param onProgress - Optional progress callback
 * @returns Promise resolving to output directory path
 * 
 * @example
 * ```ts
 * const unlisten = await listenToSplitExtractProgress((progress) => {
 *   console.log(`Extracting: ${progress.percent}% - ${progress.status}`);
 * });
 * 
 * try {
 *   await extractSplitArchive(
 *     "/evidence/large-disk.7z.001",
 *     "/working/extracted/",
 *     "password123"
 *   );
 * } finally {
 *   unlisten();
 * }
 * ```
 */
export async function extractSplitArchive(
  firstVolume: string,
  outputDir: string,
  password?: string,
  onProgress?: (progress: SplitExtractProgress) => void
): Promise<string> {
  let unlisten: UnlistenFn | null = null;

  try {
    // Set up progress listener if callback provided
    if (onProgress) {
      unlisten = await listen<SplitExtractProgress>(
        "split-extract-progress",
        (event) => onProgress(event.payload)
      );
    }

    const result = await invoke<string>("extract_split_7z_archive", {
      firstVolume,
      outputDir,
      password: password || null,
    });

    return result;
  } finally {
    if (unlisten) {
      unlisten();
    }
  }
}

/**
 * Listen to split archive extraction progress events
 * 
 * @param callback - Function to call with progress updates
 * @returns Unlisten function to stop listening
 */
export async function listenToSplitExtractProgress(
  callback: (progress: SplitExtractProgress) => void
): Promise<UnlistenFn> {
  return await listen<SplitExtractProgress>("split-extract-progress", (event) =>
    callback(event.payload)
  );
}

/**
 * Encrypt data using native Rust AES-256
 * 
 * @param data - Data to encrypt (as Uint8Array)
 * @param password - Encryption password
 * @returns Promise resolving to encrypted data
 * 
 * @example
 * ```ts
 * const plaintext = new TextEncoder().encode("sensitive data");
 * const encrypted = await encryptDataNative(plaintext, "strong-password");
 * // Store encrypted data...
 * ```
 */
export async function encryptDataNative(
  data: Uint8Array,
  password: string
): Promise<Uint8Array> {
  const encrypted = await invoke<number[]>("encrypt_data_native", {
    data: Array.from(data),
    password,
  });
  return new Uint8Array(encrypted);
}

/**
 * Decrypt data using native Rust AES-256
 * 
 * @param encryptedData - Encrypted data (as Uint8Array)
 * @param password - Decryption password
 * @returns Promise resolving to decrypted data
 * 
 * @example
 * ```ts
 * const decrypted = await decryptDataNative(encryptedData, "strong-password");
 * const text = new TextDecoder().decode(decrypted);
 * console.log(text);
 * ```
 */
export async function decryptDataNative(
  encryptedData: Uint8Array,
  password: string
): Promise<Uint8Array> {
  const decrypted = await invoke<number[]>("decrypt_data_native", {
    encryptedData: Array.from(encryptedData),
    password,
  });
  return new Uint8Array(decrypted);
}

/**
 * Get detailed error information from last archive operation
 * 
 * @returns Promise resolving to error details, or null if no error
 * 
 * @example
 * ```ts
 * const error = await getLastArchiveError();
 * if (error) {
 *   console.error(`Error ${error.code}: ${error.message}`);
 *   console.log(`Context: ${error.fileContext}`);
 *   console.log(`Suggestion: ${error.suggestion}`);
 * }
 * ```
 */
export async function getLastArchiveError(): Promise<DetailedArchiveError | null> {
  return await invoke<DetailedArchiveError | null>("get_last_archive_error");
}

/**
 * Clear the last archive error
 * 
 * @returns Promise resolving when cleared
 */
export async function clearLastArchiveError(): Promise<void> {
  return await invoke<void>("clear_last_archive_error");
}
