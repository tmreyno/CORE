// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * LeftPanelContent - Tab-based and unified content for the left sidebar.
 *
 * Renders evidence tree, processed databases, case documents, activity,
 * and bookmarks panels based on the active left-panel tab or unified mode.
 */

import { Show, lazy, type Component, type Accessor, type Setter } from "solid-js";
import { EvidenceTree, CaseDocumentsPanel, CollapsiblePanelContent } from "../index";
import { ActivityPanel } from "../ActivityPanel";
import { BookmarksPanel } from "../BookmarksPanel";
import { ProjectDashboard } from "../ProjectDashboard";
import { logger } from "../../utils/logger";
import type { LeftPanelTab, LeftPanelMode } from "./Sidebar";
import type { SelectedEntry, TreeExpansionState } from "../index";
import type {
  DiscoveredFile,
  CaseDocument,
  ContainerInfo,
  HashHistoryEntry,
  ProcessedDatabase,
  ArtifactInfo,
} from "../../types";
import type { useProcessedDatabases, useProject, FileStatus, FileHashInfo } from "../../hooks";

const log = logger.scope("LeftPanelContent");

const ProcessedDatabasePanel = lazy(() =>
  import("../ProcessedDatabasePanel").then((m) => ({
    default: m.ProcessedDatabasePanel,
  })),
);

export interface LeftPanelContentProps {
  leftPanelMode: Accessor<LeftPanelMode>;
  leftPanelTab: Accessor<LeftPanelTab>;

  discoveredFiles: Accessor<DiscoveredFile[]>;
  activeFile: Accessor<DiscoveredFile | null>;
  busy: Accessor<boolean>;
  onSelectContainer: (file: DiscoveredFile) => void;
  onSelectEntry: (entry: SelectedEntry) => void;
  typeFilter: Accessor<string | null>;
  onToggleTypeFilter: (type: string) => void;
  onClearTypeFilter: () => void;
  containerStats: Accessor<Record<string, number>>;
  onOpenNestedContainer: (
    tempPath: string,
    originalName: string,
    containerType: string,
    parentPath: string,
  ) => void;
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
  setActiveFile: Setter<DiscoveredFile | null>;

  processedDbManager: ReturnType<typeof useProcessedDatabases>;
  onSelectProcessedDb: (db: ProcessedDatabase) => void;
  onOpenProcessedDatabase: (db: ProcessedDatabase) => void;

  caseDocumentsPath: Accessor<string | null>;
  stableCaseDocsPath: Accessor<string | null>;
  caseDocuments: Accessor<CaseDocument[] | null>;
  setCaseDocuments: Setter<CaseDocument[] | null>;
  onDocumentSelect: (doc: CaseDocument) => void;
  onViewHex: (doc: CaseDocument) => void;
  onViewText: (doc: CaseDocument) => void;

  projectManager: ReturnType<typeof useProject>;

  toast: {
    success: (title: string, message?: string) => void;
    error: (title: string, message?: string) => void;
    info: (title: string, message?: string) => void;
  };
}

export const LeftPanelContent: Component<LeftPanelContentProps> = (props) => {
  return (
    <div class="flex-1 flex flex-col min-w-0 overflow-hidden">
      {/* Tab-based Content */}
      <Show when={props.leftPanelMode() === "tabs"}>
        <Show when={props.leftPanelTab() === "dashboard"}>
          <ProjectDashboard
            project={() => props.projectManager.project()}
            discoveredFiles={props.discoveredFiles}
            fileHashMap={props.fileHashMap}
            bookmarkCount={() => props.projectManager.project()?.bookmarks?.length ?? 0}
            noteCount={() => props.projectManager.project()?.notes?.length ?? 0}
            onNavigateTab={(tab) => {
              // Allow dashboard to navigate to other tabs
              log.debug(`Dashboard navigating to tab: ${tab}`);
            }}
          />
        </Show>

        <div
          class={`flex-1 flex flex-col min-w-0 overflow-hidden ${
            props.leftPanelTab() === "evidence" ? "" : "hidden"
          }`}
        >
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
            onOpenNestedContainer={props.onOpenNestedContainer}
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

        <div
          class={`flex-1 flex flex-col min-w-0 overflow-hidden ${
            props.leftPanelTab() === "processed" ? "" : "hidden"
          }`}
        >
          <ProcessedDatabasePanel
            manager={props.processedDbManager}
            onSelectDatabase={props.onSelectProcessedDb}
            onSelectArtifact={(db: ProcessedDatabase, artifact: ArtifactInfo) => {
              log.debug(`Selected artifact: ${artifact.name} from ${db.path}`);
              props.processedDbManager.selectDatabase(db);
              props.onOpenProcessedDatabase(db);
            }}
          />
        </div>

        <Show when={props.leftPanelTab() === "casedocs"}>
          <CaseDocumentsPanel
            evidencePath={props.stableCaseDocsPath() ?? undefined}
            onDocumentSelect={props.onDocumentSelect}
            onViewHex={props.onViewHex}
            onViewText={props.onViewText}
            cachedDocuments={props.caseDocuments() ?? undefined}
            onDocumentsLoaded={(docs, _searchPath) => props.setCaseDocuments(docs)}
          />
        </Show>

        <Show when={props.leftPanelTab() === "activity"}>
          <ActivityPanel project={props.projectManager.project()} />
        </Show>

        <Show when={props.leftPanelTab() === "bookmarks"}>
          <BookmarksPanel
            bookmarks={props.projectManager.project()?.bookmarks ?? []}
            onNavigate={(bookmark) => {
              if (bookmark.target_type === "file") {
                const file = props.discoveredFiles().find(
                  (f) => f.path === bookmark.target_path,
                );
                if (file) {
                  props.onSelectContainer(file);
                }
              }
              props.toast.info("Navigated to bookmark", bookmark.name);
            }}
            onRemove={(bookmarkId) => {
              props.projectManager.removeBookmark(bookmarkId);
              props.toast.success("Bookmark removed");
            }}
            onEdit={(bookmark) => {
              props.toast.info("Edit bookmark", bookmark.name);
            }}
            onUpdate={(bookmarkId, updates) => {
              props.projectManager.updateBookmark(bookmarkId, updates);
              props.toast.success("Bookmark updated");
            }}
          />
        </Show>
      </Show>

      {/* Unified/Collapsible View */}
      <Show when={props.leftPanelMode() === "unified"}>
        <CollapsiblePanelContent
          discoveredFiles={props.discoveredFiles}
          activeFile={props.activeFile}
          busy={props.busy}
          onSelectContainer={props.onSelectContainer}
          onSelectEntry={props.onSelectEntry}
          typeFilter={props.typeFilter}
          onToggleTypeFilter={props.onToggleTypeFilter}
          onClearTypeFilter={props.onClearTypeFilter}
          containerStats={props.containerStats}
          onOpenNestedContainer={props.onOpenNestedContainer}
          treeExpansionState={props.treeExpansionState}
          onTreeExpansionStateChange={props.onTreeExpansionStateChange}
          selectedFiles={props.selectedFiles}
          fileHashMap={props.fileHashMap}
          hashHistory={props.hashHistory}
          fileStatusMap={props.fileStatusMap}
          fileInfoMap={props.fileInfoMap}
          onToggleFileSelection={props.onToggleFileSelection}
          onHashFile={props.onHashFile}
          onContextMenu={props.onContextMenu}
          allFilesSelected={props.allFilesSelected}
          onToggleSelectAll={props.onToggleSelectAll}
          totalSize={props.totalSize}
          processedDbManager={props.processedDbManager}
          onSelectProcessedDb={(db) => {
            props.processedDbManager.selectDatabase(db);
            props.setActiveFile(null);
          }}
          setActiveFile={props.setActiveFile}
          caseDocumentsPath={props.caseDocumentsPath}
          evidencePath={props.stableCaseDocsPath() ?? undefined}
          projectLocations={props.projectManager.projectLocations() ?? undefined}
          caseDocuments={props.caseDocuments}
          onCaseDocumentsLoaded={(docs, _searchPath) => props.setCaseDocuments(docs)}
          onDocumentSelect={props.onDocumentSelect}
          onViewHex={props.onViewHex}
          onViewText={props.onViewText}
          project={props.projectManager.project}
          toast={props.toast}
        />
      </Show>
    </div>
  );
};
