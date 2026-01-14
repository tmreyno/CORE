// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Container Detection Utilities
 * 
 * Centralized container type detection functions used across the application.
 * This consolidates the detection logic that was duplicated in multiple places.
 */

/** Container types that use VFS mounting (disk images) - E01, L01, Raw images, Virtual disks */
export const VFS_CONTAINER_TYPES = [
  // EnCase formats
  "e01", "ex01", "ewf", "encase",
  // Raw/physical images
  "raw", "dd", "img", "001", "raw image",
  // Optical/Apple disk images
  "dmg", "iso", "iso 9660",
  // Logical evidence
  "l01", "lx01", "lvf",
  // Virtual disks (not yet implemented)
  "vmdk", "vhd", "vhdx", "qcow2", "vdi",
  // Other forensic formats (not yet implemented)
  "aff", "aff4", "smart",
] as const;

/** Container types that are logical evidence (L01/Lx01) - subset of VFS for special messaging */
export const LOGICAL_EVIDENCE_TYPES = ["l01", "lx01", "lvf"] as const;

/** Container types that are archives */
export const ARCHIVE_CONTAINER_TYPES = [
  // Standard archives
  "zip", "7z", "7-zip", "rar", "tar", "archive",
  // Compressed archives
  "gz", "gzip", "bz2", "bzip2", "xz", "zst", "zstd", "lz4",
  // Combined tar archives
  "tar.gz", "tgz", "tar.xz", "txz", "tar.bz2", "tbz2", "tar.zst", "tar.lz4",
] as const;

/** Container types that are UFED */
export const UFED_CONTAINER_TYPES = ["ufed", "ufd", "ufdr", "ufdx"] as const;

/** File extensions that are known forensic containers */
export const CONTAINER_EXTENSIONS = [
  // AD1
  "ad1",
  // EnCase
  "e01", "ex01", "ewf",
  // Raw/physical images
  "raw", "dd", "img", "001",
  // Optical/Apple disk images
  "dmg", "iso",
  // Logical evidence
  "l01", "lx01", "lvf",
  // Archives (can be nested)
  "zip", "7z", "rar", "tar", "gz", "tgz", "tar.gz", "tar.bz2", "tbz2", "tar.xz", "txz",
  // UFED
  "ufd", "ufdr", "ufdx",
] as const;

/** Check if container type uses VFS mounting (disk images) */
export const isVfsContainer = (type: string): boolean => {
  const lower = type.toLowerCase();
  return VFS_CONTAINER_TYPES.some(vt => lower.includes(vt));
};

/** Check if container type is L01 logical evidence */
export const isL01Container = (type: string): boolean => {
  const lower = type.toLowerCase();
  return LOGICAL_EVIDENCE_TYPES.some(lt => lower.includes(lt));
};

/** Check if container type is AD1 */
export const isAd1Container = (type: string): boolean => 
  type.toLowerCase().includes("ad1");

/** Check if container type is an archive */
export const isArchiveContainer = (type: string): boolean => {
  const lower = type.toLowerCase();
  return ARCHIVE_CONTAINER_TYPES.some(at => lower.includes(at));
};

/** Check if container type is UFED */
export const isUfedContainer = (type: string): boolean => {
  const lower = type.toLowerCase();
  return UFED_CONTAINER_TYPES.some(ut => lower.includes(ut));
};

/** Check if a filename has a known container extension */
export const isContainerFile = (filename: string): boolean => {
  const lower = filename.toLowerCase();
  return CONTAINER_EXTENSIONS.some(ext => lower.endsWith(`.${ext}`));
};

/** Get the container type from filename extension */
export const getContainerType = (filename: string): string => {
  const lower = filename.toLowerCase();
  const ext = CONTAINER_EXTENSIONS.find(ext => lower.endsWith(`.${ext}`));
  return ext || "unknown";
};

/** Get the container category for unified handling */
export const getContainerCategory = (type: string): "ad1" | "vfs" | "archive" | "ufed" | "unknown" => {
  if (isAd1Container(type)) return "ad1";
  if (isVfsContainer(type)) return "vfs";
  if (isArchiveContainer(type)) return "archive";
  if (isUfedContainer(type)) return "ufed";
  return "unknown";
};
