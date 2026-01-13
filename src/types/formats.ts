// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Forensic Format Definitions (TypeScript)
 *
 * This module mirrors the Rust formats.rs definitions for use in the frontend.
 * It provides type-safe format detection, display names, and metadata.
 *
 * @module formats
 */

// =============================================================================
// FORMAT CATEGORIES
// =============================================================================

/**
 * High-level category of forensic format
 */
export type FormatCategory =
  | "forensic-container"  // E01, AD1, L01
  | "raw-image"           // DD, RAW, IMG
  | "mobile-forensic"     // UFED
  | "archive"             // ZIP, 7z
  | "virtual-disk"        // VMDK, VHD
  | "optical-disc"        // ISO, DMG
  | "unknown";

/**
 * Category metadata for display
 */
export interface FormatCategoryInfo {
  id: FormatCategory;
  name: string;
  icon: string;
  description: string;
}

/**
 * All format categories with metadata
 */
export const FORMAT_CATEGORIES: Record<FormatCategory, FormatCategoryInfo> = {
  "forensic-container": {
    id: "forensic-container",
    name: "Forensic Container",
    icon: "📦",
    description: "Evidence preservation formats (E01, AD1, L01)",
  },
  "raw-image": {
    id: "raw-image",
    name: "Raw Disk Image",
    icon: "💾",
    description: "Uncompressed bit-for-bit disk images",
  },
  "mobile-forensic": {
    id: "mobile-forensic",
    name: "Mobile Forensic",
    icon: "📱",
    description: "Mobile device extraction formats (UFED)",
  },
  "archive": {
    id: "archive",
    name: "Archive",
    icon: "🗜️",
    description: "Compressed archive formats",
  },
  "virtual-disk": {
    id: "virtual-disk",
    name: "Virtual Disk",
    icon: "💿",
    description: "Virtual machine disk formats",
  },
  "optical-disc": {
    id: "optical-disc",
    name: "Optical Disc",
    icon: "📀",
    description: "CD/DVD/Blu-ray disc images",
  },
  "unknown": {
    id: "unknown",
    name: "Unknown",
    icon: "❓",
    description: "Unknown or unsupported format",
  },
};

// =============================================================================
// FORMAT DEFINITIONS
// =============================================================================

/**
 * Definition of a forensic container format
 */
export interface ContainerFormatDef {
  /** Internal identifier */
  id: string;
  /** Display name */
  displayName: string;
  /** Short type name */
  typeName: string;
  /** File extensions (lowercase, without dot) */
  extensions: readonly string[];
  /** Format category */
  category: FormatCategory;
  /** Whether format supports segmentation */
  supportsSegments: boolean;
  /** Description */
  description: string;
  /** Vendor/creator */
  vendor?: string;
}

// =============================================================================
// FORMAT REGISTRY
// =============================================================================

/**
 * Expert Witness Format (E01)
 */
export const FORMAT_E01: ContainerFormatDef = {
  id: "e01",
  displayName: "Expert Witness Format",
  typeName: "E01",
  extensions: ["e01", "e02", "e03", "e04", "e05", "ewf"],
  category: "forensic-container",
  supportsSegments: true,
  description: "EnCase Expert Witness disk image format",
  vendor: "OpenText (formerly Guidance Software)",
};

/**
 * Expert Witness Format v2 (Ex01)
 */
export const FORMAT_EX01: ContainerFormatDef = {
  id: "ex01",
  displayName: "Expert Witness Format v2",
  typeName: "Ex01",
  extensions: ["ex01"],
  category: "forensic-container",
  supportsSegments: true,
  description: "EnCase Expert Witness v2 format with improved compression",
  vendor: "OpenText (formerly Guidance Software)",
};

/**
 * EnCase Logical Evidence (L01)
 */
export const FORMAT_L01: ContainerFormatDef = {
  id: "l01",
  displayName: "EnCase Logical Evidence",
  typeName: "L01",
  extensions: ["l01", "l02", "l03"],
  category: "forensic-container",
  supportsSegments: true,
  description: "EnCase logical evidence container",
  vendor: "OpenText (formerly Guidance Software)",
};

/**
 * EnCase Logical Evidence v2 (Lx01)
 */
export const FORMAT_LX01: ContainerFormatDef = {
  id: "lx01",
  displayName: "EnCase Logical Evidence v2",
  typeName: "Lx01",
  extensions: ["lx01"],
  category: "forensic-container",
  supportsSegments: true,
  description: "EnCase logical evidence v2 format",
  vendor: "OpenText (formerly Guidance Software)",
};

/**
 * AccessData AD1
 */
export const FORMAT_AD1: ContainerFormatDef = {
  id: "ad1",
  displayName: "AccessData Logical Image",
  typeName: "AD1",
  extensions: ["ad1", "ad2", "ad3"],
  category: "forensic-container",
  supportsSegments: true,
  description: "AccessData FTK Imager logical evidence format",
  vendor: "AccessData (now Exterro)",
};

/**
 * Raw Disk Image
 */
export const FORMAT_RAW: ContainerFormatDef = {
  id: "raw",
  displayName: "Raw Disk Image",
  typeName: "Raw",
  extensions: ["dd", "raw", "img", "bin", "001"],
  category: "raw-image",
  supportsSegments: true,
  description: "Uncompressed bit-for-bit disk image",
};

/**
 * Cellebrite UFED UFD
 */
export const FORMAT_UFED_UFD: ContainerFormatDef = {
  id: "ufed_ufd",
  displayName: "Cellebrite UFED Extraction",
  typeName: "UFD",
  extensions: ["ufd"],
  category: "mobile-forensic",
  supportsSegments: false,
  description: "Cellebrite UFED extraction metadata file",
  vendor: "Cellebrite",
};

/**
 * Cellebrite UFED UFDR
 */
export const FORMAT_UFED_UFDR: ContainerFormatDef = {
  id: "ufed_ufdr",
  displayName: "Cellebrite UFED Report",
  typeName: "UFDR",
  extensions: ["ufdr"],
  category: "mobile-forensic",
  supportsSegments: false,
  description: "Cellebrite UFED report package",
  vendor: "Cellebrite",
};

/**
 * 7-Zip Archive
 */
export const FORMAT_7Z: ContainerFormatDef = {
  id: "7z",
  displayName: "7-Zip Archive",
  typeName: "7z",
  extensions: ["7z"],
  category: "archive",
  supportsSegments: true,
  description: "7-Zip compressed archive",
  vendor: "Igor Pavlov",
};

/**
 * ZIP Archive
 */
export const FORMAT_ZIP: ContainerFormatDef = {
  id: "zip",
  displayName: "ZIP Archive",
  typeName: "ZIP",
  extensions: ["zip", "z01"],
  category: "archive",
  supportsSegments: true,
  description: "ZIP compressed archive",
};

/**
 * AFF (Advanced Forensic Format)
 */
export const FORMAT_AFF: ContainerFormatDef = {
  id: "aff",
  displayName: "Advanced Forensic Format",
  typeName: "AFF",
  extensions: ["aff", "afd"],
  category: "forensic-container",
  supportsSegments: false,
  description: "Open-source forensic disk image format",
};

/**
 * AFF4
 */
export const FORMAT_AFF4: ContainerFormatDef = {
  id: "aff4",
  displayName: "Advanced Forensic Format 4",
  typeName: "AFF4",
  extensions: ["aff4"],
  category: "forensic-container",
  supportsSegments: false,
  description: "AFF4 forensic container (ZIP-based)",
};

/**
 * VMDK (VMware)
 */
export const FORMAT_VMDK: ContainerFormatDef = {
  id: "vmdk",
  displayName: "VMware Virtual Disk",
  typeName: "VMDK",
  extensions: ["vmdk"],
  category: "virtual-disk",
  supportsSegments: true,
  description: "VMware virtual machine disk image",
  vendor: "VMware",
};

/**
 * VHD (Microsoft)
 */
export const FORMAT_VHD: ContainerFormatDef = {
  id: "vhd",
  displayName: "Virtual Hard Disk",
  typeName: "VHD",
  extensions: ["vhd"],
  category: "virtual-disk",
  supportsSegments: false,
  description: "Microsoft Virtual Hard Disk format",
  vendor: "Microsoft",
};

/**
 * VHDX (Microsoft)
 */
export const FORMAT_VHDX: ContainerFormatDef = {
  id: "vhdx",
  displayName: "Virtual Hard Disk v2",
  typeName: "VHDX",
  extensions: ["vhdx"],
  category: "virtual-disk",
  supportsSegments: false,
  description: "Microsoft VHDX format (Hyper-V)",
  vendor: "Microsoft",
};

/**
 * QCOW2 (QEMU)
 */
export const FORMAT_QCOW2: ContainerFormatDef = {
  id: "qcow2",
  displayName: "QEMU Copy-on-Write v2",
  typeName: "QCOW2",
  extensions: ["qcow2", "qcow"],
  category: "virtual-disk",
  supportsSegments: false,
  description: "QEMU/KVM virtual disk format",
  vendor: "QEMU",
};

/**
 * ISO 9660
 */
export const FORMAT_ISO: ContainerFormatDef = {
  id: "iso",
  displayName: "ISO 9660 Disc Image",
  typeName: "ISO",
  extensions: ["iso"],
  category: "optical-disc",
  supportsSegments: false,
  description: "Optical disc image (CD/DVD/BD)",
};

/**
 * macOS DMG
 */
export const FORMAT_DMG: ContainerFormatDef = {
  id: "dmg",
  displayName: "Apple Disk Image",
  typeName: "DMG",
  extensions: ["dmg"],
  category: "optical-disc",
  supportsSegments: false,
  description: "macOS disk image format",
  vendor: "Apple",
};

// =============================================================================
// ALL FORMATS REGISTRY
// =============================================================================

/**
 * All supported formats
 */
export const ALL_FORMATS: readonly ContainerFormatDef[] = [
  // Forensic containers
  FORMAT_E01,
  FORMAT_EX01,
  FORMAT_L01,
  FORMAT_LX01,
  FORMAT_AD1,
  FORMAT_AFF,
  FORMAT_AFF4,
  // Raw images
  FORMAT_RAW,
  // Mobile forensic
  FORMAT_UFED_UFD,
  FORMAT_UFED_UFDR,
  // Archives
  FORMAT_7Z,
  FORMAT_ZIP,
  // Virtual disks
  FORMAT_VMDK,
  FORMAT_VHD,
  FORMAT_VHDX,
  FORMAT_QCOW2,
  // Optical discs
  FORMAT_ISO,
  FORMAT_DMG,
] as const;

// =============================================================================
// FORMAT DETECTION UTILITIES
// =============================================================================

/**
 * Detect format by file extension
 * @param path File path or filename
 * @returns Matching format definition or undefined
 */
export function detectFormatByExtension(path: string): ContainerFormatDef | undefined {
  const lower = path.toLowerCase();
  const lastDot = lower.lastIndexOf(".");
  if (lastDot === -1) return undefined;

  const ext = lower.slice(lastDot + 1);

  // Search all formats
  for (const format of ALL_FORMATS) {
    if (format.extensions.includes(ext)) {
      return format;
    }
  }

  // Handle numbered segments (.001, .002, etc.)
  if (ext.length === 3 && /^\d{3}$/.test(ext)) {
    return FORMAT_RAW;
  }

  // Handle E01 extended segments (.e10 - .e99)
  if (ext.length === 3 && ext.startsWith("e") && /^\d{2}$/.test(ext.slice(1))) {
    return FORMAT_E01;
  }

  // Handle L01 extended segments (.l10 - .l99)
  if (ext.length === 3 && ext.startsWith("l") && /^\d{2}$/.test(ext.slice(1))) {
    return FORMAT_L01;
  }

  // Handle AD1 extended segments (.ad10 - .ad99)
  if (ext.length === 4 && ext.startsWith("ad") && /^\d{2}$/.test(ext.slice(2))) {
    return FORMAT_AD1;
  }

  return undefined;
}

/**
 * Get format by ID
 */
export function getFormatById(id: string): ContainerFormatDef | undefined {
  return ALL_FORMATS.find((f) => f.id === id);
}

/**
 * Get all formats in a category
 */
export function getFormatsByCategory(category: FormatCategory): ContainerFormatDef[] {
  return ALL_FORMATS.filter((f) => f.category === category);
}

/**
 * Get display name for a file
 */
export function getFormatDisplayName(path: string): string {
  return detectFormatByExtension(path)?.displayName ?? "Unknown Format";
}

/**
 * Get short type name for a file
 */
export function getFormatTypeName(path: string): string {
  return detectFormatByExtension(path)?.typeName ?? "Unknown";
}

/**
 * Get category info for a file
 */
export function getFormatCategoryInfo(path: string): FormatCategoryInfo {
  const format = detectFormatByExtension(path);
  const categoryId = format?.category ?? "unknown";
  return FORMAT_CATEGORIES[categoryId];
}

/**
 * Check if a file is a recognized forensic format
 */
export function isRecognizedFormat(path: string): boolean {
  return detectFormatByExtension(path) !== undefined;
}

/**
 * Check if format supports multi-segment files
 */
export function supportsSegments(path: string): boolean {
  return detectFormatByExtension(path)?.supportsSegments ?? false;
}

// =============================================================================
// EXTENSION PATTERN UTILITIES
// =============================================================================

/**
 * Get all extensions for a format category
 */
export function getExtensionsForCategory(category: FormatCategory): string[] {
  return getFormatsByCategory(category).flatMap((f) => [...f.extensions]);
}

/**
 * Get glob patterns for a format category (for file dialogs)
 */
export function getGlobPatternsForCategory(category: FormatCategory): string[] {
  return getExtensionsForCategory(category).map((ext) => `*.${ext}`);
}

/**
 * Get all forensic container extensions
 */
export function getForensicContainerExtensions(): string[] {
  return getExtensionsForCategory("forensic-container");
}

/**
 * Build file filter for open dialog
 */
export function buildFileFilter(categories: FormatCategory[]): { name: string; extensions: string[] }[] {
  return categories.map((cat) => ({
    name: FORMAT_CATEGORIES[cat].name,
    extensions: getExtensionsForCategory(cat),
  }));
}
