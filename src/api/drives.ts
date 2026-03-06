// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Drive / Volume enumeration API.
 *
 * Wraps the `list_drives` Tauri command to enumerate physical and logical
 * drives visible to the OS. Used by the export panel so users can select
 * a drive as a source for E01 / L01 imaging.
 */

import { invoke } from "@tauri-apps/api/core";

/** Information about a single disk or volume. */
export interface DriveInfo {
  /** OS device path (e.g. "/dev/disk0s1" on macOS) */
  devicePath: string;
  /** Human-readable name assigned by the OS */
  name: string;
  /** Mount point (e.g. "/" or "/Volumes/MyUSB") */
  mountPoint: string;
  /** Filesystem type (e.g. "apfs", "ntfs", "fat32") */
  fileSystem: string;
  /** Total capacity in bytes */
  totalBytes: number;
  /** Available (free) space in bytes */
  availableBytes: number;
  /** Used space in bytes */
  usedBytes: number;
  /** Disk media kind: "SSD", "HDD", or "Unknown" */
  kind: string;
  /** Whether the disk is removable (USB, SD card, etc.) */
  isRemovable: boolean;
  /** Whether the disk is mounted read-only */
  isReadOnly: boolean;
  /** Whether this is the boot / system volume (e.g. "/" on macOS/Linux) */
  isSystemDisk: boolean;
}

/**
 * List all mounted drives / volumes visible to the OS.
 *
 * @returns Array of drive information objects, one per mounted volume.
 */
export async function listDrives(): Promise<DriveInfo[]> {
  return invoke<DriveInfo[]>("list_drives");
}

/** Result of a path writability check. */
export interface WritabilityCheck {
  /** Whether the path is writable */
  writable: boolean;
  /** Human-readable reason if not writable */
  reason: string;
  /** Filesystem type (e.g. "ntfs", "apfs") if detected */
  fileSystem: string;
  /** Whether the volume is mounted read-only */
  isReadOnly: boolean;
}

/**
 * Check whether a path (or its parent volume) is writable.
 *
 * Tries to create and remove a probe file. Returns detailed info about
 * why the path may not be writable (read-only FS, NTFS on macOS, permissions).
 */
export async function checkPathWritable(path: string): Promise<WritabilityCheck> {
  return invoke<WritabilityCheck>("check_path_writable", { path });
}

/**
 * Format a byte count into a human-readable string (e.g. "256 GB").
 */
export function formatDriveSize(bytes: number): string {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB", "PB"];
  const i = Math.floor(Math.log(bytes) / Math.log(1024));
  const value = bytes / Math.pow(1024, i);
  return `${value.toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
}

// =============================================================================
// Read-Only Remount for Forensic Imaging
// =============================================================================

/** Result of a mount-state change operation. */
export interface MountResult {
  /** Whether the operation succeeded */
  success: boolean;
  /** Human-readable message */
  message: string;
  /** The mount point affected */
  mountPoint: string;
  /** Whether the volume is now read-only */
  isReadOnly: boolean;
}

/**
 * Remount a volume as read-only for forensic imaging.
 *
 * On macOS this uses `diskutil unmount` + `diskutil mount readOnly`.
 * The original mount state is recorded so it can be restored afterward.
 *
 * - Removable drives: works without administrator privileges
 * - Internal drives: may require admin
 * - System boot volume: always refused
 */
export async function remountReadOnly(mountPoint: string): Promise<MountResult> {
  return invoke<MountResult>("remount_read_only", { mountPoint });
}

/**
 * Restore a volume to its original mount state (read-write) after imaging.
 *
 * Only restores if the volume was originally read-write before
 * `remountReadOnly` was called. If it was already read-only, this is a no-op.
 */
export async function restoreMount(mountPoint: string): Promise<MountResult> {
  return invoke<MountResult>("restore_mount", { mountPoint });
}
