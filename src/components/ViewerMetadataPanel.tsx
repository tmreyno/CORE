// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ViewerMetadataPanel - Right panel metadata display for specialized viewers
 *
 * Shows tabbed metadata when a viewer (image, registry, database, binary,
 * email, plist, document, spreadsheet, office, archive) is active.
 *
 * Displays:
 * - File Info tab (always present)
 * - Viewer-specific metadata tabs based on the active viewer type
 *
 * Replaces the TreePanel default view in RightPanel when viewer metadata
 * is available.
 *
 * Sub-components live in ./viewerMetadata/:
 *   shared.tsx     – CollapsibleGroup, MetadataRow primitives
 *   FileInfoTab.tsx – always-visible file info tab
 *   sections.tsx   – per-viewer-type section renderers + router
 */

import { Show, For, createSignal, createMemo } from "solid-js";
import type { ViewerMetadata } from "../types/viewerMetadata";
import { FileInfoTab } from "./viewerMetadata/FileInfoTab";
import { MetadataSectionRenderer } from "./viewerMetadata/MetadataSectionRenderer";

// =============================================================================
// Props
// =============================================================================

export interface ViewerMetadataPanelProps {
  /** Viewer metadata to display */
  metadata: ViewerMetadata;
}

// =============================================================================
// Tab definitions
// =============================================================================

type MetadataTabId = "file" | "viewer";

// =============================================================================
// Component
// =============================================================================

export function ViewerMetadataPanel(props: ViewerMetadataPanelProps) {
  const [activeTab, setActiveTab] = createSignal<MetadataTabId>("viewer");

  /** Get a human-readable label for the viewer type */
  const viewerLabel = createMemo(() => {
    const sections = props.metadata.sections;
    if (sections.length === 0) return "Details";
    const first = sections[0];
    switch (first.kind) {
      case "exif": return "EXIF";
      case "registry": return "Registry";
      case "database": return "Database";
      case "binary": return "Binary";
      case "email": return "Email";
      case "plist": return "Plist";
      case "document": return "Document";
      case "spreadsheet": return "Spreadsheet";
      case "office": return "Office";
      case "archive": return "Archive";
      default: return "Details";
    }
  });

  /** Switch to viewer tab if sections are available, otherwise file tab */
  const effectiveTab = createMemo(() => {
    if (activeTab() === "viewer" && props.metadata.sections.length > 0) return "viewer";
    return "file";
  });

  return (
    <div class="flex flex-col h-full bg-bg">
      {/* Tab header */}
      <div class="flex items-center border-b border-border bg-bg-secondary">
        <button
          class={`px-3 py-2 text-xs font-medium transition-colors ${
            effectiveTab() === "file"
              ? "text-accent border-b-2 border-accent"
              : "text-txt-muted hover:text-txt"
          }`}
          onClick={() => setActiveTab("file")}
        >
          File Info
        </button>
        <Show when={props.metadata.sections.length > 0}>
          <button
            class={`px-3 py-2 text-xs font-medium transition-colors ${
              effectiveTab() === "viewer"
                ? "text-accent border-b-2 border-accent"
                : "text-txt-muted hover:text-txt"
            }`}
            onClick={() => setActiveTab("viewer")}
          >
            {viewerLabel()}
          </button>
        </Show>
      </div>

      {/* Tab content */}
      <div class="flex-1 overflow-y-auto">
        <Show when={effectiveTab() === "file"}>
          <FileInfoTab metadata={props.metadata} />
        </Show>
        <Show when={effectiveTab() === "viewer"}>
          <For each={props.metadata.sections}>
            {(section) => <MetadataSectionRenderer section={section} />}
          </For>
        </Show>
      </div>
    </div>
  );
}
