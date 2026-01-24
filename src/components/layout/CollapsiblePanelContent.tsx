// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * CollapsiblePanelContent - Unified view with collapsible sections
 * 
 * Shows all panel content (Evidence, Processed, Case Docs, Activity) in a single
 * scrollable view with collapsible sections. Alternative to tab-based navigation.
 */

import { Show, For, lazy, createSignal, createMemo, type Component, type Accessor, type Setter } from "solid-js";
import { EvidenceTree, CaseDocumentsPanel } from "../index";
import { ActivityPanel } from "../ActivityPanel";
import {
  HiOutlineArchiveBox,
  HiOutlineChartBar,
  HiOutlineClipboardDocumentList,
  HiOutlineClock,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
} from "../icons";
import type { SelectedEntry, TreeExpansionState } from "../index";
import type { DiscoveredFile, CaseDocument, ContainerInfo, HashHistoryEntry, ProcessedDatabase } from "../../types";
import type { useProcessedDatabases, FileStatus, FileHashInfo } from "../../hooks";

// Lazy-loaded components
const ProcessedDatabasePanel = lazy(() => import("../ProcessedDatabasePanel"));

// =============================================================================
// Types
// =============================================================================

type SectionId = "evidence" | "processed" | "casedocs" | "activity";

interface SectionConfig {
  id: SectionId;
  title: string;
  icon: Component<{ class?: string }>;
  badge?: () => number | string | null;
  color?: string;
}

export interface CollapsiblePanelContentProps {
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

// =============================================================================
// Component
// =============================================================================

export const CollapsiblePanelContent: Component<CollapsiblePanelContentProps> = (props) => {
  // Track which sections are expanded (default: evidence expanded)
  const [expandedSections, setExpandedSections] = createSignal<Set<SectionId>>(new Set(["evidence"]));
  
  // Section configurations with badges
  const sections = createMemo<SectionConfig[]>(() => [
    {
      id: "evidence",
      title: "Evidence Containers",
      icon: HiOutlineArchiveBox,
      badge: () => props.discoveredFiles().length || null,
      color: "text-type-e01",
    },
    {
      id: "processed",
      title: "Processed Results",
      icon: HiOutlineChartBar,
      badge: () => props.processedDbManager.databases().length || null,
      color: "text-info",
    },
    {
      id: "casedocs",
      title: "Case Documents",
      icon: HiOutlineClipboardDocumentList,
      badge: () => props.caseDocuments()?.length || null,
      color: "text-warning",
    },
    {
      id: "activity",
      title: "Activity",
      icon: HiOutlineClock,
      badge: () => null,
      color: "text-txt-muted",
    },
  ]);
  
  const toggleSection = (id: SectionId) => {
    setExpandedSections(prev => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };
  
  const isExpanded = (id: SectionId) => expandedSections().has(id);

  return (
    <div class="flex-1 flex flex-col min-w-0 overflow-y-auto overflow-x-hidden">
      <For each={sections()}>
        {(section) => (
          <div class="border-b border-border last:border-b-0">
            {/* Section Header */}
            <button
              class="w-full flex items-center gap-2 px-3 py-2 bg-bg-secondary hover:bg-bg-hover transition-colors text-left"
              onClick={() => toggleSection(section.id)}
            >
              {/* Expand/Collapse Icon */}
              <span class="text-txt-muted">
                <Show when={isExpanded(section.id)} fallback={<HiOutlineChevronRight class="w-3.5 h-3.5" />}>
                  <HiOutlineChevronDown class="w-3.5 h-3.5" />
                </Show>
              </span>
              
              {/* Section Icon */}
              <span class={section.color || "text-txt-secondary"}>
                <section.icon class="w-4 h-4" />
              </span>
              
              {/* Title */}
              <span class="flex-1 text-xs font-medium text-txt truncate">
                {section.title}
              </span>
              
              {/* Badge */}
              <Show when={section.badge?.()}>
                <span class="text-[10px] px-1.5 py-0.5 rounded-full bg-bg-hover text-txt-secondary font-medium">
                  {section.badge?.()}
                </span>
              </Show>
            </button>
            
            {/* Section Content */}
            <Show when={isExpanded(section.id)}>
              <div class="bg-bg" style={{ "max-height": "400px", "overflow-y": "auto" }}>
                {/* Evidence Tree */}
                <Show when={section.id === "evidence"}>
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
                </Show>
                
                {/* Processed Databases */}
                <Show when={section.id === "processed"}>
                  <ProcessedDatabasePanel 
                    manager={props.processedDbManager}
                    onSelectDatabase={(db) => {
                      props.processedDbManager.selectDatabase(db);
                      props.setActiveFile(null);
                    }}
                    onSelectArtifact={(db, artifact) => console.log('Selected artifact:', artifact.name, 'from', db.path)}
                  />
                </Show>
                
                {/* Case Documents */}
                <Show when={section.id === "casedocs"}>
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
                <Show when={section.id === "activity"}>
                  <ActivityPanel project={props.project()} />
                </Show>
              </div>
            </Show>
          </div>
        )}
      </For>
    </div>
  );
};

export default CollapsiblePanelContent;
