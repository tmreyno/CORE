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
  // ISO disk images (read-only filesystem)
  "iso", "iso 9660",
  // Logical evidence
  "l01", "lx01", "lvf",
  // Virtual disks (not yet implemented)
  "vmdk", "vhd", "vhdx", "qcow2", "vdi",
  // Other forensic formats (not yet implemented)
  "aff", "aff4", "smart",
] as const;

/** Container types that are optical/disk images but NOT yet VFS-supported */
export const UNSUPPORTED_VFS_TYPES = [
  // DMG moved to ARCHIVE_CONTAINER_TYPES - now fully supported via archive interface
  // Reserved for future formats that need special VFS handling
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
  // Disk images (browsable via archive interface)
  "dmg", "iso",
] as const;

/** Container types that are UFED */
export const UFED_CONTAINER_TYPES = ["ufed", "ufd", "ufdr", "ufdx"] as const;

/** Container types that are memory dumps */
export const MEMORY_DUMP_TYPES = ["memory", "memdump", "ramdump"] as const;

/** File patterns that indicate memory dumps (for filename detection) */
export const MEMORY_DUMP_PATTERNS = [
  "_mem.raw",      // Standard memory dump naming: *_mem.raw.001
  ".mem",          // Simple memory extension
  ".vmem",         // VMware memory
  ".dmp",          // Windows memory dump
  ".hiberfil",     // Hibernation file
  ".pagefile",     // Page file
  "_memdump",      // Alternative naming
  ".raw.mem",      // Combined raw memory
] as const;

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

/** Check if container type is a disk image that's NOT YET supported by VFS */
export const isUnsupportedVfsContainer = (type: string): boolean => {
  const lower = type.toLowerCase();
  return UNSUPPORTED_VFS_TYPES.some(ut => lower.includes(ut));
};

/** Check if container type is L01 logical evidence */
export const isL01Container = (type: string): boolean => {
  const lower = type.toLowerCase();
  return LOGICAL_EVIDENCE_TYPES.some(lt => lower.includes(lt));
};

/** EnCase/E01 container types for specific E01 detection */
const ENCASE_TYPES = ["e01", "ex01", "ewf", "encase"] as const;

/** Check if container type is EnCase/E01 format */
export const isE01Container = (type: string): boolean => {
  const lower = type.toLowerCase();
  return ENCASE_TYPES.some(et => lower.includes(et));
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

/** Check if container type is a memory dump */
export const isMemoryDump = (type: string): boolean => {
  const lower = type.toLowerCase();
  return MEMORY_DUMP_TYPES.some(mt => lower.includes(mt));
};

/** Check if a filename indicates a memory dump based on naming patterns */
export const isMemoryDumpFile = (filename: string): boolean => {
  const lower = filename.toLowerCase();
  return MEMORY_DUMP_PATTERNS.some(pattern => lower.includes(pattern));
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
export const getContainerCategory = (type: string): "ad1" | "vfs" | "archive" | "ufed" | "memory" | "unknown" => {
  if (isAd1Container(type)) return "ad1";
  if (isMemoryDump(type)) return "memory";
  if (isVfsContainer(type)) return "vfs";
  if (isArchiveContainer(type)) return "archive";
  if (isUfedContainer(type)) return "ufed";
  return "unknown";
};

/** 
 * Extensions for nested containers - containers that can appear inside other containers
 * Includes forensic images and archives
 */
export const NESTED_CONTAINER_EXTENSIONS = [
  // Forensic images
  "ad1", "e01", "ex01", "ewf", "l01", "lx01", "raw", "dd", "img", "001", "dmg", "iso",
  // Archives
  "zip", "7z", "rar", "tar", "gz", "tgz", "tar.gz", "tar.bz2", "tbz2", "tar.xz", "txz",
  // UFED
  "ufd", "ufdr", "ufdx",
] as const;

/** Check if a filename is a potential nested container */
export const isNestedContainerFile = (filename: string): boolean => {
  const lower = filename.toLowerCase();
  return NESTED_CONTAINER_EXTENSIONS.some(ext => lower.endsWith(`.${ext}`));
};

/** Get the nested container type from filename */
export const getNestedContainerType = (filename: string): string | null => {
  const lower = filename.toLowerCase();
  const ext = NESTED_CONTAINER_EXTENSIONS.find(ext => lower.endsWith(`.${ext}`));
  return ext || null;
};
