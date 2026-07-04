// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { HashSourceInput } from "../api/commands";
import type { DiscoveredFile } from "../types";
import type { SelectedEntry } from "./EvidenceTree/types";

export function buildEvidenceSourceInput(
  file: DiscoveredFile | null,
  entry?: SelectedEntry,
  pathOverride?: string,
): HashSourceInput | null {
  if (entry) {
    if (entry.isArchiveEntry && entry.entryPath.includes("::")) {
      const delimiterIndex = entry.entryPath.indexOf("::");
      const nestedArchivePath = entry.entryPath.slice(0, delimiterIndex);
      const nestedEntryPath = entry.entryPath.slice(delimiterIndex + 2);
      return {
        containerPath: entry.containerPath,
        nestedArchivePath,
        entryPath: nestedEntryPath,
        containerType: entry.containerType?.toLowerCase() ?? extensionOrDefault(nestedArchivePath, "archive"),
        size: entry.size,
      };
    }

    if (entry.isDiskFile) {
      return {
        path: pathOverride ?? entry.entryPath,
        entryPath: entry.entryPath,
        containerType: "disk",
        size: entry.size,
      };
    }

    return {
      containerPath: entry.containerPath,
      entryPath: entry.entryPath,
      containerType: inferEntryContainerType(entry),
      size: entry.size,
    };
  }

  if (file) {
    return {
      path: pathOverride ?? file.path,
      containerType: "disk",
      size: file.size,
    };
  }

  return null;
}

function inferEntryContainerType(entry: SelectedEntry): string {
  const explicitType = entry.containerType?.toLowerCase();
  if (explicitType && explicitType !== "vfs" && explicitType !== "lazy") return explicitType;
  if (entry.isArchiveEntry) return extensionOrDefault(entry.containerPath, "archive");
  if (entry.isVfsEntry) return extensionOrDefault(entry.containerPath, "e01");
  return "ad1";
}

function extensionOrDefault(path: string, fallback: string): string {
  const name = path.split(/[\\/]/).pop() ?? path;
  const lowerName = name.toLowerCase();
  for (const compoundExtension of ["tar.gz", "tar.bz2", "tar.xz", "tar.zst", "tar.lz4"]) {
    if (lowerName.endsWith(`.${compoundExtension}`)) return compoundExtension;
  }
  const dot = name.lastIndexOf(".");
  if (dot < 0 || dot === name.length - 1) return fallback;
  const extension = name.slice(dot + 1).toLowerCase();
  if (/^\d+$/.test(extension)) return fallback;
  return extension;
}
