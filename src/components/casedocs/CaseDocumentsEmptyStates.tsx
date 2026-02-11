// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * CaseDocumentsEmptyStates Component
 * 
 * All empty/loading/error states for the case documents panel:
 * - Loading state (spinner)
 * - Error state (with retry)
 * - Empty state (no documents found)
 * - No filter results state
 */

import { Component, Show } from "solid-js";
import {
  HiOutlineArrowPath,
  HiOutlineExclamationTriangle,
  HiOutlineInbox,
  HiOutlineMagnifyingGlass,
} from "../icons";

interface CaseDocumentsEmptyStatesProps {
  loading: boolean;
  error: string | null;
  hasDocuments: boolean;
  hasFilteredResults: boolean;
  searchPath: string | null;
  evidencePath?: string;
  filterText: string;
  onRetry: () => void;
}

export const CaseDocumentsEmptyStates: Component<CaseDocumentsEmptyStatesProps> = (props) => {
  return (
    <>
      {/* Loading State */}
      <Show when={props.loading}>
        <div class="flex flex-col items-center justify-center h-full py-8">
          <HiOutlineArrowPath class="w-8 h-8 text-accent animate-spin" />
          <p class="mt-2 text-sm text-txt-secondary">Searching for case documents...</p>
        </div>
      </Show>

      {/* Error State */}
      <Show when={props.error && !props.loading}>
        <div class="flex flex-col items-center justify-center h-full py-8 px-4">
          <HiOutlineExclamationTriangle class="w-8 h-8 text-red-400" />
          <p class="mt-2 text-sm text-red-400 text-center">{props.error}</p>
          <button
            onClick={props.onRetry}
            class="btn-sm mt-4"
          >
            Try Again
          </button>
        </div>
      </Show>

      {/* Empty State */}
      <Show when={!props.loading && !props.error && !props.hasDocuments}>
        <div class="flex flex-col items-center justify-center h-full py-8 px-4">
          <HiOutlineInbox class="w-12 h-12 text-txt-muted" />
          <p class="mt-2 text-sm text-txt-secondary text-center">
            {props.searchPath 
              ? "No case documents found in this location"
              : props.evidencePath
                ? "No case documents found near evidence"
                : "Open an evidence file to discover case documents"}
          </p>
          <Show when={props.searchPath || props.evidencePath}>
            <p class="mt-3 text-xs text-txt-muted text-center max-w-xs">
              Expected folder names: <span class="text-txt-secondary">Case Documents</span>, <span class="text-txt-secondary">Documents</span>, <span class="text-txt-secondary">Forms</span>, <span class="text-txt-secondary">Paperwork</span>, <span class="text-txt-secondary">COC</span>, <span class="text-txt-secondary">Intake</span>
            </p>
            <p class="mt-1 text-xs text-txt-muted text-center max-w-xs">
              Supported files: PDF, DOCX, DOC, XLSX, XLS, TXT, RTF
            </p>
          </Show>
          <Show when={!props.searchPath && props.evidencePath}>
            <button
              onClick={props.onRetry}
              class="btn-sm-primary mt-4"
            >
              Retry Search
            </button>
          </Show>
        </div>
      </Show>

      {/* No Filter Results */}
      <Show when={!props.loading && !props.error && props.hasDocuments && !props.hasFilteredResults}>
        <div class="flex flex-col items-center justify-center h-full py-8 px-4">
          <HiOutlineMagnifyingGlass class="w-8 h-8 text-txt-muted" />
          <p class="mt-2 text-sm text-txt-secondary text-center">
            No documents match "{props.filterText}"
          </p>
        </div>
      </Show>
    </>
  );
};
