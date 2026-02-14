// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useActivityLogging — Tracks user activities and logs them to the project
 * 
 * Extracts activity logging effects from App.tsx that monitor:
 * - File selection changes (opening files)
 * - Hash computation completions
 * - Directory scan completions
 */

import { createEffect, on, type Accessor } from "solid-js";
import type { FileManager } from "./useFileManager";
import type { HashManager } from "./useHashManager";
import type { ActivityCategory } from "../types/project";
import type { Activity } from "../types/activity";

export interface UseActivityLoggingDeps {
  /** File manager — provides activeFile, discoveredFiles, scanDir */
  fileManager: Pick<FileManager, "activeFile" | "discoveredFiles" | "scanDir">;
  /** Hash manager — provides fileHashMap */
  hashManager: Pick<HashManager, "fileHashMap">;
  /** Project manager — provides logActivity */
  projectManager: {
    logActivity: (
      category: ActivityCategory,
      action: string,
      description: string,
      filePath?: string,
      details?: Record<string, unknown>,
    ) => void;
  };
  /** Runtime activities — tracks export/archive/copy operations (optional) */
  activities?: Accessor<Activity[]>;
}

/**
 * Sets up reactive effects that automatically log user activities.
 * 
 * Activities logged:
 * - `file/open` — when a new file is selected
 * - `hash/compute` — when a new hash is computed for a file
 * - `hash/verify` — when a hash verification completes
 * - `file/scan` — when files are discovered in a directory
 * - `export/complete` — when an export/archive/copy operation completes
 * - `export/fail` — when an export/archive/copy operation fails
 */
export function useActivityLogging(deps: UseActivityLoggingDeps): void {
  const { fileManager, hashManager, projectManager, activities } = deps;

  // Track file selection changes
  createEffect(on(
    () => fileManager.activeFile(),
    (file, prevFile) => {
      if (file && file.path !== prevFile?.path) {
        projectManager.logActivity(
          "file",
          "open",
          `Opened file: ${file.filename}`,
          file.path,
          { containerType: file.container_type, size: file.size },
        );
      }
    },
    { defer: true },
  ));

  // Track hash computation completions by watching fileHashMap changes.
  // We use a local ref for previous state because SolidJS deferred effects
  // pass `undefined` as `prev` on their first invocation, which would make
  // it impossible to distinguish "compute" from "verify".
  let prevHashMapRef: Map<string, import("../types/hash").FileHashInfo> | undefined;

  createEffect(on(
    () => hashManager.fileHashMap(),
    (hashMap) => {
      if (!hashMap || hashMap.size === 0) {
        prevHashMapRef = hashMap;
        return;
      }

      // Find newly added or updated entries
      hashMap.forEach((hashInfo, path) => {
        const prevInfo = prevHashMapRef?.get(path);
        const fileName = path.split("/").pop() || path;

        if (!prevInfo || (hashInfo.hash && !prevInfo.hash)) {
          // New hash computed
          projectManager.logActivity(
            "hash",
            "compute",
            `Computed ${hashInfo.algorithm} hash for: ${fileName}`,
            path,
            {
              algorithm: hashInfo.algorithm,
              hash: hashInfo.hash?.slice(0, 16) + "...",
              verified: hashInfo.verified,
            },
          );
        } else if (
          hashInfo.verified !== undefined &&
          hashInfo.verified !== null &&
          (prevInfo.verified === undefined || prevInfo.verified === null)
        ) {
          // Hash verification completed
          projectManager.logActivity(
            "hash",
            "verify",
            `Verified hash for: ${fileName} (${hashInfo.verified ? "MATCH" : "MISMATCH"})`,
            path,
            { algorithm: hashInfo.algorithm, verified: hashInfo.verified },
          );
        }
      });

      prevHashMapRef = hashMap;
    },
    { defer: true },
  ));

  // Track directory scans
  createEffect(on(
    () => fileManager.discoveredFiles().length,
    (count, prevCount) => {
      // Only log when files are discovered (not when cleared)
      if (count > 0 && (prevCount === undefined || prevCount === 0)) {
        const scanDir = fileManager.scanDir();
        projectManager.logActivity(
          "file",
          "scan",
          `Discovered ${count} evidence files in: ${scanDir.split("/").pop() || scanDir}`,
          scanDir,
          { fileCount: count },
        );
      }
    },
    { defer: true },
  ));

  // Track export/archive/copy completion by watching runtime activities
  if (activities) {
    // Keep a set of activity IDs we've already logged to avoid duplicates
    const loggedActivityIds = new Set<string>();

    createEffect(on(
      activities,
      (activityList) => {
        if (!activityList || activityList.length === 0) return;

        for (const activity of activityList) {
          // Skip if already logged
          if (loggedActivityIds.has(activity.id)) continue;

          const fileName = activity.destination.split("/").pop() || activity.destination;

          if (activity.status === "completed") {
            loggedActivityIds.add(activity.id);
            const durationMs = activity.endTime && activity.startTime
              ? activity.endTime.getTime() - activity.startTime.getTime()
              : undefined;

            projectManager.logActivity(
              "export",
              "complete",
              `${activity.type === "archive" ? "Archive created" : activity.type === "copy" ? "Files copied" : "Export completed"}: ${fileName}`,
              activity.destination,
              {
                type: activity.type,
                sourceCount: activity.sourceCount,
                durationMs,
                bytesProcessed: activity.progress?.bytesProcessed,
              },
            );
          } else if (activity.status === "failed") {
            loggedActivityIds.add(activity.id);
            projectManager.logActivity(
              "export",
              "fail",
              `${activity.type === "archive" ? "Archive creation" : activity.type === "copy" ? "File copy" : "Export"} failed: ${fileName}`,
              activity.destination,
              {
                type: activity.type,
                error: activity.error,
              },
            );
          } else if (activity.status === "cancelled") {
            loggedActivityIds.add(activity.id);
            projectManager.logActivity(
              "export",
              "cancel",
              `${activity.type === "archive" ? "Archive creation" : activity.type === "copy" ? "File copy" : "Export"} cancelled: ${fileName}`,
              activity.destination,
              { type: activity.type },
            );
          }
        }
      },
      { defer: true },
    ));
  }
}
