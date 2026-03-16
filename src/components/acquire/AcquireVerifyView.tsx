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
  HiOutlineArrowLeft,
  HiOutlineDocument,
  HiOutlineFolderOpen,
  HiOutlineXMark,
  HiOutlineTrash,
  HiOutlineCheck,
  HiOutlineExclamationTriangle,
  HiOutlineClipboard,
} from "../icons";

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
  const [files, setFiles] = createSignal<FileEntry[]>([]);
  const [hashAlgorithm, setHashAlgorithm] = createSignal<"MD5" | "SHA-1" | "SHA-256">("SHA-256");
  const [isHashing, setIsHashing] = createSignal(false);
  const [overallProgress, setOverallProgress] = createSignal({ completed: 0, total: 0 });
  const [copiedPath, setCopiedPath] = createSignal<string | null>(null);

  let unlistenProgress: UnlistenFn | null = null;

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

  const handleAddFiles = async () => {
    try {
      const { open } = await import("@tauri-apps/plugin-dialog");
      const selected = await open({
        multiple: true,
        title: "Select files to hash",
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
        title: "Select folder to hash",
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
        containerType: "raw",
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
      setFiles(prev => prev.map(f => f.hashing ? { ...f, error: msg, hashing: false } : f));
    } finally {
      if (progressFlushTimer) clearTimeout(progressFlushTimer);
      unlistenProgress?.();
      unlistenProgress = null;
      setIsHashing(false);
      setOverallProgress(prev => ({ ...prev, completed: prev.total }));
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
    <div class="flex flex-col flex-1 min-h-0 overflow-hidden">
      {/* Back nav */}
      <div class="flex items-center px-3 py-1.5 border-b border-border bg-bg-secondary shrink-0">
        <button class="btn btn-ghost gap-1 text-xs py-1 px-2" onClick={props.onBack}>
          <HiOutlineArrowLeft class="w-3.5 h-3.5" />
          Dashboard
        </button>
        <span class="ml-2 text-xs font-medium text-txt-muted">Verify Hashes</span>

        {/* Overall progress */}
        <Show when={isHashing()}>
          <div class="ml-auto flex items-center gap-2 text-xs text-txt-muted">
            <div class="w-24 h-1.5 bg-bg rounded-full overflow-hidden" role="progressbar" aria-valuenow={overallProgress().total > 0 ? Math.round((overallProgress().completed / overallProgress().total) * 100) : 0} aria-valuemin={0} aria-valuemax={100} aria-label="Overall hash progress">
              <div
                class="h-full bg-accent rounded-full transition-all duration-200"
                style={{ width: `${overallProgress().total > 0 ? (overallProgress().completed / overallProgress().total) * 100 : 0}%` }}
              />
            </div>
            <span>{overallProgress().completed}/{overallProgress().total}</span>
          </div>
        </Show>

        {/* Completed summary */}
        <Show when={!isHashing() && (completedCount() > 0 || errorCount() > 0)}>
          <div class="ml-auto flex items-center gap-2 text-xs">
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
      </div>

      {/* Body */}
      <div class="flex-1 overflow-y-auto">
        <div class="p-4 max-w-2xl mx-auto space-y-4">
          {/* Hash all evidence button (if project loaded) */}
          <Show when={props.hasProject() && props.evidenceCount() > 0}>
            <div class="flex items-center justify-between p-3 bg-bg-secondary rounded-lg border border-border">
              <div class="flex items-center gap-2">
                <HiOutlineFingerPrint class="w-4 h-4 text-rose-400" />
                <span class="text-sm text-txt">
                  Hash all <span class="font-medium text-accent">{props.evidenceCount()}</span> evidence file{props.evidenceCount() !== 1 ? "s" : ""}
                </span>
              </div>
              <button class="btn-sm gap-1" onClick={props.onHashAll} disabled={isHashing()}>
                Hash All Evidence
              </button>
            </div>
          </Show>

          {/* Add files section */}
          <div class="space-y-3">
            <div class="flex items-center gap-2">
              <button class="btn-sm gap-1" onClick={handleAddFiles} disabled={isHashing()}>
                <HiOutlineDocument class="w-3.5 h-3.5" />
                Add Files
              </button>
              <button class="btn-sm gap-1" onClick={handleAddFolder} disabled={isHashing()}>
                <HiOutlineFolderOpen class="w-3.5 h-3.5" />
                Add Folder
              </button>
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
                    <HiOutlineFingerPrint class="w-3.5 h-3.5" />
                    {isHashing() ? "Hashing…" : "Hash All"}
                  </button>
                </Show>
                <Show when={files().length > 0 && !isHashing()}>
                  <button class="icon-btn-sm text-txt-muted hover:text-error" onClick={() => setFiles([])} title="Clear all">
                    <HiOutlineTrash class="w-3.5 h-3.5" />
                  </button>
                </Show>
              </div>
            </div>

            {/* File list with progress bars */}
            <Show when={files().length > 0}>
              <div class="space-y-1">
                <For each={files()}>
                  {(entry, index) => (
                    <div class="border border-border rounded-lg bg-bg-secondary overflow-hidden">
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
                            <div class="text-2xs text-error mt-0.5 truncate" title={entry.error!}>
                              {entry.error}
                            </div>
                          </Show>
                        </div>

                        {/* Duration / throughput */}
                        <Show when={entry.durationMs !== null && entry.hash}>
                          <div class="text-2xs text-txt-muted shrink-0 text-right">
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
                            <HiOutlineXMark class="w-3.5 h-3.5" />
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

          {/* Empty state */}
          <Show when={files().length === 0 && !(props.hasProject() && props.evidenceCount() > 0)}>
            <div class="flex flex-col items-center justify-center py-8 text-txt-muted">
              <HiOutlineFingerPrint class="w-8 h-8 opacity-20 mb-2" />
              <p class="text-sm">Add files or folders to compute their hashes</p>
              <p class="text-2xs mt-1">Supports MD5, SHA-1, and SHA-256</p>
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
};

export default AcquireVerifyView;
