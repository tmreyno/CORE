// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Toolbar Helpers - Utility functions and constants for toolbar
 */

import type { Accessor } from "solid-js";
import type { ProjectLocation } from "./ProjectLocationSelector";

/**
 * Build project locations array from project paths.
 * 
 * Falls back to scanDir when project locations are not set
 * (common for projects created before the setup wizard or saved without it).
 */
export const buildProjectLocations = (
  evidencePath: Accessor<string | null> | undefined,
  processedDbPath: Accessor<string | null> | undefined,
  caseDocumentsPath: Accessor<string | null> | undefined,
  /** Fallback: current scan directory (root_path from project) */
  scanDir?: string,
): ProjectLocation[] => {
  const locations: ProjectLocation[] = [];
  
  const evidence = evidencePath?.();
  const processed = processedDbPath?.();
  const caseDocs = caseDocumentsPath?.();
  
  if (evidence) {
    locations.push({ id: "evidence", label: "Evidence", path: evidence, icon: "evidence" });
  } else if (scanDir) {
    // Fall back to scan directory when project doesn't have explicit locations
    locations.push({ id: "evidence", label: "Evidence", path: scanDir, icon: "evidence" });
  }
  if (processed) {
    locations.push({ id: "processed", label: "Processed Database", path: processed, icon: "database" });
  }
  if (caseDocs) {
    locations.push({ id: "documents", label: "Case Documents", path: caseDocs, icon: "documents" });
  }
  
  return locations;
};
