// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * VFS (Virtual Filesystem) types for mounting disk images
 */

/** Entry in a mounted virtual filesystem */
export type VfsEntry = {
  /** Entry name */
  name: string;
  /** Full path within the VFS */
  path: string;
  /** Is this a directory? */
  isDir: boolean;
  /** File size (0 for directories) */
  size: number;
  /** File type hint */
  fileType?: string;
};

/** Information about a partition in a mounted disk image */
export type VfsPartitionInfo = {
  /** Partition number (1-based) */
  number: number;
  /** Mount name (e.g., "Partition1_NTFS") */
  mountName: string;
  /** Filesystem type (NTFS, FAT32, etc.) */
  fsType: string;
  /** Partition size in bytes */
  size: number;
  /** Start offset in the disk image */
  startOffset: number;
};

/** Information about a mounted disk image */
export type VfsMountInfo = {
  /** Container path */
  containerPath: string;
  /** Container type (e01, raw, etc.) */
  containerType: string;
  /** Total disk size */
  diskSize: number;
  /** Detected partitions */
  partitions: VfsPartitionInfo[];
  /** Mount mode (physical or filesystem) */
  mode: string;
};
