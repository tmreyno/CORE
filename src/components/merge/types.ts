// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Types for MergeProjectsWizard and its sub-components.
 */

import type {
  MergeExaminerInfo,
  ProjectMergeSummary,
  MergeResult,
  MergeSourceAssignment,
} from "../../api/projectMerge";

// Re-export API types for convenience
export type {
  MergeExaminerInfo,
  ProjectMergeSummary,
  MergeResult,
  MergeSourceAssignment,
};

export interface MergeProjectsWizardProps {
  onClose: () => void;
  /** Callback after successful merge — load the merged project */
  onMergeComplete?: (cffxPath: string) => void;
}

export type WizardStep = "select" | "review" | "merging" | "complete";

/** Global examiner with list of which projects they appear in */
export type GlobalExaminer = MergeExaminerInfo & { projects: string[] };
