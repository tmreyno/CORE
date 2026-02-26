// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useExportState — Orchestrator hook that composes sub-hooks for the ExportPanel.
 *
 * Sub-hooks (in src/hooks/export/):
 *   - useExportCommon   — shared state (mode, sources, destination, drives)
 *   - useEwfExportState — E01/EWF physical image creation
 *   - useL01ExportState — L01 logical evidence creation
 *   - useNativeExportState — 7z archive, file export, tools, LZMA
 *
 * The return type is a flat object identical to the pre-refactor API so that
 * ExportPanel.tsx (the sole consumer) requires zero changes.
 */

import type { Activity } from "../types/activity";
import { useExportCommon } from "./export/useExportCommon";
import { useEwfExportState } from "./export/useEwfExportState";
import { useL01ExportState } from "./export/useL01ExportState";
import { useNativeExportState } from "./export/useNativeExportState";

// Re-export types so existing `import { ExportMode } from "../hooks/useExportState"` works
export type { ExportMode } from "./export/types";

// ─── Types ──────────────────────────────────────────────────────────────────

export interface UseExportStateOptions {
  initialSources?: string[];
  onComplete?: (destination: string) => void;
  onActivityCreate?: (activity: Activity) => void;
  onActivityUpdate?: (id: string, updates: Partial<Activity>) => void;
  /** Toast interface for notifications */
  toast: {
    success: (title: string, message: string) => void;
    error: (title: string, message: string) => void;
    warning: (title: string, message: string) => void;
    info: (title: string, message: string) => void;
  };
}

// ─── Hook ───────────────────────────────────────────────────────────────────

export function useExportState(options: UseExportStateOptions) {
  const { toast } = options;

  const activityCallbacks = {
    onActivityCreate: options.onActivityCreate,
    onActivityUpdate: options.onActivityUpdate,
    onComplete: options.onComplete,
  };

  // ── Compose sub-hooks ─────────────────────────────────────────────────

  const common = useExportCommon({
    initialSources: options.initialSources,
    toast,
  });

  const ewf = useEwfExportState({
    toast,
    common,
    ...activityCallbacks,
  });

  const l01 = useL01ExportState({
    toast,
    common,
    ...activityCallbacks,
  });

  const native = useNativeExportState({
    toast,
    common,
    ...activityCallbacks,
  });

  // ─── Main Start Handler ─────────────────────────────────────────────────

  const handleStart = async () => {
    if (common.sources().length === 0) {
      toast.error("No Sources", "Please select files or folders to export");
      return;
    }

    if (!common.destination()) {
      toast.error("No Destination", "Please select a destination folder");
      return;
    }

    // Confirmation check when drive sources are selected
    if (common.hasDriveSources()) {
      const driveList = Array.from(common.driveSources()).join(", ");
      const roNote = common.mountDrivesReadOnly()
        ? "\n\nThe selected drive(s) will be temporarily remounted as read-only for forensic integrity."
        : "";
      const confirmed = window.confirm(
        `You are about to image the following drive(s):\n\n${driveList}\n\n` +
          `This operation may take a long time depending on drive size. ` +
          `Make sure the destination has sufficient free space.${roNote}\n\nContinue?`,
      );
      if (!confirmed) return;
    }

    // Remount drives as read-only if requested
    const mountOk = await common.remountDrivesIfNeeded();
    if (!mountOk) return;

    const currentMode = common.mode();

    if (currentMode === "physical") {
      await ewf.handleCreateE01Image();
    } else if (currentMode === "logical") {
      await l01.handleCreateL01Image();
    } else if (currentMode === "native" && common.nativeExportTab() === "archive") {
      await native.handleCreateArchive();
    } else {
      await native.handleCopyOrExport();
    }
  };

  // ─── Reset ──────────────────────────────────────────────────────────────

  const handleReset = () => {
    common.clearAllSources();
    common.setDestination("");
    common.setIsProcessing(false);

    ewf.resetEwfState();
    l01.resetL01State();
    native.resetNativeState();

    toast.info("Form Reset", "All fields cleared");
  };

  // ─── Return (flat API — backwards-compatible) ──────────────────────────

  return {
    // Core state (from common)
    mode: common.mode,
    setMode: common.setMode,
    nativeExportTab: common.nativeExportTab,
    setNativeExportTab: common.setNativeExportTab,
    sources: common.sources,
    destination: common.destination,
    isProcessing: common.isProcessing,
    showAdvanced: common.showAdvanced,
    setShowAdvanced: common.setShowAdvanced,

    // File export names (from native)
    archiveName: native.archiveName,
    setArchiveName: native.setArchiveName,
    exportName: native.exportName,
    setExportName: native.setExportName,

    // Export options (from native)
    computeHashes: native.computeHashes,
    setComputeHashes: native.setComputeHashes,
    verifyAfterCopy: native.verifyAfterCopy,
    setVerifyAfterCopy: native.setVerifyAfterCopy,
    generateJsonManifest: native.generateJsonManifest,
    setGenerateJsonManifest: native.setGenerateJsonManifest,
    generateTxtReport: native.generateTxtReport,
    setGenerateTxtReport: native.setGenerateTxtReport,

    // Archive options (from native)
    compressionLevel: native.compressionLevel,
    setCompressionLevel: native.setCompressionLevel,
    password: native.password,
    setPassword: native.setPassword,
    showPassword: native.showPassword,
    setShowPassword: native.setShowPassword,
    numThreads: native.numThreads,
    setNumThreads: native.setNumThreads,
    solid: native.solid,
    setSolid: native.setSolid,
    splitSizeMb: native.splitSizeMb,
    setSplitSizeMb: native.setSplitSizeMb,

    // Forensic archive options (from native)
    generateManifest: native.generateManifest,
    setGenerateManifest: native.setGenerateManifest,
    verifyAfterCreate: native.verifyAfterCreate,
    setVerifyAfterCreate: native.setVerifyAfterCreate,
    hashAlgorithm: native.hashAlgorithm,
    setHashAlgorithm: native.setHashAlgorithm,
    includeExaminerInfo: native.includeExaminerInfo,
    setIncludeExaminerInfo: native.setIncludeExaminerInfo,
    examinerName: native.examinerName,
    setExaminerName: native.setExaminerName,
    caseNumber: native.caseNumber,
    setCaseNumber: native.setCaseNumber,
    evidenceDescription: native.evidenceDescription,
    setEvidenceDescription: native.setEvidenceDescription,

    // Size estimation (from native)
    estimatedUncompressed: native.estimatedUncompressed,
    estimatedCompressed: native.estimatedCompressed,

    // Tools state (from native)
    toolsTab: native.toolsTab,
    setToolsTab: native.setToolsTab,
    testArchivePath: native.testArchivePath,
    setTestArchivePath: native.setTestArchivePath,
    repairCorruptedPath: native.repairCorruptedPath,
    setRepairCorruptedPath: native.setRepairCorruptedPath,
    repairOutputPath: native.repairOutputPath,
    setRepairOutputPath: native.setRepairOutputPath,
    validateArchivePath: native.validateArchivePath,
    setValidateArchivePath: native.setValidateArchivePath,
    extractFirstVolume: native.extractFirstVolume,
    setExtractFirstVolume: native.setExtractFirstVolume,
    extractOutputDir: native.extractOutputDir,
    setExtractOutputDir: native.setExtractOutputDir,

    // LZMA state (from native)
    lzmaInputPath: native.lzmaInputPath,
    setLzmaInputPath: native.setLzmaInputPath,
    lzmaOutputPath: native.lzmaOutputPath,
    setLzmaOutputPath: native.setLzmaOutputPath,
    lzmaAlgorithm: native.lzmaAlgorithm,
    setLzmaAlgorithm: native.setLzmaAlgorithm,
    lzmaLevel: native.lzmaLevel,
    setLzmaLevel: native.setLzmaLevel,
    lzmaDecompressInput: native.lzmaDecompressInput,
    setLzmaDecompressInput: native.setLzmaDecompressInput,
    lzmaDecompressOutput: native.lzmaDecompressOutput,
    setLzmaDecompressOutput: native.setLzmaDecompressOutput,

    // EWF state (from ewf)
    ewfFormat: ewf.ewfFormat,
    setEwfFormat: ewf.setEwfFormat,
    ewfCompression: ewf.ewfCompression,
    setEwfCompression: ewf.setEwfCompression,
    ewfCompressionMethod: ewf.ewfCompressionMethod,
    setEwfCompressionMethod: ewf.setEwfCompressionMethod,
    ewfComputeMd5: ewf.ewfComputeMd5,
    setEwfComputeMd5: ewf.setEwfComputeMd5,
    ewfComputeSha1: ewf.ewfComputeSha1,
    setEwfComputeSha1: ewf.setEwfComputeSha1,
    ewfSegmentSize: ewf.ewfSegmentSize,
    setEwfSegmentSize: ewf.setEwfSegmentSize,
    ewfImageName: ewf.ewfImageName,
    setEwfImageName: ewf.setEwfImageName,
    ewfCaseNumber: ewf.ewfCaseNumber,
    setEwfCaseNumber: ewf.setEwfCaseNumber,
    ewfEvidenceNumber: ewf.ewfEvidenceNumber,
    setEwfEvidenceNumber: ewf.setEwfEvidenceNumber,
    ewfExaminerName: ewf.ewfExaminerName,
    setEwfExaminerName: ewf.setEwfExaminerName,
    ewfDescription: ewf.ewfDescription,
    setEwfDescription: ewf.setEwfDescription,
    ewfNotes: ewf.ewfNotes,
    setEwfNotes: ewf.setEwfNotes,

    // L01 state (from l01)
    l01ImageName: l01.l01ImageName,
    setL01ImageName: l01.setL01ImageName,
    l01Compression: l01.l01Compression,
    setL01Compression: l01.setL01Compression,
    l01HashAlgorithm: l01.l01HashAlgorithm,
    l01SegmentSize: l01.l01SegmentSize,
    setL01SegmentSize: l01.setL01SegmentSize,
    l01CaseNumber: l01.l01CaseNumber,
    setL01CaseNumber: l01.setL01CaseNumber,
    l01EvidenceNumber: l01.l01EvidenceNumber,
    setL01EvidenceNumber: l01.setL01EvidenceNumber,
    l01ExaminerName: l01.l01ExaminerName,
    setL01ExaminerName: l01.setL01ExaminerName,
    l01Description: l01.l01Description,
    setL01Description: l01.setL01Description,
    l01Notes: l01.l01Notes,
    setL01Notes: l01.setL01Notes,

    // Drive selector state (from common)
    showDriveSelector: common.showDriveSelector,
    setShowDriveSelector: common.setShowDriveSelector,
    driveSources: common.driveSources,
    mountDrivesReadOnly: common.mountDrivesReadOnly,

    // Handlers
    handleAddSources: common.handleAddSources,
    handleAddFolder: common.handleAddFolder,
    handleDriveSelected: common.handleDriveSelected,
    handleSelectDestination: common.handleSelectDestination,
    handleRemoveSource: common.handleRemoveSource,
    handleStart,
    handleToolAction: native.handleToolAction,
    handleReset,
    hasDriveSources: common.hasDriveSources,
  } as const;
}

export type ExportState = ReturnType<typeof useExportState>;
