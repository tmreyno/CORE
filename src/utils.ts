// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Shared utility functions for forensic container analysis

/**
 * Get the lowercase file extension from a filename
 * @example getExtension("file.PDF") => "pdf"
 * @example getExtension("file") => ""
 */
export function getExtension(filename: string): string {
  return filename.split('.').pop()?.toLowerCase() || '';
}

/**
 * Get basename (filename) from a path
 * @example getBasename("/path/to/file.txt") => "file.txt"
 * @example getBasename("file.txt") => "file.txt"
 * @example getBasename("/path/to/") => ""
 */
export function getBasename(path: string): string {
  return path.split('/').pop() || '';
}

/**
 * Format byte count to human-readable string (B, KB, MB, GB, TB)
 */
export function formatBytes(value: number): string {
  if (!value) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.min(Math.floor(Math.log(value) / Math.log(1024)), units.length - 1);
  const scaled = value / Math.pow(1024, i);
  return `${scaled.toFixed(scaled < 10 ? 2 : 1)} ${units[i]}`;
}

/**
 * Normalize various error types to a string message
 */
export function normalizeError(err: unknown): string {
  if (!err) return "Unknown error";
  if (typeof err === "string") return err;
  if (typeof err === "object" && "message" in err) {
    return String((err as { message: string }).message);
  }
  return JSON.stringify(err);
}
/**
 * Get CSS class for container type styling
 */
export function typeClass(type: string): string {
  const t = type.toLowerCase();
  if (t.includes("ad1")) return "type-ad1";
  if (t.includes("e01") || t.includes("encase")) return "type-e01";
  if (t.includes("l01")) return "type-l01";
  if (t.includes("raw") || t.includes("dd")) return "type-raw";
  if (t.includes("ufed") || t.includes("ufd")) return "type-ufed";
  if (t.includes("tar") || t.includes("7z") || t.includes("zip") || t.includes("rar") || t.includes("gz")) return "type-archive";
  return "type-other";
}

/**
 * Parse various date formats (handles UFED DD/MM/YYYY HH:MM:SS (timezone) format)
 * 
 * Supported formats:
 * - Standard ISO 8601 / JS Date parseable strings
 * - UFED: DD/MM/YYYY HH:MM:SS (timezone) e.g., "26/08/2024 17:48:01 (-4)"
 * - DD/MM/YYYY without time
 */
export function parseTimestamp(timestamp: string): Date | null {
  // Try standard Date parsing first
  const standardDate = new Date(timestamp);
  if (!isNaN(standardDate.getTime())) {
    return standardDate;
  }
  
  // Try UFED format: DD/MM/YYYY HH:MM:SS (timezone) e.g., "26/08/2024 17:48:01 (-4)"
  const ufedMatch = timestamp.match(/^(\d{2})\/(\d{2})\/(\d{4})\s+(\d{2}):(\d{2}):(\d{2})\s*\(([+-]?\d+)\)?$/);
  if (ufedMatch) {
    const [, day, month, year, hour, minute, second, tzOffset] = ufedMatch;
    // Parse timezone offset - UFED uses (-4) meaning UTC-4
    const offset = parseInt(tzOffset, 10);
    // Create date string in ISO format
    const isoString = `${year}-${month}-${day}T${hour}:${minute}:${second}${offset >= 0 ? '+' : ''}${String(Math.abs(offset)).padStart(2, '0')}:00`;
    const d = new Date(isoString);
    if (!isNaN(d.getTime())) {
      return d;
    }
    // Fallback: just use date parts directly
    return new Date(parseInt(year), parseInt(month) - 1, parseInt(day), parseInt(hour), parseInt(minute), parseInt(second));
  }
  
  // Try DD/MM/YYYY format without time
  const dmyMatch = timestamp.match(/^(\d{2})\/(\d{2})\/(\d{4})$/);
  if (dmyMatch) {
    const [, day, month, year] = dmyMatch;
    return new Date(parseInt(year), parseInt(month) - 1, parseInt(day));
  }
  
  return null;
}

/**
 * Format hash timestamp for display (short date format)
 * Uses parseTimestamp internally to handle various formats
 */
export function formatHashDate(timestamp: string): string {
  try {
    const d = parseTimestamp(timestamp);
    if (d) {
      return d.toLocaleDateString(undefined, { month: 'short', day: 'numeric', year: '2-digit' });
    }
    return timestamp;
  } catch {
    return timestamp;
  }
}

/**
 * Format duration in seconds to human-readable string
 */
export function formatDuration(seconds: number): string {
  if (seconds < 60) return `${seconds.toFixed(1)}s`;
  const mins = Math.floor(seconds / 60);
  const secs = seconds % 60;
  return `${mins}m ${secs.toFixed(0)}s`;
}

/**
 * Create a debounced function that delays execution until after 
 * `delay` milliseconds have elapsed since the last call.
 */
export function debounce<T extends (...args: Parameters<T>) => void>(
  fn: T,
  delay: number
): (...args: Parameters<T>) => void {
  let timeoutId: ReturnType<typeof setTimeout> | null = null;
  return (...args: Parameters<T>) => {
    if (timeoutId) clearTimeout(timeoutId);
    timeoutId = setTimeout(() => fn(...args), delay);
  };
}

/**
 * Format a byte offset as a hexadecimal string.
 * @param offset - The byte offset to format
 * @param options - Formatting options
 * @param options.width - Minimum width with zero-padding (default: 8)
 * @param options.prefix - Whether to include '0x' prefix (default: false)
 * @returns Formatted hex string (e.g., "0000A1B2" or "0x0000A1B2")
 */
export function formatOffset(
  offset: number | undefined | null,
  options: { width?: number; prefix?: boolean } = {}
): string {
  if (offset === undefined || offset === null) return "";
  const { width = 8, prefix = false } = options;
  const hex = offset.toString(16).toUpperCase().padStart(width, '0');
  return prefix ? `0x${hex}` : hex;
}

/**
 * Format a byte offset with "@ 0x" prefix for display in metadata panels.
 * @param offset - The byte offset to format
 * @returns Formatted string like "@ 0x0000A1B2" or empty string if null/undefined
 */
export function formatOffsetLabel(offset: number | undefined | null): string {
  if (offset === undefined || offset === null) return "";
  return `@ 0x${offset.toString(16).toUpperCase()}`;
}

/**
 * Convert a single byte (0-255) to a 2-character uppercase hex string.
 * @param byte - The byte value to convert
 * @returns Formatted hex string (e.g., "0A", "FF")
 */
export function byteToHex(byte: number): string {
  return byte.toString(16).toUpperCase().padStart(2, '0');
}

/**
 * Convert a byte (0-255) to its ASCII character representation.
 * Returns '.' for non-printable characters.
 * @param byte - The byte value to convert
 * @returns Single character string
 */
export function byteToAscii(byte: number): string {
  // Printable ASCII range: 32-126
  if (byte >= 32 && byte <= 126) {
    return String.fromCharCode(byte);
  }
  return '.';
}
