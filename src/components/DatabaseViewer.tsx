// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * DatabaseViewer - SQLite database viewer for forensic analysis
 *
 * Displays SQLite databases with:
 * - Table list with row counts
 * - Column schema view
 * - Paginated row data grid
 * - Read-only access (forensic safe)
 */

import { createSignal, createEffect, Show, For, createMemo, on } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import {
  HiOutlineExclamationTriangle,
} from "./icons";
import { ChevronDownIcon, ChevronRightIcon } from "./icons";
import { logger } from "../utils/logger";
import { formatBytes } from "../utils";
import type { DatabaseMetadataSection } from "../types/viewerMetadata";
const log = logger.scope("DatabaseViewer");

// ============================================================================
// Types (matching Rust structs)
// ============================================================================

interface TableSummary {
  name: string;
  rowCount: number;
  columnCount: number;
  isSystem: boolean;
}

interface ColumnInfo {
  index: number;
  name: string;
  dataType: string;
  nullable: boolean;
  defaultValue: string | null;
  isPrimaryKey: boolean;
}

interface DatabaseInfo {
  path: string;
  fileSize: number;
  pageSize: number;
  pageCount: number;
  sqliteVersion: string;
  tables: TableSummary[];
}

interface TableSchema {
  name: string;
  columns: ColumnInfo[];
  rowCount: number;
  createSql: string | null;
  indexes: string[];
}

interface TableRows {
  tableName: string;
  columns: string[];
  rows: string[][];
  page: number;
  pageSize: number;
  totalCount: number;
  hasMore: boolean;
}

// ============================================================================
// Props
// ============================================================================

interface DatabaseViewerProps {
  /** Path to the SQLite database file */
  path: string;
  /** Optional class name */
  class?: string;
  /** Callback to emit metadata section for right panel */
  onMetadata?: (section: DatabaseMetadataSection) => void;
}

// ============================================================================
// Helpers
// ============================================================================

function getColumnTypeColor(type: string): string {
  const upper = type.toUpperCase();
  if (upper.includes("INT")) return "text-blue-400";
  if (upper.includes("TEXT") || upper.includes("CHAR") || upper.includes("CLOB")) return "text-green-400";
  if (upper.includes("REAL") || upper.includes("FLOAT") || upper.includes("DOUBLE")) return "text-purple-400";
  if (upper.includes("BLOB")) return "text-orange-400";
  if (upper.includes("DATE") || upper.includes("TIME")) return "text-cyan-400";
  if (upper === "" || upper === "NULL") return "text-txt-muted";
  return "text-txt-secondary";
}

// ============================================================================
// Component
// ============================================================================

export function DatabaseViewer(props: DatabaseViewerProps) {
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [dbInfo, setDbInfo] = createSignal<DatabaseInfo | null>(null);
  const [selectedTable, setSelectedTable] = createSignal<string>("");
  const [schema, setSchema] = createSignal<TableSchema | null>(null);
  const [tableRows, setTableRows] = createSignal<TableRows | null>(null);
  const [rowsLoading, setRowsLoading] = createSignal(false);
  const [schemaExpanded, setSchemaExpanded] = createSignal(true);
  const [currentPage, setCurrentPage] = createSignal(0);
  const pageSize = 100;

  // Load database info
  const loadDatabase = async () => {
    setLoading(true);
    setError(null);

    try {
      const info = await invoke<DatabaseInfo>("database_get_info", { path: props.path });
      setDbInfo(info);

      // Auto-select first non-system table
      const userTable = info.tables.find((t) => !t.isSystem);
      if (userTable) {
        selectTable(userTable.name);
      } else if (info.tables.length > 0) {
        selectTable(info.tables[0].name);
      }
    } catch (e) {
      log.error("Failed to load database:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      setLoading(false);
    }
  };

  // Select a table and load schema + rows
  const selectTable = async (tableName: string) => {
    setSelectedTable(tableName);
    setCurrentPage(0);
    setRowsLoading(true);

    try {
      const [schemaResult, rowsResult] = await Promise.all([
        invoke<TableSchema>("database_get_table_schema", {
          dbPath: props.path,
          tableName,
        }),
        invoke<TableRows>("database_query_table", {
          dbPath: props.path,
          tableName,
          page: 0,
          pageSize,
        }),
      ]);
      setSchema(schemaResult);
      setTableRows(rowsResult);
    } catch (e) {
      log.error("Failed to load table:", tableName, e);
    } finally {
      setRowsLoading(false);
    }
  };

  // Navigate pages
  const loadPage = async (page: number) => {
    const table = selectedTable();
    if (!table) return;
    setRowsLoading(true);
    setCurrentPage(page);

    try {
      const rows = await invoke<TableRows>("database_query_table", {
        dbPath: props.path,
        tableName: table,
        page,
        pageSize,
      });
      setTableRows(rows);
    } catch (e) {
      log.error("Failed to load page:", page, e);
    } finally {
      setRowsLoading(false);
    }
  };

  // Computed values
  const totalPages = createMemo(() => {
    const rows = tableRows();
    if (!rows) return 0;
    return Math.ceil(rows.totalCount / rows.pageSize);
  });

  const userTables = createMemo(() => {
    const info = dbInfo();
    if (!info) return [];
    return info.tables.filter((t) => !t.isSystem);
  });

  const systemTables = createMemo(() => {
    const info = dbInfo();
    if (!info) return [];
    return info.tables.filter((t) => t.isSystem);
  });

  // Load on mount / path change
  createEffect(on(() => props.path, () => loadDatabase()));

  // Emit metadata section when db info or selection changes
  createEffect(() => {
    const info = dbInfo();
    if (!info || !props.onMetadata) return;
    const section: DatabaseMetadataSection = {
      kind: "database",
      path: info.path,
      pageSize: info.pageSize,
      pageCount: info.pageCount,
      sizeBytes: info.fileSize,
      tableCount: info.tables.length,
      tables: info.tables.map(t => ({
        name: t.name,
        rowCount: t.rowCount,
        columnCount: t.columnCount,
        isSystem: t.isSystem,
      })),
      selectedTable: selectedTable() || undefined,
    };
    props.onMetadata(section);
  });

  return (
    <div class={`flex flex-col h-full bg-bg ${props.class || ""}`}>
      {/* Loading */}
      <Show when={loading()}>
        <div class="flex flex-col items-center justify-center h-full gap-4">
          <div class="relative w-12 h-12">
            <div class="absolute inset-0 rounded-full border-2 border-border opacity-30" />
            <div class="absolute inset-0 rounded-full border-2 border-accent border-t-transparent animate-spin" />
          </div>
          <p class="text-txt-secondary text-sm">Loading database...</p>
        </div>
      </Show>

      {/* Error */}
      <Show when={error()}>
        <div class="m-4 p-4 rounded-lg bg-error/10 border border-error/30">
          <div class="flex items-start gap-3">
            <HiOutlineExclamationTriangle class="w-5 h-5 text-error shrink-0 mt-0.5" />
            <div>
              <p class="text-error font-medium text-sm">Failed to load database</p>
              <p class="text-txt-muted text-xs mt-1">{error()}</p>
            </div>
          </div>
        </div>
      </Show>

      {/* Main Content */}
      <Show when={!loading() && !error() && dbInfo()}>
        {/* Header bar */}
        <div class="panel-header gap-3 border-b border-border">
          <span class="badge badge-success text-xs">SQLite</span>
          <span class="text-sm text-txt font-medium">{dbInfo()!.tables.length} tables</span>
          <span class="text-xs text-txt-muted">{formatBytes(dbInfo()!.fileSize)}</span>
          <span class="text-xs text-txt-muted">v{dbInfo()!.sqliteVersion}</span>
          <span class="text-xs text-txt-muted ml-auto">
            {dbInfo()!.pageCount.toLocaleString()} pages · {dbInfo()!.pageSize} B/page
          </span>
        </div>

        {/* Split: Table List + Data */}
        <div class="flex flex-1 min-h-0">
          {/* Left: Table List */}
          <div class="w-56 border-r border-border flex flex-col">
            <div class="p-2 text-xs text-txt-muted font-medium uppercase tracking-wider border-b border-border">
              Tables
            </div>
            <div class="flex-1 overflow-y-auto p-1">
              {/* User tables */}
              <Show when={userTables().length > 0}>
                <For each={userTables()}>
                  {(table) => (
                    <button
                      class={`w-full flex items-center gap-2 px-2 py-1 rounded text-xs text-left hover:bg-bg-hover ${
                        selectedTable() === table.name ? "bg-accent/20 text-accent" : "text-txt"
                      }`}
                      onClick={() => selectTable(table.name)}
                    >
                      {/* Table icon */}
                      <svg class="w-3.5 h-3.5 shrink-0 text-blue-400" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                        <path stroke-linecap="round" stroke-linejoin="round" d="M3 10h18M3 14h18m-9-4v8m-7 0h14a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
                      </svg>
                      <span class="truncate flex-1" title={table.name}>{table.name}</span>
                      <span class="text-[10px] text-txt-muted">{table.rowCount.toLocaleString()}</span>
                    </button>
                  )}
                </For>
              </Show>

              {/* System tables */}
              <Show when={systemTables().length > 0}>
                <div class="mt-2 pt-2 border-t border-border">
                  <div class="px-2 py-0.5 text-[10px] text-txt-muted uppercase">System</div>
                  <For each={systemTables()}>
                    {(table) => (
                      <button
                        class={`w-full flex items-center gap-2 px-2 py-1 rounded text-xs text-left hover:bg-bg-hover ${
                          selectedTable() === table.name ? "bg-accent/20 text-accent" : "text-txt-muted"
                        }`}
                        onClick={() => selectTable(table.name)}
                      >
                        <svg class="w-3.5 h-3.5 shrink-0 text-txt-muted" fill="none" viewBox="0 0 24 24" stroke="currentColor" stroke-width="2">
                          <path stroke-linecap="round" stroke-linejoin="round" d="M3 10h18M3 14h18m-9-4v8m-7 0h14a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z" />
                        </svg>
                        <span class="truncate flex-1" title={table.name}>{table.name}</span>
                        <span class="text-[10px] text-txt-muted">{table.rowCount.toLocaleString()}</span>
                      </button>
                    )}
                  </For>
                </div>
              </Show>
            </div>
          </div>

          {/* Right: Schema + Data */}
          <div class="flex-1 flex flex-col min-w-0">
            <Show when={selectedTable()}>
              {/* Schema Section (collapsible) */}
              <Show when={schema()}>
                <div class="border-b border-border">
                  <button
                    class="w-full flex items-center gap-2 px-3 py-2 text-xs text-txt-secondary hover:bg-bg-hover"
                    onClick={() => setSchemaExpanded(!schemaExpanded())}
                  >
                    <Show when={schemaExpanded()} fallback={<ChevronRightIcon class="w-3 h-3" />}>
                      <ChevronDownIcon class="w-3 h-3" />
                    </Show>
                    <span class="font-medium">Schema</span>
                    <span class="text-txt-muted">
                      {schema()!.columns.length} columns · {schema()!.rowCount.toLocaleString()} rows
                      {schema()!.indexes.length > 0 ? ` · ${schema()!.indexes.length} indexes` : ""}
                    </span>
                  </button>
                  <Show when={schemaExpanded()}>
                    <div class="px-3 pb-2">
                      <div class="flex flex-wrap gap-x-4 gap-y-1">
                        <For each={schema()!.columns}>
                          {(col) => (
                            <div class="flex items-center gap-1.5 text-xs">
                              <Show when={col.isPrimaryKey}>
                                <span class="text-yellow-400 text-[10px]" title="Primary Key">🔑</span>
                              </Show>
                              <span class="text-txt font-mono">{col.name}</span>
                              <span class={`font-mono text-[10px] ${getColumnTypeColor(col.dataType)}`}>
                                {col.dataType || "ANY"}
                              </span>
                              <Show when={!col.nullable}>
                                <span class="text-[10px] text-error/70">NN</span>
                              </Show>
                            </div>
                          )}
                        </For>
                      </div>
                    </div>
                  </Show>
                </div>
              </Show>

              {/* Data Grid */}
              <div class="flex-1 overflow-auto">
                <Show when={rowsLoading()}>
                  <div class="flex items-center justify-center py-8">
                    <div class="w-6 h-6 rounded-full border-2 border-accent border-t-transparent animate-spin" />
                  </div>
                </Show>
                <Show when={!rowsLoading() && tableRows()}>
                  <table class="w-full text-xs border-collapse">
                    <thead class="sticky top-0 bg-bg-panel z-10">
                      <tr class="text-left text-txt-muted border-b border-border">
                        <th class="px-2 py-1.5 font-medium text-center w-10 text-[10px]">#</th>
                        <For each={tableRows()!.columns}>
                          {(col) => (
                            <th class="px-2 py-1.5 font-medium">{col}</th>
                          )}
                        </For>
                      </tr>
                    </thead>
                    <tbody>
                      <For each={tableRows()!.rows} fallback={
                        <tr>
                          <td colspan={tableRows()!.columns.length + 1} class="px-3 py-4 text-center text-txt-muted italic">
                            No rows
                          </td>
                        </tr>
                      }>
                        {(row, idx) => (
                          <tr class="border-b border-border/30 hover:bg-bg-hover">
                            <td class="px-2 py-1 text-center text-txt-muted text-[10px]">
                              {currentPage() * pageSize + idx() + 1}
                            </td>
                            <For each={row}>
                              {(cell) => (
                                <td class="px-2 py-1 text-txt-secondary font-mono truncate max-w-[300px]" title={cell}>
                                  {cell === "NULL" ? <span class="text-txt-muted italic">NULL</span> : cell}
                                </td>
                              )}
                            </For>
                          </tr>
                        )}
                      </For>
                    </tbody>
                  </table>
                </Show>
              </div>

              {/* Pagination */}
              <Show when={tableRows() && totalPages() > 1}>
                <div class="flex items-center justify-between px-3 py-2 border-t border-border bg-bg-panel text-xs">
                  <span class="text-txt-muted">
                    Showing {currentPage() * pageSize + 1}-{Math.min((currentPage() + 1) * pageSize, tableRows()!.totalCount)} of {tableRows()!.totalCount.toLocaleString()} rows
                  </span>
                  <div class="flex items-center gap-1">
                    <button
                      class="btn-sm"
                      disabled={currentPage() === 0}
                      onClick={() => loadPage(currentPage() - 1)}
                    >
                      ← Prev
                    </button>
                    <span class="px-2 text-txt-secondary">
                      Page {currentPage() + 1} / {totalPages()}
                    </span>
                    <button
                      class="btn-sm"
                      disabled={!tableRows()!.hasMore}
                      onClick={() => loadPage(currentPage() + 1)}
                    >
                      Next →
                    </button>
                  </div>
                </div>
              </Show>
            </Show>

            {/* No selection placeholder */}
            <Show when={!selectedTable()}>
              <div class="flex items-center justify-center h-full text-txt-muted text-sm">
                Select a table from the list to view its data
              </div>
            </Show>
          </div>
        </div>
      </Show>
    </div>
  );
}
