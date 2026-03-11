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
  MergeCollectionSummary,
  ProjectMergeSummary,
  MergeResult,
  MergeSourceAssignment,
  MergeExclusions,
  MergeDataCategory,
  MergeCocSummary,
  MergeFormSummary,
  MergeEvidenceFileSummary,
} from "../../api/projectMerge";

// Re-export API types for convenience
export type {
  MergeExaminerInfo,
  MergeCollectionSummary,
  ProjectMergeSummary,
  MergeResult,
  MergeSourceAssignment,
  MergeExclusions,
  MergeDataCategory,
  MergeCocSummary,
  MergeFormSummary,
  MergeEvidenceFileSummary,
};

export interface MergeProjectsWizardProps {
  onClose: () => void;
  /** Callback after successful merge — load the merged project */
  onMergeComplete?: (cffxPath: string) => void;
  /**
   * When set, the wizard operates in "merge into open project" mode:
   *  - The current project is pinned in the select step (can't be removed)
   *  - Only 1 additional project is required (instead of 2+)
   *  - Output path defaults to the current project's location
   *  - After merge, the current project is reloaded
   */
  currentProjectPath?: string;
}

export type WizardStep = "select" | "review" | "merging" | "complete";

/** Global examiner with list of which projects they appear in */
export type GlobalExaminer = MergeExaminerInfo & { projects: string[] };

// ---------------------------------------------------------------------------
// Collection Reconciliation Types
// ---------------------------------------------------------------------------

/** A potential duplicate pair between collections from different projects */
export interface CollectionConflict {
  /** Unique key for this conflict */
  key: string;
  /** Collection from the "current" (target) project */
  current: CollectionWithSource;
  /** Collection from the "incoming" (source) project */
  incoming: CollectionWithSource;
  /** Match reason: what made these look like duplicates */
  matchReason: string;
}

/** A collection annotated with its source project name */
export interface CollectionWithSource {
  collection: MergeCollectionSummary;
  projectName: string;
  cffxPath: string;
}

/** User's choice for how to resolve a collection conflict */
export type ConflictResolution = "keep-current" | "use-incoming";

/** Map of conflict key → resolution choice */
export type ReconciliationChoices = Record<string, ConflictResolution>;
