// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * SpreadsheetViewer - Native viewer for Excel/CSV/ODS files
 * 
 * Renders spreadsheet data directly as SolidJS components for better
 * performance and styling compared to HTML injection.
 */

import { createSignal, createEffect, createMemo, Show, For } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineTableCells,
  HiOutlineExclamationTriangle,
  HiOutlineChevronLeft,
  HiOutlineChevronRight,
} from "./icons";
import { logger } from "../utils/logger";
const log = logger.scope("SpreadsheetViewer");

// ============================================================================
// ============================================================================

interface SpreadsheetViewerProps {
  /** Path to the spreadsheet file */
  path: string;
  /** Optional class name */
  class?: string;
}

interface SheetInfo {
  name: string;
  row_count: number;
  col_count: number;
}

interface SpreadsheetInfo {
  path: string;
  format: string;
  sheets: SheetInfo[];
  total_sheets: number;
}

interface CellValue {
  type: "Empty" | "String" | "Int" | "Float" | "Bool" | "DateTime" | "Error";
  value?: string | number | boolean;
}

// ============================================================================
// Component
// ============================================================================

export function SpreadsheetViewer(props: SpreadsheetViewerProps) {
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [info, setInfo] = createSignal<SpreadsheetInfo | null>(null);
  const [activeSheet, setActiveSheet] = createSignal(0);
  const [rows, setRows] = createSignal<CellValue[][]>([]);
  const [loadingSheet, setLoadingSheet] = createSignal(false);

  // Memoized computed values
  const sheets = createMemo(() => info()?.sheets ?? []);
  const sheetCount = createMemo(() => sheets().length);
  const hasMultipleSheets = createMemo(() => sheetCount() > 1);
  const formatLabel = createMemo(() => info()?.format?.toUpperCase() || "Spreadsheet");
  const rowCount = createMemo(() => rows().length);
  const hasRows = createMemo(() => rowCount() > 0);

  // Load spreadsheet info
  const loadInfo = async () => {
    setLoading(true);
    setError(null);

    try {
      const result = await invoke<SpreadsheetInfo>("spreadsheet_info", {
        path: props.path,
      });
      setInfo(result);
      
      // Load first sheet
      if (result.sheets.length > 0) {
        await loadSheet(0);
      }
    } catch (e) {
      log.error("Failed to load spreadsheet:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  // Load a specific sheet's data
  const loadSheet = async (sheetIndex: number) => {
    const sheetInfo = info();
    if (!sheetInfo || sheetIndex >= sheetInfo.sheets.length) return;

    setLoadingSheet(true);
    try {
      const sheetName = sheetInfo.sheets[sheetIndex].name;
      const data = await invoke<CellValue[][]>("spreadsheet_read_sheet", {
        path: props.path,
        sheetName,
        startRow: 0,
        maxRows: 500, // Limit for performance
      });
      setRows(data);
      setActiveSheet(sheetIndex);
    } catch (e) {
      log.error("Failed to load sheet:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoadingSheet(false);
    }
  };

  // Effect to load when path changes
  createEffect(() => {
    if (props.path) {
      loadInfo();
    }
  });

  // Format cell value for display
  const formatCell = (cell: CellValue): string => {
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
  };

  // Get column letter (A, B, C, ... AA, AB, etc.)
  const getColumnLetter = (index: number): string => {
    let result = "";
    let n = index;
    while (n >= 0) {
      result = String.fromCharCode(65 + (n % 26)) + result;
      n = Math.floor(n / 26) - 1;
    }
    return result;
  };

  return (
    <div class={`spreadsheet-viewer flex flex-col h-full bg-bg ${props.class || ""}`}>
      {/* Toolbar */}
      <div class="flex items-center gap-2 px-2 py-1.5 border-b border-border bg-bg-secondary">
        {/* Format indicator */}
        <div class="flex items-center gap-1.5 text-sm text-txt-secondary">
          <HiOutlineTableCells class="w-4 h-4" />
          <span class="font-medium">{formatLabel()}</span>
        </div>

        <div class="flex-1" />

        {/* Sheet tabs */}
        <Show when={hasMultipleSheets()}>
          <div class="flex items-center gap-1">
            <button
              onClick={() => loadSheet(Math.max(0, activeSheet() - 1))}
              disabled={activeSheet() === 0 || loadingSheet()}
              class="p-1 rounded hover:bg-bg-hover disabled:opacity-30"
              title="Previous sheet"
            >
              <HiOutlineChevronLeft class="w-4 h-4" />
            </button>
            
            <select
              value={activeSheet()}
              onChange={(e) => loadSheet(parseInt(e.currentTarget.value))}
              disabled={loadingSheet()}
              class="input-xs"
            >
              <For each={sheets()}>
                {(sheet, i) => (
                  <option value={i()}>{sheet.name}</option>
                )}
              </For>
            </select>
            
            <button
              onClick={() => loadSheet(Math.min(sheetCount() - 1, activeSheet() + 1))}
              disabled={activeSheet() >= sheetCount() - 1 || loadingSheet()}
              class="p-1 rounded hover:bg-bg-hover disabled:opacity-30"
              title="Next sheet"
            >
              <HiOutlineChevronRight class="w-4 h-4" />
            </button>
          </div>
        </Show>

        {/* Row count */}
        <Show when={hasRows()}>
          <span class="text-xs text-txt-muted">
            {rowCount()} rows
          </span>
        </Show>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-auto">
        <Show
          when={!loading()}
          fallback={
            <div class="flex items-center justify-center h-full">
              <div class="flex flex-col items-center gap-2">
                <div class="animate-spin w-6 h-6 border-2 border-accent border-t-transparent rounded-full" />
                <span class="text-sm text-txt-muted">Loading spreadsheet...</span>
              </div>
            </div>
          }
        >
          <Show
            when={!error()}
            fallback={
              <div class="flex items-center justify-center h-full">
                <div class="flex flex-col items-center gap-2 text-error">
                  <HiOutlineExclamationTriangle class="w-10 h-10" />
                  <span class="font-medium">Failed to load spreadsheet</span>
                  <span class="text-sm text-txt-muted">{error()}</span>
                  <button
                    onClick={loadInfo}
                    class="btn-sm-primary mt-2"
                  >
                    Retry
                  </button>
                </div>
              </div>
            }
          >
            <Show when={loadingSheet()}>
              <div class="absolute inset-0 bg-bg/50 flex items-center justify-center z-10">
                <div class="animate-spin w-6 h-6 border-2 border-accent border-t-transparent rounded-full" />
              </div>
            </Show>

            {/* Table */}
            <table class="w-full border-collapse text-sm">
              <thead class="sticky top-0 z-10">
                <tr class="bg-bg-secondary">
                  {/* Row number header */}
                  <th class="px-2 py-1.5 text-xs text-txt-muted font-medium border-b border-r border-border bg-bg-secondary sticky left-0 z-20 w-12">
                    #
                  </th>
                  {/* Column headers */}
                  <For each={rows()[0] || []}>
                    {(_, colIndex) => (
                      <th class="px-3 py-1.5 text-xs text-txt-muted font-medium border-b border-r border-border bg-bg-secondary text-left whitespace-nowrap">
                        {getColumnLetter(colIndex())}
                      </th>
                    )}
                  </For>
                </tr>
              </thead>
              <tbody>
                <For each={rows()}>
                  {(row, rowIndex) => (
                    <tr class="hover:bg-bg-hover/50 group">
                      {/* Row number */}
                      <td class="px-2 py-1 text-xs text-txt-muted border-b border-r border-border bg-bg-secondary sticky left-0 text-right font-mono">
                        {rowIndex() + 1}
                      </td>
                      {/* Cells */}
                      <For each={row}>
                        {(cell) => (
                          <td 
                            class="px-3 py-1 border-b border-r border-border whitespace-nowrap max-w-xs truncate"
                            classList={{
                              "text-right font-mono": cell.type === "Int" || cell.type === "Float",
                              "text-accent": cell.type === "Bool",
                              "text-error": cell.type === "Error",
                              "text-txt-muted italic": cell.type === "Empty",
                            }}
                            title={formatCell(cell)}
                          >
                            {formatCell(cell)}
                          </td>
                        )}
                      </For>
                    </tr>
                  )}
                </For>
              </tbody>
            </table>

            {/* Empty state */}
            <Show when={rows().length === 0 && !loadingSheet()}>
              <div class="flex items-center justify-center h-full text-txt-muted">
                No data in this sheet
              </div>
            </Show>
          </Show>
        </Show>
      </div>
    </div>
  );
}
