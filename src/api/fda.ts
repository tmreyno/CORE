// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Full Disk Access (FDA) detection API.
 *
 * On macOS, many forensic-relevant directories (Mail, Messages, Safari, etc.)
 * are protected by TCC (Transparency, Consent, and Control). Without FDA,
 * triage and acquisition operations silently skip these paths.
 */

import { invoke } from "@tauri-apps/api/core";
import { isTauri } from "../utils/platform";

/** Result of probing macOS Full Disk Access status. */
export interface FullDiskAccessStatus {
  /** Whether the app appears to have FDA granted */
  hasFullDiskAccess: boolean;
  /** Human-readable summary */
  message: string;
  /** TCC-protected paths that were inaccessible */
  blockedPaths: string[];
}

/** Probe TCC-protected directories to determine FDA status. */
export async function checkFullDiskAccess(): Promise<FullDiskAccessStatus> {
  if (!isTauri) {
    return {
      hasFullDiskAccess: false,
      message: "Full Disk Access checks are available in the desktop app.",
      blockedPaths: [],
    };
  }

  return invoke<FullDiskAccessStatus>("check_full_disk_access");
}

/** Open macOS System Settings → Privacy & Security → Full Disk Access. */
export async function openFullDiskAccessSettings(): Promise<void> {
  if (!isTauri) {
    return;
  }

  return invoke<void>("open_full_disk_access_settings");
}
