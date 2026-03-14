// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useEwfExportState — E01/EWF physical image creation state and handler.
 */

import { createSignal } from "solid-js";
import { createE01Image, buildEwfExportOptions } from "../../api/ewfExport";
import { formatBytes } from "../../api/archiveCreate";
import { getErrorMessage } from "../../utils/errorUtils";
import { joinPath } from "../../utils/pathUtils";
import {
  createActivity,
  updateProgress,
  completeActivity,
  failActivity,
} from "../../types/activity";
import type { ExportToast, ExportActivityCallbacks } from "./types";
import type { ExportCommonState } from "./useExportCommon";

export interface UseEwfExportStateOptions extends ExportActivityCallbacks {
  toast: ExportToast;
  common: ExportCommonState;
}

export function useEwfExportState(options: UseEwfExportStateOptions) {
  const { toast, common } = options;

  // === EWF/E01 Export State ===
  const [ewfFormat, setEwfFormat] = createSignal("e01");
  const [ewfCompression, setEwfCompression] = createSignal("none");
  const [ewfCompressionMethod, setEwfCompressionMethod] = createSignal("deflate");
  const [ewfComputeMd5, setEwfComputeMd5] = createSignal(true);
  const [ewfComputeSha1, setEwfComputeSha1] = createSignal(false);
  const [ewfSegmentSize, setEwfSegmentSize] = createSignal(2048);
  const [ewfImageName, setEwfImageName] = createSignal("evidence");
  const [ewfCaseNumber, setEwfCaseNumber] = createSignal("");
  const [ewfEvidenceNumber, setEwfEvidenceNumber] = createSignal("");
  const [ewfExaminerName, setEwfExaminerName] = createSignal("");
  const [ewfDescription, setEwfDescription] = createSignal("");
  const [ewfNotes, setEwfNotes] = createSignal("");

  // ─── Handler ────────────────────────────────────────────────────────────

  const handleCreateE01Image = async () => {
    common.setIsProcessing(true);
    common.setIsAcquiring(true);

    const outputPath = joinPath(common.destination(), ewfImageName());
    const shouldRestoreMounts = common.hasDriveSources() && common.mountDrivesReadOnly();

    const activity = createActivity("export", outputPath, common.sources().length, {
      operation: `E01 Image Creation (${ewfFormat()}, ${ewfCompression()})`,
    });

    options.onActivityCreate?.(activity);

    try {
      const ewfOptions = buildEwfExportOptions({
        sourcePaths: common.sources(),
        outputPath,
        format: ewfFormat(),
        compression: ewfCompression(),
        compressionMethod: ewfCompressionMethod(),
        caseNumber: ewfCaseNumber() || undefined,
        evidenceNumber: ewfEvidenceNumber() || undefined,
        examinerName: ewfExaminerName() || undefined,
        description: ewfDescription() || undefined,
        notes: ewfNotes() || undefined,
        computeMd5: ewfComputeMd5(),
        computeSha1: ewfComputeSha1(),
      });

      if (ewfSegmentSize() > 0) {
        ewfOptions.segmentSize = ewfSegmentSize() * 1024 * 1024;
      }

      createE01Image(ewfOptions, (prog) => {
        options.onActivityUpdate?.(
          activity.id,
          updateProgress(activity, {
            bytesProcessed: prog.bytesWritten,
            bytesTotal: prog.totalBytes,
            percent: prog.percent,
            currentFile: prog.currentFile || undefined,
          }),
        );
      })
        .then((result) => {
          options.onActivityUpdate?.(activity.id, completeActivity(activity));
          const hashInfo = result.md5Hash ? ` | MD5: ${result.md5Hash.substring(0, 16)}...` : "";
          toast.success(
            "E01 Image Created",
            `${result.format} image created (${formatBytes(result.bytesWritten)})${hashInfo}`,
          );
          options.onComplete?.(result.outputPath);
        })
        .catch((error: unknown) => {
          options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
          toast.error("E01 Creation Failed", getErrorMessage(error));
        })
        .finally(() => {
          common.setIsAcquiring(false);
          if (shouldRestoreMounts) {
            common.restoreAllDriveMounts();
          }
        });

      common.clearAllSources();
      setEwfImageName("evidence");
      common.setIsProcessing(false);

      toast.success("E01 Export Started", `Creating ${ewfImageName()}.E01 - check Activity panel for progress`);
    } catch (error: unknown) {
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("E01 Creation Failed", getErrorMessage(error));
      common.setIsProcessing(false);
      common.setIsAcquiring(false);
    }
  };

  // ─── Reset ──────────────────────────────────────────────────────────────

  const resetEwfState = () => {
    setEwfFormat("e01");
    setEwfCompression("none");
    setEwfCompressionMethod("deflate");
    setEwfComputeMd5(true);
    setEwfComputeSha1(false);
    setEwfSegmentSize(2048);
    setEwfImageName("evidence");
    setEwfCaseNumber("");
    setEwfEvidenceNumber("");
    setEwfExaminerName("");
    setEwfDescription("");
    setEwfNotes("");
  };

  return {
    // EWF state
    ewfFormat,
    setEwfFormat,
    ewfCompression,
    setEwfCompression,
    ewfCompressionMethod,
    setEwfCompressionMethod,
    ewfComputeMd5,
    setEwfComputeMd5,
    ewfComputeSha1,
    setEwfComputeSha1,
    ewfSegmentSize,
    setEwfSegmentSize,
    ewfImageName,
    setEwfImageName,
    ewfCaseNumber,
    setEwfCaseNumber,
    ewfEvidenceNumber,
    setEwfEvidenceNumber,
    ewfExaminerName,
    setEwfExaminerName,
    ewfDescription,
    setEwfDescription,
    ewfNotes,
    setEwfNotes,

    // Handler
    handleCreateE01Image,

    // Reset
    resetEwfState,
  } as const;
}

export type EwfExportState = ReturnType<typeof useEwfExportState>;
