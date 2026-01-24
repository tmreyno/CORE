// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Shared utility functions for forensic container analysis

// Path utilities - re-export from centralized pathUtils
export { 
  getExtension, 
  getBasename, 
  getBasenameWithoutExt,
  getDirname,
  joinPath,
  normalizePath,
  hasExtension,
  hasAnyExtension,
  isAbsolutePath,
  isHiddenFile
} from './utils/pathUtils';

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
 * Format a date according to user preferences
 * @param date - Date object, ISO string, or timestamp string
 * @param includeTime - Whether to include time in output (default: true)
 * @returns Formatted date string
 */
export function formatDateByPreference(date: Date | string | null | undefined, includeTime = true): string {
  if (!date) return "";
  
  const d = typeof date === "string" ? parseTimestamp(date) || new Date(date) : date;
  if (isNaN(d.getTime())) return typeof date === "string" ? date : "";
  
  // Get preference from localStorage (avoid circular import)
  let dateFormat: "iso" | "us" | "eu" | "relative" = "iso";
  try {
    const stored = localStorage.getItem("ffx-preferences");
    if (stored) {
      const prefs = JSON.parse(stored);
      if (prefs.dateFormat) dateFormat = prefs.dateFormat;
    }
  } catch {
    // Use default
  }
  
  // Relative format: "2 hours ago", "yesterday", etc.
  if (dateFormat === "relative") {
    const now = new Date();
    const diffMs = now.getTime() - d.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);
    
    if (diffMins < 1) return "just now";
    if (diffMins < 60) return `${diffMins} minute${diffMins === 1 ? "" : "s"} ago`;
    if (diffHours < 24) return `${diffHours} hour${diffHours === 1 ? "" : "s"} ago`;
    if (diffDays < 7) return `${diffDays} day${diffDays === 1 ? "" : "s"} ago`;
    // Fall back to ISO for older dates
    dateFormat = "iso";
  }
  
  // Format options based on preference
  const dateOptions: Intl.DateTimeFormatOptions = 
    dateFormat === "us" ? { month: "2-digit", day: "2-digit", year: "numeric" } :  // MM/DD/YYYY
    dateFormat === "eu" ? { day: "2-digit", month: "2-digit", year: "numeric" } :  // DD/MM/YYYY
    { year: "numeric", month: "2-digit", day: "2-digit" };  // ISO: YYYY-MM-DD
  
  const locale = dateFormat === "us" ? "en-US" : dateFormat === "eu" ? "en-GB" : "sv-SE"; // sv-SE gives ISO format
  
  if (includeTime) {
    return d.toLocaleString(locale, { 
      ...dateOptions, 
      hour: "2-digit", 
      minute: "2-digit",
      hour12: dateFormat === "us"
    });
  }
  
  return d.toLocaleDateString(locale, dateOptions);
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
