// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * DocumentGroupHeader Component
 * 
 * Collapsible header for document type groups showing:
 * - Expand/collapse chevron
 * - Document type icon
 * - Type label
 * - Document count
 */

import { Component } from "solid-js";
import { HiOutlineChevronDown, HiOutlineChevronRight } from "../icons";
import type { CaseDocumentType } from "../../types";
import { documentTypeIcons, documentTypeColors } from "./DocumentTypeConfig";
import { getDocumentTypeLabel } from "../../discovery";

interface DocumentGroupHeaderProps {
  type: CaseDocumentType;
  count: number;
  isExpanded: boolean;
  onToggle: () => void;
}

export const DocumentGroupHeader: Component<DocumentGroupHeaderProps> = (props) => {
  const Icon = documentTypeIcons[props.type];
  
  return (
    <button
      onClick={props.onToggle}
      class="w-full flex items-center gap-2 px-3 py-2 hover:bg-bg-panel/50 transition-colors"
    >
      {props.isExpanded 
        ? <HiOutlineChevronDown class="w-4 h-4 text-txt-muted" />
        : <HiOutlineChevronRight class="w-4 h-4 text-txt-muted" />
      }
      <Icon class={`w-4 h-4 ${documentTypeColors[props.type].split(" ")[0]}`} />
      <span class="text-sm font-medium text-txt">
        {getDocumentTypeLabel(props.type)}
      </span>
      <span class="ml-auto text-xs text-txt-muted">
        {props.count}
      </span>
    </button>
  );
};
