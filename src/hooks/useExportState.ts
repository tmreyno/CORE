// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useExportState — Manages all state and handlers for the ExportPanel.
 *
 * Extracted from ExportPanel.tsx to keep the component render-only.
 * Contains ~70 signals and ~15 async handler functions for:
 *   - Source/destination management
 *   - Archive creation (7z)
 *   - E01 physical image creation
 *   - L01 logical evidence creation
 *   - Native file export (copy with hashes)
 *   - Archive tools (test, repair, validate, extract, LZMA)
 *   - Drive selection & read-only mounting
 */

import { createSignal, createEffect } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
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
} from "../api/archiveCreate";
import { createE01Image, buildEwfExportOptions } from "../api/ewfExport";
import { createL01Image, buildL01ExportOptions } from "../api/l01Export";
import {
  compressToLzma,
  compressToLzma2,
  decompressLzma,
  decompressLzma2,
} from "../api/lzmaApi";
import { exportFiles, type CopyProgress, type ExportOptions } from "../api/fileExport";
import { getErrorMessage } from "../utils/errorUtils";
import {
  createActivity,
  updateProgress,
  completeActivity,
  failActivity,
  type Activity,
} from "../types/activity";
import { remountReadOnly, restoreMount } from "../api/drives";
import type { NativeExportTab, ForensicHashAlgorithm } from "../components/export/NativeExportMode";
import type { ToolsTabId } from "../components/export/ToolsMode";

/** Export operation mode */
export type ExportMode = "physical" | "logical" | "native" | "tools";

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

  // === Core State ===
  const [mode, setMode] = createSignal<ExportMode>("native");
  const [nativeExportTab, setNativeExportTab] = createSignal<NativeExportTab>("files");
  const [sources, setSources] = createSignal<string[]>(options.initialSources || []);
  const [destination, setDestination] = createSignal("");
  const [archiveName, setArchiveName] = createSignal("evidence.7z");
  const [exportName, setExportName] = createSignal("forensic_export");
  const [isProcessing, setIsProcessing] = createSignal(false);
  const [showAdvanced, setShowAdvanced] = createSignal(false);

  // === Export Options ===
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

  // === L01 Logical Evidence State ===
  const [l01ImageName, setL01ImageName] = createSignal("evidence");
  const [l01Compression, setL01Compression] = createSignal("none");
  const [l01HashAlgorithm, _setL01HashAlgorithm] = createSignal("md5");
  const [l01SegmentSize, setL01SegmentSize] = createSignal(2048);
  const [l01CaseNumber, setL01CaseNumber] = createSignal("");
  const [l01EvidenceNumber, setL01EvidenceNumber] = createSignal("");
  const [l01ExaminerName, setL01ExaminerName] = createSignal("");
  const [l01Description, setL01Description] = createSignal("");
  const [l01Notes, setL01Notes] = createSignal("");

  // === Drive Selector State ===
  const [showDriveSelector, setShowDriveSelector] = createSignal(false);
  const [driveSources, setDriveSources] = createSignal<Set<string>>(new Set());
  const [mountDrivesReadOnly, setMountDrivesReadOnly] = createSignal(false);

  // ─── Effects ────────────────────────────────────────────────────────────

  // Update size estimate when sources or compression level changes
  createEffect(() => {
    const sourceList = sources();
    const level = compressionLevel();
    if (sourceList.length > 0 && mode() === "native" && nativeExportTab() === "archive") {
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

  // ─── Export Handlers ────────────────────────────────────────────────────

  const handleCreateArchive = async () => {
    setIsProcessing(true);

    const activity = createActivity("archive", `${destination()}/${archiveName()}`, sources().length, {
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
      const archivePath = `${destination()}/${archiveName()}`;

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

      createArchive(archivePath, sources(), archiveOptions)
        .then((result) => {
          options.onActivityUpdate?.(activity.id, completeActivity(activity));
          toast.success("Archive Created", `Successfully created: ${result}`);
          options.onComplete?.(result);
        })
        .catch((error: unknown) => {
          options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
          toast.error("Archive Creation Failed", getErrorMessage(error));
        })
        .finally(() => {
          unlisten();
        });

      clearAllSources();
      setArchiveName("evidence.7z");
      setPassword("");
      setIsProcessing(false);

      toast.success("Archive Started", `Creating ${archiveName()} - check Activity panel for progress`);
    } catch (error: unknown) {
      unlisten();
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("Archive Creation Failed", getErrorMessage(error));
      setIsProcessing(false);
    }
  };

  const handleCreateE01Image = async () => {
    setIsProcessing(true);

    const outputPath = `${destination()}/${ewfImageName()}`;
    const shouldRestoreMounts = hasDriveSources() && mountDrivesReadOnly();

    const activity = createActivity("export", outputPath, sources().length, {
      operation: `E01 Image Creation (${ewfFormat()}, ${ewfCompression()})`,
    });

    options.onActivityCreate?.(activity);

    try {
      const ewfOptions = buildEwfExportOptions({
        sourcePaths: sources(),
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
          if (shouldRestoreMounts) {
            restoreAllDriveMounts();
          }
        });

      clearAllSources();
      setEwfImageName("evidence");
      setIsProcessing(false);

      toast.success("E01 Export Started", `Creating ${ewfImageName()}.E01 - check Activity panel for progress`);
    } catch (error: unknown) {
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("E01 Creation Failed", getErrorMessage(error));
      setIsProcessing(false);
    }
  };

  const handleCreateL01Image = async () => {
    setIsProcessing(true);

    const outputPath = `${destination()}/${l01ImageName()}`;
    const shouldRestoreMounts = hasDriveSources() && mountDrivesReadOnly();

    const activity = createActivity("export", outputPath, sources().length, {
      operation: `L01 Logical Evidence Creation (${l01Compression()})`,
    });

    options.onActivityCreate?.(activity);

    try {
      const l01Options = buildL01ExportOptions({
        sourcePaths: sources(),
        outputPath,
        compression: l01Compression(),
        hashAlgorithm: l01HashAlgorithm(),
        segmentSize: l01SegmentSize() > 0 ? l01SegmentSize() * 1024 * 1024 : undefined,
        caseNumber: l01CaseNumber() || undefined,
        evidenceNumber: l01EvidenceNumber() || undefined,
        examinerName: l01ExaminerName() || undefined,
        description: l01Description() || undefined,
        notes: l01Notes() || undefined,
      });

      createL01Image(l01Options, (prog) => {
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
          const ratio = result.compressionRatio
            ? ` | Ratio: ${(result.compressionRatio * 100).toFixed(1)}%`
            : "";
          toast.success(
            "L01 Image Created",
            `${result.totalFiles} files packaged (${formatBytes(result.totalDataBytes)})${hashInfo}${ratio}`,
          );
          options.onComplete?.(outputPath);
        })
        .catch((error: unknown) => {
          options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
          toast.error("L01 Creation Failed", getErrorMessage(error));
        })
        .finally(() => {
          if (shouldRestoreMounts) {
            restoreAllDriveMounts();
          }
        });

      clearAllSources();
      setL01ImageName("evidence");
      setIsProcessing(false);

      toast.success("L01 Export Started", `Creating ${l01ImageName()}.L01 - check Activity panel for progress`);
    } catch (error: unknown) {
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("L01 Creation Failed", getErrorMessage(error));
      setIsProcessing(false);
    }
  };

  const handleCopyOrExport = async () => {
    setIsProcessing(true);

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

    const activity = createActivity("export", destination(), sources().length, {
      includeHashes: copyOptions.computeHashes,
    });

    options.onActivityCreate?.(activity);

    exportFiles(sources(), destination(), copyOptions, (prog: CopyProgress) => {
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
          message += `\nManifest: ${result.jsonManifestPath.split("/").pop()}`;
        }
        if (result.txtReportPath) {
          message += `\nReport: ${result.txtReportPath.split("/").pop()}`;
        }

        toast.success("Export Complete", message);
        options.onComplete?.(destination());
      })
      .catch((error: unknown) => {
        options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
        toast.error("Export Failed", getErrorMessage(error));
      });

    clearAllSources();
    setExportName("forensic_export");
    setIsProcessing(false);

    toast.success("Export Started", `Exporting to ${destination()} - check Activity panel for progress`);
  };

  // ─── Main Start Handler ─────────────────────────────────────────────────

  const handleStart = async () => {
    if (sources().length === 0) {
      toast.error("No Sources", "Please select files or folders to export");
      return;
    }

    if (!destination()) {
      toast.error("No Destination", "Please select a destination folder");
      return;
    }

    // Confirmation check when drive sources are selected
    if (hasDriveSources()) {
      const driveList = Array.from(driveSources()).join(", ");
      const roNote = mountDrivesReadOnly()
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
    const mountOk = await remountDrivesIfNeeded();
    if (!mountOk) return;

    const currentMode = mode();

    if (currentMode === "physical") {
      await handleCreateE01Image();
    } else if (currentMode === "logical") {
      await handleCreateL01Image();
    } else if (currentMode === "native" && nativeExportTab() === "archive") {
      await handleCreateArchive();
    } else {
      await handleCopyOrExport();
    }
  };

  // ─── Tool Operation Handlers ────────────────────────────────────────────

  const handleTestArchive = async () => {
    setIsProcessing(true);
    const archivePath = testArchivePath();

    const activity = createActivity("tool", archivePath, 1, { operation: "Test Archive" });
    options.onActivityCreate?.(activity);

    try {
      const isValid = await testArchive(archivePath);
      options.onActivityUpdate?.(activity.id, completeActivity(activity));

      if (isValid) {
        toast.success("Archive Test Passed", `${archivePath.split("/").pop()} is valid`);
      } else {
        toast.warning("Archive Test Failed", `${archivePath.split("/").pop()} has integrity issues`);
      }

      setTestArchivePath("");
    } catch (error: unknown) {
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("Test Failed", getErrorMessage(error));
    } finally {
      setIsProcessing(false);
    }
  };

  const handleRepairArchive = async () => {
    setIsProcessing(true);
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
      toast.success("Archive Repaired", `Saved to: ${result.split("/").pop()}`);

      setRepairCorruptedPath("");
      setRepairOutputPath("");
    } catch (error: unknown) {
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("Repair Failed", getErrorMessage(error));
    } finally {
      unlisten();
      setIsProcessing(false);
    }
  };

  const handleValidateArchive = async () => {
    setIsProcessing(true);
    const archivePath = validateArchivePath();

    const activity = createActivity("tool", archivePath, 1, { operation: "Validate Archive" });
    options.onActivityCreate?.(activity);

    try {
      const validation = await validateArchive(archivePath);
      options.onActivityUpdate?.(activity.id, completeActivity(activity));

      if (validation.isValid) {
        toast.success("Validation Passed", `${archivePath.split("/").pop()} structure is valid`);
      } else {
        toast.error("Validation Failed", validation.errorMessage || "Archive has structural errors");
      }

      setValidateArchivePath("");
    } catch (error: unknown) {
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("Validation Failed", getErrorMessage(error));
    } finally {
      setIsProcessing(false);
    }
  };

  const handleExtractSplit = async () => {
    setIsProcessing(true);
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
      toast.success("Extraction Complete", `Files extracted to: ${result.split("/").pop()}`);

      setExtractFirstVolume("");
      setExtractOutputDir("");
    } catch (error: unknown) {
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("Extraction Failed", getErrorMessage(error));
    } finally {
      unlisten();
      setIsProcessing(false);
    }
  };

  const handleLzmaCompress = async () => {
    setIsProcessing(true);
    const input = lzmaInputPath();
    const output = lzmaOutputPath();
    const algo = lzmaAlgorithm();
    const level = lzmaLevel();

    const activity = createActivity(
      "tool",
      `${input.split("/").pop()} → ${output.split("/").pop()}`,
      1,
      { operation: `${algo.toUpperCase()} Compress (level ${level})` },
    );
    options.onActivityCreate?.(activity);

    try {
      const result =
        algo === "lzma" ? await compressToLzma(input, output, level) : await compressToLzma2(input, output, level);

      options.onActivityUpdate?.(activity.id, completeActivity(activity));
      toast.success("Compression Complete", `Output: ${result.split("/").pop()}`);

      setLzmaInputPath("");
      setLzmaOutputPath("");
    } catch (error: unknown) {
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("Compression Failed", getErrorMessage(error));
    } finally {
      setIsProcessing(false);
    }
  };

  const handleLzmaDecompress = async () => {
    setIsProcessing(true);
    const input = lzmaDecompressInput();
    const output = lzmaDecompressOutput();

    const activity = createActivity(
      "tool",
      `${input.split("/").pop()} → ${output.split("/").pop()}`,
      1,
      { operation: "LZMA Decompress" },
    );
    options.onActivityCreate?.(activity);

    try {
      const isXz = input.toLowerCase().endsWith(".xz");
      const result = isXz ? await decompressLzma2(input, output) : await decompressLzma(input, output);

      options.onActivityUpdate?.(activity.id, completeActivity(activity));
      toast.success("Decompression Complete", `Output: ${result.split("/").pop()}`);

      setLzmaDecompressInput("");
      setLzmaDecompressOutput("");
    } catch (error: unknown) {
      options.onActivityUpdate?.(activity.id, failActivity(activity, getErrorMessage(error)));
      toast.error("Decompression Failed", getErrorMessage(error));
    } finally {
      setIsProcessing(false);
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

  const handleReset = () => {
    clearAllSources();
    setDestination("");
    setArchiveName("evidence.7z");
    setExportName("forensic_export");
    setPassword("");
    setIsProcessing(false);

    // Reset forensic fields
    setGenerateManifest(true);
    setVerifyAfterCreate(true);
    setHashAlgorithm("SHA-256");
    setIncludeExaminerInfo(true);
    setExaminerName("");
    setCaseNumber("");
    setEvidenceDescription("");

    // Reset archive fields
    setCompressionLevel(CompressionLevel.Store);
    setSplitSizeMb(2048);
    setSolid(false);
    setNumThreads(0);

    // Reset tools fields
    setTestArchivePath("");
    setRepairCorruptedPath("");
    setRepairOutputPath("");
    setValidateArchivePath("");
    setExtractFirstVolume("");
    setExtractOutputDir("");

    // Reset EWF fields
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

    // Reset L01 fields
    setL01ImageName("evidence");
    setL01Compression("none");
    setL01SegmentSize(2048);
    setL01CaseNumber("");
    setL01EvidenceNumber("");
    setL01ExaminerName("");
    setL01Description("");
    setL01Notes("");

    toast.info("Form Reset", "All fields cleared");
  };

  // ─── Return ─────────────────────────────────────────────────────────────

  return {
    // Core state
    mode,
    setMode,
    nativeExportTab,
    setNativeExportTab,
    sources,
    destination,
    archiveName,
    setArchiveName,
    exportName,
    setExportName,
    isProcessing,
    showAdvanced,
    setShowAdvanced,

    // Export options
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

    // L01 state
    l01ImageName,
    setL01ImageName,
    l01Compression,
    setL01Compression,
    l01HashAlgorithm,
    l01SegmentSize,
    setL01SegmentSize,
    l01CaseNumber,
    setL01CaseNumber,
    l01EvidenceNumber,
    setL01EvidenceNumber,
    l01ExaminerName,
    setL01ExaminerName,
    l01Description,
    setL01Description,
    l01Notes,
    setL01Notes,

    // Drive selector state
    showDriveSelector,
    setShowDriveSelector,
    driveSources,
    mountDrivesReadOnly,

    // Handlers
    handleAddSources,
    handleAddFolder,
    handleDriveSelected,
    handleSelectDestination,
    handleRemoveSource,
    handleStart,
    handleToolAction,
    handleReset,
    hasDriveSources,
  } as const;
}

export type ExportState = ReturnType<typeof useExportState>;
