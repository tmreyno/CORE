// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Evidence item folder naming utilities.
 *
 * Generates standardized folder names for per-evidence-item directories
 * using the same naming convention as the forensic image auto-naming:
 *   [ProjectName]-[Last5SN]-[Hostname]-[Username]-[YYYYMMDD]
 */

import type { SystemStats } from "../hooks";

/** Sanitize a string segment for use in folder names */
const sanitize = (s: string): string =>
  s.replace(/[^a-zA-Z0-9._-]/g, "_").slice(0, 40);

/**
 * Generate a standardized evidence item folder name from system identification.
 *
 * Pattern: `[ProjectName]-[Last5SN]-[Hostname]-[Username]-[YYYYMMDD]`
 *
 * Example: `CaseAlpha-BC123-MacBookPro-terry-20250318`
 */
export function generateEvidenceFolderName(
  projectName: string | undefined,
  stats: SystemStats | null,
  username?: string,
): string {
  const project = projectName || "evidence";
  const sn = stats?.systemSerialNumber?.slice(-5) || "NOSN0";
  const host =
    stats?.hostname && stats.hostname !== "unknown" ? stats.hostname : "host";
  const user = username || "user";
  const date = new Date().toISOString().slice(0, 10).replace(/-/g, "");

  return [
    sanitize(project),
    sanitize(sn),
    sanitize(host),
    sanitize(user),
    date,
  ].join("-");
}
