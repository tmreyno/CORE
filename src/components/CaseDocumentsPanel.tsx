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

import { createSignal, createEffect, Show, For, onMount, createMemo } from "solid-js";
import { formatBytes } from "../utils";
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
  getDocumentTypeIcon,
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
  /** Called when a document is selected */
  onDocumentSelect?: (doc: CaseDocument) => void;
  /** Called when user wants to open document externally */
  onDocumentOpen?: (doc: CaseDocument) => void;
  /** Additional CSS classes */
  class?: string;
}

// ============================================================================
// Document Type Styling
// ============================================================================

const documentTypeColors: Record<CaseDocumentType, string> = {
  ChainOfCustody: "text-cyan-400 bg-cyan-500/10 border-cyan-500/30",
  EvidenceIntake: "text-green-400 bg-green-500/10 border-green-500/30",
  CaseNotes: "text-yellow-400 bg-yellow-500/10 border-yellow-500/30",
  EvidenceReceipt: "text-purple-400 bg-purple-500/10 border-purple-500/30",
  LabRequest: "text-blue-400 bg-blue-500/10 border-blue-500/30",
  ExternalReport: "text-orange-400 bg-orange-500/10 border-orange-500/30",
  Other: "text-zinc-400 bg-zinc-500/10 border-zinc-500/30",
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

  // Load documents when path changes
  createEffect(() => {
    const path = props.searchPath || props.evidencePath;
    if (path && path !== searchPath()) {
      setSearchPath(path);
      loadDocuments(path);
    }
  });

  // Also try loading on mount if path is already available
  onMount(() => {
    const path = props.searchPath || props.evidencePath;
    if (path) {
      console.log("CaseDocumentsPanel: onMount loading from", path);
      setSearchPath(path);
      loadDocuments(path);
    } else {
      console.log("CaseDocumentsPanel: onMount - no path available");
    }
  });

  async function loadDocuments(path: string) {
    console.log("CaseDocumentsPanel: loadDocuments called with path:", path);
    setLoading(true);
    setError(null);
    
    try {
      let docs: CaseDocument[];
      
      if (props.cocOnly) {
        // Only search for COC forms
        console.log("CaseDocumentsPanel: searching for COC forms only");
        docs = await findCocForms(path, true);
      } else if (props.searchPath) {
        // Direct search in specified path
        console.log("CaseDocumentsPanel: direct search in specified path");
        docs = await findCaseDocuments(path, true);
      } else {
        // Discover from evidence path (finds case document folders)
        console.log("CaseDocumentsPanel: discovering from evidence path");
        docs = await discoverCaseDocuments(path);
      }
      
      console.log("CaseDocumentsPanel: found", docs.length, "documents:", docs);
      setDocuments(docs);
      
      // Auto-select first COC if found
      const firstCoc = docs.find(d => d.document_type === "ChainOfCustody");
      if (firstCoc) {
        setSelectedDoc(firstCoc);
        props.onDocumentSelect?.(firstCoc);
      }
    } catch (e) {
      console.error("Failed to load case documents:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
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
    if (!dateStr) return "";
    try {
      const date = new Date(dateStr);
      return date.toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
      });
    } catch {
      return "";
    }
  }

  return (
    <div class={`flex flex-col h-full bg-zinc-900 ${props.class || ""}`}>
      {/* Header */}
      <div class="flex items-center justify-between px-3 py-2 border-b border-zinc-800">
        <div class="flex items-center gap-2">
          <HiOutlineClipboardDocumentList class="w-4 h-4 text-cyan-400" />
          <span class="text-sm font-medium text-zinc-200">Case Documents</span>
          <Show when={documentCount() > 0}>
            <span class="px-1.5 py-0.5 text-xs bg-zinc-800 text-zinc-400 rounded">
              {documentCount()}
            </span>
          </Show>
          <Show when={cocCount() > 0}>
            <span class="px-1.5 py-0.5 text-xs bg-cyan-500/20 text-cyan-400 rounded">
              {cocCount()} COC
            </span>
          </Show>
        </div>
        
        <Show when={searchPath()}>
          <button
            onClick={() => loadDocuments(searchPath()!)}
            class="p-1 text-zinc-400 hover:text-zinc-200 hover:bg-zinc-800 rounded"
            title="Refresh"
          >
            <HiOutlineArrowPath class={`w-4 h-4 ${loading() ? "animate-spin" : ""}`} />
          </button>
        </Show>
      </div>

      {/* Search Filter */}
      <div class="px-3 py-2 border-b border-zinc-800">
        <div class="relative">
          <HiOutlineMagnifyingGlass class="absolute left-2 top-1/2 -translate-y-1/2 w-4 h-4 text-zinc-500" />
          <input
            type="text"
            value={filterText()}
            onInput={(e) => setFilterText(e.currentTarget.value)}
            placeholder="Filter documents..."
            class="w-full pl-8 pr-8 py-1.5 text-sm bg-zinc-800 border border-zinc-700 rounded 
                   text-zinc-200 placeholder-zinc-500 focus:outline-none focus:border-cyan-500/50"
          />
          <Show when={filterText()}>
            <button
              onClick={() => setFilterText("")}
              class="absolute right-2 top-1/2 -translate-y-1/2 p-0.5 text-zinc-500 hover:text-zinc-300"
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
            <HiOutlineArrowPath class="w-8 h-8 text-cyan-400 animate-spin" />
            <p class="mt-2 text-sm text-zinc-400">Searching for case documents...</p>
          </div>
        </Show>

        {/* Error State */}
        <Show when={error() && !loading()}>
          <div class="flex flex-col items-center justify-center h-full py-8 px-4">
            <HiOutlineExclamationTriangle class="w-8 h-8 text-red-400" />
            <p class="mt-2 text-sm text-red-400 text-center">{error()}</p>
            <button
              onClick={() => searchPath() && loadDocuments(searchPath()!)}
              class="mt-4 px-3 py-1.5 text-sm bg-zinc-800 text-zinc-200 rounded hover:bg-zinc-700"
            >
              Try Again
            </button>
          </div>
        </Show>

        {/* Empty State */}
        <Show when={!loading() && !error() && documents().length === 0}>
          <div class="flex flex-col items-center justify-center h-full py-8 px-4">
            <HiOutlineInbox class="w-12 h-12 text-zinc-600" />
            <p class="mt-2 text-sm text-zinc-400 text-center">
              {searchPath() 
                ? "No case documents found"
                : "Open an evidence file to discover case documents"}
            </p>
            <Show when={!searchPath() && props.evidencePath}>
              <button
                onClick={() => props.evidencePath && loadDocuments(props.evidencePath)}
                class="mt-4 px-3 py-1.5 text-sm bg-cyan-600 text-white rounded hover:bg-cyan-500"
              >
                Search for Documents
              </button>
            </Show>
          </div>
        </Show>

        {/* No Filter Results */}
        <Show when={!loading() && !error() && documents().length > 0 && groupedDocuments().length === 0}>
          <div class="flex flex-col items-center justify-center h-full py-8 px-4">
            <HiOutlineMagnifyingGlass class="w-8 h-8 text-zinc-600" />
            <p class="mt-2 text-sm text-zinc-400 text-center">
              No documents match "{filterText()}"
            </p>
          </div>
        </Show>

        {/* Document Groups */}
        <Show when={!loading() && !error() && groupedDocuments().length > 0}>
          <div class="divide-y divide-zinc-800">
            <For each={groupedDocuments()}>
              {(group) => {
                const Icon = documentTypeIcons[group.type];
                const isExpanded = () => expandedTypes().has(group.type);
                
                return (
                  <div>
                    {/* Group Header */}
                    <button
                      onClick={() => toggleType(group.type)}
                      class="w-full flex items-center gap-2 px-3 py-2 hover:bg-zinc-800/50 transition-colors"
                    >
                      {isExpanded() 
                        ? <HiOutlineChevronDown class="w-4 h-4 text-zinc-500" />
                        : <HiOutlineChevronRight class="w-4 h-4 text-zinc-500" />
                      }
                      <Icon class={`w-4 h-4 ${documentTypeColors[group.type].split(" ")[0]}`} />
                      <span class="text-sm font-medium text-zinc-200">
                        {getDocumentTypeLabel(group.type)}
                      </span>
                      <span class="ml-auto text-xs text-zinc-500">
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
                                      hover:bg-zinc-800/50 transition-colors
                                      ${selectedDoc()?.path === doc.path ? "bg-cyan-500/10" : ""}`}
                            >
                              <div class="flex-1 min-w-0">
                                <div class="flex items-center gap-2">
                                  <span class="text-sm text-zinc-200 truncate">
                                    {doc.filename}
                                  </span>
                                  <span class={`px-1.5 py-0.5 text-[10px] font-medium rounded border
                                                ${documentTypeColors[doc.document_type]}`}>
                                    {doc.format}
                                  </span>
                                </div>
                                
                                <div class="flex items-center gap-2 mt-0.5">
                                  <span class="text-xs text-zinc-500">
                                    {formatBytes(doc.size)}
                                  </span>
                                  <Show when={doc.modified}>
                                    <span class="text-xs text-zinc-600">•</span>
                                    <span class="text-xs text-zinc-500">
                                      {formatDate(doc.modified)}
                                    </span>
                                  </Show>
                                  <Show when={doc.case_number}>
                                    <span class="text-xs text-zinc-600">•</span>
                                    <span class="text-xs text-cyan-500">
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
                                class="p-1 opacity-0 group-hover:opacity-100 text-zinc-500 
                                       hover:text-zinc-200 hover:bg-zinc-700 rounded transition-opacity"
                                title="Open in default application"
                              >
                                <HiOutlineArrowTopRightOnSquare class="w-4 h-4" />
                              </button>
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
        <div class="px-3 py-2 border-t border-zinc-800 bg-zinc-900/50">
          <div class="flex items-center gap-2">
            <HiOutlineFolderOpen class="w-4 h-4 text-zinc-500 flex-shrink-0" />
            <span class="text-xs text-zinc-500 truncate" title={selectedDoc()!.path}>
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
