// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * DocumentItem Component
 * 
 * Individual document row displaying:
 * - Document name with format badge
 * - Size, modified date, case number
 * - Action buttons (open external, hex view, text view)
 */

import { Component, Show } from "solid-js";
import { HiOutlineArrowTopRightOnSquare } from "../icons";
import type { CaseDocument } from "../../types";
import { formatBytes, formatDateByPreference } from "../../utils";
import { documentTypeColors } from "./DocumentTypeConfig";

interface DocumentItemProps {
  document: CaseDocument;
  isSelected: boolean;
  onClick: () => void;
  onOpenExternal: () => void;
  onViewHex?: () => void;
  onViewText?: () => void;
}

export const DocumentItem: Component<DocumentItemProps> = (props) => {
  const formatDate = (dateStr?: string | null): string => {
    return formatDateByPreference(dateStr, false);
  };

  return (
    <div
      onClick={props.onClick}
      class={`group flex items-start gap-2 px-3 py-2 pl-9 cursor-pointer
              hover:bg-bg-panel/50 transition-colors
              ${props.isSelected ? "bg-accent/10" : ""}`}
    >
      <div class="flex-1 min-w-0">
        <div class="flex items-center gap-2">
          <span class="text-sm text-txt truncate">
            {props.document.filename}
          </span>
          <span class={`px-1.5 py-0.5 text-[10px] font-medium rounded border
                        ${documentTypeColors[props.document.document_type]}`}>
            {props.document.format}
          </span>
        </div>
        
        <div class="flex items-center gap-2 mt-0.5">
          <span class="text-xs text-txt-muted">
            {formatBytes(props.document.size)}
          </span>
          <Show when={props.document.modified}>
            <span class="text-xs text-txt-muted">•</span>
            <span class="text-xs text-txt-muted">
              {formatDate(props.document.modified)}
            </span>
          </Show>
          <Show when={props.document.case_number}>
            <span class="text-xs text-txt-muted">•</span>
            <span class="text-xs text-accent">
              Case: {props.document.case_number}
            </span>
          </Show>
        </div>
      </div>

      {/* Open External Button */}
      <button
        onClick={(e) => {
          e.stopPropagation();
          props.onOpenExternal();
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
            props.onViewHex?.();
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
            props.onViewText?.();
          }}
          class="p-1 opacity-0 group-hover:opacity-100 text-txt-muted 
                 hover:text-green-400 hover:bg-bg-hover rounded transition-opacity"
          title="View as Text"
        >
          <span class="text-[10px] font-mono font-bold">TXT</span>
        </button>
      </Show>
    </div>
  );
};
