// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Database viewer helper functions.
 */

/** Get color class for a SQLite column data type */
export function getColumnTypeColor(type: string): string {
  const upper = type.toUpperCase();
  if (upper.includes("INT")) return "text-blue-400";
  if (upper.includes("TEXT") || upper.includes("CHAR") || upper.includes("CLOB"))
    return "text-green-400";
  if (upper.includes("REAL") || upper.includes("FLOAT") || upper.includes("DOUBLE"))
    return "text-purple-400";
  if (upper.includes("BLOB")) return "text-orange-400";
  if (upper.includes("DATE") || upper.includes("TIME")) return "text-cyan-400";
  if (upper === "" || upper === "NULL") return "text-txt-muted";
  return "text-txt-secondary";
}
