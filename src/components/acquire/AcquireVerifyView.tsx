// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * AcquireVerifyView — Hash verification panel for CORE Acquire edition.
 *
 * Simplified interface for computing and verifying file hashes.
 * Supports file/folder selection followed by hash computation.
 * Shows computed hash results inline per file.
 */

import {
  Component,
  Show,
  For,
  createSignal,
  type Accessor,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineFingerPrint,
  HiOutlineArrowLeft,
  HiOutlineDocument,
  HiOutlineFolderOpen,
  HiOutlineXMark,
  HiOutlineTrash,
  HiOutlineCheck,
  HiOutlineExclamationTriangle,
} from "../icons";

// =============================================================================
// Types
// =============================================================================

interface FileEntry {
  path: string;
  hash: string | null;
  error: string | null;
  hashing: boolean;
}

export interface AcquireVerifyViewProps {
  onBack: () => void;
  /** Hash all discovered evidence files */
  onHashAll: () => void;
  /** Number of discovered files */
  evidenceCount: Accessor<number>;
  /** Whether a project is loaded */
  hasProject: Accessor<boolean>;
}

// =============================================================================
// Component
// =============================================================================

const AcquireVerifyView: Component<AcquireVerifyViewProps> = (props) => {
  const [files, setFiles] = createSignal<FileEntry[]>([]);
  const [hashAlgorithm, setHashAlgorithm] = createSignal<"MD5" | "SHA-1" | "SHA-256">("SHA-256");
  const [isHashing, setIsHashing] = createSignal(false);

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
          .map(p => ({ path: p, hash: null, error: null, hashing: false }));
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
        setFiles(prev => [...prev, { path: selected, hash: null, error: null, hashing: false }]);
      }
    } catch { /* user cancelled */ }
  };

  const handleRemoveFile = (index: number) => {
    setFiles(prev => prev.filter((_, i) => i !== index));
  };

  const handleHashAll = async () => {
    if (files().length === 0) return;
    setIsHashing(true);

    // Mark all as hashing
    setFiles(prev => prev.map(f => ({ ...f, hashing: true, hash: null, error: null })));

    const algo = hashAlgorithm();
    for (let i = 0; i < files().length; i++) {
      const entry = files()[i];
      try {
        const hash = await invoke<string>("raw_verify", {
          inputPath: entry.path,
          algorithm: algo,
        });
        setFiles(prev =>
          prev.map((f, idx) => idx === i ? { ...f, hash, hashing: false } : f)
        );
      } catch (err) {
        const msg = err instanceof Error ? err.message : String(err);
        setFiles(prev =>
          prev.map((f, idx) => idx === i ? { ...f, error: msg, hashing: false } : f)
        );
      }
    }

    setIsHashing(false);
  };

  const hasUnhashed = () => files().some(f => !f.hash && !f.error);

  const basename = (path: string) => {
    const parts = path.replace(/\\/g, "/").split("/");
    return parts[parts.length - 1] || path;
  };

  return (
    <div class="acquire-panel">
      {/* Header */}
      <div class="acquire-panel-header">
        <button class="btn btn-ghost gap-1.5" onClick={props.onBack}>
          <HiOutlineArrowLeft class="w-4 h-4" />
          Back
        </button>
        <h2 class="text-lg font-medium text-txt">Verify Hashes</h2>
        <div class="w-20" />
      </div>

      {/* Body */}
      <div class="acquire-panel-body overflow-y-auto">
        <div class="p-6 max-w-2xl mx-auto space-y-6">
          {/* Intro */}
          <div class="text-center">
            <div class="inline-flex items-center justify-center w-14 h-14 rounded-2xl bg-rose-500/10 mb-3">
              <HiOutlineFingerPrint class="w-7 h-7 text-rose-400" />
            </div>
            <h3 class="text-base font-medium text-txt mb-1">Hash Verification</h3>
            <p class="text-xs text-txt-muted max-w-md mx-auto">
              Compute cryptographic hashes to verify the integrity of evidence files.
              Select individual files or hash all evidence in the current project.
            </p>
          </div>

          {/* Hash all evidence button (if project loaded) */}
          <Show when={props.hasProject() && props.evidenceCount() > 0}>
            <div class="card-interactive p-4 text-center">
              <p class="text-sm text-txt mb-2">
                Hash all <span class="font-medium text-accent">{props.evidenceCount()}</span> evidence file{props.evidenceCount() !== 1 ? "s" : ""} in the current project
              </p>
              <button
                class="btn btn-primary gap-1.5"
                onClick={props.onHashAll}
              >
                <HiOutlineFingerPrint class="w-4 h-4" />
                Hash All Evidence
              </button>
            </div>
          </Show>

          {/* Divider */}
          <Show when={props.hasProject() && props.evidenceCount() > 0}>
            <div class="flex items-center gap-3">
              <div class="flex-1 border-t border-border" />
              <span class="text-xs text-txt-muted uppercase">or hash individual files</span>
              <div class="flex-1 border-t border-border" />
            </div>
          </Show>

          {/* Add files section */}
          <div class="space-y-3">
            <div class="flex items-center gap-2">
              <button class="btn btn-secondary gap-1.5" onClick={handleAddFiles} disabled={isHashing()}>
                <HiOutlineDocument class="w-4 h-4" />
                Add Files
              </button>
              <button class="btn btn-secondary gap-1.5" onClick={handleAddFolder} disabled={isHashing()}>
                <HiOutlineFolderOpen class="w-4 h-4" />
                Add Folder
              </button>
              <Show when={files().length > 0 && !isHashing()}>
                <button
                  class="btn btn-ghost text-error gap-1.5 ml-auto"
                  onClick={() => setFiles([])}
                >
                  <HiOutlineTrash class="w-4 h-4" />
                  Clear
                </button>
              </Show>
            </div>

            {/* Algorithm selector + Start button */}
            <div class="flex items-center gap-3">
              <div class="form-group mb-0">
                <label class="label">Hash Algorithm</label>
                <select
                  class="input w-36"
                  value={hashAlgorithm()}
                  disabled={isHashing()}
                  onChange={(e) => setHashAlgorithm(e.currentTarget.value as "MD5" | "SHA-1" | "SHA-256")}
                >
                  <option value="SHA-256">SHA-256</option>
                  <option value="SHA-1">SHA-1</option>
                  <option value="MD5">MD5</option>
                </select>
              </div>
              <Show when={files().length > 0 && hasUnhashed()}>
                <button
                  class="btn btn-primary gap-1.5 mt-5"
                  disabled={isHashing()}
                  onClick={handleHashAll}
                >
                  <HiOutlineFingerPrint class="w-4 h-4" />
                  {isHashing() ? "Hashing…" : `Hash ${files().length} File${files().length !== 1 ? "s" : ""}`}
                </button>
              </Show>
            </div>

            {/* File list */}
            <Show when={files().length > 0}>
              <div class="space-y-1">
                <For each={files()}>
                  {(entry, index) => (
                    <div class="flex items-start gap-2 px-3 py-2 bg-bg-secondary rounded-lg border border-border">
                      {/* Status icon */}
                      <div class="shrink-0 mt-0.5">
                        <Show when={entry.hashing}>
                          <div class="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin" />
                        </Show>
                        <Show when={entry.hash && !entry.hashing}>
                          <HiOutlineCheck class="w-4 h-4 text-success" />
                        </Show>
                        <Show when={entry.error && !entry.hashing}>
                          <HiOutlineExclamationTriangle class="w-4 h-4 text-error" />
                        </Show>
                        <Show when={!entry.hash && !entry.error && !entry.hashing}>
                          <HiOutlineDocument class="w-4 h-4 text-txt-muted" />
                        </Show>
                      </div>
                      {/* Content */}
                      <div class="flex-1 min-w-0">
                        <div class="text-sm text-txt truncate">{basename(entry.path)}</div>
                        <div class="text-xs text-txt-muted truncate">{entry.path}</div>
                        <Show when={entry.hash}>
                          <div class="text-compact font-mono text-success mt-1 break-all select-all">
                            {entry.hash}
                          </div>
                        </Show>
                        <Show when={entry.error}>
                          <div class="text-xs text-error mt-1">{entry.error}</div>
                        </Show>
                      </div>
                      {/* Remove button (only when not hashing) */}
                      <Show when={!isHashing()}>
                        <button
                          class="icon-btn-sm text-txt-muted hover:text-error shrink-0"
                          onClick={() => handleRemoveFile(index())}
                        >
                          <HiOutlineXMark class="w-3.5 h-3.5" />
                        </button>
                      </Show>
                    </div>
                  )}
                </For>
              </div>
            </Show>
          </div>

          {/* Empty state */}
          <Show when={files().length === 0 && !(props.hasProject() && props.evidenceCount() > 0)}>
            <div class="flex flex-col items-center justify-center py-12 text-txt-muted">
              <HiOutlineFingerPrint class="w-10 h-10 opacity-20 mb-3" />
              <p class="text-sm">Add files to compute their hashes</p>
              <p class="text-xs mt-1">or open a project to hash all evidence files</p>
            </div>
          </Show>
        </div>
      </div>
    </div>
  );
};

export default AcquireVerifyView;
