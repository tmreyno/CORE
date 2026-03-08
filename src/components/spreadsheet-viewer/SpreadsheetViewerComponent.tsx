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
import { save } from "@tauri-apps/plugin-dialog";
import {
  HiOutlineTableCells,
  HiOutlineExclamationTriangle,
  HiOutlineChevronLeft,
  HiOutlineChevronRight,
  HiOutlineMagnifyingGlass,
  HiOutlineArrowDownTray,
  HiOutlinePrinter,
  HiOutlineArrowUp,
  HiOutlineArrowDown,
} from "../icons";
import { logger } from "../../utils/logger";
import {
  formatCell,
  getColumnLetter,
  sortRows,
  filterRows,
  rowsToCsv,
  rowsToHtmlTable,
} from "./helpers";
import { printDocument } from "../document/documentHelpers";
import type { SpreadsheetMetadataSection } from "../../types/viewerMetadata";
import type {
  SpreadsheetViewerProps,
  SpreadsheetInfo,
  CellValue,
} from "./types";

const log = logger.scope("SpreadsheetViewer");

export function SpreadsheetViewerComponent(props: SpreadsheetViewerProps) {
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [info, setInfo] = createSignal<SpreadsheetInfo | null>(null);
  const [activeSheet, setActiveSheet] = createSignal(0);
  const [rows, setRows] = createSignal<CellValue[][]>([]);
  const [loadingSheet, setLoadingSheet] = createSignal(false);

  // Search & sort state
  const [searchQuery, setSearchQuery] = createSignal("");
  const [showSearch, setShowSearch] = createSignal(false);
  const [sortCol, setSortCol] = createSignal<number | null>(null);
  const [sortAsc, setSortAsc] = createSignal(true);
  const [copiedCell, setCopiedCell] = createSignal<string | null>(null);

  // Memoized computed values
  const sheets = createMemo(() => info()?.sheets ?? []);
  const sheetCount = createMemo(() => sheets().length);
  const hasMultipleSheets = createMemo(() => sheetCount() > 1);
  const formatLabel = createMemo(
    () => info()?.format?.toUpperCase() || "Spreadsheet",
  );

  // Processed rows (filtered, then sorted)
  const processedRows = createMemo(() => {
    let result = rows();
    const q = searchQuery();
    if (q.trim()) {
      result = filterRows(result, q);
    }
    const col = sortCol();
    if (col !== null) {
      result = sortRows(result, col, sortAsc());
    }
    return result;
  });

  const rowCount = createMemo(() => processedRows().length);
  const totalRowCount = createMemo(() => rows().length);
  const hasRows = createMemo(() => rowCount() > 0);
  const isFiltered = createMemo(
    () => searchQuery().trim().length > 0 || sortCol() !== null,
  );

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
    // Reset search/sort on sheet change
    setSearchQuery("");
    setSortCol(null);
    setSortAsc(true);
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

  // Column header click toggles sort
  const handleColumnSort = (colIndex: number) => {
    if (sortCol() === colIndex) {
      if (sortAsc()) {
        setSortAsc(false);
      } else {
        // Third click clears sort
        setSortCol(null);
        setSortAsc(true);
      }
    } else {
      setSortCol(colIndex);
      setSortAsc(true);
    }
  };

  // Copy cell value to clipboard
  const handleCellClick = async (cell: CellValue) => {
    const text = formatCell(cell);
    if (!text) return;
    try {
      await navigator.clipboard.writeText(text);
      setCopiedCell(text);
      setTimeout(() => setCopiedCell(null), 1500);
    } catch {
      // clipboard may not be available
    }
  };

  // Build column headers from first row
  const columnHeaders = createMemo(() => {
    const first = rows()[0];
    if (!first) return [];
    return first.map((_, i) => getColumnLetter(i));
  });

  // Export current sheet data to CSV
  const handleExportCsv = async () => {
    const data = processedRows();
    if (data.length === 0) return;

    const sheetName =
      sheets()[activeSheet()]?.name || "Sheet1";
    const headers = columnHeaders();

    try {
      const path = await save({
        title: "Export Spreadsheet as CSV",
        defaultPath: `${sheetName}.csv`,
        filters: [{ name: "CSV", extensions: ["csv"] }],
      });
      if (!path) return;

      const csv = rowsToCsv(data, headers);
      await invoke("write_text_file", { path, content: csv });
    } catch (e) {
      log.error("CSV export failed:", e);
    }
  };

  // Print current sheet data
  const handlePrint = () => {
    const data = processedRows();
    if (data.length === 0) return;

    const sheetName =
      sheets()[activeSheet()]?.name || "Sheet1";
    const headers = columnHeaders();
    const html = rowsToHtmlTable(data, sheetName, headers);
    printDocument(html);
  };

  // Effect to load when path changes
  createEffect(() => {
    if (props.path) {
      loadInfo();
    }
  });

  // Emit metadata section when spreadsheet info or active sheet changes
  createEffect(() => {
    const sheetInfo = info();
    if (!sheetInfo || !props.onMetadata) return;
    const section: SpreadsheetMetadataSection = {
      kind: "spreadsheet",
      format: sheetInfo.format,
      sheetCount: sheetInfo.total_sheets,
      sheets: sheetInfo.sheets.map((s) => ({
        name: s.name,
        rowCount: s.row_count,
        columnCount: s.col_count,
      })),
      selectedSheet: sheetInfo.sheets[activeSheet()]?.name,
    };
    props.onMetadata(section);
  });

  return (
    <div
      class={`spreadsheet-viewer flex flex-col h-full bg-bg ${props.class || ""}`}
    >
      {/* Toolbar */}
      <div class="flex items-center gap-2 px-2 py-1.5 border-b border-border bg-bg-secondary">
        {/* Format indicator */}
        <div class="flex items-center gap-1.5 text-sm text-txt-secondary">
          <HiOutlineTableCells class="w-4 h-4" />
          <span class="font-medium">{formatLabel()}</span>
        </div>

        {/* Search toggle */}
        <button
          onClick={() => {
            setShowSearch((v) => !v);
            if (showSearch()) {
              setSearchQuery("");
            }
          }}
          class="icon-btn-sm"
          classList={{ "text-accent": showSearch() }}
          title="Search data"
        >
          <HiOutlineMagnifyingGlass class="w-4 h-4" />
        </button>

        {/* Search input (inline expand) */}
        <Show when={showSearch()}>
          <input
            type="text"
            placeholder="Search rows…"
            value={searchQuery()}
            onInput={(e) => setSearchQuery(e.currentTarget.value)}
            class="input-xs w-36"
            autofocus
          />
        </Show>

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
                {(sheet, i) => <option value={i()}>{sheet.name}</option>}
              </For>
            </select>

            <button
              onClick={() =>
                loadSheet(Math.min(sheetCount() - 1, activeSheet() + 1))
              }
              disabled={activeSheet() >= sheetCount() - 1 || loadingSheet()}
              class="p-1 rounded hover:bg-bg-hover disabled:opacity-30"
              title="Next sheet"
            >
              <HiOutlineChevronRight class="w-4 h-4" />
            </button>
          </div>
        </Show>

        {/* Export CSV */}
        <button
          onClick={handleExportCsv}
          disabled={!hasRows()}
          class="icon-btn-sm"
          title="Export as CSV"
        >
          <HiOutlineArrowDownTray class="w-4 h-4" />
        </button>

        {/* Print */}
        <button
          onClick={handlePrint}
          disabled={!hasRows()}
          class="icon-btn-sm"
          title="Print sheet"
        >
          <HiOutlinePrinter class="w-4 h-4" />
        </button>

        {/* Row count */}
        <Show when={hasRows()}>
          <span class="text-xs text-txt-muted">
            {isFiltered()
              ? `${rowCount()} / ${totalRowCount()} rows`
              : `${rowCount()} rows`}
          </span>
        </Show>

        {/* Copied indicator */}
        <Show when={copiedCell() !== null}>
          <span class="text-xs text-success animate-fade-in">Copied</span>
        </Show>
      </div>

      {/* Content */}
      <div class="flex-1 overflow-auto relative">
        <Show
          when={!loading()}
          fallback={
            <div class="flex items-center justify-center h-full">
              <div class="flex flex-col items-center gap-2">
                <div class="animate-spin w-6 h-6 border-2 border-accent border-t-transparent rounded-full" />
                <span class="text-sm text-txt-muted">
                  Loading spreadsheet...
                </span>
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
                  <button onClick={loadInfo} class="btn-sm-primary mt-2">
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
                  {/* Column headers - sortable */}
                  <For each={rows()[0] || []}>
                    {(_, colIndex) => (
                      <th
                        class="px-3 py-1.5 text-xs text-txt-muted font-medium border-b border-r border-border bg-bg-secondary text-left whitespace-nowrap cursor-pointer hover:bg-bg-hover select-none"
                        onClick={() => handleColumnSort(colIndex())}
                        title={`Sort by column ${getColumnLetter(colIndex())}${sortCol() === colIndex() ? (sortAsc() ? " (ascending)" : " (descending)") : ""}`}
                      >
                        <span class="inline-flex items-center gap-1">
                          {getColumnLetter(colIndex())}
                          <Show when={sortCol() === colIndex()}>
                            {sortAsc() ? (
                              <HiOutlineArrowUp class="w-3 h-3 text-accent" />
                            ) : (
                              <HiOutlineArrowDown class="w-3 h-3 text-accent" />
                            )}
                          </Show>
                        </span>
                      </th>
                    )}
                  </For>
                </tr>
              </thead>
              <tbody>
                <For each={processedRows()}>
                  {(row, rowIndex) => (
                    <tr class="hover:bg-bg-hover/50 group">
                      {/* Row number */}
                      <td class="px-2 py-1 text-xs text-txt-muted border-b border-r border-border bg-bg-secondary sticky left-0 text-right font-mono">
                        {rowIndex() + 1}
                      </td>
                      {/* Cells - clickable for copy */}
                      <For each={row}>
                        {(cell) => (
                          <td
                            class="px-3 py-1 border-b border-r border-border whitespace-nowrap max-w-xs truncate cursor-pointer hover:bg-accent/10"
                            classList={{
                              "text-right font-mono":
                                cell.type === "Int" || cell.type === "Float",
                              "text-accent": cell.type === "Bool",
                              "text-error": cell.type === "Error",
                              "text-txt-muted italic": cell.type === "Empty",
                            }}
                            title={`${formatCell(cell)} (click to copy)`}
                            onClick={() => handleCellClick(cell)}
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
            <Show when={processedRows().length === 0 && !loadingSheet()}>
              <div class="flex items-center justify-center h-full text-txt-muted">
                <Show
                  when={searchQuery().trim()}
                  fallback="No data in this sheet"
                >
                  No rows match "{searchQuery()}"
                </Show>
              </div>
            </Show>
          </Show>
        </Show>
      </div>
    </div>
  );
}
