// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useNativeExportState — Native file export (copy + hashes), 7z archive creation,
 * archive tools (test/repair/validate/extract), and LZMA compression state.
 */

import { createSignal, createEffect } from "solid-js";
import {
  createArchive,
  listenToProgress,
  estimateSize,
  formatBytes,
  CompressionLevel,
  testArchive,
  repairArchive,
  validateArchive,
  extractSplitArchive,
  listenToRepairProgress,
  listenToSplitExtractProgress,
} from "../../api/archiveCreate";
import {
  compressToLzma,
  compressToLzma2,
  decompressLzma,
  decompressLzma2,
} from "../../api/lzmaApi";
import { exportFiles, type CopyProgress, type ExportOptions } from "../../api/fileExport";
import { getErrorMessage } from "../../utils/errorUtils";
import { getBasename, joinPath } from "../../utils/pathUtils";
import {
  createActivity,
  updateProgress,
  completeActivity,
  failActivity,
} from "../../types/activity";
import type { ForensicHashAlgorithm } from "../../components/export/NativeExportMode";
import type { ToolsTabId } from "../../components/export/ToolsMode";
import type { ExportToast, ExportActivityCallbacks } from "./types";
import type { ExportCommonState } from "./useExportCommon";
import { dbSync } from "../project/useProjectDbSync";
import type { DbExportRecord } from "../../types/projectDb";

export interface UseNativeExportStateOptions extends ExportActivityCallbacks {
  toast: ExportToast;
  common: ExportCommonState;
}

export function useNativeExportState(options: UseNativeExportStateOptions) {
  const { toast, common } = options;

  // === File Export Options ===
  const [archiveName, setArchiveName] = createSignal("evidence.7z");
  const [exportName, setExportName] = createSignal("forensic_export");
  const [computeHashes, setComputeHashes] = createSignal(true);
  const [verifyAfterCopy, setVerifyAfterCopy] = createSignal(true);
  const [generateJsonManifest, setGenerateJsonManifest] = createSignal(true);
  const [generateTxtReport, setGenerateTxtReport] = createSignal(true);

  // === Archive Options ===
  const [compressionLevel, setCompressionLevel] = createSignal<number>(CompressionLevel.Store);
  const [password, setPassword] = createSignal("");
  const [showPassword, setShowPassword] = createSignal(false);
  const [numThreads, setNumThreads] = createSignal(0); // 0 = auto
  const [solid, setSolid] = createSignal(false);
  const [splitSizeMb, setSplitSizeMb] = createSignal(2048); // 2GB default

  // === Forensic Archive Options ===
  const [generateManifest, setGenerateManifest] = createSignal(true);
  const [verifyAfterCreate, setVerifyAfterCreate] = createSignal(true);
  const [hashAlgorithm, setHashAlgorithm] = createSignal<ForensicHashAlgorithm>("SHA-256");
  const [includeExaminerInfo, setIncludeExaminerInfo] = createSignal(true);
  const [examinerName, setExaminerName] = createSignal("");
  const [caseNumber, setCaseNumber] = createSignal("");
  const [evidenceDescription, setEvidenceDescription] = createSignal("");

  // === Size Estimation ===
  const [estimatedUncompressed, setEstimatedUncompressed] = createSignal(0);
  const [estimatedCompressed, setEstimatedCompressed] = createSignal(0);

  // === Archive Tools State ===
  const [toolsTab, setToolsTab] = createSignal<ToolsTabId>("test");
  const [testArchivePath, setTestArchivePath] = createSignal("");
  const [repairCorruptedPath, setRepairCorruptedPath] = createSignal("");
  const [repairOutputPath, setRepairOutputPath] = createSignal("");
  const [validateArchivePath, setValidateArchivePath] = createSignal("");
  const [extractFirstVolume, setExtractFirstVolume] = createSignal("");
  const [extractOutputDir, setExtractOutputDir] = createSignal("");

  // === LZMA Compression State ===
  const [lzmaInputPath, setLzmaInputPath] = createSignal("");
  const [lzmaOutputPath, setLzmaOutputPath] = createSignal("");
  const [lzmaAlgorithm, setLzmaAlgorithm] = createSignal<"lzma" | "lzma2">("lzma2");
  const [lzmaLevel, setLzmaLevel] = createSignal(5);
  const [lzmaDecompressInput, setLzmaDecompressInput] = createSignal("");
  const [lzmaDecompressOutput, setLzmaDecompressOutput] = createSignal("");

  // ─── Effects ────────────────────────────────────────────────────────────

  // Update size estimate when sources or compression level changes
  createEffect(() => {
    const sourceList = common.sources();
    const level = compressionLevel();
    if (sourceList.length > 0 && common.mode() === "native" && common.nativeExportTab() === "archive") {
      estimateSize(sourceList, level)
        .then(([uncompressed, compressed]) => {
          setEstimatedUncompressed(uncompressed);
          setEstimatedCompressed(compressed);
        })
        .catch(() => {
          setEstimatedUncompressed(0);
          setEstimatedCompressed(0);
        });
    }
  });

  // ─── Archive Handler ───────────────────────────────────────────────────

  const handleCreateArchive = async () => {
    common.setIsProcessing(true);

    const activity = createActivity("archive", joinPath(common.destination(), archiveName()), common.sources().length, {
      compressionLevel: compressionLevel(),
      encrypted: !!password(),
    });

    options.onActivityCreate?.(activity);

    let lastUpdate = 0;
    const UPDATE_INTERVAL = 250;

    const unlisten = await listenToProgress((prog) => {
      const now = Date.now();
      if (now - lastUpdate < UPDATE_INTERVAL && prog.percent < 100) return;
      lastUpdate = now;

      options.onActivityUpdate?.(
        activity.id,
        updateProgress(activity, {
          bytesProcessed: prog.bytesProcessed,
          bytesTotal: prog.bytesTotal,
          percent: prog.percent,
          currentFile: prog.currentFile || undefined,
          currentFileBytes: prog.currentFileBytes,
          currentFileTotal: prog.currentFileTotal,
        }),
      );
    });

    try {
      const archivePath = joinPath(common.destination(), archiveName());

      const archiveOptions = {
        compressionLevel: compressionLevel(),
        password: password() || undefined,
        numThreads: numThreads() || undefined,
        solid: solid(),
        splitSizeMb: splitSizeMb() || undefined,
        generateManifest: generateManifest(),
        verifyAfterCreate: verifyAfterCreate(),
        hashAlgorithm: generateManifest() ? hashAlgorithm() : undefined,
        examinerName: includeExaminerInfo() ? examinerName() || undefined : undefined,
        caseNumber: includeExaminerInfo() ? caseNumber() || undefined : undefined,
        evidenceDescription: includeExaminerInfo() ? evidenceDescription() || undefined : undefined,
      };

      // Track in DB
      const exportId = `archive-${Date.now()}`;
      const dbRecord: DbExportRecord = {
        id: exportId,
        exportType: "archive",
        sourcePathsJson: JSON.stringify(common.sources()),
        destination: archivePath,
        startedAt: new Date().toISOString(),
        initiatedBy: archiveOptions.examinerName || "",
        status: "in_progress",
        totalFiles: common.sources().length,
        totalBytes: 0,
        archiveName: archiveName(),
        archiveFormat: "7z",
        compressionLevel: String(compressionLevel()),
        encrypted: !!password(),
        optionsJson: JSON.stringify(archiveOptions),
      };
      dbSync.insertExport(dbRecord);

      createArchive(archivePath, common.sources(), archiveOptions)
        .then((result) => {
          options.onActivityUpdate?.(activity.id, completeActivity(activity));
          toast.success("Archive Created", `Successfully created: ${result}`);
          options.onComplete?.(result);

          dbSync.updateExport({ ...dbRecord, status: "completed", completedAt: new Date().toISOString() });
        })
        .catch((error: unknown) => {
          options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
          toast.error("Archive Creation Failed", getErrorMessage(error));

          dbSync.updateExport({ ...dbRecord, status: "failed", completedAt: new Date().toISOString(), error: getErrorMessage(error) });
        })
        .finally(() => {
          unlisten();
        });

      common.clearAllSources();
      setArchiveName("evidence.7z");
      setPassword("");
      common.setIsProcessing(false);

      toast.success("Archive Started", `Creating ${archiveName()} - check Activity panel for progress`);
    } catch (error: unknown) {
      unlisten();
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("Archive Creation Failed", getErrorMessage(error));
      common.setIsProcessing(false);
    }
  };

  // ─── Copy/Export Handler ────────────────────────────────────────────────

  const handleCopyOrExport = async () => {
    common.setIsProcessing(true);

    const copyOptions: ExportOptions = {
      computeHashes: computeHashes(),
      verifyAfterCopy: verifyAfterCopy(),
      generateJsonManifest: generateJsonManifest(),
      generateTxtReport: generateTxtReport(),
      preserveTimestamps: true,
      overwrite: false,
      createDirs: true,
      exportName: exportName(),
    };

    const activity = createActivity("export", common.destination(), common.sources().length, {
      includeHashes: copyOptions.computeHashes,
    });

    options.onActivityCreate?.(activity);

    // Track in DB — will be updated with operationId from CopyResult
    const exportId = `file-export-${Date.now()}`;
    const dbRecord: DbExportRecord = {
      id: exportId,
      exportType: "file",
      sourcePathsJson: JSON.stringify(common.sources()),
      destination: common.destination(),
      startedAt: new Date().toISOString(),
      initiatedBy: "",
      status: "in_progress",
      totalFiles: common.sources().length,
      totalBytes: 0,
      encrypted: false,
      optionsJson: JSON.stringify(copyOptions),
    };
    dbSync.insertExport(dbRecord);

    exportFiles(common.sources(), common.destination(), copyOptions, (prog: CopyProgress) => {
      options.onActivityUpdate?.(
        activity.id,
        updateProgress(activity, {
          bytesProcessed: prog.totalBytesCopied,
          bytesTotal: prog.totalBytes,
          percent: prog.percent,
          currentFile: prog.currentFile,
          filesProcessed: prog.currentIndex,
          totalFiles: prog.totalFiles,
        }),
      );
    })
      .then((result) => {
        options.onActivityUpdate?.(activity.id, completeActivity(activity));

        let message = `Exported ${result.filesCopied} files (${formatBytes(result.bytesCopied)}) in ${(result.durationMs / 1000).toFixed(1)}s`;
        if (result.jsonManifestPath) {
          message += `\nManifest: ${getBasename(result.jsonManifestPath)}`;
        }
        if (result.txtReportPath) {
          message += `\nReport: ${getBasename(result.txtReportPath)}`;
        }

        toast.success("Export Complete", message);
        options.onComplete?.(common.destination());

        // Update DB with results — use backend operation_id as the canonical ID
        dbSync.updateExport({
          ...dbRecord,
          id: result.operationId || exportId,
          status: "completed",
          completedAt: new Date().toISOString(),
          totalFiles: result.filesCopied,
          totalBytes: result.bytesCopied,
          manifestHash: result.jsonManifestPath || undefined,
        });
      })
      .catch((error: unknown) => {
        options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
        toast.error("Export Failed", getErrorMessage(error));

        dbSync.updateExport({
          ...dbRecord,
          status: "failed",
          completedAt: new Date().toISOString(),
          error: getErrorMessage(error),
        });
      });

    common.clearAllSources();
    setExportName("forensic_export");
    common.setIsProcessing(false);

    toast.success("Export Started", `Exporting to ${common.destination()} - check Activity panel for progress`);
  };

  // ─── Tool Operation Handlers ────────────────────────────────────────────

  const handleTestArchive = async () => {
    common.setIsProcessing(true);
    const archivePath = testArchivePath();

    const activity = createActivity("tool", archivePath, 1, { operation: "Test Archive" });
    options.onActivityCreate?.(activity);

    try {
      const isValid = await testArchive(archivePath);
      options.onActivityUpdate?.(activity.id, completeActivity(activity));

      if (isValid) {
        toast.success("Archive Test Passed", `${getBasename(archivePath)} is valid`);
      } else {
        toast.warning("Archive Test Failed", `${getBasename(archivePath)} has integrity issues`);
      }

      setTestArchivePath("");
    } catch (error: unknown) {
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("Test Failed", getErrorMessage(error));
    } finally {
      common.setIsProcessing(false);
    }
  };

  const handleRepairArchive = async () => {
    common.setIsProcessing(true);
    const corruptedPath = repairCorruptedPath();
    const repairedPath = repairOutputPath();

    const activity = createActivity("tool", `${corruptedPath} → ${repairedPath}`, 1, {
      operation: "Repair Archive",
    });
    options.onActivityCreate?.(activity);

    const unlisten = await listenToRepairProgress((prog) => {
      options.onActivityUpdate?.(activity.id, updateProgress(activity, { percent: prog.percent }));
    });

    try {
      const result = await repairArchive(corruptedPath, repairedPath);
      options.onActivityUpdate?.(activity.id, completeActivity(activity));
      toast.success("Archive Repaired", `Saved to: ${getBasename(result)}`);

      setRepairCorruptedPath("");
      setRepairOutputPath("");
    } catch (error: unknown) {
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("Repair Failed", getErrorMessage(error));
    } finally {
      unlisten();
      common.setIsProcessing(false);
    }
  };

  const handleValidateArchive = async () => {
    common.setIsProcessing(true);
    const archivePath = validateArchivePath();

    const activity = createActivity("tool", archivePath, 1, { operation: "Validate Archive" });
    options.onActivityCreate?.(activity);

    try {
      const validation = await validateArchive(archivePath);
      options.onActivityUpdate?.(activity.id, completeActivity(activity));

      if (validation.isValid) {
        toast.success("Validation Passed", `${getBasename(archivePath)} structure is valid`);
      } else {
        toast.error("Validation Failed", validation.errorMessage || "Archive has structural errors");
      }

      setValidateArchivePath("");
    } catch (error: unknown) {
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("Validation Failed", getErrorMessage(error));
    } finally {
      common.setIsProcessing(false);
    }
  };

  const handleExtractSplit = async () => {
    common.setIsProcessing(true);
    const firstVolume = extractFirstVolume();
    const outputDir = extractOutputDir();

    const activity = createActivity("tool", `${firstVolume} → ${outputDir}`, 1, {
      operation: "Extract Split Archive",
    });
    options.onActivityCreate?.(activity);

    const unlisten = await listenToSplitExtractProgress((prog) => {
      options.onActivityUpdate?.(activity.id, updateProgress(activity, { percent: prog.percent }));
    });

    try {
      const result = await extractSplitArchive(firstVolume, outputDir);
      options.onActivityUpdate?.(activity.id, completeActivity(activity));
      toast.success("Extraction Complete", `Files extracted to: ${getBasename(result)}`);

      setExtractFirstVolume("");
      setExtractOutputDir("");
    } catch (error: unknown) {
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("Extraction Failed", getErrorMessage(error));
    } finally {
      unlisten();
      common.setIsProcessing(false);
    }
  };

  const handleLzmaCompress = async () => {
    common.setIsProcessing(true);
    const input = lzmaInputPath();
    const output = lzmaOutputPath();
    const algo = lzmaAlgorithm();
    const level = lzmaLevel();

    const activity = createActivity(
      "tool",
      `${getBasename(input)} → ${getBasename(output)}`,
      1,
      { operation: `${algo.toUpperCase()} Compress (level ${level})` },
    );
    options.onActivityCreate?.(activity);

    try {
      const result =
        algo === "lzma" ? await compressToLzma(input, output, level) : await compressToLzma2(input, output, level);

      options.onActivityUpdate?.(activity.id, completeActivity(activity));
      toast.success("Compression Complete", `Output: ${getBasename(result)}`);

      setLzmaInputPath("");
      setLzmaOutputPath("");
    } catch (error: unknown) {
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("Compression Failed", getErrorMessage(error));
    } finally {
      common.setIsProcessing(false);
    }
  };

  const handleLzmaDecompress = async () => {
    common.setIsProcessing(true);
    const input = lzmaDecompressInput();
    const output = lzmaDecompressOutput();

    const activity = createActivity(
      "tool",
      `${getBasename(input)} → ${getBasename(output)}`,
      1,
      { operation: "LZMA Decompress" },
    );
    options.onActivityCreate?.(activity);

    try {
      const isXz = input.toLowerCase().endsWith(".xz");
      const result = isXz ? await decompressLzma2(input, output) : await decompressLzma(input, output);

      options.onActivityUpdate?.(activity.id, completeActivity(activity));
      toast.success("Decompression Complete", `Output: ${getBasename(result)}`);

      setLzmaDecompressInput("");
      setLzmaDecompressOutput("");
    } catch (error: unknown) {
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("Decompression Failed", getErrorMessage(error));
    } finally {
      common.setIsProcessing(false);
    }
  };

  const handleToolAction = async () => {
    const currentTab = toolsTab();

    switch (currentTab) {
      case "test":
        await handleTestArchive();
        break;
      case "repair":
        await handleRepairArchive();
        break;
      case "validate":
        await handleValidateArchive();
        break;
      case "extract":
        await handleExtractSplit();
        break;
      case "compress":
        await handleLzmaCompress();
        break;
      case "decompress":
        await handleLzmaDecompress();
        break;
    }
  };

  // ─── Reset ──────────────────────────────────────────────────────────────

  const resetNativeState = () => {
    setArchiveName("evidence.7z");
    setExportName("forensic_export");
    setPassword("");

    // Forensic fields
    setGenerateManifest(true);
    setVerifyAfterCreate(true);
    setHashAlgorithm("SHA-256");
    setIncludeExaminerInfo(true);
    setExaminerName("");
    setCaseNumber("");
    setEvidenceDescription("");

    // Archive fields
    setCompressionLevel(CompressionLevel.Store);
    setSplitSizeMb(2048);
    setSolid(false);
    setNumThreads(0);

    // Tools fields
    setTestArchivePath("");
    setRepairCorruptedPath("");
    setRepairOutputPath("");
    setValidateArchivePath("");
    setExtractFirstVolume("");
    setExtractOutputDir("");
  };

  return {
    // File export options
    archiveName,
    setArchiveName,
    exportName,
    setExportName,
    computeHashes,
    setComputeHashes,
    verifyAfterCopy,
    setVerifyAfterCopy,
    generateJsonManifest,
    setGenerateJsonManifest,
    generateTxtReport,
    setGenerateTxtReport,

    // Archive options
    compressionLevel,
    setCompressionLevel,
    password,
    setPassword,
    showPassword,
    setShowPassword,
    numThreads,
    setNumThreads,
    solid,
    setSolid,
    splitSizeMb,
    setSplitSizeMb,

    // Forensic archive options
    generateManifest,
    setGenerateManifest,
    verifyAfterCreate,
    setVerifyAfterCreate,
    hashAlgorithm,
    setHashAlgorithm,
    includeExaminerInfo,
    setIncludeExaminerInfo,
    examinerName,
    setExaminerName,
    caseNumber,
    setCaseNumber,
    evidenceDescription,
    setEvidenceDescription,

    // Size estimation
    estimatedUncompressed,
    estimatedCompressed,

    // Tools state
    toolsTab,
    setToolsTab,
    testArchivePath,
    setTestArchivePath,
    repairCorruptedPath,
    setRepairCorruptedPath,
    repairOutputPath,
    setRepairOutputPath,
    validateArchivePath,
    setValidateArchivePath,
    extractFirstVolume,
    setExtractFirstVolume,
    extractOutputDir,
    setExtractOutputDir,

    // LZMA state
    lzmaInputPath,
    setLzmaInputPath,
    lzmaOutputPath,
    setLzmaOutputPath,
    lzmaAlgorithm,
    setLzmaAlgorithm,
    lzmaLevel,
    setLzmaLevel,
    lzmaDecompressInput,
    setLzmaDecompressInput,
    lzmaDecompressOutput,
    setLzmaDecompressOutput,

    // Handlers
    handleCreateArchive,
    handleCopyOrExport,
    handleToolAction,

    // Reset
    resetNativeState,
  } as const;
}

export type NativeExportState = ReturnType<typeof useNativeExportState>;
