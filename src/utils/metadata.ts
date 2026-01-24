/**
 * Metadata Display Utilities
 * @module utils/metadata
 * 
 * Provides shared utilities for consistent metadata display across components.
 */

import { createMemo, type Accessor } from "solid-js";
import { formatDateByPreference } from "../utils";

// =============================================================================
// Types
// =============================================================================

/** Generic metadata field for display */
export interface DisplayField {
  label: string;
  value: string | number | null | undefined;
  category?: string;
  highlight?: boolean;
  monospace?: boolean;
  copyable?: boolean;
}

/** Hash verification status */
export type VerificationStatus = 'verified' | 'mismatch' | 'pending' | 'unknown';

/** Stored hash information */
export interface HashInfo {
  algorithm: string;
  hash: string;
  verified?: boolean | null;
  source?: 'container' | 'companion' | 'computed';
}

// =============================================================================
// Formatting Utilities
// =============================================================================

/**
 * Format a date string for display
 */
export function formatDate(dateStr: string | null | undefined): string {
  return formatDateByPreference(dateStr, true);
}

/**
 * Format a Unix timestamp to readable date
 */
export function formatTimestamp(timestamp: number | null | undefined): string {
  if (timestamp == null) return '';
  
  try {
    const date = new Date(timestamp * 1000);
    if (isNaN(date.getTime())) return '';
    return date.toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch {
    return '';
  }
}

/**
 * Truncate a hash for display with ellipsis
 */
export function truncateHash(hash: string, length: number = 16): string {
  if (!hash || hash.length <= length) return hash;
  return `${hash.substring(0, length)}...`;
}

/**
 * Format a hash algorithm name consistently
 */
export function formatAlgorithm(algorithm: string): string {
  if (!algorithm) return '';
  const upper = algorithm.toUpperCase();
  // Normalize common variants
  if (upper === 'SHA256' || upper === 'SHA-256') return 'SHA-256';
  if (upper === 'SHA1' || upper === 'SHA-1') return 'SHA-1';
  if (upper === 'MD5') return 'MD5';
  return upper;
}

/**
 * Get verification status from hash info
 */
export function getVerificationStatus(verified: boolean | null | undefined): VerificationStatus {
  if (verified === true) return 'verified';
  if (verified === false) return 'mismatch';
  if (verified === null) return 'pending';
  return 'unknown';
}

/**
 * Get verification status icon/symbol
 */
export function getVerificationIcon(status: VerificationStatus): string {
  switch (status) {
    case 'verified': return '✓✓';
    case 'mismatch': return '✗';
    case 'pending': return '?';
    default: return '';
  }
}

/**
 * Get verification status CSS class
 */
export function getVerificationClass(status: VerificationStatus): string {
  switch (status) {
    case 'verified': return 'text-green-400';
    case 'mismatch': return 'text-red-400';
    case 'pending': return 'text-yellow-400';
    default: return 'text-txt-muted';
  }
}

/**
 * Format a number with locale-aware separators
 * @param value - Number to format
 * @param options - Intl.NumberFormat options
 */
export function formatNumber(
  value: number | null | undefined,
  options?: Intl.NumberFormatOptions
): string {
  if (value == null) return '';
  return value.toLocaleString(undefined, options);
}

/**
 * Format a count with optional singular/plural label
 * @param count - Number to format
 * @param singular - Singular label (e.g., "item")
 * @param plural - Optional plural label (defaults to singular + "s")
 */
export function formatCount(
  count: number | null | undefined,
  singular: string,
  plural?: string
): string {
  if (count == null) return '';
  const formattedCount = count.toLocaleString();
  const label = count === 1 ? singular : (plural ?? `${singular}s`);
  return `${formattedCount} ${label}`;
}

/**
 * Format an offset for hex display (e.g., "@ 0x1234")
 */
export function formatOffset(offset: number | null | undefined, prefix = '@'): string {
  if (offset == null) return '';
  return `${prefix} 0x${offset.toString(16).toUpperCase()}`;
}

/**
 * Format a decimal offset with separators
 */
export function formatDecimalOffset(offset: number | null | undefined, prefix = '@'): string {
  if (offset == null) return '';
  return `${prefix} ${offset.toLocaleString()}`;
}

// =============================================================================
// Memoization Helpers
// =============================================================================

/**
 * Create memoized hash display info
 */
export function createHashDisplayMemo(hashAccessor: Accessor<HashInfo | undefined>) {
  const algorithm = createMemo(() => formatAlgorithm(hashAccessor()?.algorithm ?? ''));
  const truncated = createMemo(() => truncateHash(hashAccessor()?.hash ?? ''));
  const fullHash = createMemo(() => hashAccessor()?.hash ?? '');
  const status = createMemo(() => getVerificationStatus(hashAccessor()?.verified));
  const statusIcon = createMemo(() => getVerificationIcon(status()));
  const statusClass = createMemo(() => getVerificationClass(status()));
  
  return { algorithm, truncated, fullHash, status, statusIcon, statusClass };
}

/**
 * Create memoized date display
 */
export function createDateDisplayMemo(dateAccessor: Accessor<string | null | undefined>) {
  const formatted = createMemo(() => formatDate(dateAccessor()));
  const hasDate = createMemo(() => !!dateAccessor());
  const raw = createMemo(() => dateAccessor() ?? '');
  
  return { formatted, hasDate, raw };
}

/**
 * Create memoized timestamp display
 */
export function createTimestampDisplayMemo(timestampAccessor: Accessor<number | null | undefined>) {
  const formatted = createMemo(() => formatTimestamp(timestampAccessor()));
  const hasTimestamp = createMemo(() => timestampAccessor() != null);
  const raw = createMemo(() => timestampAccessor() ?? 0);
  
  return { formatted, hasTimestamp, raw };
}

// =============================================================================
// Field Normalization
// =============================================================================

/**
 * Filter out empty/null fields from a list
 */
export function filterEmptyFields(fields: DisplayField[]): DisplayField[] {
  return fields.filter(f => 
    f.value !== null && 
    f.value !== undefined && 
    f.value !== ''
  );
}

/**
 * Group fields by category
 */
export function groupFieldsByCategory(fields: DisplayField[]): Map<string, DisplayField[]> {
  const groups = new Map<string, DisplayField[]>();
  
  for (const field of fields) {
    const category = field.category ?? 'General';
    const existing = groups.get(category) ?? [];
    existing.push(field);
    groups.set(category, existing);
  }
  
  return groups;
}

/**
 * Sort fields by label alphabetically
 */
export function sortFieldsByLabel(fields: DisplayField[]): DisplayField[] {
  return [...fields].sort((a, b) => a.label.localeCompare(b.label));
}

// =============================================================================
// Metadata Source Helpers
// =============================================================================

/** Source indicator symbols */
export const SOURCE_INDICATORS = {
  container: '◆',
  companion: '◇',
  computed: '▣',
} as const;

/**
 * Get source indicator for hash display
 */
export function getSourceIndicator(source: HashInfo['source']): string {
  if (!source) return '';
  return SOURCE_INDICATORS[source] ?? '';
}

/**
 * Get source description for tooltip
 */
export function getSourceDescription(source: HashInfo['source']): string {
  switch (source) {
    case 'container': return 'Stored in container header';
    case 'companion': return 'From companion log file';
    case 'computed': return 'Computed during verification';
    default: return '';
  }
}
