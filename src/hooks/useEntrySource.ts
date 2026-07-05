// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useEntrySource - Unified data reading from various evidence sources
 * 
 * This hook provides a consistent interface for reading bytes/text from:
 * - Regular disk files (DiscoveredFile)
 * - AD1 container entries
 * - VFS entries (E01/Raw filesystem)
 * - Archive entries (ZIP, 7z, TAR, etc.)
 * - Nested archive entries
 * 
 * Used by HexViewer, TextViewer, and other content viewers.
 */

import type { DiscoveredFile } from "../types";
import type { SelectedEntry } from "../components/EvidenceTree/types";
import { commands } from "../api/commands";
import { buildEvidenceSourceInput } from "../components/evidenceSourceInput";
import { isTauri } from "../utils/platform";

/**
 * Result of reading bytes from a source
 */
export interface ByteReadResult {
  bytes: number[];
  bytesRead: number;
  totalSize: number;
}

/**
 * Result of reading text from a source
 */
export interface TextReadResult {
  text: string;
  bytesRead: number;
  totalSize: number;
}

/**
 * Read bytes from any source: disk file, AD1 container entry, VFS entry, or archive entry
 */
export async function readBytesFromSource(
  file: DiscoveredFile | null,
  entry: SelectedEntry | undefined,
  offset: number,
  size: number
): Promise<ByteReadResult> {
  if (!isTauri) {
    throw new Error("Evidence content viewing is available in the desktop app.");
  }

  const source = buildEvidenceSourceInput(file, entry);
  if (!source) throw new Error("No file or entry provided");

  const chunk = await commands.viewer.readBinarySourceBase64Chunk(source, offset, size);
  return {
    bytes: base64ToBytes(chunk.data),
    bytesRead: chunk.bytesRead,
    totalSize: chunk.totalSize,
  };
}

/**
 * Read text from any source: disk file, AD1 container entry, VFS entry, or archive entry
 */
export async function readTextFromSource(
  file: DiscoveredFile | null,
  entry: SelectedEntry | undefined,
  offset: number,
  maxChars: number
): Promise<TextReadResult> {
  const { bytes, totalSize } = await readBytesFromSource(file, entry, offset, maxChars * 4);
  const prefixLength = completeUtf8PrefixLength(bytes);
  const completeBytes = bytes.slice(0, prefixLength === 0 && bytes.length > 0 ? bytes.length : prefixLength);
  const decoded = new TextDecoder("utf-8", { fatal: false }).decode(new Uint8Array(completeBytes));
  const text = decoded.length > maxChars ? Array.from(decoded).slice(0, maxChars).join("") : decoded;
  const bytesRead = text.length === decoded.length
    ? completeBytes.length
    : new TextEncoder().encode(text).length;
  return { text, bytesRead, totalSize };
}

/**
 * Get a unique source key for change detection (memoization/effect dependencies)
 */
export function getSourceKey(
  file: DiscoveredFile | null | undefined,
  entry: SelectedEntry | undefined
): string | null {
  if (entry) return `entry:${entry.containerPath}:${entry.entryPath}`;
  if (file) return `file:${file.path}`;
  return null;
}

/**
 * Get the display filename from any source
 */
export function getSourceFilename(
  file: DiscoveredFile | null | undefined,
  entry: SelectedEntry | undefined
): string {
  if (entry) return entry.name;
  if (file) return file.filename;
  return "";
}

function base64ToBytes(data: string): number[] {
  const binaryString = atob(data);
  const bytes = new Array<number>(binaryString.length);
  for (let i = 0; i < binaryString.length; i++) {
    bytes[i] = binaryString.charCodeAt(i);
  }
  return bytes;
}

function completeUtf8PrefixLength(bytes: number[]): number {
  if (bytes.length === 0) return 0;

  let start = bytes.length - 1;
  while (start >= 0 && (bytes[start] & 0b1100_0000) === 0b1000_0000) {
    start -= 1;
  }
  if (start < 0) return bytes.length;

  const first = bytes[start];
  const expectedLength =
    (first & 0b1000_0000) === 0 ? 1 :
    (first & 0b1110_0000) === 0b1100_0000 ? 2 :
    (first & 0b1111_0000) === 0b1110_0000 ? 3 :
    (first & 0b1111_1000) === 0b1111_0000 ? 4 :
    1;

  return bytes.length - start < expectedLength ? start : bytes.length;
}
