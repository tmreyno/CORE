// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, createMemo, For, Show, onMount, onCleanup } from "solid-js";
import { open, ask } from "@tauri-apps/plugin-dialog";
import {
  transferStart,
  transferCancel,
  onTransferProgress,
  onTransferComplete,
  type TransferRequest,
  type TransferResult,
} from "../transfer";
import type { HashAlgorithm, HashHistoryEntry } from "../types";
import { getPreference, getLastPath, setLastPath } from "./preferences";
import {
  HiOutlineFolderOpen,
  HiOutlineArrowUpTray,
  HiOutlinePlay,
  HiOutlineCog6Tooth,
  HiOutlineChevronRight,
  HiOutlineChevronDown,
} from "./icons";
import {
  SourceList,
  TransferJobCard,
  TransferOptions,
  detectContainerType,
  type TransferJob,
  type TransferPanelProps,
} from "./transfer";

export type { TransferJob, TransferPanelProps };

export function TransferPanel(props: TransferPanelProps) {
  const [sources, setSources] = createSignal<string[]>([]);
  const [destination, setDestination] = createSignal<string>("");
  const [verify, setVerify] = createSignal(true);
  const [hashAlgorithm, setHashAlgorithm] = createSignal<HashAlgorithm>("sha256");
  const [preserveTimestamps, setPreserveTimestamps] = createSignal(true);
  const [overwrite, setOverwrite] = createSignal(false);
  const [recursive, setRecursive] = createSignal(true);
  const [parallelThreads, setParallelThreads] = createSignal(4);
  const [showAdvanced, setShowAdvanced] = createSignal(false);
  const [internalJobs, setInternalJobs] = createSignal<TransferJob[]>([]);
  
  let unsubProgress: (() => void) | null = null;
  let unsubComplete: (() => void) | null = null;
  
  const activeJobs = () => props.activeJobs ?? internalJobs();
  const setActiveJobs = (updater: TransferJob[] | ((jobs: TransferJob[]) => TransferJob[])) => {
    const newJobs = typeof updater === "function" ? updater(activeJobs()) : updater;
    if (props.onActiveJobsChange) {
      props.onActiveJobsChange(newJobs);
    } else {
      setInternalJobs(newJobs);
    }
  };
  
  const runningJobs = createMemo(() => activeJobs().filter(j => j.status === "running" || j.status === "pending"));
  const completedJobs = createMemo(() => activeJobs().filter(j => j.status === "completed" || j.status === "failed" || j.status === "cancelled"));
  const hasContainers = createMemo(() => sources().some(p => detectContainerType(p) !== "unknown"));
  const canStart = createMemo(() => sources().length > 0 && destination().trim() !== "");
  
  createEffect(() => {
    const scanDir = props.scanDir;
    if (scanDir && scanDir.trim() !== "" && sources().length === 0) {
      setSources([scanDir]);
    }
  });
  
  createEffect(() => {
    props.onProgressUpdate?.(activeJobs());
  });
  
  onMount(async () => {
    unsubProgress = await onTransferProgress((progress) => {
      setActiveJobs(jobs => jobs.map(job => 
        job.id === progress.operation_id ? { ...job, progress, status: "running" as const } : job
      ));
    });
    
    unsubComplete = await onTransferComplete((result) => {
      setActiveJobs(jobs => jobs.map(job => {
        if (job.id === result.operation_id) {
          const updatedJob: TransferJob = { ...job, status: result.success ? "completed" : "failed", result, endTime: new Date() };
          processTransferHashes(updatedJob, result);
          return updatedJob;
        }
        return job;
      }));
      props.onTransferComplete?.(result);
    });
  });
  
  onCleanup(() => {
    unsubProgress?.();
    unsubComplete?.();
  });
  
  const processTransferHashes = (job: TransferJob, result: TransferResult) => {
    if (!props.onHashComputed || !result.files) return;
    const entries: HashHistoryEntry[] = [];
    const now = new Date();
    for (const file of result.files) {
      if (file.source_hash) {
        entries.push({
          algorithm: job.hashAlgorithm.toUpperCase(),
          hash: file.source_hash,
          timestamp: now,
          source: "computed",
          verified: file.verified,
          verified_against: file.destination_hash || undefined,
        });
      }
    }
    if (entries.length > 0) {
      props.onHashComputed(entries);
    }
  };
  
  const browseSource = async () => {
    try {
      const selected = await open({ multiple: true, directory: false, title: "Select Files to Export" });
      if (selected) {
        const paths = Array.isArray(selected) ? selected : [selected];
        setSources(prev => [...new Set([...prev, ...paths])]);
      }
    } catch (err) {
      console.error("Browse error:", err);
    }
  };
  
  const browseSourceFolder = async () => {
    try {
      const selected = await open({ multiple: false, directory: true, title: "Select Folder to Export" });
      if (selected && typeof selected === "string") {
        setSources(prev => [...new Set([...prev, selected])]);
      }
    } catch (err) {
      console.error("Browse error:", err);
    }
  };
  
  const browseDestination = async () => {
    try {
      const defaultPath = getLastPath("export");
      const selected = await open({ 
        multiple: false, 
        directory: true, 
        title: "Select Destination Folder",
        defaultPath,
      });
      if (selected && typeof selected === "string") {
        setLastPath("export", selected);
        setDestination(selected);
      }
    } catch (err) {
      console.error("Browse error:", err);
    }
  };
  
  const removeSource = (path: string) => {
    setSources(prev => prev.filter(p => p !== path));
  };
  
  const startTransfer = async () => {
    const src = sources();
    const dest = destination();
    if (src.length === 0 || !dest) return;
    
    // Check if confirmation is required
    const confirmRequired = getPreference("confirmBeforeExport");
    if (confirmRequired) {
      const confirmed = await ask(
        `Export ${src.length} file(s) to "${dest}"?\n\nThis operation will copy files with hash verification.`,
        { title: "Confirm Export", kind: "info" }
      );
      if (!confirmed) return;
    }
    
    const request: TransferRequest = {
      sources: src,
      destination: dest,
      verify: verify(),
      hashAlgorithm: hashAlgorithm(),
      preserveTimestamps: preserveTimestamps(),
      overwrite: overwrite(),
      recursive: recursive(),
      containerAware: hasContainers(),
      parallelThreads: parallelThreads(),
    };
    
    try {
      const jobId = await transferStart(request);
      const newJob: TransferJob = {
        id: jobId,
        sources: src,
        destination: dest,
        status: "pending",
        progress: null,
        result: null,
        startTime: new Date(),
        hashAlgorithm: hashAlgorithm(),
        containerAware: hasContainers(),
      };
      setActiveJobs(jobs => [newJob, ...jobs]);
    } catch (err) {
      console.error("Transfer start error:", err);
    }
  };
  
  const cancelTransfer = async (jobId: string) => {
    try {
      await transferCancel(jobId);
      setActiveJobs(jobs => jobs.map(job => job.id === jobId ? { ...job, status: "cancelled" as const } : job));
    } catch (err) {
      console.error("Cancel error:", err);
    }
  };
  
  const clearCompleted = () => {
    setActiveJobs(jobs => jobs.filter(j => j.status === "running" || j.status === "pending"));
  };

  return (
    <div class={`flex flex-col h-full bg-bg-primary overflow-hidden`}>
      <div class="flex items-center justify-between px-3 py-2 border-b border-border bg-bg-secondary">
        <div class="flex items-center gap-2">
          <HiOutlineArrowUpTray class="w-3.5 h-3.5" />
          <span class={`font-medium text-xs leading-normal`}>Export</span>
        </div>
        <Show when={runningJobs().length > 0}>
          <div class="flex items-center gap-1.5">
            <div class="bg-accent/20" />
            <span class={`text-[11px] leading-tight text-accent`}>{runningJobs().length} active</span>
          </div>
        </Show>
      </div>

      <div class="flex-1 overflow-y-auto">
        <Show when={activeJobs().length > 0}>
          <div class="px-3 pt-3 pb-2 space-y-2">
            <div class="flex items-center justify-between">
              <span class="text-xs font-medium text-txt-muted uppercase tracking-wide">Transfers</span>
              <Show when={completedJobs().length > 0}>
                <button onClick={clearCompleted} class={`text-[11px] leading-tight text-txt-secondary hover:text-txt-tertiary transition-colors`}>
                  Clear completed
                </button>
              </Show>
            </div>
            <div class="space-y-2">
              <For each={activeJobs()}>
                {(job) => <TransferJobCard job={job} onCancel={cancelTransfer} />}
              </For>
            </div>
          </div>
          <div class={`border-b border-border/50`} />
        </Show>

        <div class="px-3 py-3 space-y-3">
          <SourceList
            sources={sources()}
            onRemoveSource={removeSource}
            onBrowseFile={browseSource}
            onBrowseFolder={browseSourceFolder}
          />
          
          <div class="space-y-1.5">
            <label class="text-xs font-medium text-txt-muted uppercase tracking-wide">Destination</label>
            <div class="flex gap-2">
              <input
                type="text"
                value={destination()}
                onInput={(e) => setDestination(e.currentTarget.value)}
                placeholder="Select destination folder..."
                class={`flex-1 w-full px-2 py-1 text-xs bg-bg-tertiary border border-border rounded text-txt-primary placeholder:text-txt-muted focus:outline-none focus:ring-1 focus:ring-accent`}
              />
              <button onClick={browseDestination} class="px-2 py-1 text-xs rounded border border-border bg-bg-tertiary text-txt-primary hover:bg-bg-secondary transition-colors">
                <HiOutlineFolderOpen class="w-4 h-4" />
              </button>
            </div>
          </div>
          
          <button onClick={() => setShowAdvanced(v => !v)} class="flex items-center gap-1 text-xs text-txt-secondary hover:text-txt-primary cursor-pointer">
            <HiOutlineCog6Tooth class="w-3 h-3" />
            <span>{showAdvanced() ? "Hide" : "Show"} options</span>
            {showAdvanced() ? <HiOutlineChevronDown class="w-3 h-3" /> : <HiOutlineChevronRight class="w-3 h-3" />}
          </button>
          
          <Show when={showAdvanced()}>
            <TransferOptions
              verify={verify()}
              setVerify={setVerify}
              hashAlgorithm={hashAlgorithm()}
              setHashAlgorithm={(v) => setHashAlgorithm(v as HashAlgorithm)}
              preserveTimestamps={preserveTimestamps()}
              setPreserveTimestamps={setPreserveTimestamps}
              overwrite={overwrite()}
              setOverwrite={setOverwrite}
              recursive={recursive()}
              setRecursive={setRecursive}
              parallelThreads={parallelThreads()}
              setParallelThreads={setParallelThreads}
            />
          </Show>
          
          <button onClick={startTransfer} disabled={!canStart()} class={canStart() ? "px-4 py-1.5 text-sm font-medium rounded bg-accent text-white hover:bg-accent/90 transition-colors" : "px-4 py-1.5 text-sm font-medium rounded bg-bg-tertiary text-txt-muted cursor-not-allowed"}>
            <HiOutlinePlay class="w-4 h-4" />
            Start Export
          </button>
        </div>
      </div>
      
      <Show when={activeJobs().length === 0 && sources().length === 0}>
        <div class="flex flex-col items-center justify-center h-32 text-txt-muted text-sm">
          <HiOutlineArrowUpTray class="w-12 h-12 mb-3 opacity-30" />
          <span class={`text-xs leading-normal text-center`}>Select files to export with hash verification</span>
        </div>
      </Show>
    </div>
  );
}
