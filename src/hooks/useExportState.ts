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
import { invoke } from "@tauri-apps/api/core";
import { createSignal } from "solid-js";
import { useExportCommon } from "./export/useExportCommon";
import { useEwfExportState } from "./export/useEwfExportState";
import { useL01ExportState } from "./export/useL01ExportState";
import { useNativeExportState } from "./export/useNativeExportState";
import { useMemoryDumpState } from "./export/useMemoryDumpState";
import { useTriageState } from "./export/useTriageState";
import { useRawExportState } from "./export/useRawExportState";
import { useAff4ExportState } from "./export/useAff4ExportState";
import { isTauri } from "../utils/platform";

// Re-export types so existing `import { ExportMode } from "../hooks/useExportState"` works
export type { ExportMode } from "./export/types";

// ─── Types ──────────────────────────────────────────────────────────────────

export interface UseExportStateOptions {
  initialSources?: string[];
  /** Pre-fill examiner name from project owner (optional) */
  initialExaminerName?: string;
  /** Case number for evidence collection records (optional) */
  caseNumber?: string;
  /** Project name for auto-generating evidence filenames (optional) */
  projectName?: string;
  /** Pre-collected system stats from Identify phase (avoids redundant fetches) */
  systemStats?: import("./useFileManager").SystemStats | null;
  /** Initial export mode (physical/logical/native/tools). Defaults to "native". */
  initialMode?: import("./export/types").ExportMode;
  /** Default destination directory from project locations (optional) */
  initialDestination?: string;
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
    initialMode: options.initialMode,
    initialDestination: options.initialDestination,
    toast,
  });

  const ewf = useEwfExportState({
    toast,
    common,
    caseNumber: options.caseNumber,
    examinerName: options.initialExaminerName,
    systemStats: options.systemStats,
    ...activityCallbacks,
  });

  const l01 = useL01ExportState({
    toast,
    common,
    caseNumber: options.caseNumber,
    examinerName: options.initialExaminerName,
    systemStats: options.systemStats,
    ...activityCallbacks,
  });

  const native = useNativeExportState({
    toast,
    common,
    caseNumber: options.caseNumber,
    examinerName: options.initialExaminerName,
    systemStats: options.systemStats,
    ...activityCallbacks,
  });

  const memory = useMemoryDumpState({
    toast,
    common,
    caseNumber: options.caseNumber,
    examinerName: options.initialExaminerName,
    systemStats: options.systemStats,
    ...activityCallbacks,
  });

  const triage = useTriageState({
    toast,
    common,
    caseNumber: options.caseNumber,
    examinerName: options.initialExaminerName,
    systemStats: options.systemStats,
    ...activityCallbacks,
  });

  const raw = useRawExportState({
    toast,
    common,
    caseNumber: options.caseNumber,
    examinerName: options.initialExaminerName,
    systemStats: options.systemStats,
    ...activityCallbacks,
  });

  const aff4 = useAff4ExportState({
    toast,
    common,
    caseNumber: options.caseNumber,
    examinerName: options.initialExaminerName,
    systemStats: options.systemStats,
    ...activityCallbacks,
  });

  // Physical imaging format signal ("ewf" = E01 container, "raw" = dd/img)
  const [physicalFormat, setPhysicalFormat] = createSignal<"ewf" | "raw">("ewf");

  // Pre-fill examiner name from project owner if provided
  if (options.initialExaminerName) {
    ewf.setEwfExaminerName(options.initialExaminerName);
    l01.setL01ExaminerName(options.initialExaminerName);
    native.setExaminerName(options.initialExaminerName);
    raw.setRawExaminerName(options.initialExaminerName);
    aff4.setAff4ExaminerName(options.initialExaminerName);
  }

  // Pre-fill case number from project if provided
  if (options.caseNumber) {
    ewf.setEwfCaseNumber(options.caseNumber);
    l01.setL01CaseNumber(options.caseNumber);
    native.setCaseNumber(options.caseNumber);
    raw.setRawCaseNumber(options.caseNumber);
    aff4.setAff4CaseNumber(options.caseNumber);
  }

  // Auto-populate description and auto-generate evidence filename from system identification.
  // When systemStats is pre-collected (Acquire edition Identify phase), use it directly.
  // Otherwise, fetch from backend (full FFX edition).
  const applySystemInfo = (hostname: string, username: string, serialNumber: string) => {
    // Auto-populate description
    if (hostname && hostname !== "unknown") {
      const desc = `Acquired on ${hostname}`;
      if (!ewf.ewfDescription()) ewf.setEwfDescription(desc);
      if (!l01.l01Description()) l01.setL01Description(desc);
      if (!native.evidenceDescription()) native.setEvidenceDescription(desc);
      if (!raw.rawDescription()) raw.setRawDescription(desc);
      if (!aff4.aff4Description()) aff4.setAff4Description(desc);
    }

    // Auto-generate evidence filename:
    // [ProjectName]-[Last5SN]-[Hostname]-[Username]-[YYYYMMDD]
    const projectName = options.projectName || options.caseNumber || "evidence";
    const sn = serialNumber?.slice(-5) || "NOSN0";
    const host = hostname && hostname !== "unknown" ? hostname : "host";
    const user = username || "user";
    const date = new Date().toISOString().slice(0, 10).replace(/-/g, "");

    // Sanitize each segment: keep alphanumeric, hyphens, dots, underscores
    const sanitize = (s: string) => s.replace(/[^a-zA-Z0-9._-]/g, "_").slice(0, 40);
    const evidenceName = [
      sanitize(projectName),
      sanitize(sn),
      sanitize(host),
      sanitize(user),
      date,
    ].join("-");

    // Set across all export modes (only if still at default "evidence")
    if (!ewf.ewfImageName() || ewf.ewfImageName() === "evidence") ewf.setEwfImageName(evidenceName);
    if (!l01.l01ImageName() || l01.l01ImageName() === "evidence") l01.setL01ImageName(evidenceName);
    if (!raw.rawImageName() || raw.rawImageName() === "evidence") raw.setRawImageName(evidenceName);
    if (!aff4.aff4ImageName() || aff4.aff4ImageName() === "evidence") aff4.setAff4ImageName(evidenceName);
  };

  if (!isTauri) {
    const stats = options.systemStats;
    applySystemInfo(stats?.hostname || "", "user", stats?.systemSerialNumber || "");
  } else if (options.systemStats) {
    // Use pre-collected system stats from Identify phase — only need username
    const stats = options.systemStats;
    invoke<string>("get_current_username").then((username) => {
      applySystemInfo(stats.hostname || "", username, stats.systemSerialNumber || "");
    }).catch(() => {
      applySystemInfo(stats.hostname || "", "user", stats.systemSerialNumber || "");
    });
  } else {
    // No pre-collected stats — fetch everything from backend
    Promise.all([
      invoke<string>("get_hostname"),
      invoke<string>("get_current_username"),
      invoke<{ systemSerialNumber: string }>("get_system_stats"),
    ]).then(([hostname, username, stats]) => {
      applySystemInfo(hostname, username, stats?.systemSerialNumber || "");
    }).catch(() => {});
  }

  // ─── Main Start Handler ─────────────────────────────────────────────────

  const handleStart = async () => {
    if (common.isAcquiring()) {
      toast.error("Acquisition In Progress", "Please wait for the current acquisition to complete before starting another.");
      return;
    }

    const currentMode = common.mode();

    // Memory mode only needs destination, not sources
    if (currentMode === "memory") {
      if (!common.destination()) {
        toast.error("No Destination", "Please select a destination folder");
        return;
      }
      await memory.handleCaptureMemory();
      return;
    }

    // Triage mode only needs destination, not sources
    if (currentMode === "triage") {
      if (!common.destination()) {
        toast.error("No Destination", "Please select a destination folder");
        return;
      }
      await triage.handleTriageCollect();
      return;
    }

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

    if (currentMode === "physical") {
      if (physicalFormat() === "raw") {
        await raw.handleCreateRawImage();
      } else {
        await ewf.handleCreateE01Image();
      }
    } else if (currentMode === "logical") {
      await l01.handleCreateL01Image();
    } else if (currentMode === "aff4") {
      await aff4.handleCreateAff4Image();
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
    memory.resetMemoryState();
    triage.resetTriageState();
    raw.resetRawState();
    aff4.resetAff4State();
    setPhysicalFormat("ewf");

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
    isAcquiring: common.isAcquiring,
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
    ewfVerifyAfterWrite: ewf.ewfVerifyAfterWrite,
    setEwfVerifyAfterWrite: ewf.setEwfVerifyAfterWrite,
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
    l01FilterExtensions: l01.l01FilterExtensions,
    setL01FilterExtensions: l01.setL01FilterExtensions,
    l01ExcludeExtensions: l01.l01ExcludeExtensions,
    setL01ExcludeExtensions: l01.setL01ExcludeExtensions,
    l01MinFileSize: l01.l01MinFileSize,
    setL01MinFileSize: l01.setL01MinFileSize,
    l01MaxFileSize: l01.l01MaxFileSize,
    setL01MaxFileSize: l01.setL01MaxFileSize,

    // Drive selector state (from common)
    showDriveSelector: common.showDriveSelector,
    setShowDriveSelector: common.setShowDriveSelector,
    driveSources: common.driveSources,
    mountDrivesReadOnly: common.mountDrivesReadOnly,

    // Handlers
    setDestination: common.setDestination,
    handleAddSources: common.handleAddSources,
    handleAddFolder: common.handleAddFolder,
    handleDriveSelected: common.handleDriveSelected,
    handleAddDriveSource: common.handleAddDriveSource,
    handleSelectDestination: common.handleSelectDestination,
    handleRemoveSource: common.handleRemoveSource,
    removeSourceByPath: common.removeSourceByPath,
    handleStart,
    handleToolAction: native.handleToolAction,
    handleCancelExport: native.handleCancelExport,
    handleReset,
    hasDriveSources: common.hasDriveSources,

    // Active export tracking (for cancel UI)
    activeExportOperationId: native.activeExportOperationId,

    // Memory capture state (from memory)
    memoryInfo: memory.memoryInfo,
    memoryInfoLoading: memory.memoryInfoLoading,
    memoryComputeHashes: memory.memoryComputeHashes,
    setMemoryComputeHashes: memory.setMemoryComputeHashes,
    memoryOutputName: memory.memoryOutputName,
    setMemoryOutputName: memory.setMemoryOutputName,
    memoryProgress: memory.memoryProgress,
    memoryResult: memory.memoryResult,
    loadMemoryInfo: memory.loadMemoryInfo,
    handleCaptureMemory: memory.handleCaptureMemory,
    handleCancelMemoryCapture: memory.handleCancelMemoryCapture,
    resetMemoryState: memory.resetMemoryState,

    // Triage collection state (from triage)
    triageProfiles: triage.triageProfiles,
    triageCategories: triage.triageCategories,
    triageProfilesLoading: triage.triageProfilesLoading,
    selectedTriageProfile: triage.selectedTriageProfile,
    setSelectedTriageProfile: triage.setSelectedTriageProfile,
    selectedTriageCategories: triage.selectedTriageCategories,
    toggleTriageCategory: triage.toggleTriageCategory,
    triageScanForSecrets: triage.triageScanForSecrets,
    setTriageScanForSecrets: triage.setTriageScanForSecrets,
    triageContainerFormat: triage.triageContainerFormat,
    setTriageContainerFormat: triage.setTriageContainerFormat,
    triageProgress: triage.triageProgress,
    triageResult: triage.triageResult,
    loadTriageProfiles: triage.loadTriageProfiles,
    handleTriageCollect: triage.handleTriageCollect,
    handleCancelTriage: triage.handleCancelTriage,
    resetTriageState: triage.resetTriageState,

    // Physical imaging format ("ewf" | "raw")
    physicalFormat,
    setPhysicalFormat,

    // Raw disk image state (from raw)
    rawVerifyAfterWrite: raw.rawVerifyAfterWrite,
    setRawVerifyAfterWrite: raw.setRawVerifyAfterWrite,
    rawComputeMd5: raw.rawComputeMd5,
    setRawComputeMd5: raw.setRawComputeMd5,
    rawComputeSha1: raw.rawComputeSha1,
    setRawComputeSha1: raw.setRawComputeSha1,
    rawComputeSha256: raw.rawComputeSha256,
    setRawComputeSha256: raw.setRawComputeSha256,
    rawSegmentSize: raw.rawSegmentSize,
    setRawSegmentSize: raw.setRawSegmentSize,
    rawImageName: raw.rawImageName,
    setRawImageName: raw.setRawImageName,
    rawCaseNumber: raw.rawCaseNumber,
    setRawCaseNumber: raw.setRawCaseNumber,
    rawEvidenceNumber: raw.rawEvidenceNumber,
    setRawEvidenceNumber: raw.setRawEvidenceNumber,
    rawExaminerName: raw.rawExaminerName,
    setRawExaminerName: raw.setRawExaminerName,
    rawDescription: raw.rawDescription,
    setRawDescription: raw.setRawDescription,
    rawNotes: raw.rawNotes,
    setRawNotes: raw.setRawNotes,
    handleCreateRawImage: raw.handleCreateRawImage,
    resetRawState: raw.resetRawState,

    // AFF4 state (from aff4)
    aff4ImageName: aff4.aff4ImageName,
    setAff4ImageName: aff4.setAff4ImageName,
    aff4Compression: aff4.aff4Compression,
    setAff4Compression: aff4.setAff4Compression,
    aff4HashAlgorithms: aff4.aff4HashAlgorithms,
    setAff4HashAlgorithms: aff4.setAff4HashAlgorithms,
    aff4CaseNumber: aff4.aff4CaseNumber,
    setAff4CaseNumber: aff4.setAff4CaseNumber,
    aff4EvidenceNumber: aff4.aff4EvidenceNumber,
    setAff4EvidenceNumber: aff4.setAff4EvidenceNumber,
    aff4ExaminerName: aff4.aff4ExaminerName,
    setAff4ExaminerName: aff4.setAff4ExaminerName,
    aff4Description: aff4.aff4Description,
    setAff4Description: aff4.setAff4Description,
    aff4Notes: aff4.aff4Notes,
    setAff4Notes: aff4.setAff4Notes,
    handleCreateAff4Image: aff4.handleCreateAff4Image,
    resetAff4State: aff4.resetAff4State,
  } as const;
}

export type ExportState = ReturnType<typeof useExportState>;
