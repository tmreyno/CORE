// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Container Utilities - Centralized Container Type Detection
 * 
 * Provides type-safe container detection functions to eliminate duplication
 * across the codebase. Uses constants from containerDetection.ts and adds
 * enum-based type safety.
 */

import type { ContainerType } from "../types/lazy-loading";

// Re-export all detection functions for backward compatibility
export {
  VFS_CONTAINER_TYPES,
  UNSUPPORTED_VFS_TYPES,
  LOGICAL_EVIDENCE_TYPES,
  ARCHIVE_CONTAINER_TYPES,
  UFED_CONTAINER_TYPES,
  MEMORY_DUMP_TYPES,
  MEMORY_DUMP_PATTERNS,
  CONTAINER_EXTENSIONS,
  NESTED_CONTAINER_EXTENSIONS,
  isVfsContainer,
  isUnsupportedVfsContainer,
  isL01Container,
  isE01Container,
  isAd1Container,
  isArchiveContainer,
  isUfedContainer,
  isMemoryDump,
  isMemoryDumpFile,
  isContainerFile,
  getContainerType as getContainerTypeString,
  getContainerCategory,
  isNestedContainerFile,
  getNestedContainerType,
} from "../components/EvidenceTree/containerDetection";

/**
 * Type-safe container type detection using ContainerType enum.
 * 
 * @param path - File path or filename
 * @returns ContainerType enum value or null if unknown
 */
export function detectContainerType(path: string): ContainerType | null {
  const ext = path.toLowerCase().split('.').pop();
  
  switch (ext) {
    case 'ad1':
      return 'Ad1' as ContainerType;
      
    case 'e01':
    case 'l01':
    case 'ex01':
    case 'lx01':
      return 'Ewf' as ContainerType;
      
    case 'ufd':
    case 'ufdr':
    case 'ufdx':
      return 'Ufed' as ContainerType;
      
    case 'zip':
      return 'Zip' as ContainerType;
      
    case '7z':
      return 'SevenZip' as ContainerType;
      
    case 'tar':
    case 'gz':
    case 'tgz':
      return 'Tar' as ContainerType;
      
    case 'rar':
      return 'Rar' as ContainerType;
      
    case 'dd':
    case 'raw':
    case 'img':
    case '001':
      return 'Raw' as ContainerType;
      
    default:
      return null;
  }
}

/**
 * Check if a file is a forensic container requiring special handling.
 * 
 * @param path - File path or filename
 * @returns true if path is a recognized forensic container
 */
export function isForensicContainer(path: string): boolean {
  return detectContainerType(path) !== null;
}

/**
 * Detect container type with fallback to string-based detection.
 * Useful for backward compatibility with code expecting string types.
 * 
 * @param path - File path or filename
 * @returns Container type string (e.g., "e01", "ad1") or "unknown"
 */
export function detectContainerTypeString(path: string): string {
  const lower = path.toLowerCase();
  
  // E01/Ex01
  if (lower.endsWith(".e01") || lower.endsWith(".ex01") || lower.includes(".e0")) {
    return "e01";
  }
  
  // L01/Lx01
  if (lower.endsWith(".l01") || lower.endsWith(".lx01") || lower.includes(".l0")) {
    return "l01";
  }
  
  // AD1
  if (lower.endsWith(".ad1")) {
    return "ad1";
  }
  
  // UFED (specific extensions only - NOT generic .zip)
  if (lower.endsWith(".ufd") || lower.endsWith(".ufdr") || lower.endsWith(".ufdx")) {
    return "ufed";
  }
  
  // Raw images
  if (lower.endsWith(".dd") || lower.endsWith(".raw") || lower.endsWith(".img") || lower.endsWith(".bin")) {
    return "raw";
  }
  
  return "unknown";
}

/**
 * Container format display names mapping.
 */
export const CONTAINER_DISPLAY_NAMES: Record<string, string> = {
  ad1: "AD1",
  e01: "E01",
  ex01: "Ex01",
  l01: "L01",
  lx01: "Lx01",
  ewf: "EWF",
  ufd: "UFED",
  ufdr: "UFED Reader",
  ufdx: "UFED XML",
  zip: "ZIP",
  "7z": "7-Zip",
  tar: "TAR",
  rar: "RAR",
  raw: "Raw Image",
  dd: "DD Image",
  img: "Image",
  dmg: "DMG",
  iso: "ISO",
};

/**
 * Get display name for a container type.
 * 
 * @param type - Container type string
 * @returns Display name or the original type if not found
 */
export function getContainerDisplayName(type: string): string {
  const lower = type.toLowerCase();
  return CONTAINER_DISPLAY_NAMES[lower] || type.toUpperCase();
}

// Re-export path utilities for convenience
export { getExtension, hasExtension } from "./pathUtils";
