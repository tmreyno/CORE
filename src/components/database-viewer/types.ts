// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Database viewer types — matching Rust structs.
 */

import type { DatabaseMetadataSection } from "../../types/viewerMetadata";

export interface TableSummary {
  name: string;
  rowCount: number;
  columnCount: number;
  isSystem: boolean;
}

export interface ColumnInfo {
  index: number;
  name: string;
  dataType: string;
  nullable: boolean;
  defaultValue: string | null;
  isPrimaryKey: boolean;
}

export interface DatabaseInfo {
  path: string;
  fileSize: number;
  pageSize: number;
  pageCount: number;
  sqliteVersion: string;
  tables: TableSummary[];
}

export interface TableSchema {
  name: string;
  columns: ColumnInfo[];
  rowCount: number;
  createSql: string | null;
  indexes: string[];
}

export interface TableRows {
  tableName: string;
  columns: string[];
  rows: string[][];
  page: number;
  pageSize: number;
  totalCount: number;
  hasMore: boolean;
}

export interface DatabaseViewerProps {
  /** Path to the SQLite database file */
  path: string;
  /** Optional class name */
  class?: string;
  /** Callback to emit metadata section for right panel */
  onMetadata?: (section: DatabaseMetadataSection) => void;
}
