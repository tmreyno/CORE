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

import { invoke } from "@tauri-apps/api/core";
import type { DiscoveredFile, FileChunk } from "../types";
import type { SelectedEntry } from "../components/EvidenceTree/types";

/**
 * Result of reading bytes from a source
 */
export interface ByteReadResult {
  bytes: number[];
  totalSize: number;
}

/**
 * Result of reading text from a source
 */
export interface TextReadResult {
  text: string;
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
  // Case 1: SelectedEntry provided (container file viewing)
  if (entry) {
    // VFS entries (E01/Raw)
    if (entry.isVfsEntry) {
      const bytes = await invoke<number[]>("vfs_read_file", {
        containerPath: entry.containerPath,
        filePath: entry.entryPath,
        offset,
        length: size
      });
      return { bytes, totalSize: entry.size };
    }
    
    // Archive entries (ZIP, 7z, TAR, etc.)
    if (entry.isArchiveEntry) {
      // Check if this is a nested archive entry (path contains "::")
      // Format: "nestedArchive.zip::file.txt" means file.txt inside nestedArchive.zip
      if (entry.entryPath.includes("::")) {
        const [nestedArchivePath, nestedEntryPath] = entry.entryPath.split("::", 2);
        const bytes = await invoke<number[]>("nested_archive_read_entry_chunk", {
          containerPath: entry.containerPath,
          nestedArchivePath,
          entryPath: nestedEntryPath,
          offset,
          size
        });
        return { bytes, totalSize: entry.size };
      }
      
      // Regular archive entry
      const bytes = await invoke<number[]>("archive_read_entry_chunk", {
        containerPath: entry.containerPath,
        entryPath: entry.entryPath,
        offset,
        size
      });
      return { bytes, totalSize: entry.size };
    }
    
    // Disk file entry (file inside container that's actually on disk)
    if (entry.isDiskFile) {
      const bytes = await invoke<number[]>("read_file_bytes", {
        path: entry.entryPath,
        offset,
        length: size
      });
      return { bytes, totalSize: entry.size };
    }
    
    // AD1 container entry - use chunk-based reading for scroll support
    const bytes = await invoke<number[]>("container_read_entry_chunk", {
      containerPath: entry.containerPath,
      entryPath: entry.entryPath,
      offset,
      size
    });
    return { bytes, totalSize: entry.size };
  }
  
  // Case 2: Regular disk file (DiscoveredFile)
  if (file) {
    const result = await invoke<FileChunk>("viewer_read_chunk", {
      path: file.path,
      offset,
      size
    });
    return { bytes: result.bytes, totalSize: result.total_size };
  }
  
  throw new Error("No file or entry provided");
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
  // For container entries, read bytes and decode as text
  if (entry) {
    const { bytes, totalSize } = await readBytesFromSource(file, entry, offset, maxChars);
    const text = new TextDecoder("utf-8", { fatal: false }).decode(new Uint8Array(bytes));
    return { text, totalSize };
  }
  
  // For disk files, use the dedicated text reading command (more efficient)
  if (file) {
    const text = await invoke<string>("viewer_read_text", {
      path: file.path,
      offset,
      maxChars
    });
    return { text, totalSize: file.size };
  }
  
  throw new Error("No file or entry provided");
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
