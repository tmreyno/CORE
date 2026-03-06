// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Registry viewer helper functions.
 */

/** Get color class for a registry data type */
export function getDataTypeColor(type: string): string {
  switch (type) {
    case "REG_SZ":
    case "REG_EXPAND_SZ":
      return "text-green-400";
    case "REG_DWORD":
    case "REG_QWORD":
    case "REG_DWORD_BIG_ENDIAN":
      return "text-blue-400";
    case "REG_BINARY":
      return "text-orange-400";
    case "REG_MULTI_SZ":
      return "text-purple-400";
    case "REG_NONE":
      return "text-txt-muted";
    default:
      return "text-txt-secondary";
  }
}

/** Format byte size to human-readable string */
export function formatSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
