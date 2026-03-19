// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useAff4ExportState — AFF4 forensic container creation state and handler.
 */

import { createSignal } from "solid-js";
import { createAff4Image } from "../../api/aff4Export";
import type { Aff4ExportOptions, Aff4ExportProgress } from "../../api/aff4Export";
import { formatBytes } from "../../api/archiveCreate";
import { getErrorMessage } from "../../utils/errorUtils";
import { logger } from "../../utils/logger";

const log = logger.scope("AFF4Export");
import { joinPath } from "../../utils/pathUtils";
import {
  createActivity,
  updateProgress,
  completeActivity,
  failActivity,
} from "../../types/activity";
import type { ExportToast, ExportActivityCallbacks } from "./types";
import type { ExportCommonState } from "./useExportCommon";
import { dbSync } from "../project/useProjectDbSync";
import type { DbExportRecord } from "../../types/projectDb";
import { handleAcquisitionComplete, startAcquisitionRecord } from "./companionHelper";

interface UseAff4ExportStateOptions extends ExportActivityCallbacks {
  toast: ExportToast;
  common: ExportCommonState;
  caseNumber?: string;
  examinerName?: string;
  systemStats?: { hostname?: string; systemModel?: string; systemSerialNumber?: string; systemManufacturer?: string; osName?: string; osVersion?: string } | null;
}

export function useAff4ExportState(options: UseAff4ExportStateOptions) {
  const { toast, common } = options;

  // ── Signals ──────────────────────────────────────────────────────────────

  const [aff4ImageName, setAff4ImageName] = createSignal("evidence");
  const [aff4Compression, setAff4Compression] = createSignal("deflate");
  const [aff4HashAlgorithms, setAff4HashAlgorithms] = createSignal<string[]>(["sha256"]);

  // Case metadata
  const [aff4CaseNumber, setAff4CaseNumber] = createSignal("");
  const [aff4EvidenceNumber, setAff4EvidenceNumber] = createSignal("");
  const [aff4ExaminerName, setAff4ExaminerName] = createSignal("");
  const [aff4Description, setAff4Description] = createSignal("");
  const [aff4Notes, setAff4Notes] = createSignal("");

  // ── Handler ──────────────────────────────────────────────────────────────

  const handleCreateAff4Image = async () => {
    const sources = [...common.sources()];
    const destination = common.destination();
    const imageName = aff4ImageName().trim() || "evidence";
    log.info(`Starting AFF4 creation: ${imageName}.aff4, compression=${aff4Compression()}, hashes=${aff4HashAlgorithms().join(",")}, sources=${sources.length}`);

    if (sources.length === 0) {
      toast.error("No Sources", "Please select files or folders to export");
      return;
    }
    if (!destination) {
      toast.error("No Destination", "Please select a destination folder");
      return;
    }

    common.setIsProcessing(true);

    const outputPath = joinPath(destination, `${imageName}.aff4`);
    const startTime = Date.now();

    // Create activity
    const activity = createActivity("export", outputPath, sources.length, {
      operation: `AFF4 Container Creation (${aff4Compression()})`,
    });
    options.onActivityCreate?.(activity);

    // Track in DB
    const exportId = `aff4-${Date.now()}`;
    const exportRecord: DbExportRecord = {
      id: exportId,
      exportType: "aff4",
      sourcePathsJson: JSON.stringify(sources),
      destination: outputPath,
      startedAt: new Date().toISOString(),
      initiatedBy: aff4ExaminerName() || "",
      status: "in_progress",
      totalFiles: sources.length,
      totalBytes: 0,
      archiveName: `${imageName}.aff4`,
      archiveFormat: "AFF4",
      compressionLevel: aff4Compression(),
      encrypted: false,
    };
    dbSync.insertExport(exportRecord);

    const acqRecord = startAcquisitionRecord({
      acquisitionType: "archive",
      outputPath,
      sources,
      caseNumber: options.caseNumber,
      examiner: options.examinerName,
      hostname: options.systemStats?.hostname,
      systemModel: options.systemStats?.systemModel,
      systemSerialNumber: options.systemStats?.systemSerialNumber,
      systemManufacturer: options.systemStats?.systemManufacturer,
    });

    const aff4Options: Aff4ExportOptions = {
      sourcePaths: sources,
      outputPath,
      compression: aff4Compression(),
      hashAlgorithms: aff4HashAlgorithms(),
      caseNumber: aff4CaseNumber() || undefined,
      evidenceNumber: aff4EvidenceNumber() || undefined,
      examinerName: aff4ExaminerName() || undefined,
      description: aff4Description() || undefined,
      notes: aff4Notes() || undefined,
    };

    try {
      const result = await createAff4Image(aff4Options, (progress: Aff4ExportProgress) => {
        options.onActivityUpdate?.(
          activity.id,
          updateProgress(activity, {
            bytesProcessed: progress.bytesProcessed,
            bytesTotal: progress.totalBytes,
            percent: progress.percent,
            currentFile: progress.currentFile || undefined,
            filesProcessed: progress.filesProcessed,
            totalFiles: progress.totalFiles,
          }),
        );
      });

      const sizeStr = formatBytes(result.containerBytes);

      // Complete activity
      options.onActivityUpdate?.(activity.id, completeActivity(activity));

      // Update DB
      dbSync.updateExport({
        ...exportRecord,
        status: "completed",
        completedAt: new Date().toISOString(),
        totalFiles: result.fileCount,
        totalBytes: result.totalBytes,
      });

      log.info(`AFF4 container created: ${imageName}.aff4 — ${sizeStr}, ${result.fileCount} files, ratio ${result.compressionRatio.toFixed(1)}%`);
      toast.success(
        "AFF4 Container Created",
        `${imageName}.aff4 — ${sizeStr}, ${result.fileCount} files, ratio ${result.compressionRatio.toFixed(1)}%`,
      );

      // Companion file + evidence collection
      handleAcquisitionComplete({
        acquisitionType: "archive",
        outputPath: result.outputPath,
        sources,
        caseNumber: aff4CaseNumber() || undefined,
        evidenceNumber: aff4EvidenceNumber() || undefined,
        examiner: aff4ExaminerName() || undefined,
        description: aff4Description() || undefined,
        notes: aff4Notes() || undefined,
        format: "AFF4",
        totalBytes: result.totalBytes,
        totalFiles: result.fileCount,
        compressed: aff4Compression() !== "stored",
        sha256: result.linearHashes?.["sha256"] || undefined,
        md5: result.linearHashes?.["md5"] || undefined,
        sha1: result.linearHashes?.["sha1"] || undefined,
        startedAt: new Date(startTime).toISOString(),
        completedAt: new Date().toISOString(),
        durationMs: result.durationMs,
        collectionId: acqRecord.collectionId,
        itemId: acqRecord.itemId,
        hostname: options.systemStats?.hostname,
        systemModel: options.systemStats?.systemModel,
        systemSerialNumber: options.systemStats?.systemSerialNumber,
        systemManufacturer: options.systemStats?.systemManufacturer,
        osName: options.systemStats?.osName,
        osVersion: options.systemStats?.osVersion,
      });

      options.onComplete?.(destination);
    } catch (err) {
      const msg = getErrorMessage(err);
      const isCancelled = msg.toLowerCase().includes("cancel");
      log.error(`AFF4 export ${isCancelled ? "cancelled" : "failed"}: ${msg}`);

      options.onActivityUpdate?.(
        activity.id,
        failActivity(activity, isCancelled ? "AFF4 export cancelled" : msg),
      );

      dbSync.updateExport({
        ...exportRecord,
        status: isCancelled ? "cancelled" : "failed",
        completedAt: new Date().toISOString(),
        error: msg,
      });

      if (isCancelled) {
        toast.warning("AFF4 Export Cancelled", "The operation was cancelled");
      } else {
        toast.error("AFF4 Export Failed", msg);
      }
    } finally {
      common.setIsProcessing(false);
      common.restoreAllDriveMounts();
    }
  };

  // ── Reset ────────────────────────────────────────────────────────────────

  const resetAff4State = () => {
    setAff4ImageName("evidence");
    setAff4Compression("deflate");
    setAff4HashAlgorithms(["sha256"]);
    setAff4CaseNumber("");
    setAff4EvidenceNumber("");
    setAff4ExaminerName("");
    setAff4Description("");
    setAff4Notes("");
  };

  return {
    aff4ImageName,
    setAff4ImageName,
    aff4Compression,
    setAff4Compression,
    aff4HashAlgorithms,
    setAff4HashAlgorithms,
    aff4CaseNumber,
    setAff4CaseNumber,
    aff4EvidenceNumber,
    setAff4EvidenceNumber,
    aff4ExaminerName,
    setAff4ExaminerName,
    aff4Description,
    setAff4Description,
    aff4Notes,
    setAff4Notes,
    handleCreateAff4Image,
    resetAff4State,
  } as const;
}

export type Aff4ExportState = ReturnType<typeof useAff4ExportState>;
