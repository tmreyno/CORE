// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Portable mode API.
 *
 * Wraps the `portable_get_status` and `portable_ensure_dirs` Tauri commands
 * for detecting and managing portable mode (zero-footprint forensic operation).
 */

import { invoke } from "@tauri-apps/api/core";

/** Configuration for portable mode path redirection. */
export interface PortableConfig {
  /** Root directory for all portable data */
  dataDir: string;
  /** Configuration storage directory */
  configDir: string;
  /** Cache storage directory (WebView2, preview) */
  cacheDir: string;
  /** Temporary files directory (extraction, preview) */
  tempDir: string;
  /** Audit and session logs directory */
  logDir: string;
  /** Default project output directory */
  projectsDir: string;
  /** How portable mode was detected */
  detectionReason: string;
  /** Mount point of the volume the executable resides on */
  volumeMountPoint: string;
  /** Whether the volume has sufficient free space (> 100 MB) */
  hasSufficientSpace: boolean;
  /** Free space on the portable volume in bytes */
  freeSpaceBytes: number;
}

/** Portable mode status. */
export interface PortableStatus {
  /** Whether the app is running in portable mode */
  isPortable: boolean;
  /** Configuration details (null if not portable) */
  config: PortableConfig | null;
}

/** Query the portable mode status. */
export async function getPortableStatus(): Promise<PortableStatus> {
  return invoke<PortableStatus>("portable_get_status");
}

/** Ensure the portable data directory structure exists. */
export async function ensurePortableDirs(): Promise<string> {
  return invoke<string>("portable_ensure_dirs");
}
