// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Tab Helper Utilities
 * 
 * Utility functions for tab management in CenterPane.
 */

import type { CenterTabType } from "./CenterPane";

export function getTabTypeColor(type: CenterTabType): string {
  switch (type) {
    case "evidence": return "text-type-ad1";
    case "document": return "text-accent";
    case "entry": return "text-type-e01";
    case "export": return "text-warning";
    case "processed": return "text-success";
    case "help": return "text-info";
    default: return "text-txt-muted";
  }
}
