// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { CellValue } from "./types";

/** Format cell value for display */
export function formatCell(cell: CellValue): string {
  if (!cell || cell.type === "Empty") return "";
  if (cell.value === undefined || cell.value === null) return "";

  switch (cell.type) {
    case "String":
    case "DateTime":
    case "Error":
      return String(cell.value);
    case "Int":
      return String(cell.value);
    case "Float":
      return typeof cell.value === "number"
        ? cell.value.toLocaleString(undefined, { maximumFractionDigits: 6 })
        : String(cell.value);
    case "Bool":
      return cell.value ? "TRUE" : "FALSE";
    default:
      return String(cell.value ?? "");
  }
}

/** Get column letter (A, B, C, ... AA, AB, etc.) */
export function getColumnLetter(index: number): string {
  let result = "";
  let n = index;
  while (n >= 0) {
    result = String.fromCharCode(65 + (n % 26)) + result;
    n = Math.floor(n / 26) - 1;
  }
  return result;
}
