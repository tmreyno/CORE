// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Report Numbering Utilities
 *
 * Generates unique, settings-driven report numbers, COC numbers, and
 * evidence item numbers using the patterns and counters stored in
 * AppPreferences.
 *
 * Sequential counters are persisted in `reportNumberCounters` keyed by
 * a scope string (report type, "coc", or "evidence") so each category
 * maintains its own independent sequence.
 */

import { getPreference } from "../../../preferences";
import type { ReportType } from "../../types";

// ============================================================================
// Internal: Counter Management
// ============================================================================

const STORAGE_KEY = "ffx-preferences";

/**
 * Atomically increment and return the next counter value for a given scope.
 * Persists the updated counter back to localStorage immediately.
 */
function nextCounter(scope: string): number {
  const counters = { ...getPreference("reportNumberCounters") };
  const current = counters[scope] ?? 0;
  const next = current + 1;
  counters[scope] = next;

  // Write-through to localStorage
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const parsed = JSON.parse(stored);
      parsed.reportNumberCounters = counters;
      localStorage.setItem(STORAGE_KEY, JSON.stringify(parsed));
    }
  } catch {
    // Swallow — preference hook will reconcile
  }

  return next;
}

/**
 * Peek at the current counter value without incrementing.
 */
export function peekCounter(scope: string): number {
  const counters = getPreference("reportNumberCounters");
  return (counters[scope] ?? 0) + 1;
}

// ============================================================================
// Report Number Generation
// ============================================================================

/**
 * Generate a unique report number for the given report type.
 *
 * Pattern: `{PREFIX}-{YEAR}-{SEQ}` (when `reportNumberIncludeYear` is true)
 *          `{PREFIX}-{SEQ}`        (when false)
 *
 * The prefix is looked up from `reportNumberPrefixes[reportType]`.
 * The sequential number is zero-padded to `reportNumberSeqDigits` digits.
 */
export function generateReportNumber(reportType: ReportType): string {
  const prefixes = getPreference("reportNumberPrefixes");
  const includeYear = getPreference("reportNumberIncludeYear");
  const seqDigits = getPreference("reportNumberSeqDigits");

  const prefix = prefixes[reportType] ?? "RPT";
  const seq = nextCounter(reportType);
  const seqStr = String(seq).padStart(seqDigits, "0");

  if (includeYear) {
    const year = new Date().getFullYear();
    return `${prefix}-${year}-${seqStr}`;
  }
  return `${prefix}-${seqStr}`;
}

/**
 * Preview what the next report number would look like (without consuming).
 */
export function previewReportNumber(reportType: ReportType): string {
  const prefixes = getPreference("reportNumberPrefixes");
  const includeYear = getPreference("reportNumberIncludeYear");
  const seqDigits = getPreference("reportNumberSeqDigits");

  const prefix = prefixes[reportType] ?? "RPT";
  const seq = peekCounter(reportType);
  const seqStr = String(seq).padStart(seqDigits, "0");

  if (includeYear) {
    const year = new Date().getFullYear();
    return `${prefix}-${year}-${seqStr}`;
  }
  return `${prefix}-${seqStr}`;
}

// ============================================================================
// COC Number Generation
// ============================================================================

/**
 * Generate a COC number using the pattern from preferences.
 *
 * Supported tokens:
 *   {case}   – replaced with the case number
 *   {agency} – replaced with the default agency
 *   {year}   – replaced with the current 4-digit year
 *   {seq}    – replaced with the next sequential counter (zero-padded)
 */
export function generateCocNumber(caseNumber?: string): string {
  const pattern = getPreference("cocNumberPattern");
  const agency = getPreference("defaultAgency");
  const seqDigits = getPreference("reportNumberSeqDigits");

  const seq = nextCounter("coc");
  const seqStr = String(seq).padStart(seqDigits < 3 ? 3 : seqDigits, "0");
  const year = String(new Date().getFullYear());

  return pattern
    .replace(/\{case\}/gi, caseNumber || "0000")
    .replace(/\{agency\}/gi, agency || "AGY")
    .replace(/\{year\}/gi, year)
    .replace(/\{seq\}/gi, seqStr);
}

/**
 * Preview the next COC number without consuming a counter.
 */
export function previewCocNumber(caseNumber?: string): string {
  const pattern = getPreference("cocNumberPattern");
  const agency = getPreference("defaultAgency");
  const seqDigits = getPreference("reportNumberSeqDigits");

  const seq = peekCounter("coc");
  const seqStr = String(seq).padStart(seqDigits < 3 ? 3 : seqDigits, "0");
  const year = String(new Date().getFullYear());

  return pattern
    .replace(/\{case\}/gi, caseNumber || "0000")
    .replace(/\{agency\}/gi, agency || "AGY")
    .replace(/\{year\}/gi, year)
    .replace(/\{seq\}/gi, seqStr);
}

// ============================================================================
// Evidence Item Number Generation
// ============================================================================

/**
 * Generate an evidence item number using the pattern from preferences.
 *
 * Supported tokens:
 *   {case} – replaced with the case number
 *   {seq}  – replaced with the next sequential counter (zero-padded)
 */
export function generateEvidenceItemNumber(caseNumber?: string): string {
  const pattern = getPreference("evidenceItemPattern");
  const seqDigits = getPreference("reportNumberSeqDigits");

  const seq = nextCounter("evidence");
  const seqStr = String(seq).padStart(seqDigits < 3 ? 3 : seqDigits, "0");

  return pattern
    .replace(/\{case\}/gi, caseNumber || "0000")
    .replace(/\{seq\}/gi, seqStr);
}

/**
 * Preview the next evidence item number without consuming a counter.
 */
export function previewEvidenceItemNumber(caseNumber?: string): string {
  const pattern = getPreference("evidenceItemPattern");
  const seqDigits = getPreference("reportNumberSeqDigits");

  const seq = peekCounter("evidence");
  const seqStr = String(seq).padStart(seqDigits < 3 ? 3 : seqDigits, "0");

  return pattern
    .replace(/\{case\}/gi, caseNumber || "0000")
    .replace(/\{seq\}/gi, seqStr);
}
