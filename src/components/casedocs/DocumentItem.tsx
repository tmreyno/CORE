// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * DocumentItem Component
 * 
 * Compact single-line document row displaying:
 * - A format-specific colored icon (PDF=red, DOCX=blue, XLS=green, etc.)
 * - The filename (truncated)
 * - A small muted format label to the right (PDF, DOCX, TXT, etc.)
 * 
 * All metadata (size, modified date, case number, evidence ID, document type)
 * is intentionally shown in the RIGHT PANEL (ViewerMetadataPanel → FileInfoTab)
 * when the document is selected — NOT in the tree.
 * 
 * Viewer options (hex, text, etc.) are also NOT shown here — they are
 * available in the center pane's view mode switcher.
 */

import { Component, createMemo } from "solid-js";
import type { CaseDocument } from "../../types";
import { getDocumentFormatIcon } from "./DocumentTypeConfig";

interface DocumentItemProps {
  document: CaseDocument;
  isSelected: boolean;
  onClick: () => void;
}

export const DocumentItem: Component<DocumentItemProps> = (props) => {
  const formatConfig = createMemo(() => getDocumentFormatIcon(props.document.format));
  const FormatIcon = () => {
    const config = formatConfig();
    const Icon = config.icon;
    return <Icon class={`w-3.5 h-3.5 ${config.color} shrink-0`} />;
  };

  return (
    <div
      onClick={props.onClick}
      class={`flex items-center gap-1.5 px-3 py-1 pl-9 cursor-pointer
              hover:bg-bg-panel/50 transition-colors
              ${props.isSelected ? "bg-accent/10" : ""}`}
    >
      <FormatIcon />
      <span class="text-sm text-txt truncate flex-1 min-w-0">
        {props.document.filename}
      </span>
      <span class="text-[9px] text-txt-muted font-medium shrink-0 uppercase opacity-50">
        {props.document.format}
      </span>
    </div>
  );
};
