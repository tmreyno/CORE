// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, type Component, type Accessor, type Setter } from "solid-js";
import { MetadataPanel, TreePanel } from "../index";
import { SimpleActivityPanel } from "../SimpleActivityPanel";
import type { ParsedMetadata, TabViewMode, SelectedEntry } from "../index";
import type { ContainerInfo, DiscoveredFile } from "../../types";
import type { Activity } from "../../types/activity";

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
  
  // Activities (simplified)
  activities: Accessor<Activity[]>;
  onCancelActivity?: (id: string) => void;
  onClearActivity?: (id: string) => void;
}

/**
 * RightPanel - Right sidebar with metadata, tree view, or activity progress.
 * Switches view based on current view mode.
 */
export const RightPanel: Component<RightPanelProps> = (props) => {
  return (
    <Show when={!props.collapsed()}>
      <aside class="right-panel" style={{ width: `${props.width()}px` }}>
        {/* Activity View */}
        <Show when={props.currentViewMode() === "export"}>
          <SimpleActivityPanel
            activities={props.activities()}
            onCancel={props.onCancelActivity}
            onClear={props.onClearActivity}
          />
        </Show>
        
        {/* Hex Metadata View */}
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
        
        {/* Tree View (default) */}
        <Show when={props.currentViewMode() !== "hex" && props.currentViewMode() !== "export"}>
          <TreePanel info={props.activeFileInfo()} />
        </Show>
      </aside>
    </Show>
  );
};
