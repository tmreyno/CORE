// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Search Engine API — Tantivy full-text search.
 *
 * Provides typed wrappers for the 8 search Tauri commands:
 * - Index lifecycle (open, close, delete)
 * - Indexing (single container, all, rebuild)
 * - Query execution
 * - Index statistics
 */

import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

// =============================================================================
// Types (mirror Rust search::query types)
// =============================================================================

/** A single search hit from the Tantivy index. */
export interface SearchHit {
  docId: string;
  containerPath: string;
  containerType: string;
  entryPath: string;
  filename: string;
  extension: string;
  size: number;
  modified: number;
  isDir: boolean;
  fileCategory: string;
  score: number;
  /** HTML snippet with `<mark>` highlighted matches */
  snippet: string;
  /** Whether this was a content match (vs filename/path match) */
  contentMatch: boolean;
}

/** Search query options sent to the backend. */
export interface TantivySearchOptions {
  query: string;
  limit?: number;
  containerTypes?: string[];
  extensions?: string[];
  categories?: string[];
  minSize?: number;
  maxSize?: number;
  includeDirs?: boolean;
  searchContent?: boolean;
  containerPath?: string;
}

/** Aggregated search results. */
export interface SearchResults {
  hits: SearchHit[];
  totalHits: number;
  elapsedMs: number;
  categoryCounts: FacetCount[];
  containerTypeCounts: FacetCount[];
}

/** Facet count for category or container type. */
export interface FacetCount {
  label: string;
  count: number;
}

/** Index statistics. */
export interface IndexStats {
  numDocs: number;
  numSegments: number;
  indexSizeBytes: number;
  contentIndexedDocs: number;
}

/** Indexing progress event payload. */
export interface IndexProgress {
  containerPath: string;
  phase: IndexPhase;
  indexed: number;
  total: number;
  percent: number;
  message: string;
}

export type IndexPhase =
  | "scanning"
  | "indexingMetadata"
  | "extractingContent"
  | "committing"
  | "done";

// =============================================================================
// Index Lifecycle
// =============================================================================

/**
 * Open or create a search index for the current project.
 * Called when a project is opened.
 */
export async function openSearchIndex(ffxdbPath: string): Promise<IndexStats> {
  return invoke<IndexStats>("search_open_index", { ffxdbPath });
}

/** Close the search index for the current window. */
export async function closeSearchIndex(): Promise<void> {
  return invoke("search_close_index");
}

/** Delete the search index files from disk. */
export async function deleteSearchIndex(ffxdbPath: string): Promise<void> {
  return invoke("search_delete_index", { ffxdbPath });
}

/** Get index statistics. */
export async function getSearchIndexStats(): Promise<IndexStats> {
  return invoke<IndexStats>("search_get_stats");
}

// =============================================================================
// Indexing
// =============================================================================

/**
 * Index a single container (metadata + optionally content).
 * Emits "search-index-progress" events.
 */
export async function indexContainer(
  containerPath: string,
  indexContent: boolean = false
): Promise<void> {
  return invoke("search_index_container", { containerPath, indexContent });
}

/**
 * Index all containers sequentially.
 * Emits "search-index-progress" events for each container.
 */
export async function indexAllContainers(
  containerPaths: string[],
  indexContent: boolean = false
): Promise<void> {
  return invoke("search_index_all", { containerPaths, indexContent });
}

/**
 * Rebuild the entire index from scratch.
 * Deletes existing data and re-indexes all containers.
 */
export async function rebuildSearchIndex(
  containerPaths: string[],
  indexContent: boolean = false
): Promise<void> {
  return invoke("search_rebuild_index", { containerPaths, indexContent });
}

// =============================================================================
// Search
// =============================================================================

/** Execute a search query against the Tantivy index. */
export async function searchQuery(
  options: TantivySearchOptions
): Promise<SearchResults> {
  return invoke<SearchResults>("search_query", { options });
}

// =============================================================================
// Event Listeners
// =============================================================================

/** Listen for indexing progress events. */
export async function onIndexProgress(
  callback: (progress: IndexProgress) => void
): Promise<UnlistenFn> {
  return listen<IndexProgress>("search-index-progress", (event) => {
    callback(event.payload);
  });
}
