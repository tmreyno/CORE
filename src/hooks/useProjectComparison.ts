// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { invoke } from "@tauri-apps/api/core";
import { createSignal } from "solid-js";

/**
 * Project comparison result from backend
 */
export interface ProjectComparison {
  project_a_name: string;
  project_b_name: string;
  compared_at: string;
  summary: ComparisonSummary;
  bookmarks: BookmarkDiff;
  notes: NoteDiff;
  evidence: EvidenceDiff;
  activity: ActivityDiff;
  conflicts: MergeConflict[];
  recommendations: string[];
}

export interface ComparisonSummary {
  unique_to_a: number;
  unique_to_b: number;
  common: number;
  modified: number;
  similarity_percent: number;
  total_differences: number;
}

export interface BookmarkDiff {
  only_in_a: string[];
  only_in_b: string[];
  common: string[];
  modified: string[];
}

export interface NoteDiff {
  only_in_a: string[];
  only_in_b: string[];
  common: string[];
  modified: string[];
}

export interface EvidenceDiff {
  only_in_a: string[];
  only_in_b: string[];
  common: string[];
}

export interface ActivityDiff {
  count_a: number;
  count_b: number;
  types_only_in_a: string[];
  types_only_in_b: string[];
  common_types: string[];
}

/**
 * Merge conflict from backend
 */
export interface MergeConflict {
  conflict_type: ConflictType;
  description: string;
  item_a: any;
  item_b: any;
  resolution: string;
}

export type ConflictType =
  | "bookmark_conflict"
  | "note_conflict"
  | "evidence_conflict"
  | "metadata_conflict";

/**
 * Merge result from backend
 */
export interface MergeResult {
  merged_project: any; // FFXProject
  conflicts: MergeConflict[];
  summary: string;
  items_merged: number;
  items_skipped: number;
}

/**
 * Merge strategy options
 */
export type MergeStrategy =
  | "prefer_a"
  | "prefer_b"
  | "keep_both"
  | "skip"
  | "manual";

/**
 * Hook for project comparison and merging
 */
export function useProjectComparison() {
  const [comparison, setComparison] = createSignal<ProjectComparison | null>(null);
  const [mergeResult, setMergeResult] = createSignal<MergeResult | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  /**
   * Compare two projects
   */
  const compareProjects = async (projectAPath: string, projectBPath: string) => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<ProjectComparison>("project_compare", {
        projectA: projectAPath,
        projectB: projectBPath,
      });
      setComparison(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to compare projects:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Merge two projects with specified strategy
   */
  const mergeProjects = async (
    projectAPath: string,
    projectBPath: string,
    strategy: MergeStrategy = "manual"
  ) => {
    try {
      setLoading(true);
      setError(null);
      const result = await invoke<MergeResult>("project_merge", {
        projectA: projectAPath,
        projectB: projectBPath,
        strategy,
      });
      setMergeResult(result);
      return result;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to merge projects:", err);
      return null;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Sync bookmarks from source to target
   */
  const syncBookmarks = async (
    targetPath: string,
    sourcePath: string,
    overwrite: boolean = false
  ): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke<void>("project_sync_bookmarks", {
        target: targetPath,
        source: sourcePath,
        overwrite,
      });
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to sync bookmarks:", err);
      return false;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Sync notes from source to target
   */
  const syncNotes = async (
    targetPath: string,
    sourcePath: string,
    overwrite: boolean = false
  ): Promise<boolean> => {
    try {
      setLoading(true);
      setError(null);
      await invoke<void>("project_sync_notes", {
        target: targetPath,
        source: sourcePath,
        overwrite,
      });
      return true;
    } catch (err) {
      const message = err instanceof Error ? err.message : String(err);
      setError(message);
      console.error("Failed to sync notes:", err);
      return false;
    } finally {
      setLoading(false);
    }
  };

  /**
   * Get unique items in project A
   */
  const getUniqueToA = () => {
    const comp = comparison();
    if (!comp) return { bookmarks: [], notes: [], evidence: [] };
    return {
      bookmarks: comp.bookmarks.only_in_a,
      notes: comp.notes.only_in_a,
      evidence: comp.evidence.only_in_a,
    };
  };

  /**
   * Get unique items in project B
   */
  const getUniqueToB = () => {
    const comp = comparison();
    if (!comp) return { bookmarks: [], notes: [], evidence: [] };
    return {
      bookmarks: comp.bookmarks.only_in_b,
      notes: comp.notes.only_in_b,
      evidence: comp.evidence.only_in_b,
    };
  };

  /**
   * Get common items between projects
   */
  const getCommonItems = () => {
    const comp = comparison();
    if (!comp) return { bookmarks: [], notes: [], evidence: [] };
    return {
      bookmarks: comp.bookmarks.common,
      notes: comp.notes.common,
      evidence: comp.evidence.common,
    };
  };

  /**
   * Get modified items
   */
  const getModifiedItems = () => {
    const comp = comparison();
    if (!comp) return { bookmarks: [], notes: [] };
    return {
      bookmarks: comp.bookmarks.modified,
      notes: comp.notes.modified,
    };
  };

  /**
   * Get merge conflicts
   */
  const getConflicts = () => {
    const comp = comparison();
    return comp?.conflicts || [];
  };

  /**
   * Check if projects are similar (>80% similarity)
   */
  const areSimilar = () => {
    const comp = comparison();
    return comp ? comp.summary.similarity_percent > 80 : false;
  };

  return {
    // State
    comparison,
    mergeResult,
    loading,
    error,
    // Actions
    compareProjects,
    mergeProjects,
    syncBookmarks,
    syncNotes,
    // Utilities
    getUniqueToA,
    getUniqueToB,
    getCommonItems,
    getModifiedItems,
    getConflicts,
    areSimilar,
  };
}
