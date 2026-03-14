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

import { Show, lazy, type Component, type Accessor, type Setter, createSignal } from "solid-js";
import { EvidenceTree, CaseDocumentsPanel, CollapsiblePanelContent } from "../index";
import { ActivityPanel } from "../ActivityPanel";
import { BookmarksPanel } from "../BookmarksPanel";
import { NotesPanel } from "../notes";
import { ProjectDashboard } from "../ProjectDashboard";
import { logger } from "../../utils/logger";
import type { LeftPanelTab, LeftPanelMode } from "./Sidebar";

const DriveSourcePanel = lazy(() => import("../drives/DriveSourcePanel"));
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
  setActiveFile: Setter<DiscoveredFile | null>;

  processedDbManager: ReturnType<typeof useProcessedDatabases>;
  onSelectProcessedDb: (db: ProcessedDatabase) => void;
  onOpenProcessedDatabase: (db: ProcessedDatabase) => void;

  caseDocumentsPath: Accessor<string | null>;
  stableCaseDocsPath: Accessor<string | null>;
  caseDocuments: Accessor<CaseDocument[] | null>;
  setCaseDocuments: Setter<CaseDocument[] | null>;
  onDocumentSelect: (doc: CaseDocument) => void;

  projectManager: ReturnType<typeof useProject>;

  toast: {
    success: (title: string, message?: string) => void;
    error: (title: string, message?: string) => void;
    info: (title: string, message?: string) => void;
  };

  /** Navigate to a different left panel tab (from dashboard) */
  onNavigateTab?: (tab: string) => void;
  /** Open the export panel tab */
  onExport?: () => void;
  /** Open the report wizard */
  onReport?: () => void;
  /** Export selected drive sources — opens export panel with given paths and optional mode */
  onExportSources?: (paths: string[], mode?: "physical" | "logical" | "native") => void;
}

export const LeftPanelContent: Component<LeftPanelContentProps> = (props) => {
  const [bookmarkNotesTab, setBookmarkNotesTab] = createSignal<"bookmarks" | "notes">("bookmarks");

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
              log.debug(`Dashboard navigating to tab: ${tab}`);
              props.onNavigateTab?.(tab);
            }}
            onExport={props.onExport}
            onReport={props.onReport}
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
            cachedDocuments={props.caseDocuments() ?? undefined}
            onDocumentsLoaded={(docs, _searchPath) => props.setCaseDocuments(docs)}
          />
        </Show>

        <Show when={props.leftPanelTab() === "activity"}>
          <ActivityPanel project={props.projectManager.project()} />
        </Show>

        <Show when={props.leftPanelTab() === "drives"}>
          <DriveSourcePanel
            onExportSources={(paths, mode) => props.onExportSources?.(paths, mode)}
          />
        </Show>

        <Show when={props.leftPanelTab() === "bookmarks"}>
          <div class="flex flex-col h-full">
            {/* Sub-tab bar for Bookmarks / Notes */}
            <div class="flex items-center border-b border-border bg-bg-secondary shrink-0">
              <button
                class={`flex-1 px-3 py-2 text-xs font-medium transition-colors ${
                  bookmarkNotesTab() === "bookmarks"
                    ? "text-accent border-b-2 border-accent"
                    : "text-txt-muted hover:text-txt"
                }`}
                onClick={() => setBookmarkNotesTab("bookmarks")}
              >
                Bookmarks
                <Show when={(props.projectManager.project()?.bookmarks?.length ?? 0) > 0}>
                  <span class="ml-1 text-2xs text-txt-muted">
                    ({props.projectManager.project()?.bookmarks?.length ?? 0})
                  </span>
                </Show>
              </button>
              <button
                class={`flex-1 px-3 py-2 text-xs font-medium transition-colors ${
                  bookmarkNotesTab() === "notes"
                    ? "text-accent border-b-2 border-accent"
                    : "text-txt-muted hover:text-txt"
                }`}
                onClick={() => setBookmarkNotesTab("notes")}
              >
                Notes
                <Show when={(props.projectManager.project()?.notes?.length ?? 0) > 0}>
                  <span class="ml-1 text-2xs text-txt-muted">
                    ({props.projectManager.project()?.notes?.length ?? 0})
                  </span>
                </Show>
              </button>
            </div>

            {/* Bookmarks sub-panel */}
            <Show when={bookmarkNotesTab() === "bookmarks"}>
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

            {/* Notes sub-panel */}
            <Show when={bookmarkNotesTab() === "notes"}>
              <NotesPanel
                notes={props.projectManager.project()?.notes ?? []}
                onNavigate={(note) => {
                  if (note.target_path) {
                    const file = props.discoveredFiles().find(
                      (f) => f.path === note.target_path,
                    );
                    if (file) {
                      props.onSelectContainer(file);
                    }
                    props.toast.info("Navigated to note target", note.title);
                  }
                }}
                onRemove={(noteId) => {
                  props.projectManager.removeNote(noteId);
                  props.toast.success("Note removed");
                }}
                onUpdate={(noteId, updates) => {
                  props.projectManager.updateNote(noteId, updates);
                  props.toast.success("Note updated");
                }}
                onCreate={(noteData) => {
                  props.projectManager.addNote(noteData);
                  props.toast.success("Note created", noteData.title);
                }}
              />
            </Show>
          </div>
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
          project={props.projectManager.project}
          toast={props.toast}
        />
      </Show>
    </div>
  );
};
