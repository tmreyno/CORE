// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Case Documents Panel - Displays discovered case documents (COC forms, etc.)
 * 
 * This panel shows Chain of Custody forms and other case-related documents
 * found in the case folder structure. Supports content-based COC detection.
 */

import { createSignal, Show, For, onMount, createMemo } from "solid-js";
import { formatBytes, formatDateByPreference } from "../utils";
import {
  HiOutlineClipboardDocumentList,
  HiOutlineDocumentText,
  HiOutlineFolderOpen,
  HiOutlineArrowPath,
  HiOutlineExclamationTriangle,
  HiOutlineMagnifyingGlass,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineXMark,
  HiOutlineArrowTopRightOnSquare,
  HiOutlineDocumentCheck,
  HiOutlineInbox,
  HiOutlineClipboard,
} from "./icons";
import type { CaseDocument, CaseDocumentType } from "../types";
import {
  findCaseDocuments,
  findCocForms,
  discoverCaseDocuments,
  getDocumentTypeLabel,
} from "../discovery";
import { invoke } from "@tauri-apps/api/core";

// ============================================================================
// Types
// ============================================================================

export interface CaseDocumentsPanelProps {
  /** Evidence path to discover documents from */
  evidencePath?: string;
  /** Direct search path (overrides evidence path discovery) */
  searchPath?: string;
  /** Show only COC forms */
  cocOnly?: boolean;
  /** Called when a document is selected (for hex/text viewing) */
  onDocumentSelect?: (doc: CaseDocument) => void;
  /** Called when user wants to open document externally */
  onDocumentOpen?: (doc: CaseDocument) => void;
  /** Called when user wants to view document as hex */
  onViewHex?: (doc: CaseDocument) => void;
  /** Called when user wants to view document as text */
  onViewText?: (doc: CaseDocument) => void;
  /** Cached documents from project (to avoid re-discovery) */
  cachedDocuments?: CaseDocument[];
  /** Callback when documents are loaded (for caching) */
  onDocumentsLoaded?: (docs: CaseDocument[], searchPath: string) => void;
  /** Additional CSS classes */
  class?: string;
}

// ============================================================================
// Document Type Styling
// ============================================================================

const documentTypeColors: Record<CaseDocumentType, string> = {
  ChainOfCustody: "text-accent bg-accent/10 border-accent/30",
  EvidenceIntake: "text-green-400 bg-green-500/10 border-green-500/30",
  CaseNotes: "text-yellow-400 bg-yellow-500/10 border-yellow-500/30",
  EvidenceReceipt: "text-purple-400 bg-purple-500/10 border-purple-500/30",
  LabRequest: "text-blue-400 bg-blue-500/10 border-blue-500/30",
  ExternalReport: "text-orange-400 bg-orange-500/10 border-orange-500/30",
  Other: "text-txt-secondary bg-bg-muted/10 border-border-subtle/30",
};

const documentTypeIcons: Record<CaseDocumentType, typeof HiOutlineClipboard> = {
  ChainOfCustody: HiOutlineClipboardDocumentList,
  EvidenceIntake: HiOutlineDocumentCheck,
  CaseNotes: HiOutlineDocumentText,
  EvidenceReceipt: HiOutlineClipboard,
  LabRequest: HiOutlineMagnifyingGlass,
  ExternalReport: HiOutlineDocumentText,
  Other: HiOutlineDocumentText,
};

// ============================================================================
// Component
// ============================================================================

export function CaseDocumentsPanel(props: CaseDocumentsPanelProps) {
  const [documents, setDocuments] = createSignal<CaseDocument[]>([]);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [searchPath, setSearchPath] = createSignal<string | null>(null);
  const [expandedTypes, setExpandedTypes] = createSignal<Set<CaseDocumentType>>(
    new Set(["ChainOfCustody", "EvidenceIntake"])
  );
  const [filterText, setFilterText] = createSignal("");
  const [selectedDoc, setSelectedDoc] = createSignal<CaseDocument | null>(null);

  // Group documents by type
  const groupedDocuments = createMemo(() => {
    const groups: Map<CaseDocumentType, CaseDocument[]> = new Map();
    const filter = filterText().toLowerCase();
    
    for (const doc of documents()) {
      // Apply text filter
      if (filter && !doc.filename.toLowerCase().includes(filter) &&
          !doc.path.toLowerCase().includes(filter)) {
        continue;
      }
      
      const list = groups.get(doc.document_type) || [];
      list.push(doc);
      groups.set(doc.document_type, list);
    }
    
    // Sort by type priority (COC first)
    const typeOrder: CaseDocumentType[] = [
      "ChainOfCustody",
      "EvidenceIntake",
      "EvidenceReceipt",
      "CaseNotes",
      "LabRequest",
      "ExternalReport",
      "Other",
    ];
    
    return typeOrder
      .filter(type => groups.has(type))
      .map(type => ({ type, docs: groups.get(type)! }));
  });

  // Count documents
  const documentCount = createMemo(() => documents().length);
  const cocCount = createMemo(() => 
    documents().filter(d => d.document_type === "ChainOfCustody").length
  );

  // Track state to prevent duplicate loads and infinite loops
  let isLoadingRef = false;

  // Single initialization effect - runs once when component mounts with valid path
  onMount(() => {
    const path = props.searchPath || props.evidencePath;
    
    // If we have cached documents, use them directly
    if (props.cachedDocuments && props.cachedDocuments.length > 0) {
      console.log("CaseDocumentsPanel: Using cached documents:", props.cachedDocuments.length);
      setDocuments(props.cachedDocuments);
      if (path) {
        setSearchPath(path);
      }
      return;
    }
    
    // Load documents if we have a path
    if (path) {
      console.log("CaseDocumentsPanel: Initial load from", path);
      setSearchPath(path);
      loadDocuments(path);
    }
  });

  async function loadDocuments(path: string) {
    // Prevent concurrent loads
    if (isLoadingRef) {
      console.log("CaseDocumentsPanel: skipping load, already loading");
      return;
    }
    
    isLoadingRef = true;
    console.log("CaseDocumentsPanel: loadDocuments called with path:", path);
    console.log("CaseDocumentsPanel: cocOnly =", props.cocOnly, ", searchPath =", props.searchPath);
    setLoading(true);
    setError(null);
    
    try {
      let docs: CaseDocument[];
      
      if (props.cocOnly) {
        // Only search for COC forms
        console.log("CaseDocumentsPanel: searching for COC forms only in:", path);
        docs = await findCocForms(path, true);
      } else if (props.searchPath) {
        // Direct search in specified path
        console.log("CaseDocumentsPanel: direct search in specified path:", path);
        docs = await findCaseDocuments(path, true);
      } else {
        // Discover from evidence path (finds case document folders)
        console.log("CaseDocumentsPanel: discovering from evidence path:", path);
        docs = await discoverCaseDocuments(path);
      }
      
      console.log("CaseDocumentsPanel: found", docs.length, "documents");
      if (docs.length > 0) {
        console.log("CaseDocumentsPanel: documents found:", docs.map(d => ({ 
          filename: d.filename, 
          type: d.document_type, 
          path: d.path 
        })));
      } else {
        console.log("CaseDocumentsPanel: No documents found. Expected folder names:", 
          ["case.documents", "case documents", "documents", "forms", "paperwork", "intake", "custody", "coc", "case.notes"]);
      }
      setDocuments(docs);
      
      // Notify parent of loaded documents for caching
      props.onDocumentsLoaded?.(docs, path);
      
      // Don't auto-select - let user choose
    } catch (e) {
      console.error("Failed to load case documents:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
      isLoadingRef = false;
    }
  }

  function toggleType(type: CaseDocumentType) {
    setExpandedTypes(prev => {
      const next = new Set(prev);
      if (next.has(type)) {
        next.delete(type);
      } else {
        next.add(type);
      }
      return next;
    });
  }

  function handleDocumentClick(doc: CaseDocument) {
    setSelectedDoc(doc);
    props.onDocumentSelect?.(doc);
  }

  async function handleOpenExternal(doc: CaseDocument) {
    try {
      // Use Tauri's shell open
      await invoke("plugin:opener|open_path", { path: doc.path });
    } catch (e) {
      console.error("Failed to open document:", e);
      // Try alternative method
      props.onDocumentOpen?.(doc);
    }
  }

  function formatDate(dateStr?: string | null): string {
    return formatDateByPreference(dateStr, false);
  }

  return (
    <div class={`flex flex-col h-full bg-bg ${props.class || ""}`}>
      {/* Header */}
      <div class="flex items-center justify-between px-3 py-2 border-b border-border">
        <div class="flex items-center gap-2">
          <HiOutlineClipboardDocumentList class="w-4 h-4 text-accent" />
          <span class="text-sm font-medium text-txt">Case Documents</span>
          <Show when={documentCount() > 0}>
            <span class="px-1.5 py-0.5 text-xs bg-bg-panel text-txt-secondary rounded">
              {documentCount()}
            </span>
          </Show>
          <Show when={cocCount() > 0}>
            <span class="px-1.5 py-0.5 text-xs bg-accent/20 text-accent rounded">
              {cocCount()} COC
            </span>
          </Show>
        </div>
        
        <Show when={searchPath()}>
          <button
            onClick={() => loadDocuments(searchPath()!)}
            class="p-1 text-txt-secondary hover:text-txt hover:bg-bg-panel rounded"
            title="Refresh"
          >
            <HiOutlineArrowPath class={`w-4 h-4 ${loading() ? "animate-spin" : ""}`} />
          </button>
        </Show>
      </div>

      {/* Search Filter */}
      <div class="px-3 py-2 border-b border-border">
        <div class="relative">
          <HiOutlineMagnifyingGlass class="absolute left-2 top-1/2 -translate-y-1/2 w-4 h-4 text-txt-muted" />
          <input
            type="text"
            value={filterText()}
            onInput={(e) => setFilterText(e.currentTarget.value)}
            placeholder="Filter documents..."
            class="w-full pl-8 pr-8 py-1.5 text-sm bg-bg-panel border border-border rounded 
                   text-txt placeholder-txt-muted focus:outline-none focus:border-accent/50"
          />
          <Show when={filterText()}>
            <button
              onClick={() => setFilterText("")}
              class="absolute right-2 top-1/2 -translate-y-1/2 p-0.5 text-txt-muted hover:text-txt-tertiary"
            >
              <HiOutlineXMark class="w-4 h-4" />
            </button>
          </Show>
        </div>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-y-auto">
        {/* Loading State */}
        <Show when={loading()}>
          <div class="flex flex-col items-center justify-center h-full py-8">
            <HiOutlineArrowPath class="w-8 h-8 text-accent animate-spin" />
            <p class="mt-2 text-sm text-txt-secondary">Searching for case documents...</p>
          </div>
        </Show>

        {/* Error State */}
        <Show when={error() && !loading()}>
          <div class="flex flex-col items-center justify-center h-full py-8 px-4">
            <HiOutlineExclamationTriangle class="w-8 h-8 text-red-400" />
            <p class="mt-2 text-sm text-red-400 text-center">{error()}</p>
            <button
              onClick={() => searchPath() && loadDocuments(searchPath()!)}
              class="mt-4 px-3 py-1.5 text-sm bg-bg-panel text-txt rounded hover:bg-bg-hover"
            >
              Try Again
            </button>
          </div>
        </Show>

        {/* Empty State */}
        <Show when={!loading() && !error() && documents().length === 0}>
          <div class="flex flex-col items-center justify-center h-full py-8 px-4">
            <HiOutlineInbox class="w-12 h-12 text-txt-muted" />
            <p class="mt-2 text-sm text-txt-secondary text-center">
              {searchPath() 
                ? "No case documents found in this location"
                : props.evidencePath
                  ? "No case documents found near evidence"
                  : "Open an evidence file to discover case documents"}
            </p>
            <Show when={searchPath() || props.evidencePath}>
              <p class="mt-3 text-xs text-txt-muted text-center max-w-xs">
                Expected folder names: <span class="text-txt-secondary">Case Documents</span>, <span class="text-txt-secondary">Documents</span>, <span class="text-txt-secondary">Forms</span>, <span class="text-txt-secondary">Paperwork</span>, <span class="text-txt-secondary">COC</span>, <span class="text-txt-secondary">Intake</span>
              </p>
              <p class="mt-1 text-xs text-txt-muted text-center max-w-xs">
                Supported files: PDF, DOCX, DOC, XLSX, XLS, TXT, RTF
              </p>
            </Show>
            <Show when={!searchPath() && props.evidencePath}>
              <button
                onClick={() => props.evidencePath && loadDocuments(props.evidencePath)}
                class="mt-4 px-3 py-1.5 text-sm bg-accent text-white rounded hover:bg-accent"
              >
                Retry Search
              </button>
            </Show>
          </div>
        </Show>

        {/* No Filter Results */}
        <Show when={!loading() && !error() && documents().length > 0 && groupedDocuments().length === 0}>
          <div class="flex flex-col items-center justify-center h-full py-8 px-4">
            <HiOutlineMagnifyingGlass class="w-8 h-8 text-txt-muted" />
            <p class="mt-2 text-sm text-txt-secondary text-center">
              No documents match "{filterText()}"
            </p>
          </div>
        </Show>

        {/* Document Groups */}
        <Show when={!loading() && !error() && groupedDocuments().length > 0}>
          <div class="divide-y divide-border">
            <For each={groupedDocuments()}>
              {(group) => {
                const Icon = documentTypeIcons[group.type];
                const isExpanded = () => expandedTypes().has(group.type);
                
                return (
                  <div>
                    {/* Group Header */}
                    <button
                      onClick={() => toggleType(group.type)}
                      class="w-full flex items-center gap-2 px-3 py-2 hover:bg-bg-panel/50 transition-colors"
                    >
                      {isExpanded() 
                        ? <HiOutlineChevronDown class="w-4 h-4 text-txt-muted" />
                        : <HiOutlineChevronRight class="w-4 h-4 text-txt-muted" />
                      }
                      <Icon class={`w-4 h-4 ${documentTypeColors[group.type].split(" ")[0]}`} />
                      <span class="text-sm font-medium text-txt">
                        {getDocumentTypeLabel(group.type)}
                      </span>
                      <span class="ml-auto text-xs text-txt-muted">
                        {group.docs.length}
                      </span>
                    </button>

                    {/* Documents List */}
                    <Show when={isExpanded()}>
                      <div class="pb-1">
                        <For each={group.docs}>
                          {(doc) => (
                            <div
                              onClick={() => handleDocumentClick(doc)}
                              class={`group flex items-start gap-2 px-3 py-2 pl-9 cursor-pointer
                                      hover:bg-bg-panel/50 transition-colors
                                      ${selectedDoc()?.path === doc.path ? "bg-accent/10" : ""}`}
                            >
                              <div class="flex-1 min-w-0">
                                <div class="flex items-center gap-2">
                                  <span class="text-sm text-txt truncate">
                                    {doc.filename}
                                  </span>
                                  <span class={`px-1.5 py-0.5 text-[10px] font-medium rounded border
                                                ${documentTypeColors[doc.document_type]}`}>
                                    {doc.format}
                                  </span>
                                </div>
                                
                                <div class="flex items-center gap-2 mt-0.5">
                                  <span class="text-xs text-txt-muted">
                                    {formatBytes(doc.size)}
                                  </span>
                                  <Show when={doc.modified}>
                                    <span class="text-xs text-txt-muted">•</span>
                                    <span class="text-xs text-txt-muted">
                                      {formatDate(doc.modified)}
                                    </span>
                                  </Show>
                                  <Show when={doc.case_number}>
                                    <span class="text-xs text-txt-muted">•</span>
                                    <span class="text-xs text-accent">
                                      Case: {doc.case_number}
                                    </span>
                                  </Show>
                                </div>
                              </div>

                              {/* Open External Button */}
                              <button
                                onClick={(e) => {
                                  e.stopPropagation();
                                  handleOpenExternal(doc);
                                }}
                                class="p-1 opacity-0 group-hover:opacity-100 text-txt-muted 
                                       hover:text-txt hover:bg-bg-hover rounded transition-opacity"
                                title="Open in default application"
                              >
                                <HiOutlineArrowTopRightOnSquare class="w-4 h-4" />
                              </button>
                              
                              {/* Hex View Button */}
                              <Show when={props.onViewHex}>
                                <button
                                  onClick={(e) => {
                                    e.stopPropagation();
                                    props.onViewHex?.(doc);
                                  }}
                                  class="p-1 opacity-0 group-hover:opacity-100 text-txt-muted 
                                         hover:text-accent hover:bg-bg-hover rounded transition-opacity"
                                  title="View as Hex"
                                >
                                  <span class="text-[10px] font-mono font-bold">HEX</span>
                                </button>
                              </Show>
                              
                              {/* Text View Button */}
                              <Show when={props.onViewText}>
                                <button
                                  onClick={(e) => {
                                    e.stopPropagation();
                                    props.onViewText?.(doc);
                                  }}
                                  class="p-1 opacity-0 group-hover:opacity-100 text-txt-muted 
                                         hover:text-green-400 hover:bg-bg-hover rounded transition-opacity"
                                  title="View as Text"
                                >
                                  <span class="text-[10px] font-mono font-bold">TXT</span>
                                </button>
                              </Show>
                            </div>
                          )}
                        </For>
                      </div>
                    </Show>
                  </div>
                );
              }}
            </For>
          </div>
        </Show>
      </div>

      {/* Footer - Selected Document Path */}
      <Show when={selectedDoc()}>
        <div class="px-3 py-2 border-t border-border bg-bg/50">
          <div class="flex items-center gap-2">
            <HiOutlineFolderOpen class="w-4 h-4 text-txt-muted flex-shrink-0" />
            <span class="text-xs text-txt-muted truncate" title={selectedDoc()!.path}>
              {selectedDoc()!.path}
            </span>
          </div>
        </div>
      </Show>
    </div>
  );
}

// ============================================================================
// Exports
// ============================================================================

export default CaseDocumentsPanel;
