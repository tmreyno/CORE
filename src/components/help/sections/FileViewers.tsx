// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import {
  HiOutlineCodeBracket,
  HiOutlineDocumentText,
  HiOutlineEye,
  HiOutlinePhoto,
  HiOutlineChartBar,
  HiOutlineEnvelope,
  HiOutlineCog6Tooth,
  HiOutlineLockClosed,
  HiOutlineCircleStack,
} from "../../icons";
import { Kbd } from "../../ui/Kbd";

export const ViewerCard: Component<{ icon: Component<{ class?: string }>; title: string; desc: string; shortcut?: string }> = (props) => (
  <div class="p-3 bg-bg-secondary rounded-lg border border-border/50 flex gap-3">
    <div class="p-2 bg-accent/10 rounded-lg text-accent flex-shrink-0 h-fit">
      <props.icon class="w-5 h-5" />
    </div>
    <div class="min-w-0">
      <div class="font-medium text-txt text-sm flex items-center gap-2">
        {props.title}
        <Show when={props.shortcut}>
          <Kbd keys={props.shortcut!} muted />
        </Show>
      </div>
      <div class="text-xs text-txt-muted mt-0.5">{props.desc}</div>
    </div>
  </div>
);

export const FileViewersContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Files inside evidence containers can be viewed directly without extraction.
      CORE-FFX auto-detects the best viewer based on file extension and content magic bytes.
    </p>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <ViewerCard
        icon={HiOutlineCodeBracket}
        title="Hex Viewer"
        desc="Raw hexadecimal byte display with ASCII sidebar, offset navigation, and header analysis"
        shortcut="Cmd+2"
      />
      <ViewerCard
        icon={HiOutlineDocumentText}
        title="Text Viewer"
        desc="Plain text and source code with syntax detection"
        shortcut="Cmd+3"
      />
      <ViewerCard
        icon={HiOutlineEye}
        title="PDF Viewer"
        desc="Embedded PDF rendering with page navigation"
      />
      <ViewerCard
        icon={HiOutlinePhoto}
        title="Image Viewer"
        desc="Image preview with EXIF metadata extraction (JPEG, PNG, TIFF, BMP, GIF, WebP)"
      />
      <ViewerCard
        icon={HiOutlineDocumentText}
        title="Office Documents"
        desc="DOCX, DOC, PPTX, PPT, ODT, ODP, RTF text extraction and preview"
      />
      <ViewerCard
        icon={HiOutlineChartBar}
        title="Spreadsheets"
        desc="Excel (XLSX/XLS), CSV, and ODS with tabular display"
      />
      <ViewerCard
        icon={HiOutlineEnvelope}
        title="Email Viewer"
        desc="EML and MBOX email parsing — headers, body, attachments"
      />
      <ViewerCard
        icon={HiOutlineEnvelope}
        title="PST Viewer"
        desc="Outlook PST file browsing — folder tree, message list, message preview"
      />
      <ViewerCard
        icon={HiOutlineCog6Tooth}
        title="Plist Viewer"
        desc="Apple property list (binary and XML) with tree display"
      />
      <ViewerCard
        icon={HiOutlineCodeBracket}
        title="Binary Viewer"
        desc="PE, ELF, and Mach-O executable analysis — headers, sections, imports, exports"
      />
      <ViewerCard
        icon={HiOutlineLockClosed}
        title="Registry Viewer"
        desc="Windows Registry hive browsing — keys, values, and types"
      />
      <ViewerCard
        icon={HiOutlineCircleStack}
        title="Database Viewer"
        desc="SQLite database browsing — tables, schemas, and data preview"
      />
    </div>
  </div>
);
