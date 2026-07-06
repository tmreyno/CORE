// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * useDatabaseData — state management hook for the SQLite database viewer.
 *
 * Manages database loading, table selection, schema display, paginated row
 * fetching, and metadata emission.
 */

import { createSignal, createEffect, createMemo, on } from "solid-js";
import { logger } from "../../utils/logger";
import { commands, type HashSourceInput } from "../../api/commands";
import type { DatabaseMetadataSection } from "../../types/viewerMetadata";
import type { DatabaseInfo, TableSchema, TableRows } from "./types";
import { isTauri } from "../../utils/platform";

const log = logger.scope("DatabaseViewer");
const BROWSER_DATABASE_VIEW_MESSAGE =
  "Database evidence viewing is available in the desktop app.";

export const PAGE_SIZE = 100;

export interface UseDatabaseDataOptions {
  path: () => string;
  source?: () => HashSourceInput | null | undefined;
  onMetadata?: (section: DatabaseMetadataSection) => void;
}

export function useDatabaseData(opts: UseDatabaseDataOptions) {
  const [loading, setLoading] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);
  const [dbInfo, setDbInfo] = createSignal<DatabaseInfo | null>(null);
  const [selectedTable, setSelectedTable] = createSignal<string>("");
  const [schema, setSchema] = createSignal<TableSchema | null>(null);
  const [tableRows, setTableRows] = createSignal<TableRows | null>(null);
  const [rowsLoading, setRowsLoading] = createSignal(false);
  const [schemaExpanded, setSchemaExpanded] = createSignal(true);
  const [currentPage, setCurrentPage] = createSignal(0);
  let databaseGeneration = 0;
  let tableGeneration = 0;
  let pageGeneration = 0;

  // ── Load database info ──
  const loadDatabase = async () => {
    const generation = ++databaseGeneration;
    tableGeneration++;
    pageGeneration++;
    setLoading(true);
    setError(null);

    try {
      if (!isTauri) {
        throw new Error(BROWSER_DATABASE_VIEW_MESSAGE);
      }

      const source = opts.source?.();
      const info = source
        ? await commands.sqlite.getInfoSource<DatabaseInfo>(source)
        : await commands.sqlite.getInfo<DatabaseInfo>(opts.path());
      if (generation !== databaseGeneration) return;
      setDbInfo(info);

      // Auto-select first non-system table
      const userTable = info.tables.find((t) => !t.isSystem);
      if (userTable) {
        void selectTable(userTable.name, generation);
      } else if (info.tables.length > 0) {
        void selectTable(info.tables[0].name, generation);
      }
    } catch (e) {
      if (generation !== databaseGeneration) return;
      log.error("Failed to load database:", e);
      setError(e instanceof Error ? e.message : String(e));
    } finally {
      if (generation === databaseGeneration) {
        setLoading(false);
      }
    }
  };

  // ── Select a table and load schema + rows ──
  const selectTable = async (tableName: string, dbGeneration = databaseGeneration) => {
    const generation = ++tableGeneration;
    pageGeneration++;
    setSelectedTable(tableName);
    setCurrentPage(0);
    setRowsLoading(true);

    try {
      if (!isTauri) {
        throw new Error(BROWSER_DATABASE_VIEW_MESSAGE);
      }

      const source = opts.source?.();
      const [schemaResult, rowsResult] = await Promise.all([
        source
          ? commands.sqlite.getTableSchemaSource<TableSchema>(source, tableName)
          : commands.sqlite.getTableSchema<TableSchema>(opts.path(), tableName),
        source
          ? commands.sqlite.queryTableSource<TableRows>(
              source,
              tableName,
              0,
              PAGE_SIZE,
            )
          : commands.sqlite.queryTable<TableRows>(
              opts.path(),
              tableName,
              0,
              PAGE_SIZE,
            ),
      ]);
      if (dbGeneration !== databaseGeneration || generation !== tableGeneration) return;
      setSchema(schemaResult);
      setTableRows(rowsResult);
    } catch (e) {
      if (dbGeneration !== databaseGeneration || generation !== tableGeneration) return;
      log.error("Failed to load table:", tableName, e);
    } finally {
      if (dbGeneration === databaseGeneration && generation === tableGeneration) {
        setRowsLoading(false);
      }
    }
  };

  // ── Navigate pages ──
  const loadPage = async (page: number) => {
    const table = selectedTable();
    if (!table) return;
    const dbGeneration = databaseGeneration;
    const tableToken = tableGeneration;
    const generation = ++pageGeneration;
    setRowsLoading(true);
    setCurrentPage(page);

    try {
      if (!isTauri) {
        throw new Error(BROWSER_DATABASE_VIEW_MESSAGE);
      }

      const source = opts.source?.();
      const rows = source
        ? await commands.sqlite.queryTableSource<TableRows>(
            source,
            table,
            page,
            PAGE_SIZE,
          )
        : await commands.sqlite.queryTable<TableRows>(
            opts.path(),
            table,
            page,
            PAGE_SIZE,
          );
      if (
        dbGeneration !== databaseGeneration ||
        tableToken !== tableGeneration ||
        generation !== pageGeneration
      ) return;
      setTableRows(rows);
    } catch (e) {
      if (
        dbGeneration !== databaseGeneration ||
        tableToken !== tableGeneration ||
        generation !== pageGeneration
      ) return;
      log.error("Failed to load page:", page, e);
    } finally {
      if (
        dbGeneration === databaseGeneration &&
        tableToken === tableGeneration &&
        generation === pageGeneration
      ) {
        setRowsLoading(false);
      }
    }
  };

  // ── Computed ──
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

  // ── Load on path change ──
  createEffect(
    on(
      () => `${opts.path()}|${JSON.stringify(opts.source?.() ?? null)}`,
      () => loadDatabase(),
    ),
  );

  // ── Emit metadata ──
  createEffect(() => {
    const info = dbInfo();
    if (!info || !opts.onMetadata) return;
    const section: DatabaseMetadataSection = {
      kind: "database",
      path: info.path,
      pageSize: info.pageSize,
      pageCount: info.pageCount,
      sizeBytes: info.fileSize,
      tableCount: info.tables.length,
      tables: info.tables.map((t) => ({
        name: t.name,
        rowCount: t.rowCount,
        columnCount: t.columnCount,
        isSystem: t.isSystem,
      })),
      selectedTable: selectedTable() || undefined,
    };
    opts.onMetadata(section);
  });

  return {
    loading,
    error,
    dbInfo,
    selectedTable,
    schema,
    tableRows,
    rowsLoading,
    schemaExpanded,
    setSchemaExpanded,
    currentPage,
    totalPages,
    userTables,
    systemTables,
    selectTable,
    loadPage,
  };
}
