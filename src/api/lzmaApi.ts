// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * LZMA/LZMA2 Compression API
 *
 * TypeScript wrappers for raw LZMA and LZMA2 (XZ) compression/decompression
 * via the sevenzip-ffi backend (LZMA SDK 24.09).
 *
 * LZMA produces .lzma files, LZMA2 produces .xz files.
 * Compression levels 0-9 map to Store/Fastest/Fast/Normal/Maximum/Ultra.
 */

import { invoke } from "@tauri-apps/api/core";

/** Compression level labels matching LZMA SDK 24.09 defaults */
export const LZMA_COMPRESSION_LEVELS = [
  { value: 0, label: "Store (no compression)", dictSize: "64 KB" },
  { value: 1, label: "Fastest", dictSize: "256 KB" },
  { value: 3, label: "Fast", dictSize: "4 MB" },
  { value: 5, label: "Normal", dictSize: "32 MB" },
  { value: 7, label: "Maximum", dictSize: "128 MB" },
  { value: 9, label: "Ultra", dictSize: "256 MB" },
] as const;

/**
 * Compress a file using LZMA algorithm.
 * Produces a .lzma output file.
 *
 * @param inputPath - Path to the source file
 * @param outputPath - Path for the compressed .lzma output
 * @param compressionLevel - 0 (store) to 9 (ultra), default 5 (normal)
 * @returns The output file path on success
 */
export async function compressToLzma(
  inputPath: string,
  outputPath: string,
  compressionLevel: number = 5,
): Promise<string> {
  return invoke<string>("compress_to_lzma", {
    inputPath,
    outputPath,
    compressionLevel,
  });
}

/**
 * Decompress a .lzma file.
 *
 * @param lzmaPath - Path to the .lzma compressed file
 * @param outputPath - Path for the decompressed output
 * @returns The output file path on success
 */
export async function decompressLzma(
  lzmaPath: string,
  outputPath: string,
): Promise<string> {
  return invoke<string>("decompress_lzma", {
    lzmaPath,
    outputPath,
  });
}

/**
 * Compress a file using LZMA2 algorithm.
 * Produces a .xz output file.
 *
 * @param inputPath - Path to the source file
 * @param outputPath - Path for the compressed .xz output
 * @param compressionLevel - 0 (store) to 9 (ultra), default 5 (normal)
 * @returns The output file path on success
 */
export async function compressToLzma2(
  inputPath: string,
  outputPath: string,
  compressionLevel: number = 5,
): Promise<string> {
  return invoke<string>("compress_to_lzma2", {
    inputPath,
    outputPath,
    compressionLevel,
  });
}

/**
 * Decompress a .xz (LZMA2) file.
 *
 * @param xzPath - Path to the .xz compressed file
 * @param outputPath - Path for the decompressed output
 * @returns The output file path on success
 */
export async function decompressLzma2(
  xzPath: string,
  outputPath: string,
): Promise<string> {
  return invoke<string>("decompress_lzma2", {
    xzPath,
    outputPath,
  });
}
