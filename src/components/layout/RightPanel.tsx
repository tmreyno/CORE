// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, type Component, type Accessor, type Setter } from "solid-js";
import { MetadataPanel, TreePanel, TransferProgressPanel } from "../index";
import type { ParsedMetadata, TabViewMode, SelectedEntry, TransferJob } from "../index";
import type { ContainerInfo, DiscoveredFile } from "../../types";
import { transferCancel } from "../../transfer";

export interface RightPanelProps {
  // Panel state
  collapsed: Accessor<boolean>;
  width: Accessor<number>;
  
  // View mode
  currentViewMode: Accessor<TabViewMode>;
  setRequestViewMode: Setter<"info" | "hex" | "text" | "pdf" | "export" | null>;
  
  // Hex/Metadata view
  hexMetadata: Accessor<ParsedMetadata | null>;
  hexNavigator: Accessor<((offset: number, size?: number) => void) | null>;
  
  // File info
  activeFile: Accessor<DiscoveredFile | null>;
  activeFileInfo: Accessor<ContainerInfo | undefined>;
  selectedEntry: Accessor<SelectedEntry | null>;
  
  // Transfer jobs
  transferJobs: Accessor<TransferJob[]>;
  setTransferJobs: Setter<TransferJob[]>;
}

/**
 * RightPanel - Right sidebar with metadata, tree view, or transfer progress.
 * Switches view based on current view mode.
 */
export const RightPanel: Component<RightPanelProps> = (props) => {
  const handleCancelTransfer = async (jobId: string) => {
    try {
      await transferCancel(jobId);
      props.setTransferJobs(jobs => jobs.map(job =>
        job.id === jobId ? { ...job, status: "cancelled" as const } : job
      ));
    } catch (err) {
      console.error("Failed to cancel transfer:", err);
    }
  };

  return (
    <Show when={!props.collapsed()}>
      <aside class="right-panel" style={{ width: `${props.width()}px` }}>
        {/* Transfer indicator banner - shows when transfers active but not in export view */}
        <Show when={props.currentViewMode() !== "export" && props.transferJobs().some(j => j.status === "running" || j.status === "pending")}>
          <button
            onClick={() => props.setRequestViewMode("export")}
            class="w-full flex items-center justify-between gap-2 px-3 py-1.5 bg-accent/20 hover:bg-accent/30 border-b border-accent/30 text-accent text-xs cursor-pointer transition-colors"
            title="Click to view transfer details"
          >
            <span class="flex items-center gap-1.5">
              <span class="w-2 h-2 bg-accent rounded-full animate-pulse" />
              <span class="font-medium">
                {props.transferJobs().filter(j => j.status === "running").length} transfer{props.transferJobs().filter(j => j.status === "running").length !== 1 ? 's' : ''} running
              </span>
            </span>
            <span class="text-accent/70">View →</span>
          </button>
        </Show>
        
        <Show when={props.currentViewMode() === "export"}>
          <TransferProgressPanel 
            jobs={props.transferJobs()} 
            onCancelJob={handleCancelTransfer}
          />
        </Show>
        <Show when={props.currentViewMode() === "hex"}>
          <MetadataPanel 
            metadata={props.hexMetadata()}
            containerInfo={props.activeFileInfo()}
            fileInfo={props.activeFile() ? {
              path: props.activeFile()!.path,
              filename: props.activeFile()!.filename,
              size: props.activeFile()!.size,
              created: props.activeFile()!.created,
              modified: props.activeFile()!.modified,
              container_type: props.activeFile()!.container_type,
              segment_count: props.activeFile()!.segment_count
            } : null}
            selectedEntry={props.selectedEntry()}
            onRegionClick={(offset) => {
              // Request DetailPanel to switch to hex view mode
              props.setRequestViewMode("hex");
              
              // Retry function to wait for HexViewer to mount
              const tryNavigate = (attempts: number) => {
                const nav = props.hexNavigator();
                if (nav) {
                  nav(offset);
                } else if (attempts > 0) {
                  // HexViewer not mounted yet, retry
                  setTimeout(() => tryNavigate(attempts - 1), 100);
                }
              };
              
              // Start retrying after a small delay for view mode to switch
              setTimeout(() => tryNavigate(5), 100);
            }}
          />
        </Show>
        <Show when={props.currentViewMode() !== "hex" && props.currentViewMode() !== "export"}>
          <TreePanel info={props.activeFileInfo()} />
        </Show>
      </aside>
    </Show>
  );
};

export default RightPanel;
