// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Show, lazy, type Component, type Accessor, type Setter } from "solid-js";
import { EvidenceTree, CaseDocumentsPanel } from "../index";
import { ActivityPanel } from "../ActivityPanel";
import type { LeftPanelTab } from "./Sidebar";
import type { SelectedEntry, TreeExpansionState } from "../index";
import type { DiscoveredFile, CaseDocument, ContainerInfo, HashHistoryEntry, ProcessedDatabase } from "../../types";
import type { useProcessedDatabases, FileStatus, FileHashInfo } from "../../hooks";

// Lazy-loaded components with named exports
const ProcessedDatabasePanel = lazy(() => 
  import("../ProcessedDatabasePanel").then(m => ({ default: m.ProcessedDatabasePanel }))
);

export interface LeftPanelContentProps {
  // Active tab
  activeTab: Accessor<LeftPanelTab>;
  
  // Evidence Tree props
  discoveredFiles: Accessor<DiscoveredFile[]>;
  activeFile: Accessor<DiscoveredFile | null>;
  busy: Accessor<boolean>;
  onSelectContainer: (file: DiscoveredFile) => void;
  onSelectEntry: (entry: SelectedEntry) => void;
  typeFilter: Accessor<string | null>;
  onToggleTypeFilter: (type: string) => void;
  onClearTypeFilter: () => void;
  containerStats: Accessor<Record<string, number>>;
  onOpenNestedContainer: (tempPath: string, originalName: string, containerType: string, parentPath: string) => void;
  treeExpansionState: Accessor<TreeExpansionState | null>;
  onTreeExpansionStateChange: Setter<TreeExpansionState | null>;
  selectedFiles: Accessor<Set<string>>;
  fileHashMap: Accessor<Map<string, FileHashInfo>>;
  hashHistory: Accessor<Map<string, HashHistoryEntry[]>>;
  fileStatusMap: Accessor<Map<string, FileStatus>>;
  fileInfoMap: Accessor<Map<string, ContainerInfo>>;
  onToggleFileSelection: (path: string) => void;
  onHashFile: (file: DiscoveredFile) => void;
  onContextMenu: (file: DiscoveredFile, e: MouseEvent) => void;
  allFilesSelected: Accessor<boolean>;
  onToggleSelectAll: () => void;
  totalSize: Accessor<number>;
  
  // Processed Database props
  processedDbManager: ReturnType<typeof useProcessedDatabases>;
  onSelectProcessedDb: (db: ProcessedDatabase) => void;
  setActiveFile: Setter<DiscoveredFile | null>;
  
  // Case Documents props
  caseDocumentsPath: Accessor<string | null>;
  evidencePath: string | undefined;
  projectLocations: { case_documents_path?: string; evidence_path?: string } | undefined;
  caseDocuments: Accessor<CaseDocument[] | null>;
  onCaseDocumentsLoaded: (docs: CaseDocument[], searchPath: string) => void;
  onDocumentSelect: (doc: CaseDocument) => void;
  onViewHex: (doc: CaseDocument) => void;
  onViewText: (doc: CaseDocument) => void;
  
  // Activity props
  project: Accessor<import("../../hooks/project/types").FFXProject | null>;
  
  // Toast for notifications
  toast: {
    success: (title: string, message?: string) => void;
    error: (title: string, message?: string) => void;
    info: (title: string, message?: string) => void;
  };
}

/**
 * LeftPanelContent - The content area of the left panel.
 * Switches between Evidence, Processed DBs, Case Documents, and Activity tabs.
 */
export const LeftPanelContent: Component<LeftPanelContentProps> = (props) => {
  return (
    <div class="flex-1 flex flex-col min-w-0 overflow-hidden">
      {/* Tab Content - Using CSS visibility to preserve state across tab switches */}
      
      {/* Evidence Tree Tab */}
      <div class={`flex-1 flex flex-col min-w-0 overflow-hidden ${props.activeTab() === "evidence" ? "" : "hidden"}`}>
        <EvidenceTree
          discoveredFiles={props.discoveredFiles()}
          activeFile={props.activeFile()}
          busy={props.busy()}
          onSelectContainer={props.onSelectContainer}
          onSelectEntry={props.onSelectEntry}
          typeFilter={props.typeFilter()}
          onToggleTypeFilter={props.onToggleTypeFilter}
          onClearTypeFilter={props.onClearTypeFilter}
          containerStats={props.containerStats()}
          onOpenNestedContainer={(tempPath, originalName, containerType, parentPath) => {
            props.onOpenNestedContainer(tempPath, originalName, containerType, parentPath);
            props.toast.success("Nested Container", `Opened ${originalName}`);
          }}
          initialExpansionState={props.treeExpansionState() || undefined}
          onExpansionStateChange={props.onTreeExpansionStateChange}
          selectedFiles={props.selectedFiles()}
          fileHashMap={props.fileHashMap()}
          hashHistory={props.hashHistory()}
          fileStatusMap={props.fileStatusMap()}
          fileInfoMap={props.fileInfoMap()}
          onToggleFileSelection={props.onToggleFileSelection}
          onHashFile={props.onHashFile}
          onContextMenu={props.onContextMenu}
          allFilesSelected={props.allFilesSelected()}
          onToggleSelectAll={props.onToggleSelectAll}
          totalSize={props.totalSize()}
        />
      </div>

      {/* Processed Databases Tab */}
      <div class={`flex-1 flex flex-col min-w-0 overflow-hidden ${props.activeTab() === "processed" ? "" : "hidden"}`}>
        <ProcessedDatabasePanel 
          manager={props.processedDbManager}
          onSelectDatabase={(db) => {
            props.processedDbManager.selectDatabase(db);
            // Clear active forensic file when switching to processed view
            props.setActiveFile(null);
          }}
          onSelectArtifact={(db, artifact) => console.log('Selected artifact:', artifact.name, 'from', db.path)}
        />
      </div>

      {/* Case Documents Tab */}
      <Show when={props.activeTab() === "casedocs"}>
        <CaseDocumentsPanel 
          evidencePath={props.caseDocumentsPath() || props.evidencePath || props.projectLocations?.case_documents_path || props.projectLocations?.evidence_path}
          onDocumentSelect={props.onDocumentSelect}
          onViewHex={props.onViewHex}
          onViewText={props.onViewText}
          cachedDocuments={props.caseDocuments() ?? undefined}
          onDocumentsLoaded={props.onCaseDocumentsLoaded}
        />
      </Show>
      
      {/* Activity Panel */}
      <Show when={props.activeTab() === "activity"}>
        <ActivityPanel project={props.project()} />
      </Show>
    </div>
  );
};
