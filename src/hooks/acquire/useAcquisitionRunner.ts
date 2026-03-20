// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useAcquisitionRunner — Orchestrates sequential forensic acquisitions.
 *
 * Tasks execute in forensic priority order:
 *   Memory (0) → Triage (1) → Physical (2) → Logical (3) → Export (4)
 *
 * Each completed task:
 *   1. Writes a .ffx-companion.json sidecar file
 *   2. Creates an evidence collection record in .ffxdb
 *   3. Registers the output with the file manager (for auto-fill)
 */

import { createSignal, type Accessor } from "solid-js";
import type {
  AcquisitionTask,
  AcquisitionTaskConfig,
  AcquisitionTaskProgress,
  AcquisitionTaskResult,
  AcquisitionTaskType,
  AcquisitionPhase,
} from "./types";
import { ACQUISITION_PRIORITY, defaultConfig } from "./types";

// API wrappers
import {
  createE01Image,
  cancelE01Export,
  type EwfExportOptions,
  type EwfExportProgress,
  type EwfExportResult,
} from "../../api/ewfExport";
import {
  createL01Image,
  cancelL01Export,
  type L01ExportOptions,
  type L01ExportProgress,
  type L01ExportResult,
} from "../../api/l01Export";
import {
  createRawImage,
  cancelRawExport,
  type RawExportOptions,
  type RawExportProgress,
  type RawExportResult,
} from "../../api/rawExport";
import {
  captureMemory,
  cancelMemoryCapture,
  listenMemoryCaptureProgress,
  type MemoryCaptureResult,
} from "../../api/memory";
import {
  triageCollect,
  triageCancel,
  listenTriageProgress,
  type TriageOptions,
  type TriageResult,
} from "../../api/triage";

// Companion file + evidence collection creation
import {
  handleAcquisitionComplete,
  startAcquisitionRecord,
  type AcquisitionInfo,
} from "../export/companionHelper";

// DB sync for export tracking
import { dbSync } from "../project/useProjectDbSync";
import type { DbExportRecord } from "../../types/projectDb";

// =============================================================================
// Hook options
// =============================================================================

export interface AcquisitionRunnerOptions {
  /** Default output destination (evidence folder) */
  destination: Accessor<string>;
  /** Case metadata for companion files */
  caseNumber?: Accessor<string | undefined>;
  examiner?: Accessor<string | undefined>;
  /** System context for evidence records */
  hostname?: string;
  systemModel?: string;
  systemSerialNumber?: string;
  systemManufacturer?: string;
  osName?: string;
  osVersion?: string;
  /** Called when a task completes (for registering outputs) */
  onTaskComplete?: (task: AcquisitionTask) => void;
  /** Called when all tasks finish */
  onAllComplete?: (tasks: AcquisitionTask[]) => void;
}

// =============================================================================
// Hook
// =============================================================================

export function useAcquisitionRunner(opts: AcquisitionRunnerOptions) {
  const [tasks, setTasks] = createSignal<AcquisitionTask[]>([]);
  const [phase, setPhase] = createSignal<AcquisitionPhase>("idle");
  const [currentTaskId, setCurrentTaskId] = createSignal<string | null>(null);

  // Track the cancel function for the currently running task
  let cancelCurrentFn: (() => Promise<void>) | null = null;

  // ─── Task management (selection phase) ──────────────────────────────────

  function addTask(
    type: AcquisitionTaskType,
    source: string,
    sourceLabel: string,
    config?: Partial<AcquisitionTaskConfig>,
  ): string {
    const id = `task-${Date.now()}-${Math.random().toString(36).substring(2, 6)}`;
    const task: AcquisitionTask = {
      id,
      type,
      label: taskLabel(type),
      source,
      sourceLabel,
      status: "pending",
      config: { ...defaultConfig(type), ...config },
    };
    setTasks((prev) => [...prev, task]);
    return id;
  }

  function removeTask(id: string) {
    setTasks((prev) => prev.filter((t) => t.id !== id));
  }

  function updateConfig(id: string, config: Partial<AcquisitionTaskConfig>) {
    setTasks((prev) =>
      prev.map((t) => (t.id === id ? { ...t, config: { ...t.config, ...config } } : t)),
    );
  }

  function clearTasks() {
    setTasks([]);
    setPhase("idle");
    setCurrentTaskId(null);
    cancelCurrentFn = null;
  }

  // ─── Execution ──────────────────────────────────────────────────────────

  async function start() {
    if (phase() === "running") return;

    const dest = opts.destination();
    if (!dest) {
      console.error("[AcquisitionRunner] Cannot start: no output destination set");
      return;
    }

    setPhase("running");

    // Sort by forensic priority
    const sorted = [...tasks()].sort(
      (a, b) => ACQUISITION_PRIORITY[a.type] - ACQUISITION_PRIORITY[b.type],
    );

    for (const task of sorted) {
      // Skip if already completed/failed/cancelled (e.g. from a previous run)
      if (task.status !== "pending") continue;

      setCurrentTaskId(task.id);
      const startedAt = new Date().toISOString();
      updateTask(task.id, { status: "running", startedAt });

      // Create in-progress evidence collection record
      const { collectionId, itemId } = startAcquisitionRecord({
        acquisitionType: mapAcquisitionType(task),
        outputPath: buildOutputPath(task),
        sources: [task.source],
        caseNumber: task.config.caseNumber || opts.caseNumber?.(),
        examiner: task.config.examiner || opts.examiner?.(),
        hostname: opts.hostname,
        systemModel: opts.systemModel,
        systemSerialNumber: opts.systemSerialNumber,
        systemManufacturer: opts.systemManufacturer,
      });
      updateTask(task.id, { collectionId });

      // Create DB export record (in-progress)
      const exportId = `${task.type}-${Date.now()}`;
      const exportRecord: DbExportRecord = {
        id: exportId,
        exportType: mapAcquisitionType(task),
        destination: buildOutputPath(task),
        status: "in_progress",
        sourcePathsJson: JSON.stringify([task.source]),
        totalFiles: 1,
        totalBytes: 0,
        startedAt,
        initiatedBy: task.config.examiner || opts.examiner?.() || "",
        encrypted: false,
        archiveFormat: task.config.format || task.type,
      };
      dbSync.insertExport(exportRecord);

      try {
        const result = await executeTask(task);
        const completedAt = new Date().toISOString();
        const durationMs = new Date(completedAt).getTime() - new Date(startedAt).getTime();

        updateTask(task.id, {
          status: "completed",
          completedAt,
          result,
        });

        // Update DB export record
        dbSync.updateExport({
          ...exportRecord,
          status: "completed",
          totalBytes: result.outputSize,
          completedAt,
          manifestHash: result.hashes.sha256 || result.hashes.sha1 || result.hashes.md5 || "",
        });

        // Write companion file + update evidence collection
        handleAcquisitionComplete({
          acquisitionType: mapAcquisitionType(task),
          outputPath: result.outputPath,
          sources: [task.source],
          format: task.config.format || task.type,
          totalBytes: result.outputSize,
          totalFiles: result.totalFiles,
          segments: result.segments,
          md5: result.hashes.md5,
          sha1: result.hashes.sha1,
          sha256: result.hashes.sha256,
          startedAt,
          completedAt,
          durationMs,
          caseNumber: task.config.caseNumber || opts.caseNumber?.(),
          evidenceNumber: task.config.evidenceNumber,
          examiner: task.config.examiner || opts.examiner?.(),
          description: task.config.description,
          notes: task.config.notes,
          hostname: opts.hostname,
          systemModel: opts.systemModel,
          systemSerialNumber: opts.systemSerialNumber,
          systemManufacturer: opts.systemManufacturer,
          osName: opts.osName,
          osVersion: opts.osVersion,
          collectionId,
          itemId,
        });

        opts.onTaskComplete?.(tasks().find((t) => t.id === task.id)!);
      } catch (err: unknown) {
        const errorMsg = err instanceof Error ? err.message : String(err);
        const completedAt = new Date().toISOString();

        updateTask(task.id, {
          status: "failed",
          completedAt,
          error: errorMsg,
        });

        // Update DB export record
        dbSync.updateExport({
          ...exportRecord,
          status: "failed",
          completedAt,
          error: errorMsg,
        });
      } finally {
        cancelCurrentFn = null;
      }
    }

    setCurrentTaskId(null);
    setPhase("complete");
    opts.onAllComplete?.(tasks());
  }

  async function cancel() {
    if (cancelCurrentFn) {
      await cancelCurrentFn();
    }
    // Mark current task as cancelled
    const id = currentTaskId();
    if (id) {
      updateTask(id, { status: "cancelled", completedAt: new Date().toISOString() });
    }
  }

  // ─── Task execution per type ────────────────────────────────────────────

  async function executeTask(task: AcquisitionTask): Promise<AcquisitionTaskResult> {
    const dest = opts.destination();

    switch (task.type) {
      case "memory":
        return executeMemory(task, dest);
      case "triage":
        return executeTriage(task, dest);
      case "physical":
        return task.config.format === "raw"
          ? executePhysicalRaw(task, dest)
          : executePhysicalE01(task, dest);
      case "logical":
        return executeLogical(task, dest);
      case "export":
        return executeLogical(task, dest); // L01 for folder export
      default:
        throw new Error(`Unknown task type: ${task.type}`);
    }
  }

  // ── Memory ──────────────────────────────────────────────────────────────

  async function executeMemory(
    task: AcquisitionTask,
    dest: string,
  ): Promise<AcquisitionTaskResult> {
    const outputPath = `${dest}/memory_${formatTimestamp()}.mem`;

    cancelCurrentFn = async () => {
      await cancelMemoryCapture();
    };

    const unlisten = await listenMemoryCaptureProgress((p) => {
      updateProgress(task.id, {
        percent: p.percent,
        bytesProcessed: p.bytesCaptured,
        totalBytes: p.totalBytes,
        phase: p.phase,
      });
    });

    try {
      const result: MemoryCaptureResult = await captureMemory(outputPath, true);
      return {
        outputPath: result.outputPath,
        outputSize: result.bytesCaptured,
        hashes: {
          ...(result.hashMd5 ? { md5: result.hashMd5 } : {}),
          ...(result.hashSha256 ? { sha256: result.hashSha256 } : {}),
        },
        durationMs: Math.round(result.durationSecs * 1000),
      };
    } finally {
      unlisten();
    }
  }

  // ── Triage ──────────────────────────────────────────────────────────────

  async function executeTriage(
    task: AcquisitionTask,
    dest: string,
  ): Promise<AcquisitionTaskResult> {
    const outputDir = `${dest}/triage_${formatTimestamp()}`;

    cancelCurrentFn = async () => {
      await triageCancel();
    };

    const unlisten = await listenTriageProgress((p) => {
      updateProgress(task.id, {
        percent: p.percent,
        bytesProcessed: p.bytesCollected,
        totalBytes: 0,
        currentFile: p.currentFile,
        phase: `${p.phase} — ${p.currentCategory}`,
      });
    });

    try {
      const triageOpts: TriageOptions = {
        outputDir,
        categories: task.config.triageCategories || [],
        scanForSecrets: task.config.scanSecrets ?? true,
        containerFormat: "7z",
      };
      const result: TriageResult = await triageCollect(triageOpts);

      return {
        outputPath: result.containerPath || result.outputDir,
        outputSize: result.bytesCollected,
        hashes: {},
        durationMs: Math.round(result.durationSecs * 1000),
        totalFiles: result.filesCollected,
      };
    } finally {
      unlisten();
    }
  }

  // ── Physical E01 ───────────────────────────────────────────────────────

  async function executePhysicalE01(
    task: AcquisitionTask,
    dest: string,
  ): Promise<AcquisitionTaskResult> {
    const baseName = sanitizeFilename(task.sourceLabel);
    const outputPath = `${dest}/${baseName}`;

    cancelCurrentFn = async () => {
      await cancelE01Export(outputPath);
    };

    const ewfOpts: EwfExportOptions = {
      sourcePaths: [task.source],
      outputPath,
      format: "e01",
      compression: task.config.compression || "none",
      segmentSize: task.config.segmentSize,
      computeMd5: task.config.hashMd5 ?? true,
      computeSha1: task.config.hashSha1 ?? false,
      caseNumber: task.config.caseNumber,
      evidenceNumber: task.config.evidenceNumber,
      examinerName: task.config.examiner,
      description: task.config.description,
      notes: task.config.notes,
    };

    const result: EwfExportResult = await createE01Image(ewfOpts, (p: EwfExportProgress) => {
      updateProgress(task.id, {
        percent: p.percent,
        bytesProcessed: p.bytesWritten,
        totalBytes: p.totalBytes,
        currentFile: p.currentFile,
        phase: p.phase,
      });
    });

    return {
      outputPath: result.outputPath,
      outputSize: result.bytesWritten,
      hashes: {
        ...(result.md5Hash ? { md5: result.md5Hash } : {}),
        ...(result.sha1Hash ? { sha1: result.sha1Hash } : {}),
      },
      durationMs: result.durationMs,
      totalFiles: result.filesIncluded,
    };
  }

  // ── Physical Raw ───────────────────────────────────────────────────────

  async function executePhysicalRaw(
    task: AcquisitionTask,
    dest: string,
  ): Promise<AcquisitionTaskResult> {
    const baseName = sanitizeFilename(task.sourceLabel);
    const outputPath = `${dest}/${baseName}`;

    cancelCurrentFn = async () => {
      await cancelRawExport(outputPath);
    };

    const rawOpts: RawExportOptions = {
      sourcePaths: [task.source],
      outputPath,
      segmentSize: task.config.segmentSize,
      computeMd5: task.config.hashMd5 ?? true,
      computeSha1: task.config.hashSha1 ?? false,
      computeSha256: task.config.hashSha256 ?? true,
      caseNumber: task.config.caseNumber,
      evidenceNumber: task.config.evidenceNumber,
      examinerName: task.config.examiner,
      description: task.config.description,
      notes: task.config.notes,
    };

    const result: RawExportResult = await createRawImage(rawOpts, (p: RawExportProgress) => {
      updateProgress(task.id, {
        percent: p.percent,
        bytesProcessed: p.bytesWritten,
        totalBytes: p.totalBytes,
        currentFile: p.currentFile,
        phase: p.phase,
      });
    });

    return {
      outputPath: result.outputPath,
      outputSize: result.bytesWritten,
      hashes: {
        ...(result.md5Hash ? { md5: result.md5Hash } : {}),
        ...(result.sha1Hash ? { sha1: result.sha1Hash } : {}),
        ...(result.sha256Hash ? { sha256: result.sha256Hash } : {}),
      },
      durationMs: result.durationMs,
      totalFiles: result.filesIncluded,
      segments: result.segmentsCreated,
    };
  }

  // ── Logical L01 ────────────────────────────────────────────────────────

  async function executeLogical(
    task: AcquisitionTask,
    dest: string,
  ): Promise<AcquisitionTaskResult> {
    const baseName = sanitizeFilename(task.sourceLabel);
    const outputPath = `${dest}/${baseName}`;

    cancelCurrentFn = async () => {
      await cancelL01Export(outputPath);
    };

    const l01Opts: L01ExportOptions = {
      sourcePaths: [task.source],
      outputPath,
      compression: task.config.compression || "none",
      segmentSize: task.config.segmentSize,
      caseNumber: task.config.caseNumber,
      evidenceNumber: task.config.evidenceNumber,
      examinerName: task.config.examiner,
      description: task.config.description,
      notes: task.config.notes,
    };

    const result: L01ExportResult = await createL01Image(l01Opts, (p: L01ExportProgress) => {
      updateProgress(task.id, {
        percent: p.percent,
        bytesProcessed: p.bytesWritten,
        totalBytes: p.totalBytes,
        currentFile: p.currentFile,
        phase: p.phase,
      });
    });

    return {
      outputPath: result.outputPaths[0] || outputPath,
      outputSize: result.totalDataBytes,
      hashes: {
        ...(result.md5Hash ? { md5: result.md5Hash } : {}),
        ...(result.sha1Hash ? { sha1: result.sha1Hash } : {}),
      },
      durationMs: result.durationMs,
      totalFiles: result.totalFiles,
      segments: result.segmentCount,
    };
  }

  // ─── Helpers ────────────────────────────────────────────────────────────

  function updateTask(id: string, updates: Partial<AcquisitionTask>) {
    setTasks((prev) => prev.map((t) => (t.id === id ? { ...t, ...updates } : t)));
  }

  function updateProgress(id: string, progress: AcquisitionTaskProgress) {
    setTasks((prev) =>
      prev.map((t) => (t.id === id ? { ...t, progress } : t)),
    );
  }

  function toggleCollectionExpanded(id: string) {
    setTasks((prev) =>
      prev.map((t) =>
        t.id === id ? { ...t, collectionExpanded: !t.collectionExpanded } : t,
      ),
    );
  }

  function buildOutputPath(task: AcquisitionTask): string {
    const dest = opts.destination();
    const base = sanitizeFilename(task.sourceLabel);
    if (task.type === "memory") return `${dest}/memory_${formatTimestamp()}.mem`;
    if (task.type === "triage") return `${dest}/triage_${formatTimestamp()}`;
    return `${dest}/${base}`;
  }

  return {
    tasks,
    phase,
    currentTaskId,
    addTask,
    removeTask,
    updateConfig,
    clearTasks,
    start,
    cancel,
    toggleCollectionExpanded,
  };
}

// =============================================================================
// Utilities
// =============================================================================

function taskLabel(type: AcquisitionTaskType): string {
  switch (type) {
    case "memory":
      return "Memory Capture";
    case "triage":
      return "Quick Triage";
    case "physical":
      return "Disk Image";
    case "logical":
      return "Logical Image";
    case "export":
      return "File Export";
  }
}

function mapAcquisitionType(
  task: AcquisitionTask,
): AcquisitionInfo["acquisitionType"] {
  switch (task.type) {
    case "memory":
      return "memory";
    case "triage":
      return "triage";
    case "physical":
      return task.config.format === "raw" ? "raw" : "e01";
    case "logical":
      return "l01";
    case "export":
      return "archive";
  }
}

function sanitizeFilename(name: string): string {
  return name
    .replace(/[/\\:*?"<>|]/g, "_")
    .replace(/\s+/g, "_")
    .substring(0, 80);
}

function formatTimestamp(): string {
  const d = new Date();
  return [
    d.getFullYear(),
    String(d.getMonth() + 1).padStart(2, "0"),
    String(d.getDate()).padStart(2, "0"),
    "_",
    String(d.getHours()).padStart(2, "0"),
    String(d.getMinutes()).padStart(2, "0"),
    String(d.getSeconds()).padStart(2, "0"),
  ].join("");
}
