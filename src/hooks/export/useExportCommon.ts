// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useExportCommon — Shared export state: mode, sources, destination, drive
 * management, and processing flag.
 */

import { createSignal } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import { getErrorMessage } from "../../utils/errorUtils";
import { remountReadOnly, restoreMount, checkPathWritable } from "../../api/drives";
import type { NativeExportTab } from "../../components/export/NativeExportMode";
import type { ExportMode, ExportToast } from "./types";

export interface UseExportCommonOptions {
  initialSources?: string[];
  toast: ExportToast;
}

export function useExportCommon(options: UseExportCommonOptions) {
  const { toast } = options;

  // === Core State ===
  const [mode, setMode] = createSignal<ExportMode>("native");
  const [nativeExportTab, setNativeExportTab] = createSignal<NativeExportTab>("files");
  const [sources, setSources] = createSignal<string[]>(options.initialSources || []);
  const [destination, setDestination] = createSignal("");
  const [isProcessing, setIsProcessing] = createSignal(false);
  const [showAdvanced, setShowAdvanced] = createSignal(false);

  // === Drive Selector State ===
  const [showDriveSelector, setShowDriveSelector] = createSignal(false);
  const [driveSources, setDriveSources] = createSignal<Set<string>>(new Set());
  const [mountDrivesReadOnly, setMountDrivesReadOnly] = createSignal(false);

  // ─── Source Management ──────────────────────────────────────────────────

  /** Add paths to sources, skipping duplicates */
  const addUniqueSources = (newPaths: string[]) => {
    const existing = new Set(sources());
    const unique = newPaths.filter((p) => !existing.has(p));
    if (unique.length > 0) {
      setSources([...sources(), ...unique]);
    }
    if (unique.length < newPaths.length) {
      const skipped = newPaths.length - unique.length;
      toast.warning(
        "Duplicate Sources",
        `${skipped} source${skipped > 1 ? "s" : ""} already in list — skipped`,
      );
    }
  };

  const handleAddSources = async () => {
    const selected = await open({
      multiple: true,
      directory: false,
      title: "Select Files to Export",
    });

    if (selected) {
      const newSources = Array.isArray(selected) ? selected : [selected];
      addUniqueSources(newSources);
    }
  };

  const handleAddFolder = async () => {
    const selected = await open({
      multiple: false,
      directory: true,
      title: "Select Folder to Export",
    });

    if (selected) {
      addUniqueSources([selected as string]);
    }
  };

  /** Handle drive selection from DriveSelector modal */
  const handleDriveSelected = (paths: string[], mountReadOnly: boolean) => {
    addUniqueSources(paths);
    setDriveSources((prev) => {
      const next = new Set(prev);
      paths.forEach((p) => next.add(p));
      return next;
    });
    setMountDrivesReadOnly(mountReadOnly);
    setShowDriveSelector(false);
  };

  const handleSelectDestination = async () => {
    const selected = await open({
      multiple: false,
      directory: true,
      title: "Select Destination Folder",
    });

    if (selected) {
      // Check if the destination is writable before accepting it
      try {
        const check = await checkPathWritable(selected as string);
        if (!check.writable) {
          toast.error("Read-Only Destination", check.reason);
          return;
        }
      } catch {
        // If the check itself fails, warn but allow selection
      }
      setDestination(selected as string);
    }
  };

  const handleRemoveSource = (index: number) => {
    const removed = sources()[index];
    setSources(sources().filter((_, i) => i !== index));
    if (removed && driveSources().has(removed)) {
      setDriveSources((prev) => {
        const next = new Set(prev);
        next.delete(removed);
        return next;
      });
    }
  };

  /** Clear all sources and drive tracking */
  const clearAllSources = () => {
    setSources([]);
    setDriveSources(new Set<string>());
    setMountDrivesReadOnly(false);
  };

  /** Whether any source is a drive/volume */
  const hasDriveSources = () => driveSources().size > 0;

  // ─── Drive Mount Helpers ────────────────────────────────────────────────

  /** Restore all drive mounts to their original state after imaging. */
  const restoreAllDriveMounts = async () => {
    const drives = Array.from(driveSources());
    for (const mp of drives) {
      try {
        const result = await restoreMount(mp);
        if (result.success) {
          toast.success("Mount Restored", result.message);
        }
      } catch (err: unknown) {
        toast.error("Mount Restore Failed", getErrorMessage(err));
      }
    }
  };

  /** Remount selected drives as read-only. Returns false if any fail. */
  const remountDrivesIfNeeded = async (): Promise<boolean> => {
    if (!hasDriveSources() || !mountDrivesReadOnly()) return true;

    const drives = Array.from(driveSources());
    for (const mp of drives) {
      try {
        const result = await remountReadOnly(mp);
        if (result.success) {
          toast.success("Mounted Read-Only", result.message);
        } else {
          toast.error("Mount Failed", result.message);
          return false;
        }
      } catch (err: unknown) {
        toast.error(
          "Read-Only Mount Failed",
          `Could not remount ${mp} as read-only: ${getErrorMessage(err)}. Export aborted.`,
        );
        await restoreAllDriveMounts();
        return false;
      }
    }
    return true;
  };

  return {
    // Core state
    mode,
    setMode,
    nativeExportTab,
    setNativeExportTab,
    sources,
    setSources,
    destination,
    setDestination,
    isProcessing,
    setIsProcessing,
    showAdvanced,
    setShowAdvanced,

    // Drive selector state
    showDriveSelector,
    setShowDriveSelector,
    driveSources,
    mountDrivesReadOnly,

    // Source handlers
    addUniqueSources,
    handleAddSources,
    handleAddFolder,
    handleDriveSelected,
    handleSelectDestination,
    handleRemoveSource,
    clearAllSources,
    hasDriveSources,

    // Drive mount helpers
    restoreAllDriveMounts,
    remountDrivesIfNeeded,
  } as const;
}

export type ExportCommonState = ReturnType<typeof useExportCommon>;
