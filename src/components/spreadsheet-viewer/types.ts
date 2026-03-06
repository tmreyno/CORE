// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { SpreadsheetMetadataSection } from "../../types/viewerMetadata";

export interface SpreadsheetViewerProps {
  /** Path to the spreadsheet file */
  path: string;
  /** Optional class name */
  class?: string;
  /** Callback to emit metadata section for right panel */
  onMetadata?: (section: SpreadsheetMetadataSection) => void;
}

export interface SheetInfo {
  name: string;
  row_count: number;
  col_count: number;
}

export interface SpreadsheetInfo {
  path: string;
  format: string;
  sheets: SheetInfo[];
  total_sheets: number;
}

export interface CellValue {
  type: "Empty" | "String" | "Int" | "Float" | "Bool" | "DateTime" | "Error";
  value?: string | number | boolean;
}
