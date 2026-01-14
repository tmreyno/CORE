// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Evidence Utilities - Helper functions for evidence grouping and detection
 */

import type { DiscoveredFile, ContainerInfo } from "../../../../types";
import type { EvidenceType } from "../../types";
import type { EvidenceGroup } from "../types";

// =============================================================================
// EVIDENCE GROUPING
// =============================================================================

/**
 * Groups multi-segment container files by base name.
 * 
 * Forensic containers like E01, L01, AD1 can span multiple segments:
 * - E01: file.E01, file.E02, ..., file.E99, file.Ex00, file.Ex01, ...
 * - L01: file.L01, file.L02, ...
 * - AD1: file.ad1, file.ad2, ...
 * 
 * This function groups segments together so they appear as a single evidence item.
 */
export function groupEvidenceFiles(files: DiscoveredFile[]): EvidenceGroup[] {
  // Map to group files by base name
  const groups = new Map<string, EvidenceGroup>();
  
  // Regex patterns for segment files
  const segmentPatterns = [
    // E01 format: .E01-.E99, .Ex00-.Ex99
    /^(.+)\.(E|Ex)(\d{2,})$/i,
    // L01 format: .L01-.L99, .Lx00-.Lx99
    /^(.+)\.(L|Lx)(\d{2,})$/i,
    // AD1 format: .ad1, .ad2, ...
    /^(.+)\.(ad)(\d+)$/i,
    // S01 format: .s01, .s02, ...
    /^(.+)\.(s)(\d{2,})$/i,
  ];
  
  for (const file of files) {
    let baseName: string | null = null;
    let segmentNum = 1;
    
    // Check if this is a segment file
    for (const pattern of segmentPatterns) {
      const match = file.filename.match(pattern);
      if (match) {
        baseName = match[1];
        segmentNum = parseInt(match[3], 10);
        break;
      }
    }
    
    // Use filename as base if not a segment
    if (!baseName) {
      baseName = file.filename.replace(/\.[^.]+$/, "");
    }
    
    // Create group key from path directory + base name
    const pathDir = file.path.substring(0, file.path.lastIndexOf("/"));
    const groupKey = `${pathDir}/${baseName}`;
    
    if (!groups.has(groupKey)) {
      groups.set(groupKey, {
        primaryFile: file,
        segments: [],
        segmentCount: 0,
        totalSize: 0,
        baseName,
      });
    }
    
    const group = groups.get(groupKey)!;
    group.segments.push(file);
    group.segmentCount++;
    group.totalSize += file.size || 0;
    
    // Update primary file if this is segment 1 or earlier
    if (segmentNum === 1) {
      group.primaryFile = file;
    }
  }
  
  // Sort segments within each group and return
  return Array.from(groups.values()).map(group => ({
    ...group,
    segments: group.segments.sort((a, b) => a.filename.localeCompare(b.filename)),
  }));
}

// =============================================================================
// EVIDENCE TYPE DETECTION
// =============================================================================

/**
 * Detects the evidence type based on filename and container type.
 */
export function detectEvidenceType(file: DiscoveredFile, _info?: ContainerInfo): EvidenceType {
  const name = file.filename.toLowerCase();
  const type = file.container_type.toLowerCase();
  
  // Mobile devices
  if (type.includes("ufed") || type.includes("cellebrite")) return "MobilePhone";
  if (name.includes("tablet") || name.includes("ipad")) return "Tablet";
  
  // Removable storage
  if (name.includes("usb") || name.includes("thumb")) return "UsbDrive";
  if (name.includes("external")) return "ExternalDrive";
  if (name.includes("sd") || name.includes("memory")) return "MemoryCard";
  
  // Fixed storage
  if (name.includes("ssd")) return "SSD";
  if (name.includes("laptop")) return "Laptop";
  if (name.includes("computer") || name.includes("desktop")) return "Computer";
  
  // Forensic formats
  if (type.includes("e01") || type.includes("ad1") || type.includes("l01")) return "ForensicImage";
  
  // Other media
  if (name.includes("dvd") || name.includes("cd") || name.includes("iso")) return "OpticalDisc";
  if (name.includes("pcap") || name.includes("network")) return "NetworkCapture";
  if (name.includes("cloud") || name.includes("onedrive") || name.includes("gdrive")) return "CloudStorage";
  
  // Default
  return "HardDrive";
}

// =============================================================================
// DISPLAY HELPERS
// =============================================================================

/**
 * Gets a clean display name for a grouped evidence item.
 * For multi-segment containers, shows the base name with .X01 extension.
 */
export function getDisplayName(group: EvidenceGroup): string {
  if (group.segmentCount > 1) {
    // Remove segment extension to show clean base name
    const match = group.primaryFile.filename.match(/^(.+)\.(E|L|Ex|Lx|ad|s)\d{2,}$/i);
    if (match) {
      return `${match[1]}.${match[2].toUpperCase()}01`;
    }
  }
  return group.primaryFile.filename;
}

/**
 * Gets the total size for display, preferring metadata over file size sum.
 */
export function getDisplaySize(
  group: EvidenceGroup,
  info?: ContainerInfo
): number | undefined {
  if (!info) return group.totalSize;
  
  const ewfInfo = info.e01 || info.l01;
  const ad1Info = info.ad1;
  
  // For segmented containers, use metadata's total_size (actual image size)
  return ewfInfo?.total_size ?? ad1Info?.total_size ?? group.totalSize;
}

/**
 * Gets acquisition date from container info if available.
 */
export function getAcquisitionDate(info?: ContainerInfo): string | undefined {
  if (!info) return undefined;
  
  const ewfInfo = info.e01 || info.l01;
  const ad1Info = info.ad1;
  
  return ewfInfo?.acquiry_date ?? ad1Info?.companion_log?.acquisition_date ?? undefined;
}
