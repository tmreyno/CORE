// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { OfficeMetadataSection } from "../../types/viewerMetadata";

// Types matching Rust OfficeDocumentInfo with camelCase serde

export interface OfficeMetadata {
  title: string | null;
  creator: string | null;
  lastModifiedBy: string | null;
  subject: string | null;
  description: string | null;
  created: string | null;
  modified: string | null;
  application: string | null;
  pageCount: number | null;
  wordCount: number | null;
  charCount: number | null;
}

export type OfficeFormat =
  | "docx" | "doc" | "pptx" | "ppt"
  | "odt" | "odp" | "rtf" | "unknown";

export type ParagraphHint =
  | "normal" | "heading1" | "heading2" | "heading3" | "heading4"
  | "title" | "subtitle" | "listItem" | "quote";

export interface OfficeParagraph {
  text: string;
  hint: ParagraphHint;
}

export interface OfficeTextSection {
  label: string | null;
  paragraphs: OfficeParagraph[];
}

export interface OfficeDocumentInfo {
  path: string;
  format: OfficeFormat;
  formatDescription: string;
  metadata: OfficeMetadata;
  sections: OfficeTextSection[];
  totalChars: number;
  totalWords: number;
  extractionComplete: boolean;
  warnings: string[];
}

export interface OfficeViewerProps {
  /** Path to the office document file */
  path: string;
  /** Optional class name */
  class?: string;
  /** Callback to emit metadata section for right panel */
  onMetadata?: (section: OfficeMetadataSection) => void;
}
