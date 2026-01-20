// =============================================================================
// CORE-FFX - Forensic File Explorer
// VFS Types - Virtual Filesystem types for disk image mounting
// =============================================================================

/**
 * VFS (Virtual Filesystem) Types
 * 
 * Types for mounting and browsing disk images as virtual filesystems.
 */

// ============================================================================
// VFS Entry Types
// ============================================================================

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

// ============================================================================
// Partition Types
// ============================================================================

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

// ============================================================================
// Mount Info Types
// ============================================================================

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
