// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * DatabaseViewerComponent — slim render shell for the SQLite database viewer.
 * All state and logic live in useDatabaseData hook.
 */

import { Show, For } from "solid-js";
import { HiOutlineExclamationTriangle } from "../icons";
import { ChevronDownIcon, ChevronRightIcon } from "../icons";
import { formatBytes } from "../../utils";
import { useDatabaseData, PAGE_SIZE } from "./useDatabaseData";
import { getColumnTypeColor } from "./helpers";
import type { DatabaseViewerProps } from "./types";

export function DatabaseViewer(props: DatabaseViewerProps) {
  const db = useDatabaseData({
    path: () => props.path,
    onMetadata: props.onMetadata,
  });

  return (
    <div class={`flex flex-col h-full bg-bg ${props.class || ""}`}>
      {/* Loading */}
      <Show when={db.loading()}>
        <div class="flex flex-col items-center justify-center h-full gap-4">
          <div class="relative w-12 h-12">
            <div class="absolute inset-0 rounded-full border-2 border-border opacity-30" />
            <div class="absolute inset-0 rounded-full border-2 border-accent border-t-transparent animate-spin" />
          </div>
          <p class="text-txt-secondary text-sm">Loading database...</p>
        </div>
      </Show>

      {/* Error */}
      <Show when={db.error()}>
        <div class="m-4 p-4 rounded-lg bg-error/10 border border-error/30">
          <div class="flex items-start gap-3">
            <HiOutlineExclamationTriangle class="w-5 h-5 text-error shrink-0 mt-0.5" />
            <div>
              <p class="text-error font-medium text-sm">Failed to load database</p>
              <p class="text-txt-muted text-xs mt-1">{db.error()}</p>
            </div>
          </div>
        </div>
      </Show>

      {/* Main Content */}
      <Show when={!db.loading() && !db.error() && db.dbInfo()}>
        {/* Header bar */}
        <div class="panel-header gap-3 border-b border-border">
          <span class="badge badge-success text-xs">SQLite</span>
          <span class="text-sm text-txt font-medium">
            {db.dbInfo()!.tables.length} tables
          </span>
          <span class="text-xs text-txt-muted">{formatBytes(db.dbInfo()!.fileSize)}</span>
          <span class="text-xs text-txt-muted">v{db.dbInfo()!.sqliteVersion}</span>
          <span class="text-xs text-txt-muted ml-auto">
            {db.dbInfo()!.pageCount.toLocaleString()} pages · {db.dbInfo()!.pageSize} B/page
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
              <Show when={db.userTables().length > 0}>
                <For each={db.userTables()}>
                  {(table) => (
                    <button
                      class={`w-full flex items-center gap-2 px-2 py-1 rounded text-xs text-left hover:bg-bg-hover ${
                        db.selectedTable() === table.name
                          ? "bg-accent/20 text-accent"
                          : "text-txt"
                      }`}
                      onClick={() => db.selectTable(table.name)}
                    >
                      <svg
                        class="w-3.5 h-3.5 shrink-0 text-blue-400"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke="currentColor"
                        stroke-width="2"
                      >
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          d="M3 10h18M3 14h18m-9-4v8m-7 0h14a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"
                        />
                      </svg>
                      <span class="truncate flex-1" title={table.name}>
                        {table.name}
                      </span>
                      <span class="text-[10px] text-txt-muted">
                        {table.rowCount.toLocaleString()}
                      </span>
                    </button>
                  )}
                </For>
              </Show>

              {/* System tables */}
              <Show when={db.systemTables().length > 0}>
                <div class="mt-2 pt-2 border-t border-border">
                  <div class="px-2 py-0.5 text-[10px] text-txt-muted uppercase">System</div>
                  <For each={db.systemTables()}>
                    {(table) => (
                      <button
                        class={`w-full flex items-center gap-2 px-2 py-1 rounded text-xs text-left hover:bg-bg-hover ${
                          db.selectedTable() === table.name
                            ? "bg-accent/20 text-accent"
                            : "text-txt-muted"
                        }`}
                        onClick={() => db.selectTable(table.name)}
                      >
                        <svg
                          class="w-3.5 h-3.5 shrink-0 text-txt-muted"
                          fill="none"
                          viewBox="0 0 24 24"
                          stroke="currentColor"
                          stroke-width="2"
                        >
                          <path
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            d="M3 10h18M3 14h18m-9-4v8m-7 0h14a2 2 0 002-2V8a2 2 0 00-2-2H5a2 2 0 00-2 2v8a2 2 0 002 2z"
                          />
                        </svg>
                        <span class="truncate flex-1" title={table.name}>
                          {table.name}
                        </span>
                        <span class="text-[10px] text-txt-muted">
                          {table.rowCount.toLocaleString()}
                        </span>
                      </button>
                    )}
                  </For>
                </div>
              </Show>
            </div>
          </div>

          {/* Right: Schema + Data */}
          <div class="flex-1 flex flex-col min-w-0">
            <Show when={db.selectedTable()}>
              {/* Schema Section (collapsible) */}
              <Show when={db.schema()}>
                <div class="border-b border-border">
                  <button
                    class="w-full flex items-center gap-2 px-3 py-2 text-xs text-txt-secondary hover:bg-bg-hover"
                    onClick={() => db.setSchemaExpanded(!db.schemaExpanded())}
                  >
                    <Show
                      when={db.schemaExpanded()}
                      fallback={<ChevronRightIcon class="w-3 h-3" />}
                    >
                      <ChevronDownIcon class="w-3 h-3" />
                    </Show>
                    <span class="font-medium">Schema</span>
                    <span class="text-txt-muted">
                      {db.schema()!.columns.length} columns ·{" "}
                      {db.schema()!.rowCount.toLocaleString()} rows
                      {db.schema()!.indexes.length > 0
                        ? ` · ${db.schema()!.indexes.length} indexes`
                        : ""}
                    </span>
                  </button>
                  <Show when={db.schemaExpanded()}>
                    <div class="px-3 pb-2">
                      <div class="flex flex-wrap gap-x-4 gap-y-1">
                        <For each={db.schema()!.columns}>
                          {(col) => (
                            <div class="flex items-center gap-1.5 text-xs">
                              <Show when={col.isPrimaryKey}>
                                <span class="text-yellow-400 text-[10px]" title="Primary Key">
                                  🔑
                                </span>
                              </Show>
                              <span class="text-txt font-mono">{col.name}</span>
                              <span
                                class={`font-mono text-[10px] ${getColumnTypeColor(col.dataType)}`}
                              >
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
                <Show when={db.rowsLoading()}>
                  <div class="flex items-center justify-center py-8">
                    <div class="w-6 h-6 rounded-full border-2 border-accent border-t-transparent animate-spin" />
                  </div>
                </Show>
                <Show when={!db.rowsLoading() && db.tableRows()}>
                  <table class="w-full text-xs border-collapse">
                    <thead class="sticky top-0 bg-bg-panel z-10">
                      <tr class="text-left text-txt-muted border-b border-border">
                        <th class="px-2 py-1.5 font-medium text-center w-10 text-[10px]">#</th>
                        <For each={db.tableRows()!.columns}>
                          {(col) => <th class="px-2 py-1.5 font-medium">{col}</th>}
                        </For>
                      </tr>
                    </thead>
                    <tbody>
                      <For
                        each={db.tableRows()!.rows}
                        fallback={
                          <tr>
                            <td
                              colspan={db.tableRows()!.columns.length + 1}
                              class="px-3 py-4 text-center text-txt-muted italic"
                            >
                              No rows
                            </td>
                          </tr>
                        }
                      >
                        {(row, idx) => (
                          <tr class="border-b border-border/30 hover:bg-bg-hover">
                            <td class="px-2 py-1 text-center text-txt-muted text-[10px]">
                              {db.currentPage() * PAGE_SIZE + idx() + 1}
                            </td>
                            <For each={row}>
                              {(cell) => (
                                <td
                                  class="px-2 py-1 text-txt-secondary font-mono truncate max-w-[300px]"
                                  title={cell}
                                >
                                  {cell === "NULL" ? (
                                    <span class="text-txt-muted italic">NULL</span>
                                  ) : (
                                    cell
                                  )}
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
              <Show when={db.tableRows() && db.totalPages() > 1}>
                <div class="flex items-center justify-between px-3 py-2 border-t border-border bg-bg-panel text-xs">
                  <span class="text-txt-muted">
                    Showing {db.currentPage() * PAGE_SIZE + 1}-
                    {Math.min(
                      (db.currentPage() + 1) * PAGE_SIZE,
                      db.tableRows()!.totalCount,
                    )}{" "}
                    of {db.tableRows()!.totalCount.toLocaleString()} rows
                  </span>
                  <div class="flex items-center gap-1">
                    <button
                      class="btn-sm"
                      disabled={db.currentPage() === 0}
                      onClick={() => db.loadPage(db.currentPage() - 1)}
                    >
                      ← Prev
                    </button>
                    <span class="px-2 text-txt-secondary">
                      Page {db.currentPage() + 1} / {db.totalPages()}
                    </span>
                    <button
                      class="btn-sm"
                      disabled={!db.tableRows()!.hasMore}
                      onClick={() => db.loadPage(db.currentPage() + 1)}
                    >
                      Next →
                    </button>
                  </div>
                </div>
              </Show>
            </Show>

            {/* No selection placeholder */}
            <Show when={!db.selectedTable()}>
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
