// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Shared types for the search system (hook, panel, sub-components).
 */

export interface SearchFilter {
  /** Filter by file type extension */
  fileTypes?: string[];
  /** Filter by size range (in bytes) */
  sizeRange?: { min?: number; max?: number };
  /** Filter by date range */
  dateRange?: { start?: Date; end?: Date };
  /** Include directories */
  includeDirs?: boolean;
  /** Include hidden files */
  includeHidden?: boolean;
  /** Search in file content */
  searchContent?: boolean;
  /** Case sensitive search */
  caseSensitive?: boolean;
  /** Regex search */
  useRegex?: boolean;
}

export interface SearchResult {
  /** Unique ID */
  id: string;
  /** File path (or entry path within container) */
  path: string;
  /** File name */
  name: string;
  /** Matched text context */
  matchContext?: string;
  /** Line number if content search */
  lineNumber?: number;
  /** File size */
  size: number;
  /** Is directory */
  isDir: boolean;
  /** Match score for ranking */
  score: number;
  /** Container path if result is from within a container */
  containerPath?: string;
  /** Container type (ad1, zip, e01, etc.) */
  containerType?: string;
  /** Match type: "name", "path", or "both" */
  matchType?: string;
}

export interface SavedSearch {
  id: string;
  name: string;
  query: string;
  filters: SearchFilter;
  createdAt: Date;
}

export interface UseSearchOptions {
  /** Debounce delay in ms */
  debounceMs?: number;
  /** Max search history items */
  maxHistory?: number;
  /** Storage key for history */
  storageKey?: string;
}

export interface SearchPanelProps {
  /** Whether the panel is open */
  isOpen: boolean;
  /** Close handler */
  onClose: () => void;
  /** Search function - called with query and filters, should return results */
  onSearch: (query: string, filters: SearchFilter) => Promise<SearchResult[]>;
  /** Called when a result is selected */
  onSelectResult: (result: SearchResult) => void;
  /** Placeholder text */
  placeholder?: string;
  /** Pre-fill search with this query when panel opens (e.g., from text selection) */
  initialQuery?: string;
  /** Called after initialQuery is consumed, so parent can clear it */
  onInitialQueryConsumed?: () => void;
}
