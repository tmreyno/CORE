// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Comparison Helper Utilities
 * 
 * Utilities for project comparison:
 * - Similarity color coding
 * - Merge strategy definitions
 */

import type { MergeStrategy } from "../../hooks/useProjectComparison";

export function getSimilarityColor(similarity: number): string {
  if (similarity >= 80) return "text-success";
  if (similarity >= 50) return "text-warning";
  return "text-error";
}

export const mergeStrategies: Array<{ value: MergeStrategy; label: string; desc: string }> = [
  {
    value: "PreferA",
    label: "Prefer A",
    desc: "Keep items from Project A when conflicts occur",
  },
  {
    value: "PreferB",
    label: "Prefer B",
    desc: "Keep items from Project B when conflicts occur",
  },
  {
    value: "KeepBoth",
    label: "Keep Both",
    desc: "Keep all items from both projects",
  },
  {
    value: "Skip",
    label: "Skip Conflicts",
    desc: "Skip conflicting items, keep only non-conflicting",
  },
  {
    value: "Manual",
    label: "Manual Review",
    desc: "Mark conflicts for manual resolution",
  },
];
