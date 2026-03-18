// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useTriageState — Forensic triage collection + credential scanning state and handler.
 */

import { createSignal } from "solid-js";
import {
  getTriageProfiles,
  triageCollect,
  triageCancel,
  listenTriageProgress,
  type TriageProfile,
  type TriageCategory,
  type TriageProgress,
  type TriageResult,
} from "../../api/triage";
import { getErrorMessage } from "../../utils/errorUtils";
import { logger } from "../../utils/logger";

const log = logger.scope("Triage");
import {
  createActivity,
  completeActivity,
  failActivity,
} from "../../types/activity";
import type { ExportToast, ExportActivityCallbacks } from "./types";
import type { ExportCommonState } from "./useExportCommon";
import { dbSync } from "../project/useProjectDbSync";
import type { DbExportRecord } from "../../types/projectDb";
import { handleAcquisitionComplete } from "./companionHelper";

export interface UseTriageStateOptions extends ExportActivityCallbacks {
  toast: ExportToast;
  common: ExportCommonState;
  /** Case number for companion file + evidence collection record */
  caseNumber?: string;
  /** Examiner name for companion file + evidence collection record */
  examinerName?: string;
}

export function useTriageState(options: UseTriageStateOptions) {
  const { toast, common } = options;

  // === Triage State ===
  const [triageProfiles, setTriageProfiles] = createSignal<TriageProfile[]>([]);
  const [triageCategories, setTriageCategories] = createSignal<TriageCategory[]>([]);
  const [triageProfilesLoading, setTriageProfilesLoading] = createSignal(false);
  const [selectedTriageProfile, setSelectedTriageProfile] = createSignal<string>("full_triage");
  const [selectedTriageCategories, setSelectedTriageCategories] = createSignal<string[]>([]);
  const [triageScanForSecrets, setTriageScanForSecrets] = createSignal(true);
  const [triageProgress, setTriageProgress] = createSignal<TriageProgress | null>(null);
  const [triageResult, setTriageResult] = createSignal<TriageResult | null>(null);

  // ─── Load Profiles ──────────────────────────────────────────────────────

  const loadTriageProfiles = async () => {
    setTriageProfilesLoading(true);
    try {
      const [profiles, categories] = await getTriageProfiles();
      setTriageProfiles(profiles);
      setTriageCategories(categories);
      // Select "full_triage" profile categories by default
      const full = profiles.find((p) => p.id === "full_triage");
      if (full) {
        setSelectedTriageCategories(full.categories);
      } else if (categories.length > 0) {
        setSelectedTriageCategories(categories.map((c) => c.id));
      }
    } catch (err) {
      toast.error("Triage", `Failed to load triage profiles: ${getErrorMessage(err)}`);
    } finally {
      setTriageProfilesLoading(false);
    }
  };

  // ─── Apply Profile ──────────────────────────────────────────────────────

  const applyTriageProfile = (profileId: string) => {
    setSelectedTriageProfile(profileId);
    const profile = triageProfiles().find((p) => p.id === profileId);
    if (profile) {
      setSelectedTriageCategories(profile.categories);
    }
  };

  // ─── Toggle Category ────────────────────────────────────────────────────

  const toggleTriageCategory = (categoryId: string) => {
    setSelectedTriageCategories((prev) => {
      if (prev.includes(categoryId)) {
        return prev.filter((c) => c !== categoryId);
      }
      return [...prev, categoryId];
    });
    // When manually toggling, switch profile to "custom" feel
    setSelectedTriageProfile("custom");
  };

  // ─── Collect Handler ────────────────────────────────────────────────────

  const handleTriageCollect = async () => {
    const dest = common.destination();
    if (!dest) {
      toast.error("No Destination", "Please select a destination folder");
      return;
    }

    const cats = selectedTriageCategories();
    log.info(`Starting triage collection: ${cats.length} categories, secrets=${triageScanForSecrets()}, profile=${selectedTriageProfile()}, dest=${dest}`);
    if (cats.length === 0) {
      toast.error("No Categories", "Please select at least one artifact category");
      return;
    }

    common.setIsProcessing(true);
    common.setIsAcquiring(true);
    setTriageProgress(null);
    setTriageResult(null);

    const activity = createActivity("export", dest, cats.length, {
      operation: "Forensic Triage Collection",
    });
    options.onActivityCreate?.(activity);

    // Listen for progress events — set up BEFORE backend call to avoid race
    let unlisten: (() => void) | undefined;
    try {
      unlisten = await listenTriageProgress((progress) => {
        setTriageProgress(progress);
      });
    } catch (err) {
      console.warn("Failed to set up triage progress listener:", err);
      toast.warning("Progress Unavailable", "Triage collection will proceed but progress updates may not display");
    }

    // Track in DB
    const exportId = `triage-${Date.now()}`;
    const dbRecord: DbExportRecord = {
      id: exportId,
      exportType: "triage",
      sourcePathsJson: JSON.stringify(cats),
      destination: dest,
      status: "in_progress",
      startedAt: new Date().toISOString(),
      initiatedBy: "",
      totalFiles: 0,
      totalBytes: 0,
      encrypted: false,
      optionsJson: JSON.stringify({
        categories: cats,
        scanForSecrets: triageScanForSecrets(),
        profile: selectedTriageProfile(),
      }),
    };
    dbSync.insertExport(dbRecord);

    try {
      const result = await triageCollect({
        outputDir: dest,
        categories: cats,
        scanForSecrets: triageScanForSecrets(),
      });
      setTriageResult(result);

      const sizeMb = (result.bytesCollected / (1024 * 1024)).toFixed(1);
      const durationStr = result.durationSecs < 60
        ? `${result.durationSecs.toFixed(1)}s`
        : `${Math.floor(result.durationSecs / 60)}m ${Math.floor(result.durationSecs % 60)}s`;

      log.info(`Triage ${result.cancelled ? "cancelled" : "complete"}: ${result.filesCollected} files (${sizeMb} MB) in ${durationStr}`);
      if (result.cancelled) {
        toast.warning("Triage Cancelled", `Collected ${result.filesCollected} files (${sizeMb} MB) before cancellation`);
      } else {
        const secretsMsg = result.secretFindings.length > 0
          ? ` — ${result.secretFindings.length} credential(s) found`
          : "";
        toast.success(
          "Triage Complete",
          `${result.filesCollected} files (${sizeMb} MB) collected in ${durationStr}${secretsMsg}`,
        );
      }

      completeActivity(activity);
      options.onActivityUpdate?.(activity.id, activity);

      dbSync.updateExport({
        ...dbRecord,
        status: result.cancelled ? "cancelled" : "completed",
        completedAt: new Date().toISOString(),
        totalFiles: result.filesCollected,
        totalBytes: result.bytesCollected,
      });

      if (!result.cancelled) {
        options.onComplete?.(dest);

        // Build rich auto-generated description and notes for evidence collection
        const categoryNames = result.categoriesCollected.length > 0
          ? result.categoriesCollected.join(", ")
          : cats.join(", ");
        const secretsSummary = result.secretFindings.length > 0
          ? ` ${result.secretFindings.length} credential/secret finding(s) detected.`
          : "";
        const autoDescription = `Forensic triage collection — ${result.filesCollected} files (${sizeMb} MB) from ${result.categoriesCollected.length || cats.length} categories`;
        const autoNotes = `Categories: ${categoryNames}. Profile: ${selectedTriageProfile()}.${secretsSummary}`;

        handleAcquisitionComplete({
          acquisitionType: "triage",
          outputPath: dest,
          sources: cats,
          format: "triage_collection",
          totalBytes: result.bytesCollected,
          totalFiles: result.filesCollected,
          startedAt: dbRecord.startedAt,
          completedAt: new Date().toISOString(),
          durationMs: result.durationSecs * 1000,
          caseNumber: options.caseNumber,
          examiner: options.examinerName,
          description: autoDescription,
          notes: autoNotes,
        });
      }
    } catch (err) {
      const msg = getErrorMessage(err);
      log.error(`Triage collection failed: ${msg}`);
      toast.error("Triage Failed", msg);

      failActivity(activity, msg);
      options.onActivityUpdate?.(activity.id, activity);

      dbSync.updateExport({
        ...dbRecord,
        status: "failed",
        completedAt: new Date().toISOString(),
        error: msg,
      });
    } finally {
      unlisten?.();
      common.setIsProcessing(false);
      common.setIsAcquiring(false);
    }
  };

  // ─── Cancel ─────────────────────────────────────────────────────────────

  const handleCancelTriage = async () => {
    try {
      await triageCancel();
      toast.info("Cancelling", "Triage collection will stop after the current file");
    } catch (err) {
      toast.error("Cancel Failed", getErrorMessage(err));
    }
  };

  // ─── Reset ──────────────────────────────────────────────────────────────

  const resetTriageState = () => {
    setTriageProgress(null);
    setTriageResult(null);
    setSelectedTriageProfile("full_triage");
    setTriageScanForSecrets(true);
  };

  return {
    triageProfiles,
    triageCategories,
    triageProfilesLoading,
    selectedTriageProfile,
    setSelectedTriageProfile: applyTriageProfile,
    selectedTriageCategories,
    toggleTriageCategory,
    triageScanForSecrets,
    setTriageScanForSecrets,
    triageProgress,
    triageResult,
    loadTriageProfiles,
    handleTriageCollect,
    handleCancelTriage,
    resetTriageState,
  };
}

export type TriageState = ReturnType<typeof useTriageState>;
