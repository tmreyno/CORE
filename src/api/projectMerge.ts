// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";

// =============================================================================
// Types (match Rust merge module)
// =============================================================================

export interface ProjectMergeSummary {
  cffxPath: string;
  ffxdbPath: string;
  ffxdbExists: boolean;
  name: string;
  projectId: string;
  /** Owner/Examiner name from project owner_name field */
  ownerName: string | null;
  createdAt: string;
  savedAt: string;
  evidenceFileCount: number;
  hashCount: number;
  sessionCount: number;
  activityCount: number;
  bookmarkCount: number;
  noteCount: number;
  tabCount: number;
  reportCount: number;
  rootPath: string;
  /** Examiners/users found in .cffx and .ffxdb */
  examiners: MergeExaminerInfo[];
  /** Evidence collection summaries from .ffxdb */
  collections: MergeCollectionSummary[];
  /** COC item summaries from .ffxdb */
  cocItems: MergeCocSummary[];
  /** Form submission summaries from .ffxdb */
  formSubmissions: MergeFormSummary[];
  /** Evidence file summaries from .ffxdb */
  evidenceFiles: MergeEvidenceFileSummary[];
}

/** Examiner/user found in .cffx or .ffxdb */
export interface MergeExaminerInfo {
  name: string;
  displayName: string | null;
  /** Where this name was found: "cffx" or "ffxdb" */
  source: string;
  /** Role context: "project owner", "session user", "collecting officer", etc. */
  role: string;
}

/** Evidence collection summary from .ffxdb */
export interface MergeCollectionSummary {
  id: string;
  caseNumber: string;
  collectionDate: string;
  collectingOfficer: string;
  collectionLocation: string;
  status: string;
  itemCount: number;
}

/** COC item summary from .ffxdb */
export interface MergeCocSummary {
  id: string;
  cocNumber: string;
  caseNumber: string;
  evidenceId: string;
  description: string;
  submittedBy: string;
  receivedBy: string;
  status: string;
}

/** Form submission summary from .ffxdb */
export interface MergeFormSummary {
  id: string;
  templateId: string;
  caseNumber: string | null;
  status: string;
  createdAt: string;
  /** Collecting officer (from data_json for evidence_collection templates) */
  collectingOfficer: string | null;
  /** Collection location (from data_json for evidence_collection templates) */
  collectionLocation: string | null;
  /** Lead examiner (from data_json for IAR/activity templates) */
  leadExaminer: string | null;
}

/** Evidence file summary from .ffxdb */
export interface MergeEvidenceFileSummary {
  id: string;
  path: string;
  filename: string;
  containerType: string;
  totalSize: number;
}

/** Owner assignment for a source project during merge */
export interface MergeSourceAssignment {
  cffxPath: string;
  ownerName: string;
}

/** Provenance record for a merged source project */
export interface MergeSource {
  sourceProjectId: string;
  sourceProjectName: string;
  sourceCffxPath: string;
  ownerName: string;
  mergedAt: string;
  evidenceFileCount: number;
  sessionCount: number;
  activityCount: number;
  bookmarkCount: number;
  noteCount: number;
}

export interface MergeStats {
  projectsMerged: number;
  usersMerged: number;
  sessionsMerged: number;
  activityEntriesMerged: number;
  evidenceFilesMerged: number;
  hashesMerged: number;
  bookmarksMerged: number;
  notesMerged: number;
  tabsMerged: number;
  reportsMerged: number;
  tagsMerged: number;
  searchesMerged: number;
  ffxdbTablesMerged: number;
}

export interface MergeResult {
  success: boolean;
  cffxPath: string | null;
  ffxdbPath: string | null;
  error: string | null;
  stats: MergeStats | null;
  /** Provenance records for each source project */
  sources: MergeSource[] | null;
}

// =============================================================================
// API Functions
// =============================================================================

/** Analyze multiple .cffx files and return summaries for the merge wizard. */
export async function analyzeProjects(
  cffxPaths: string[],
): Promise<ProjectMergeSummary[]> {
  return invoke<ProjectMergeSummary[]>("project_merge_analyze", {
    cffxPaths,
  });
}

/** Execute a full project merge. */
export async function executeMerge(
  cffxPaths: string[],
  outputPath: string,
  mergedName: string,
  newRoot?: string,
  ownerAssignments?: MergeSourceAssignment[],
  excludeCollectionIds?: string[],
): Promise<MergeResult> {
  return invoke<MergeResult>("project_merge_execute", {
    cffxPaths,
    outputPath,
    mergedName,
    newRoot: newRoot ?? null,
    ownerAssignments: ownerAssignments ?? null,
    excludeCollectionIds: excludeCollectionIds ?? null,
  });
}
