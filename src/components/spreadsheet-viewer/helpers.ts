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

/** Get raw numeric value for sorting purposes */
export function cellSortValue(cell: CellValue): string | number {
  if (!cell || cell.type === "Empty") return "";
  if (cell.value === undefined || cell.value === null) return "";
  switch (cell.type) {
    case "Int":
    case "Float":
      return typeof cell.value === "number" ? cell.value : parseFloat(String(cell.value)) || 0;
    case "Bool":
      return cell.value ? 1 : 0;
    default:
      return String(cell.value).toLowerCase();
  }
}

/** Sort rows by a column index. Returns a new array. */
export function sortRows(
  rows: CellValue[][],
  colIndex: number,
  ascending: boolean,
): CellValue[][] {
  return [...rows].sort((a, b) => {
    const va = cellSortValue(a[colIndex]);
    const vb = cellSortValue(b[colIndex]);
    if (typeof va === "number" && typeof vb === "number") {
      return ascending ? va - vb : vb - va;
    }
    const sa = String(va);
    const sb = String(vb);
    return ascending ? sa.localeCompare(sb) : sb.localeCompare(sa);
  });
}

/** Filter rows by a search query string across all cells */
export function filterRows(rows: CellValue[][], query: string): CellValue[][] {
  if (!query.trim()) return rows;
  const q = query.toLowerCase();
  return rows.filter((row) =>
    row.some((cell) => formatCell(cell).toLowerCase().includes(q)),
  );
}

/** Convert rows to CSV string */
export function rowsToCsv(rows: CellValue[][], headers?: string[]): string {
  const lines: string[] = [];
  if (headers) {
    lines.push(headers.map((h) => csvEscape(h)).join(","));
  }
  for (const row of rows) {
    lines.push(row.map((cell) => csvEscape(formatCell(cell))).join(","));
  }
  return lines.join("\n");
}

function csvEscape(value: string): string {
  if (value.includes(",") || value.includes('"') || value.includes("\n")) {
    return `"${value.replace(/"/g, '""')}"`;
  }
  return value;
}

/** Build printable HTML table from rows */
export function rowsToHtmlTable(
  rows: CellValue[][],
  sheetName: string,
  headers?: string[],
): string {
  const hdr = headers
    ? `<tr>${headers.map((h) => `<th style="border:1px solid #ccc;padding:4px 8px;background:#f5f5f5;text-align:left;font-size:12px;">${escHtml(h)}</th>`).join("")}</tr>`
    : "";
  const body = rows
    .map(
      (row) =>
        `<tr>${row.map((cell) => `<td style="border:1px solid #eee;padding:3px 8px;font-size:12px;">${escHtml(formatCell(cell))}</td>`).join("")}</tr>`,
    )
    .join("");
  return `<!DOCTYPE html><html><head><title>${escHtml(sheetName)}</title><style>body{font-family:system-ui,sans-serif;margin:20px}table{border-collapse:collapse;width:100%}h2{font-size:16px;margin-bottom:8px}@media print{body{margin:0}}</style></head><body><h2>${escHtml(sheetName)}</h2><table>${hdr}${body}</table></body></html>`;
}

function escHtml(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

/** Copy selected rows to clipboard as tab-separated text */
export function rowsToTsv(rows: CellValue[][]): string {
  return rows.map((row) => row.map((cell) => formatCell(cell)).join("\t")).join("\n");
}
