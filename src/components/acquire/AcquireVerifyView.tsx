// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * AcquireVerifyView — Hash verification panel for CORE Acquire edition.
 *
 * Uses batch_hash backend with per-file progress events for parallel hashing.
 * Supports file/folder selection, algorithm choice, and shows real-time
 * progress bars per file with hash results, duration, and throughput.
 */

import {
  Component,
  Show,
  For,
  createSignal,
  createEffect,
  on,
  onCleanup,
  type Accessor,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import {
  HiOutlineFingerPrint,
  HiOutlineArrowDownTray,
  HiOutlineDocument,
  HiOutlineFolderOpen,
  HiOutlineXMark,
  HiOutlineTrash,
  HiOutlineCheck,
  HiOutlineExclamationTriangle,
  HiOutlineClipboard,
} from "../icons";
import { getContainerType } from "../EvidenceTree/containerDetection";
import { logger } from "../../utils/logger";
import AcquireProcessShell from "./AcquireProcessShell";

// =============================================================================
// Types
// =============================================================================

interface FileEntry {
  path: string;
  hash: string | null;
  error: string | null;
  hashing: boolean;
  percent: number;
  durationMs: number | null;
  throughputMbs: number | null;
}

export interface AcquireVerifyViewProps {
  onBack: () => void;
  /** When true, hides the top back-bar (used for inline dashboard expansion) */
  inline?: boolean;
  /** Hash all discovered evidence files */
  onHashAll: () => void;
  /** Number of discovered files */
  evidenceCount: Accessor<number>;
  /** Whether a project is loaded */
  hasProject: Accessor<boolean>;
  /** Pre-selected file paths from dashboard quick-verify */
  initialFiles?: Accessor<string[] | null>;
  /** Called after initialFiles have been consumed */
  onInitialFilesConsumed?: () => void;
}

// =============================================================================
// Component
// =============================================================================

const AcquireVerifyView: Component<AcquireVerifyViewProps> = (props) => {
  const log = logger.scope("AcquireVerify");
  const [files, setFiles] = createSignal<FileEntry[]>([]);
  const [hashAlgorithm, setHashAlgorithm] = createSignal<"MD5" | "SHA-1" | "SHA-256">("SHA-256");
  const [isHashing, setIsHashing] = createSignal(false);
  const [overallProgress, setOverallProgress] = createSignal({ completed: 0, total: 0 });
  const [copiedPath, setCopiedPath] = createSignal<string | null>(null);
  const [isDragOver, setIsDragOver] = createSignal(false);

  let unlistenProgress: UnlistenFn | null = null;
  let dragCounter = 0;

  onCleanup(() => {
    unlistenProgress?.();
  });

  // Auto-populate files from dashboard quick-verify
  createEffect(on(
    () => props.initialFiles?.(),
    (paths) => {
      if (paths && paths.length > 0) {
        const existing = new Set(files().map(f => f.path));
        const newEntries: FileEntry[] = paths
          .filter(p => !existing.has(p))
          .map(p => ({ path: p, hash: null, error: null, hashing: false, percent: 0, durationMs: null, throughputMbs: null }));
        if (newEntries.length > 0) setFiles(prev => [...prev, ...newEntries]);
        props.onInitialFilesConsumed?.();
      }
    },
  ));

  // --- Drag-and-drop handlers ---
  const handleDragEnter = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (isHashing()) return;
    dragCounter++;
    setIsDragOver(true);
  };

  const handleDragOver = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    if (e.dataTransfer) e.dataTransfer.dropEffect = "copy";
  };

  const handleDragLeave = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter--;
    if (dragCounter <= 0) {
      dragCounter = 0;
      setIsDragOver(false);
    }
  };

  const handleDrop = (e: DragEvent) => {
    e.preventDefault();
    e.stopPropagation();
    dragCounter = 0;
    setIsDragOver(false);
    if (isHashing()) return;

    const dt = e.dataTransfer;
    if (!dt?.files?.length) return;

    const existing = new Set(files().map(f => f.path));
    const newEntries: FileEntry[] = [];
    for (let i = 0; i < dt.files.length; i++) {
      const file = dt.files[i];
      // @ts-expect-error — path is available in Tauri/Electron runtimes
      const path: string | undefined = file.path;
      if (path && !existing.has(path)) {
        existing.add(path);
        newEntries.push({ path, hash: null, error: null, hashing: false, percent: 0, durationMs: null, throughputMbs: null });
      }
    }
    if (newEntries.length > 0) setFiles(prev => [...prev, ...newEntries]);
  };

  const handleAddFiles = async () => {
    log.debug("Opening file dialog for hash verification");
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: true,
        title: "Select files or forensic containers to hash",
      });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        const existing = new Set(files().map(f => f.path));
        const newEntries: FileEntry[] = paths
          .filter(p => !existing.has(p))
          .map(p => ({ path: p, hash: null, error: null, hashing: false, percent: 0, durationMs: null, throughputMbs: null }));
        if (newEntries.length > 0) setFiles(prev => [...prev, ...newEntries]);
      }
    } catch { /* user cancelled */ }
  };

  const handleAddFolder = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        directory: true,
        title: "Select folder containing files or containers to hash",
      });
      if (selected && !files().some(f => f.path === selected)) {
        setFiles(prev => [...prev, { path: selected, hash: null, error: null, hashing: false, percent: 0, durationMs: null, throughputMbs: null }]);
      }
    } catch { /* user cancelled */ }
  };

  const handleRemoveFile = (index: number) => {
    setFiles(prev => prev.filter((_, i) => i !== index));
  };

  const handleCopyHash = async (hash: string, path: string) => {
    try {
      await navigator.clipboard.writeText(hash);
      setCopiedPath(path);
      setTimeout(() => setCopiedPath(null), 2000);
    } catch { /* ignore */ }
  };

  const handleHashAll = async () => {
    const currentFiles = files();
    if (currentFiles.length === 0) return;

    log.info(`Starting batch hash: ${currentFiles.length} file(s), algorithm=${hashAlgorithm()}`);
    setIsHashing(true);
    setOverallProgress({ completed: 0, total: currentFiles.length });

    // Reset all entries
    setFiles(prev => prev.map(f => ({
      ...f, hashing: true, hash: null, error: null, percent: 0,
      durationMs: null, throughputMbs: null,
    })));

    // Track terminal events for safety net
    const terminatedFiles = new Set<string>();

    // Buffer progress updates (200ms throttle like useHashComputation)
    let pendingProgress = new Map<string, number>();
    let progressFlushTimer: ReturnType<typeof setTimeout> | null = null;

    const flushProgress = () => {
      const batch = pendingProgress;
      pendingProgress = new Map();
      progressFlushTimer = null;
      setFiles(prev => prev.map(f => {
        const pct = batch.get(f.path);
        return pct !== undefined ? { ...f, percent: Math.max(f.percent, pct) } : f;
      }));
    };

    unlistenProgress = await listen<{
      path: string;
      status: string;
      percent: number;
      filesCompleted: number;
      filesTotal: number;
      hash?: string;
      error?: string;
    }>("batch-progress", (e) => {
      const { path, status, percent, filesCompleted, filesTotal, hash, error } = e.payload;

      if (status === "progress" || status === "started") {
        pendingProgress.set(path, percent);
        if (!progressFlushTimer) {
          progressFlushTimer = setTimeout(flushProgress, 200);
        }
        setOverallProgress({ completed: filesCompleted, total: filesTotal });
      } else if (status === "completed") {
        terminatedFiles.add(path);
        setFiles(prev => prev.map(f =>
          f.path === path ? { ...f, hash: hash || null, hashing: false, percent: 100 } : f
        ));
        setOverallProgress({ completed: filesCompleted, total: filesTotal });
      } else if (status === "error") {
        terminatedFiles.add(path);
        setFiles(prev => prev.map(f =>
          f.path === path ? { ...f, error: error || "Unknown error", hashing: false, percent: 0 } : f
        ));
      }
    });

    try {
      const algo = hashAlgorithm();
      const batchFiles = currentFiles.map(f => ({
        path: f.path,
        containerType: getContainerType(f.path),
      }));

      const results = await invoke<Array<{
        path: string;
        hash: string | null;
        error: string | null;
        durationMs: number | null;
        throughputMbs: number | null;
      }>>("batch_hash", { files: batchFiles, algorithm: algo });

      // Apply results for any files that didn't get terminal progress events
      for (const result of results) {
        if (!terminatedFiles.has(result.path)) {
          setFiles(prev => prev.map(f =>
            f.path === result.path ? {
              ...f,
              hash: result.hash || null,
              error: result.error || null,
              hashing: false,
              percent: result.hash ? 100 : 0,
              durationMs: result.durationMs,
              throughputMbs: result.throughputMbs,
            } : f
          ));
        } else {
          // Update duration/throughput from results
          setFiles(prev => prev.map(f =>
            f.path === result.path ? {
              ...f,
              durationMs: result.durationMs,
              throughputMbs: result.throughputMbs,
            } : f
          ));
        }
      }
    } catch (err) {
      const msg = err instanceof Error ? err.message : String(err);
      log.error(`Batch hash failed: ${msg}`);
      setFiles(prev => prev.map(f => f.hashing ? { ...f, error: msg, hashing: false } : f));
    } finally {
      if (progressFlushTimer) clearTimeout(progressFlushTimer);
      unlistenProgress?.();
      unlistenProgress = null;
      setIsHashing(false);
      setOverallProgress(prev => ({ ...prev, completed: prev.total }));
      log.info(`Batch hash finished: ${completedCount()} succeeded, ${errorCount()} failed`);
    }
  };

  const hasUnhashed = () => files().some(f => !f.hash && !f.error);
  const completedCount = () => files().filter(f => f.hash).length;
  const errorCount = () => files().filter(f => f.error).length;

  const basename = (path: string) => {
    const parts = path.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || path;
  };

  const formatDuration = (ms: number) => {
    if (ms < 1000) return `${ms}ms`;
    if (ms < 60000) return `${(ms / 1000).toFixed(1)}s`;
    return `${Math.floor(ms / 60000)}m ${Math.round((ms % 60000) / 1000)}s`;
  };

  return (
    <AcquireProcessShell
      title="Verify Hashes"
      onBack={props.onBack}
      inline={props.inline}
      shellClass=""
      headerActions={(
        <>
          <Show when={isHashing()}>
            <div class="text-xs text-txt-muted flex items-center gap-2">
              <div class="w-24 h-1.5 bg-bg rounded-full overflow-hidden" role="progressbar" aria-valuenow={overallProgress().total > 0 ? Math.round((overallProgress().completed / overallProgress().total) * 100) : 0} aria-valuemin={0} aria-valuemax={100} aria-label="Overall hash progress">
                <div
                  class="h-full bg-accent rounded-full transition-all duration-200"
                  style={{ width: `${overallProgress().total > 0 ? (overallProgress().completed / overallProgress().total) * 100 : 0}%` }}
                />
              </div>
              <span>{overallProgress().completed}/{overallProgress().total}</span>
            </div>
          </Show>
          <Show when={!isHashing() && (completedCount() > 0 || errorCount() > 0)}>
            <div class="text-xs flex items-center gap-2">
              <Show when={completedCount() > 0}>
                <span class="text-success flex items-center gap-0.5">
                  <HiOutlineCheck class="w-3 h-3" />
                  {completedCount()} verified
                </span>
              </Show>
              <Show when={errorCount() > 0}>
                <span class="text-error flex items-center gap-0.5">
                  <HiOutlineExclamationTriangle class="w-3 h-3" />
                  {errorCount()} failed
                </span>
              </Show>
            </div>
          </Show>
        </>
      )}
    >
      <div
        class={`flex-1 overflow-y-auto relative transition-colors duration-150 ${
          isDragOver() ? "bg-accent/5" : ""
        }`}
        onDragEnter={handleDragEnter}
        onDragOver={handleDragOver}
        onDragLeave={handleDragLeave}
        onDrop={handleDrop}
      >
        {/* Drop overlay */}
        <Show when={isDragOver()}>
          <div class="absolute inset-0 z-10 flex items-center justify-center pointer-events-none">
            <div class="p-6 rounded-xl border-2 border-dashed border-accent bg-bg-panel/90 flex items-center gap-3">
              <HiOutlineArrowDownTray class="w-6 h-6 text-accent" />
              <span class="text-sm font-medium text-accent">Drop files or containers to add</span>
            </div>
          </div>
        </Show>

        <div class="flex-1 min-h-0 overflow-y-auto px-3 py-2 w-full max-w-[640px] mx-auto space-y-5">
          {/* Hash all evidence button (if project loaded) */}
          <Show when={props.hasProject() && props.evidenceCount() > 0}>
            <div class="callout">
              <div class="flex items-center gap-2">
                <HiOutlineFingerPrint class="w-5 h-5 text-accent" />
                <span class="text-sm text-txt">
                  Hash all <span class="font-medium text-accent">{props.evidenceCount()}</span> evidence file{props.evidenceCount() !== 1 ? "s" : ""}
                </span>
              </div>
              <button class="btn-action-primary gap-1" onClick={props.onHashAll} disabled={isHashing()}>
                <HiOutlineFingerPrint class="w-icon-sm h-icon-sm" />
                Hash All Evidence
              </button>
            </div>
          </Show>

          {/* Add files section */}
          <div class="space-y-3">
            <div class="flex items-center gap-2 flex-wrap">
              <div class="flex items-center gap-1.5">
                <button class="btn-sm gap-1" onClick={handleAddFiles} disabled={isHashing()} title="Add files or forensic containers (E01, L01, AD1, AFF4, disk images)">
                  <HiOutlineDocument class="w-icon-sm h-icon-sm" />
                  Add Files
                </button>
                <button class="btn-sm gap-1" onClick={handleAddFolder} disabled={isHashing()} title="Add a folder to hash all files within it">
                  <HiOutlineFolderOpen class="w-icon-sm h-icon-sm" />
                  Add Folder
                </button>
              </div>
              <div class="flex items-center gap-2 ml-auto">
                <select
                  class="input-xs w-28"
                  value={hashAlgorithm()}
                  disabled={isHashing()}
                  onChange={(e) => setHashAlgorithm(e.currentTarget.value as "MD5" | "SHA-1" | "SHA-256")}
                >
                  <option value="SHA-256">SHA-256</option>
                  <option value="SHA-1">SHA-1</option>
                  <option value="MD5">MD5</option>
                </select>
                <Show when={files().length > 0 && hasUnhashed()}>
                  <button
                    class="btn-sm gap-1"
                    disabled={isHashing()}
                    onClick={handleHashAll}
                  >
                    <HiOutlineFingerPrint class="w-icon-sm h-icon-sm" />
                    {isHashing() ? "Hashing…" : "Hash All"}
                  </button>
                </Show>
                <Show when={files().length > 0 && !isHashing()}>
                  <button class="icon-btn-sm text-txt-muted hover:text-error" onClick={() => setFiles([])} title="Clear all">
                    <HiOutlineTrash class="w-icon-sm h-icon-sm" />
                  </button>
                </Show>
              </div>
            </div>

            {/* File list with progress bars */}
            <Show when={files().length > 0}>
              <div class="space-y-1" role="list" aria-label="Files to hash">
                <For each={files()}>
                  {(entry, index) => (
                    <div
                      class="border border-border rounded-lg bg-bg-secondary overflow-hidden"
                      role="listitem"
                      tabIndex={0}
                      onKeyDown={(e) => {
                        if (e.key === "Delete" || e.key === "Backspace") {
                          if (!isHashing()) handleRemoveFile(index());
                        }
                      }}
                    >
                      <div class="flex items-center gap-2 px-3 py-2">
                        {/* Status icon */}
                        <Show when={entry.hashing}>
                          <div class="w-4 h-4 shrink-0">
                            <svg class="animate-spin w-4 h-4 text-accent" viewBox="0 0 24 24" fill="none">
                              <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" />
                              <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                            </svg>
                          </div>
                        </Show>
                        <Show when={entry.hash && !entry.hashing}>
                          <HiOutlineCheck class="w-4 h-4 shrink-0 text-success" />
                        </Show>
                        <Show when={entry.error && !entry.hashing}>
                          <HiOutlineExclamationTriangle class="w-4 h-4 shrink-0 text-error" />
                        </Show>
                        <Show when={!entry.hashing && !entry.hash && !entry.error}>
                          <HiOutlineDocument class="w-4 h-4 shrink-0 text-txt-muted" />
                        </Show>

                        {/* Filename + hash/error */}
                        <div class="flex-1 min-w-0">
                          <div class="text-sm text-txt truncate" title={entry.path}>
                            {basename(entry.path)}
                          </div>
                          <Show when={entry.hash}>
                            <div class="flex items-center gap-1 mt-0.5">
                              <span class="font-mono text-compact text-txt-muted truncate">{entry.hash}</span>
                              <button
                                class="icon-btn-sm shrink-0"
                                onClick={() => handleCopyHash(entry.hash!, entry.path)}
                                title="Copy hash"
                                aria-label="Copy hash to clipboard"
                              >
                                <Show when={copiedPath() === entry.path} fallback={
                                  <HiOutlineClipboard class="w-3 h-3 text-txt-muted" />
                                }>
                                  <HiOutlineCheck class="w-3 h-3 text-success" />
                                </Show>
                              </button>
                            </div>
                          </Show>
                          <Show when={entry.error}>
                            <div class="text-xs text-error mt-0.5 truncate" title={entry.error!}>
                              {entry.error}
                            </div>
                          </Show>
                        </div>

                        {/* Duration / throughput */}
                        <Show when={entry.durationMs !== null && entry.hash}>
                          <div class="text-xs text-txt-muted shrink-0 text-right">
                            <div>{formatDuration(entry.durationMs!)}</div>
                            <Show when={entry.throughputMbs !== null}>
                              <div>{entry.throughputMbs!.toFixed(0)} MB/s</div>
                            </Show>
                          </div>
                        </Show>

                        {/* Percent while hashing */}
                        <Show when={entry.hashing && entry.percent > 0}>
                          <span class="text-xs text-accent font-medium shrink-0 w-10 text-right">
                            {entry.percent.toFixed(0)}%
                          </span>
                        </Show>

                        {/* Remove button */}
                        <Show when={!isHashing()}>
                          <button
                            class="icon-btn-sm text-txt-muted hover:text-error shrink-0"
                            onClick={() => handleRemoveFile(index())}
                            title="Remove"
                            aria-label="Remove file from list"
                          >
                            <HiOutlineXMark class="w-icon-sm h-icon-sm" />
                          </button>
                        </Show>
                      </div>

                      {/* Progress bar */}
                      <Show when={entry.hashing}>
                        <div class="h-1 bg-bg" role="progressbar" aria-valuenow={Math.round(entry.percent)} aria-valuemin={0} aria-valuemax={100} aria-label={`Hashing ${entry.path.split("/").pop() || entry.path}`}>
                          <div
                            class="h-full bg-accent transition-all duration-200 ease-out"
                            style={{ width: `${entry.percent}%` }}
                          />
                        </div>
                      </Show>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </div>

          {/* Empty state — guided steps */}
          <Show when={files().length === 0 && !(props.hasProject() && props.evidenceCount() > 0)}>
            <div class="flex flex-col items-center justify-center py-16">
              <HiOutlineFingerPrint class="w-12 h-12 text-accent/15 mb-5" />
              <p class="text-base font-medium text-txt mb-1">Verify File Integrity</p>
              <p class="text-xs text-txt-muted mb-8">Compute and verify cryptographic hashes for forensic evidence</p>
              <div class="space-y-5 text-left max-w-sm w-full">
                <div class="flex items-start gap-3">
                  <div class="flex items-center justify-center w-6 h-6 rounded-full bg-accent/10 text-accent text-xs font-bold shrink-0 mt-0.5">1</div>
                  <div>
                    <p class="text-sm text-txt">Add files or folders</p>
                    <p class="text-xs text-txt-muted">Use the buttons above, or drag & drop files here</p>
                  </div>
                </div>
                <div class="flex items-start gap-3">
                  <div class="flex items-center justify-center w-6 h-6 rounded-full bg-accent/10 text-accent text-xs font-bold shrink-0 mt-0.5">2</div>
                  <div>
                    <p class="text-sm text-txt">Choose a hash algorithm</p>
                    <p class="text-xs text-txt-muted">SHA-256 (recommended), SHA-1, or MD5</p>
                  </div>
                </div>
                <div class="flex items-start gap-3">
                  <div class="flex items-center justify-center w-6 h-6 rounded-full bg-accent/10 text-accent text-xs font-bold shrink-0 mt-0.5">3</div>
                  <div>
                    <p class="text-sm text-txt">Click “Hash All” to compute</p>
                    <p class="text-xs text-txt-muted">Supports E01, L01, AD1, AFF4, and disk images</p>
                  </div>
                </div>
              </div>
            </div>
          </Show>
        </div>
      </div>
    </AcquireProcessShell>
  );
};

export default AcquireVerifyView;
