// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * TransferProgressPanel - Right panel component showing transfer progress details
 * 
 * Displays detailed information about active and completed transfers when
 * Export view is active, replacing the standard metadata panel.
 */

import { For, Show, createMemo } from "solid-js";
import type { TransferJob } from "./TransferPanel";
import { formatBytes, formatSpeed, formatEta } from "../transfer";
import {
  HiOutlineArrowUpTray,
  HiOutlineCheckCircle,
  HiOutlineXCircle,
  HiOutlineClock,
  HiOutlineArrowPath,
  HiOutlineFolder,
  HiOutlineDocument,
  HiOutlineFingerPrint,
  HiOutlineChartBar,
  HiOutlineArchiveBox,
  HiOutlineStop,
} from "./icons";

interface TransferProgressPanelProps {
  /** Active transfer jobs */
  jobs: TransferJob[];
  /** Optional callback to cancel a job */
  onCancelJob?: (jobId: string) => void;
}

export type { TransferProgressPanelProps };

/** Format duration in human readable form */
function formatDuration(ms: number): string {
  if (ms < 1000) return `${ms}ms`;
  const seconds = Math.floor(ms / 1000);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  if (minutes < 60) return `${minutes}m ${remainingSeconds}s`;
  const hours = Math.floor(minutes / 60);
  const remainingMinutes = minutes % 60;
  return `${hours}h ${remainingMinutes}m`;
}

/** Format short date */
function formatShortTime(date: Date): string {
  return date.toLocaleTimeString(undefined, {
    hour: "2-digit",
    minute: "2-digit",
  });
}

export function TransferProgressPanel(props: TransferProgressPanelProps) {
  // Separate active and completed jobs
  const activeJobs = createMemo(() => 
    props.jobs.filter(j => j.status === "running" || j.status === "pending")
  );
  
  const completedJobs = createMemo(() => 
    props.jobs.filter(j => j.status === "completed" || j.status === "failed" || j.status === "cancelled")
  );
  
  // Stats
  const totalStats = createMemo(() => {
    const jobs = props.jobs;
    const active = activeJobs();
    const completed = completedJobs();
    
    let bytesTransferred = 0;
    let filesTransferred = 0;
    let currentSpeed = 0;
    
    for (const job of jobs) {
      if (job.progress) {
        bytesTransferred += job.progress.bytes_transferred;
        filesTransferred += job.progress.files_completed;
        if (job.status === "running") {
          currentSpeed += job.progress.bytes_per_second;
        }
      }
      if (job.result) {
        bytesTransferred += job.result.bytes_transferred;
        filesTransferred += job.result.successful_files;
      }
    }
    
    return {
      activeCount: active.length,
      completedCount: completed.length,
      bytesTransferred,
      filesTransferred,
      currentSpeed,
    };
  });

  // Row styles
  const rowBase = `grid gap-2 py-1 px-2 text-[10px] leading-tight items-baseline transition-colors`;
  const rowGrid = "grid-cols-[minmax(60px,1fr)_minmax(80px,2fr)]";
  const keyStyle = "text-txt-muted truncate";
  const valueStyle = "font-mono text-txt-tertiary truncate";
  const categoryHeader = "flex items-center gap-1.5 py-1.5 px-2 bg-bg-panel/50 cursor-pointer select-none hover:bg-bg-panel transition-colors";

  return (
    <div class={`flex flex-col h-full bg-bg-primary text-xs overflow-auto`}>
      {/* Header */}
      <div class="panel-header">
        <span class="flex items-center gap-1.5">
          <HiOutlineArrowUpTray class={`w-3.5 h-3.5 text-accent`} />
          <span class="font-semibold text-txt">Export Progress</span>
        </span>
      </div>

      {/* Show empty state when no jobs */}
      <Show when={props.jobs.length === 0}>
        <div class="flex flex-col items-center justify-center py-8 text-txt-muted">
          <HiOutlineArrowUpTray class="w-8 h-8 mb-2 opacity-40" />
          <span class="text-xs">No active transfers</span>
          <span class={`text-[10px] leading-tight text-txt-muted mt-1`}>Start an export to see progress here</span>
        </div>
      </Show>

      {/* Summary Stats */}
      <Show when={props.jobs.length > 0}>
        <div class={`border-b border-border/50 py-2 px-2`}>
          <div class="grid grid-cols-2 gap-2">
            <div class={`bg-bg-secondary rounded p-2`}>
              <div class={`text-[11px] leading-tight text-txt-muted`}>Active</div>
              <div class="font-mono text-lg text-accent">{totalStats().activeCount}</div>
            </div>
            <div class={`bg-bg-secondary rounded p-2`}>
              <div class={`text-[11px] leading-tight text-txt-muted`}>Completed</div>
              <div class="font-mono text-lg text-success">{totalStats().completedCount}</div>
            </div>
          </div>
          <div class="grid grid-cols-2 gap-2 mt-2">
            <div class={`bg-bg-secondary rounded p-2`}>
              <div class={`text-[11px] leading-tight text-txt-muted`}>Transferred</div>
              <div class="font-mono text-sm text-txt-tertiary">{formatBytes(totalStats().bytesTransferred)}</div>
            </div>
            <div class={`bg-bg-secondary rounded p-2`}>
              <div class={`text-[11px] leading-tight text-txt-muted`}>Speed</div>
              <div class="font-mono text-sm text-txt-tertiary">
                {totalStats().currentSpeed > 0 ? formatSpeed(totalStats().currentSpeed) : "—"}
              </div>
            </div>
          </div>
        </div>
      </Show>

      {/* Active Jobs */}
      <Show when={activeJobs().length > 0}>
        <div class={categoryHeader}>
          <HiOutlineArrowPath class={`w-3 h-3 animate-spin text-accent`} />
          <span class={`text-[11px] leading-tight font-medium text-txt-tertiary`}>
            Active Transfers ({activeJobs().length})
          </span>
        </div>
        <For each={activeJobs()}>
          {(job) => (
            <div class={`border-b border-border/50`}>
              {/* Job header with status and stop button */}
              <div class={`${rowBase} ${rowGrid} bg-accent/10`}>
                <span class={keyStyle}>Job ID</span>
                <div class="flex items-center justify-between">
                  <span class={`${valueStyle} text-accent`}>{job.id.slice(0, 8)}...</span>
                  <Show when={props.onCancelJob}>
                    <button
                      onClick={() => props.onCancelJob?.(job.id)}
                      class={`flex items-center gap-1 px-2 py-0.5 rounded text-red-400 hover:text-red-300 hover:bg-red-500/20 border border-red-500/30 hover:border-red-500/50 transition-colors ml-2`}
                      title="Stop transfer"
                    >
                      <HiOutlineStop class="w-2 h-2" />
                      <span class="text-[11px] leading-tight">Stop</span>
                    </button>
                  </Show>
                </div>
              </div>
              
              {/* Source */}
              <div class={`${rowBase} ${rowGrid}`}>
                <span class={keyStyle}>Source</span>
                <span class={valueStyle} title={job.sources.join(", ")}>
                  <Show when={job.containerAware} fallback={
                    <HiOutlineFolder class={`w-2 h-2 inline mr-1`} />
                  }>
                    <HiOutlineArchiveBox class={`w-2 h-2 inline mr-1 text-accent`} title="Forensic container" />
                  </Show>
                  {job.sources.length === 1 
                    ? job.sources[0].split("/").pop() 
                    : `${job.sources.length} items`}
                  <Show when={job.containerAware}>
                    <span class={`ml-1 text-accent text-[11px] leading-tight`}>(Container)</span>
                  </Show>
                </span>
              </div>
              
              {/* Destination */}
              <div class={`${rowBase} ${rowGrid}`}>
                <span class={keyStyle}>Dest</span>
                <span class={valueStyle} title={job.destination}>
                  {job.destination.split("/").pop()}
                </span>
              </div>

              {/* Progress details */}
              <Show when={job.progress}>
                {(progress) => (
                  <>
                    {/* Phase */}
                    <div class={`${rowBase} ${rowGrid}`}>
                      <span class={keyStyle}>Phase</span>
                      <span class={`${valueStyle} text-warning`}>{progress().phase}</span>
                    </div>

                    {/* Current file */}
                    <Show when={progress().current_file}>
                      <div class={`${rowBase} ${rowGrid}`}>
                        <span class={keyStyle}>File</span>
                        <span class={valueStyle} title={progress().current_file || ""}>
                          <HiOutlineDocument class={`w-2 h-2 inline mr-1`} />
                          {progress().current_file?.split("/").pop()}
                        </span>
                      </div>
                    </Show>
                    
                    {/* Progress bar */}
                    <div class="px-2 py-1">
                      <div class="flex items-center justify-between mb-1">
                        <span class={`text-[11px] leading-tight text-txt-muted`}>Progress</span>
                        <span class={`text-[11px] leading-tight font-mono text-txt-tertiary`}>
                          {progress().overall_percent.toFixed(1)}%
                        </span>
                      </div>
                      <div class="w-full h-2 bg-bg-hover rounded-full overflow-hidden">
                        <div 
                          class="h-full bg-accent transition-all duration-300"
                          style={{ width: `${progress().overall_percent}%` }}
                        />
                      </div>
                    </div>
                    
                    {/* Files */}
                    <div class={`${rowBase} ${rowGrid}`}>
                      <span class={keyStyle}>Files</span>
                      <span class={valueStyle}>
                        {progress().files_completed} / {progress().total_files}
                      </span>
                    </div>
                    
                    {/* Bytes */}
                    <div class={`${rowBase} ${rowGrid}`}>
                      <span class={keyStyle}>Data</span>
                      <span class={valueStyle}>
                        {formatBytes(progress().bytes_transferred)} / {formatBytes(progress().total_bytes)}
                      </span>
                    </div>
                    
                    {/* Speed */}
                    <div class={`${rowBase} ${rowGrid}`}>
                      <span class={keyStyle}>Speed</span>
                      <span class={`${valueStyle} text-accent`}>
                        {formatSpeed(progress().bytes_per_second)}
                      </span>
                    </div>
                    
                    {/* ETA */}
                    <div class={`${rowBase} ${rowGrid}`}>
                      <span class={keyStyle}>ETA</span>
                      <span class={valueStyle}>
                        <HiOutlineClock class={`w-2 h-2 inline mr-1`} />
                        {formatEta(progress().eta_seconds)}
                      </span>
                    </div>
                    
                    {/* Algorithm */}
                    <div class={`${rowBase} ${rowGrid}`}>
                      <span class={keyStyle}>Hash</span>
                      <span class={valueStyle}>
                        <HiOutlineFingerPrint class={`w-2 h-2 inline mr-1`} />
                        {job.hashAlgorithm.toUpperCase()}
                        {job.containerAware && " (container)"}
                      </span>
                    </div>
                  </>
                )}
              </Show>
            </div>
          )}
        </For>
      </Show>

      {/* Completed Jobs */}
      <Show when={completedJobs().length > 0}>
        <div class={categoryHeader}>
          <HiOutlineChartBar class={`w-3 h-3 text-txt-secondary`} />
          <span class={`text-[11px] leading-tight font-medium text-txt-tertiary`}>
            History ({completedJobs().length})
          </span>
        </div>
        <For each={completedJobs()}>
          {(job) => (
            <div class={`border-b border-border/50`}>
              {/* Status icon and time */}
              <div class={`${rowBase} ${rowGrid}`}>
                <span class="flex items-center gap-1">
                  <Show when={job.status === "completed"}>
                    <HiOutlineCheckCircle class={`w-3 h-3 text-success`} />
                  </Show>
                  <Show when={job.status === "failed"}>
                    <HiOutlineXCircle class={`w-3 h-3 text-error`} />
                  </Show>
                  <Show when={job.status === "cancelled"}>
                    <HiOutlineXCircle class={`w-3 h-3 text-warning`} />
                  </Show>
                </span>
                <span class={valueStyle}>
                  {formatShortTime(job.startTime)}
                  {job.endTime && ` — ${formatShortTime(job.endTime)}`}
                </span>
              </div>
              
              {/* Result details */}
              <Show when={job.result}>
                {(result) => (
                  <>
                    <div class={`${rowBase} ${rowGrid}`}>
                      <span class={keyStyle}>Files</span>
                      <span class={valueStyle}>
                        <span class="text-success">{result().successful_files}</span>
                        <Show when={result().failed_files > 0}>
                          {" / "}
                          <span class="text-error">{result().failed_files} failed</span>
                        </Show>
                      </span>
                    </div>
                    <div class={`${rowBase} ${rowGrid}`}>
                      <span class={keyStyle}>Size</span>
                      <span class={valueStyle}>{formatBytes(result().bytes_transferred)}</span>
                    </div>
                    <div class={`${rowBase} ${rowGrid}`}>
                      <span class={keyStyle}>Time</span>
                      <span class={valueStyle}>{formatDuration(result().duration_ms)}</span>
                    </div>
                    <Show when={result().error}>
                      <div class={`${rowBase} ${rowGrid}`}>
                        <span class={keyStyle}>Error</span>
                        <span class={`${valueStyle} text-error`}>{result().error}</span>
                      </div>
                    </Show>
                  </>
                )}
              </Show>
            </div>
          )}
        </For>
      </Show>
    </div>
  );
}
