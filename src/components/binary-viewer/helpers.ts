// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Format a number as a hex string (e.g. 0x1A2B).
 */
export function formatHex(value: number | null): string {
  if (value === null) return "N/A";
  return `0x${value.toString(16).toUpperCase()}`;
}

/**
 * Format a Unix timestamp to an ISO-style UTC string.
 */
export function formatTimestamp(ts: number | null): string {
  if (ts === null) return "N/A";
  try {
    const d = new Date(ts * 1000);
    return d.toISOString().replace("T", " ").replace("Z", " UTC");
  } catch {
    return `0x${ts.toString(16)}`;
  }
}

/**
 * Return a label and color class string for the binary format badge.
 */
export function formatBadge(format: string): { label: string; color: string } {
  if (format.startsWith("PE")) return { label: format, color: "bg-blue-500/20 text-blue-400 border-blue-500/30" };
  if (format.startsWith("ELF")) return { label: format, color: "bg-green-500/20 text-green-400 border-green-500/30" };
  if (format.startsWith("MachO")) return { label: format, color: "bg-purple-500/20 text-purple-400 border-purple-500/30" };
  return { label: format, color: "bg-bg-secondary text-txt-muted border-border" };
}
