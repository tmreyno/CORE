// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For, createMemo } from "solid-js";
import { useParallelExtractor, type ExtractionJob } from "../hooks/useParallelExtractor";

const statusColors: Record<string, string> = {
  Queued: "text-txt-muted",
  Extracting: "text-accent",
  Verifying: "text-info",
  Completed: "text-success",
  Failed: "text-error",
  Cancelled: "text-txt-muted",
};

const statusBgColors: Record<string, string> = {
  Queued: "bg-bg-secondary",
  Extracting: "bg-accent",
  Verifying: "bg-info",
  Completed: "bg-success",
  Failed: "bg-error",
  Cancelled: "bg-bg-secondary",
};

export const ParallelExtractionPanel: Component = () => {
  const extractor = useParallelExtractor();

  const handleCancelBatch = async (batchId: string) => {
    if (confirm("Cancel this extraction batch?")) {
      await extractor.cancelBatch(batchId);
    }
  };

  const handleRemoveBatch = (batchId: string) => {
    extractor.removeBatch(batchId);
  };

  return (
    <div class="card flex flex-col gap-base h-full">
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-semibold text-txt">Parallel Extraction</h3>
        <Show when={!extractor.initialized()}>
          <span class="text-sm text-txt-muted">Initializing...</span>
        </Show>
      </div>

      <Show when={extractor.error()}>
        <div class="bg-error/10 border border-error text-error px-4 py-2 rounded-md text-sm">
          {extractor.error()}
        </div>
      </Show>

      {/* Overall Statistics */}
      <Show when={extractor.activeBatches().length > 0}>
        <div class="grid grid-cols-4 gap-4">
          <div class="stat-box">
            <div class="text-txt-muted text-xs">Total Batches</div>
            <div class="text-xl font-semibold text-txt">
              {extractor.overallStats().totalBatches}
            </div>
          </div>
          <div class="stat-box">
            <div class="text-txt-muted text-xs">Files</div>
            <div class="text-xl font-semibold text-txt">
              {extractor.overallStats().completedFiles} / {extractor.overallStats().totalFiles}
            </div>
          </div>
          <div class="stat-box">
            <div class="text-txt-muted text-xs">Data Extracted</div>
            <div class="text-xl font-semibold text-txt">
              {extractor.formatBytes(extractor.overallStats().extractedBytes)}
            </div>
          </div>
          <div class="stat-box">
            <div class="text-txt-muted text-xs">Avg Throughput</div>
            <div class="text-xl font-semibold text-txt">
              {extractor.overallStats().avgThroughputMbps.toFixed(1)} MB/s
            </div>
          </div>
        </div>
      </Show>

      {/* Active Batches */}
      <div class="border-t border-border pt-4 flex-1 overflow-auto">
        <h4 class="text-sm font-medium text-txt mb-2">Active Batches</h4>
        <Show
          when={extractor.activeBatches().length > 0}
          fallback={<div class="text-txt-muted text-sm">No active extractions</div>}
        >
          <div class="space-y-4">
            <For each={extractor.activeBatches()}>
              {(batch) => (
                <div class="bg-bg rounded-md p-4 border border-border">
                  {/* Batch Header */}
                  <div class="flex items-center justify-between mb-3">
                    <div>
                      <div class="text-sm font-medium text-txt">{batch.batchId}</div>
                      <div class="text-xs text-txt-muted">
                        {batch.completedFiles + batch.failedFiles + batch.cancelledFiles} /{" "}
                        {batch.totalFiles} files
                      </div>
                    </div>
                    <div class="flex items-center gap-2">
                      <div class="text-sm text-txt-secondary">
                        {batch.throughputMbps.toFixed(1)} MB/s
                      </div>
                      <Show when={batch.estimatedSecondsRemaining !== null}>
                        <div class="text-sm text-txt-muted">
                          ETA: {extractor.formatTime(batch.estimatedSecondsRemaining)}
                        </div>
                      </Show>
                      <button
                        onClick={() => handleCancelBatch(batch.batchId)}
                        class="px-2 py-1 text-xs hover:bg-bg-hover rounded-md transition-colors text-error"
                        disabled={
                          batch.completedFiles + batch.failedFiles + batch.cancelledFiles >=
                          batch.totalFiles
                        }
                      >
                        Cancel
                      </button>
                      <Show
                        when={
                          batch.completedFiles + batch.failedFiles + batch.cancelledFiles >=
                          batch.totalFiles
                        }
                      >
                        <button
                          onClick={() => handleRemoveBatch(batch.batchId)}
                          class="px-2 py-1 text-xs hover:bg-bg-hover rounded-md transition-colors text-txt-muted"
                        >
                          Remove
                        </button>
                      </Show>
                    </div>
                  </div>

                  {/* Overall Progress Bar */}
                  <div class="mb-3">
                    <div class="flex items-center justify-between text-xs text-txt-muted mb-1">
                      <span>Overall Progress</span>
                      <span>{batch.percentComplete.toFixed(1)}%</span>
                    </div>
                    <div class="w-full bg-bg-secondary rounded-full h-2">
                      <div
                        class="bg-accent h-2 rounded-full transition-all duration-300"
                        style={{ width: `${batch.percentComplete}%` }}
                      />
                    </div>
                    <div class="flex items-center justify-between text-xs text-txt-muted mt-1">
                      <span>{extractor.formatBytes(batch.extractedBytes)}</span>
                      <span>{extractor.formatBytes(batch.totalBytes)}</span>
                    </div>
                  </div>

                  {/* Status Breakdown */}
                  <div class="grid grid-cols-4 gap-2 mb-3">
                    <div class="text-center">
                      <div class="text-xs text-txt-muted">Completed</div>
                      <div class="text-sm font-medium text-success">{batch.completedFiles}</div>
                    </div>
                    <div class="text-center">
                      <div class="text-xs text-txt-muted">Failed</div>
                      <div class="text-sm font-medium text-error">{batch.failedFiles}</div>
                    </div>
                    <div class="text-center">
                      <div class="text-xs text-txt-muted">Cancelled</div>
                      <div class="text-sm font-medium text-txt-muted">{batch.cancelledFiles}</div>
                    </div>
                    <div class="text-center">
                      <div class="text-xs text-txt-muted">Active</div>
                      <div class="text-sm font-medium text-accent">
                        {batch.activeJobs.length}
                      </div>
                    </div>
                  </div>

                  {/* Active Jobs */}
                  <Show when={batch.activeJobs.length > 0}>
                    <div class="border-t border-border pt-3">
                      <div class="text-xs font-medium text-txt-muted mb-2">Active Files</div>
                      <div class="space-y-2 max-h-48 overflow-y-auto">
                        <For each={batch.activeJobs}>
                          {(job) => (
                            <JobCard job={job} formatBytes={extractor.formatBytes} />
                          )}
                        </For>
                      </div>
                    </div>
                  </Show>
                </div>
              )}
            </For>
          </div>
        </Show>
      </div>
    </div>
  );
};

// Individual job card component
const JobCard: Component<{
  job: ExtractionJob;
  formatBytes: (bytes: number) => string;
}> = (props) => {
  const fileName = createMemo(() => {
    const parts = props.job.entryPath.split("/");
    return parts[parts.length - 1];
  });

  return (
    <div class="bg-bg-secondary rounded p-2">
      <div class="flex items-center justify-between mb-1">
        <div class="flex items-center gap-2 flex-1 min-w-0">
          <span class={`text-xs font-medium ${statusColors[props.job.status]}`}>
            {props.job.status}
          </span>
          <span class="text-xs text-txt truncate" title={props.job.entryPath}>
            {fileName()}
          </span>
        </div>
        <span class="text-xs text-txt-muted">{props.formatBytes(props.job.sizeBytes)}</span>
      </div>

      <Show when={props.job.status === "Extracting" || props.job.status === "Verifying"}>
        <div class="w-full bg-bg rounded-full h-1.5 mb-1">
          <div
            class={`h-1.5 rounded-full transition-all duration-300 ${
              statusBgColors[props.job.status]
            }`}
            style={{ width: `${props.job.percentComplete}%` }}
          />
        </div>
        <div class="flex items-center justify-between text-xs text-txt-muted">
          <span>
            {props.formatBytes(props.job.bytesExtracted)} / {props.formatBytes(props.job.sizeBytes)}
          </span>
          <span>{props.job.percentComplete.toFixed(0)}%</span>
        </div>
      </Show>

      <Show when={props.job.status === "Failed" && props.job.errorMessage}>
        <div class="text-xs text-error mt-1">{props.job.errorMessage}</div>
      </Show>

      <Show when={props.job.status === "Completed" && props.job.computedHash}>
        <div class="text-xs text-success mt-1">
          ✓ {props.job.hashAlgorithm}: {props.job.computedHash?.substring(0, 16)}...
        </div>
      </Show>
    </div>
  );
};
