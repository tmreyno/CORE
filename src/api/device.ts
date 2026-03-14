// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Raw device access and privilege detection API.
 *
 * Wraps the `device` Tauri commands for:
 * - Checking process privilege level (root/admin)
 * - Querying raw device sizes via ioctl/DeviceIoControl
 * - Enumerating physical disks (not just mounted volumes)
 * - Streaming raw device data for forensic imaging
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// =============================================================================
// Types
// =============================================================================

/** Information about the current process's privilege level. */
export interface PrivilegeInfo {
  /** Whether the process has elevated privileges (root/admin) */
  isElevated: boolean;
  /** Human-readable description of the privilege level */
  description: string;
  /** OS username of the current user */
  username: string;
  /** Whether elevation is required for raw device access */
  elevationRequired: boolean;
}

/** A physical disk on the system (not a partition/volume). */
export interface PhysicalDisk {
  /** Raw device path (e.g. "/dev/rdisk2", "/dev/sda", "\\\\.\\PhysicalDrive0") */
  devicePath: string;
  /** Whole-disk device path (e.g. "/dev/disk2" on macOS) */
  wholeDiskPath: string;
  /** Display name / model (e.g. "Samsung SSD 980") */
  model: string;
  /** Total size in bytes */
  sizeBytes: number;
  /** Media type: "SSD", "HDD", "USB", "Unknown" */
  mediaType: string;
  /** Whether this is the boot disk */
  isBootDisk: boolean;
  /** Whether this is removable (USB, SD card) */
  isRemovable: boolean;
  /** Serial number if available */
  serial: string;
  /** Partitions / volumes on this disk */
  partitions: string[];
}

/** Progress event emitted during raw device reading. */
export interface DeviceReadProgress {
  /** Device path being read */
  devicePath: string;
  /** Bytes read so far */
  bytesRead: number;
  /** Total bytes to read */
  totalBytes: number;
  /** Progress percentage (0–100) */
  percent: number;
}

// =============================================================================
// API Functions
// =============================================================================

/**
 * Check the current process's privilege level.
 *
 * On macOS/Linux, checks if running as root (UID 0) or SUID.
 * On Windows, checks if running with Administrator privileges.
 */
export async function checkPrivilege(): Promise<PrivilegeInfo> {
  return invoke<PrivilegeInfo>("check_privilege");
}

/**
 * Get the total size in bytes of a raw block device.
 *
 * Uses platform-specific ioctl/DeviceIoControl calls.
 * Requires elevated privileges on most platforms.
 *
 * @param devicePath - Raw device path (e.g. "/dev/rdisk2", "/dev/sda")
 */
export async function getDeviceSize(devicePath: string): Promise<number> {
  return invoke<number>("get_device_size", { devicePath });
}

/**
 * Enumerate physical disks on the system.
 *
 * Unlike `listDrives()` which returns mounted volumes, this returns
 * physical hardware devices. Each device may have multiple partitions.
 *
 * May require elevated privileges to detect all disks.
 */
export async function listPhysicalDisks(): Promise<PhysicalDisk[]> {
  return invoke<PhysicalDisk[]>("list_physical_disks");
}

/**
 * Request information about how to restart with elevated privileges.
 *
 * Returns platform-specific instructions (sudo, pkexec, runas).
 * Does NOT automatically restart the application.
 */
export async function requestElevation(): Promise<string> {
  return invoke<string>("request_elevation");
}

/**
 * Read raw bytes from a device to a file, streaming with progress events.
 *
 * This is the low-level building block for physical disk imaging.
 * Requires elevated privileges.
 *
 * @param devicePath - Raw device path to read from
 * @param outputPath - File path to write raw data to
 * @returns Total bytes read
 */
export async function readRawDevice(
  devicePath: string,
  outputPath: string
): Promise<number> {
  return invoke<number>("read_raw_device", { devicePath, outputPath });
}

/**
 * Listen for raw device read progress events.
 *
 * @param callback - Called with progress updates during device reading
 * @returns Unlisten function to stop receiving events
 */
export async function listenDeviceReadProgress(
  callback: (progress: DeviceReadProgress) => void
): Promise<UnlistenFn> {
  return listen<DeviceReadProgress>("device-read-progress", (event) => {
    callback(event.payload);
  });
}
