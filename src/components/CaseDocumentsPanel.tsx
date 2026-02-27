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
import { HiOutlineXMark, HiOutlineMagnifyingGlass, HiOutlineFolderOpen } from "./icons";
import { logger } from '../utils/logger';

const log = logger.scope('CaseDocuments');
import type { CaseDocument, CaseDocumentType } from "../types";
import {
  findCaseDocuments,
  findCocForms,
  discoverCaseDocuments,
} from "../discovery";
import { dbSync } from "../hooks/project/useProjectDbSync";
import { CaseDocumentsHeader } from "./casedocs/CaseDocumentsHeader";
import { CaseDocumentsEmptyStates } from "./casedocs/CaseDocumentsEmptyStates";
import { DocumentGroupHeader } from "./casedocs/DocumentGroupHeader";
import { DocumentItem } from "./casedocs/DocumentItem";

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
  /** Called when a document is selected — opens in the center pane viewer */
  onDocumentSelect?: (doc: CaseDocument) => void;
  /** Called when user wants to open document externally */
  onDocumentOpen?: (doc: CaseDocument) => void;
  /** Cached documents from project (to avoid re-discovery) */
  cachedDocuments?: CaseDocument[];
  /** Callback when documents are loaded (for caching) */
  onDocumentsLoaded?: (docs: CaseDocument[], searchPath: string) => void;
  /** Additional CSS classes */
  class?: string;
}

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
      log.debug(" Using cached documents:", props.cachedDocuments.length);
      setDocuments(props.cachedDocuments);
      if (path) {
        setSearchPath(path);
      }
      return;
    }
    
    // Load documents if we have a path
    if (path) {
      log.debug(" Initial load from", path);
      setSearchPath(path);
      loadDocuments(path);
    }
  });

  async function loadDocuments(path: string) {
    // Prevent concurrent loads
    if (isLoadingRef) {
      log.debug(" skipping load, already loading");
      return;
    }
    
    isLoadingRef = true;
    log.debug(" loadDocuments called with path:", path);
    log.debug(" cocOnly =", props.cocOnly, ", searchPath =", props.searchPath);
    setLoading(true);
    setError(null);
    
    try {
      let docs: CaseDocument[];
      
      if (props.cocOnly) {
        // Only search for COC forms
        log.debug(" searching for COC forms only in:", path);
        docs = await findCocForms(path, true);
      } else if (props.searchPath) {
        // Direct search in specified path
        log.debug(" direct search in specified path:", path);
        docs = await findCaseDocuments(path, true);
      } else {
        // Discover from evidence path (finds case document folders)
        log.debug(" discovering from evidence path:", path);
        docs = await discoverCaseDocuments(path);
      }
      
      log.debug(" found", docs.length, "documents");
      if (docs.length > 0) {
        log.debug(" documents found:", docs.map(d => ({ 
          filename: d.filename, 
          type: d.document_type, 
          path: d.path 
        })));
      } else {
        log.debug(" No documents found. Expected folder names:", 
          ["case.documents", "case documents", "documents", "forms", "paperwork", "intake", "custody", "coc", "case.notes"]);
      }
      setDocuments(docs);
      
      // Write-through discovered documents to .ffxdb for queryable storage
      for (const doc of docs) {
        dbSync.upsertCaseDocument(doc);
      }
      
      // Notify parent of loaded documents for caching
      props.onDocumentsLoaded?.(docs, path);
      
      // Don't auto-select - let user choose
    } catch (e) {
      log.error("Failed to load case documents:", e);
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

  return (
    <div class={`flex flex-col h-full bg-bg ${props.class || ""}`}>
      {/* Header */}
      <CaseDocumentsHeader
        documentCount={documentCount()}
        cocCount={cocCount()}
        searchPath={searchPath()}
        loading={loading()}
        onRefresh={() => loadDocuments(searchPath()!)}
      />

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
        <CaseDocumentsEmptyStates
          loading={loading()}
          error={error()}
          hasDocuments={documents().length > 0}
          hasFilteredResults={groupedDocuments().length > 0}
          searchPath={searchPath()}
          evidencePath={props.evidencePath}
          filterText={filterText()}
          onRetry={() => searchPath() && loadDocuments(searchPath()!)}
        />

        {/* Document Groups */}
        <Show when={!loading() && !error() && groupedDocuments().length > 0}>
          <div class="divide-y divide-border">
            <For each={groupedDocuments()}>
              {(group) => {
                const isExpanded = () => expandedTypes().has(group.type);
                
                return (
                  <div>
                    {/* Group Header */}
                    <DocumentGroupHeader
                      type={group.type}
                      count={group.docs.length}
                      isExpanded={isExpanded()}
                      onToggle={() => toggleType(group.type)}
                    />

                    {/* Documents List */}
                    <Show when={isExpanded()}>
                      <div class="pb-1">
                        <For each={group.docs}>
                          {(doc) => (
                            <DocumentItem
                              document={doc}
                              isSelected={selectedDoc()?.path === doc.path}
                              onClick={() => handleDocumentClick(doc)}
                            />
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
