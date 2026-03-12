// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * DeduplicationPanel — modal panel for file deduplication analysis.
 *
 * Shows duplicate file groups found across evidence containers, with:
 * - Summary statistics (total files, duplicates, wasted space)
 * - Sortable/filterable group list
 * - Expandable group details showing all files
 * - Export to CSV
 */

import {
  Component,
  Show,
  For,
  createSignal,
  createMemo,
  onMount,
} from "solid-js";
import {
  HiOutlineDocumentDuplicate,
  HiOutlineXMark,
  HiOutlineArrowPath,
  HiOutlineArrowDownTray,
  HiOutlineFunnel,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineCheckBadge,
  HiOutlineExclamationTriangle,
  HiOutlineArchiveBox,
  HiOutlineDocumentText,
} from "../icons";
import {
  analyzeDuplicates,
  exportDedupCsv,
  formatBytes,
  matchTypeLabel,
  type DedupResults,
  type DedupOptions,
  type DuplicateGroup,
  type DuplicateMatchType,
} from "../../api/dedup";

// =============================================================================
// Props
// =============================================================================

export interface DeduplicationPanelProps {
  isOpen: boolean;
  onClose: () => void;
}

// =============================================================================
// Sort options
// =============================================================================

type SortField = "wastedBytes" | "fileCount" | "fileSize" | "name";

// =============================================================================
// Main Component
// =============================================================================

const DeduplicationPanel: Component<DeduplicationPanelProps> = (props) => {
  const [results, setResults] = createSignal<DedupResults | null>(null);
  const [loading, setLoading] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [sortBy, setSortBy] = createSignal<SortField>("wastedBytes");
  const [filterType, setFilterType] = createSignal<DuplicateMatchType | "all">(
    "all"
  );
  const [expandedGroups, setExpandedGroups] = createSignal<Set<string>>(
    new Set<string>()
  );
  const [includeSizeOnly, setIncludeSizeOnly] = createSignal(false);
  const [exporting, setExporting] = createSignal(false);
  const [crossContainerOnly, setCrossContainerOnly] = createSignal(false);

  // Run scan
  const runScan = async () => {
    setLoading(true);
    setError(null);
    setResults(null);
    setExpandedGroups(new Set<string>());

    try {
      const options: DedupOptions = {
        includeSizeOnlyMatches: includeSizeOnly(),
        includeEmptyFiles: false,
      };
      const res = await analyzeDuplicates(options);
      setResults(res);
    } catch (e) {
      setError(typeof e === "string" ? e : String(e));
    } finally {
      setLoading(false);
    }
  };

  // Auto-scan on first open
  let hasScanned = false;
  onMount(() => {
    if (!hasScanned) {
      hasScanned = true;
      runScan();
    }
  });

  // Toggle group expansion
  const toggleGroup = (id: string) => {
    setExpandedGroups((prev) => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  };

  // Sorted + filtered groups
  const filteredGroups = createMemo(() => {
    const r = results();
    if (!r) return [];

    let groups = [...r.groups];

    // Filter by match type
    const ft = filterType();
    if (ft !== "all") {
      groups = groups.filter((g) => g.matchType === ft);
    }

    // Filter cross-container only
    if (crossContainerOnly()) {
      groups = groups.filter((g) => g.crossContainer);
    }

    // Sort
    const field = sortBy();
    groups.sort((a, b) => {
      switch (field) {
        case "wastedBytes":
          return b.wastedBytes - a.wastedBytes;
        case "fileCount":
          return b.fileCount - a.fileCount;
        case "fileSize":
          return b.fileSize - a.fileSize;
        case "name":
          return a.representativeName.localeCompare(b.representativeName);
        default:
          return 0;
      }
    });

    return groups;
  });

  // Export CSV
  const handleExport = async () => {
    const r = results();
    if (!r) return;
    setExporting(true);
    try {
      const csv = await exportDedupCsv(r);
      const blob = new Blob([csv], { type: "text/csv" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `dedup_report_${new Date().toISOString().slice(0, 10)}.csv`;
      a.click();
      URL.revokeObjectURL(url);
    } catch (e) {
      setError(typeof e === "string" ? e : String(e));
    } finally {
      setExporting(false);
    }
  };

  // Keyboard: Escape to close
  const handleKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") {
      props.onClose();
    }
  };

  return (
    <Show when={props.isOpen}>
      <div
        class="fixed inset-0 bg-black/60 backdrop-blur-sm z-50 flex items-start justify-center pt-12 animate-fade-in"
        onKeyDown={handleKeyDown}
      >
        <div class="dedup-panel w-[800px] max-h-[80vh] flex flex-col bg-bg-panel border border-border rounded-xl shadow-2xl overflow-hidden animate-slide-up">
          {/* Header */}
          <div class="flex items-center justify-between px-5 py-3.5 border-b border-border bg-bg-secondary/30">
            <div class="flex items-center gap-2.5">
              <div class="p-1.5 rounded-md bg-accent/10 text-accent">
                <HiOutlineDocumentDuplicate class="w-5 h-5" />
              </div>
              <div>
                <h2 class="text-base font-semibold text-txt">
                  File Deduplication
                </h2>
                <p class="text-2xs text-txt-muted">
                  Find duplicate files across evidence containers
                </p>
              </div>
            </div>
            <div class="flex items-center gap-1.5">
              <button
                class="icon-btn"
                onClick={runScan}
                disabled={loading()}
                title="Re-scan"
              >
                <HiOutlineArrowPath
                  class={`w-4 h-4 ${loading() ? "animate-spin" : ""}`}
                />
              </button>
              <Show when={results()}>
                <button
                  class="icon-btn"
                  onClick={handleExport}
                  disabled={exporting()}
                  title="Export CSV"
                >
                  <HiOutlineArrowDownTray class="w-4 h-4" />
                </button>
              </Show>
              <button
                class="icon-btn"
                onClick={props.onClose}
                title="Close (Esc)"
              >
                <HiOutlineXMark class="w-4 h-4" />
              </button>
            </div>
          </div>

          {/* Stats Bar */}
          <Show when={results()}>
            {(r) => (
              <div class="grid grid-cols-4 gap-3 px-5 py-3 border-b border-border/50 bg-bg-secondary/10">
                <StatBox
                  label="Files Scanned"
                  value={r().stats.totalFilesScanned.toLocaleString()}
                />
                <StatBox
                  label="Duplicate Groups"
                  value={r().stats.totalDuplicateGroups.toLocaleString()}
                  accent={r().stats.totalDuplicateGroups > 0}
                />
                <StatBox
                  label="Duplicate Files"
                  value={r().stats.totalDuplicateFiles.toLocaleString()}
                  accent={r().stats.totalDuplicateFiles > 0}
                />
                <StatBox
                  label="Wasted Space"
                  value={formatBytes(r().stats.totalWastedBytes)}
                  accent={r().stats.totalWastedBytes > 0}
                />
              </div>
            )}
          </Show>

          {/* Filters & Sort */}
          <Show when={results() && results()!.groups.length > 0}>
            <div class="flex items-center gap-3 px-5 py-2.5 border-b border-border/30">
              <HiOutlineFunnel class="w-3.5 h-3.5 text-txt-muted shrink-0" />

              {/* Match type filter */}
              <select
                class="input-xs"
                value={filterType()}
                onChange={(e) =>
                  setFilterType(
                    e.currentTarget.value as DuplicateMatchType | "all"
                  )
                }
              >
                <option value="all">All Match Types</option>
                <option value="exactHash">Exact Hash</option>
                <option value="sizeAndName">Size + Name</option>
                <option value="sizeOnly">Size Only</option>
              </select>

              {/* Sort */}
              <select
                class="input-xs"
                value={sortBy()}
                onChange={(e) =>
                  setSortBy(e.currentTarget.value as SortField)
                }
              >
                <option value="wastedBytes">Most Wasted Space</option>
                <option value="fileCount">Most Duplicates</option>
                <option value="fileSize">Largest Files</option>
                <option value="name">Name (A-Z)</option>
              </select>

              {/* Cross-container toggle */}
              <label class="flex items-center gap-1.5 text-xs text-txt-secondary cursor-pointer ml-auto whitespace-nowrap">
                <input
                  type="checkbox"
                  checked={crossContainerOnly()}
                  onChange={(e) =>
                    setCrossContainerOnly(e.currentTarget.checked)
                  }
                  class="rounded"
                />
                Cross-container only
              </label>

              {/* Size-only toggle */}
              <label class="flex items-center gap-1.5 text-xs text-txt-secondary cursor-pointer whitespace-nowrap">
                <input
                  type="checkbox"
                  checked={includeSizeOnly()}
                  onChange={(e) => {
                    setIncludeSizeOnly(e.currentTarget.checked);
                    // Re-scan with new option
                    runScan();
                  }}
                  class="rounded"
                />
                Size-only matches
              </label>

              <span class="text-2xs text-txt-muted ml-2">
                {filteredGroups().length} group
                {filteredGroups().length !== 1 ? "s" : ""}
              </span>
            </div>
          </Show>

          {/* Content Area */}
          <div class="flex-1 overflow-y-auto">
            {/* Loading */}
            <Show when={loading()}>
              <div class="flex flex-col items-center justify-center py-16 gap-3">
                <div class="w-8 h-8 border-2 border-accent border-t-transparent rounded-full animate-spin" />
                <span class="text-sm text-txt-muted">
                  Scanning for duplicates...
                </span>
              </div>
            </Show>

            {/* Error */}
            <Show when={error()}>
              <div class="flex flex-col items-center justify-center py-12 gap-3 text-error">
                <HiOutlineExclamationTriangle class="w-8 h-8 opacity-50" />
                <span class="text-sm">{error()}</span>
                <button class="btn btn-secondary btn-sm" onClick={runScan}>
                  Retry
                </button>
              </div>
            </Show>

            {/* No results */}
            <Show
              when={
                !loading() &&
                !error() &&
                results() &&
                results()!.groups.length === 0
              }
            >
              <div class="flex flex-col items-center justify-center py-16 gap-3">
                <HiOutlineCheckBadge class="w-10 h-10 text-success opacity-40" />
                <span class="text-sm text-txt-muted">
                  No duplicate files found
                </span>
                <span class="text-xs text-txt-muted">
                  {results()!.stats.totalFilesScanned.toLocaleString()} files
                  scanned in {results()!.stats.elapsedMs}ms
                </span>
              </div>
            </Show>

            {/* Results list */}
            <Show when={!loading() && !error() && filteredGroups().length > 0}>
              <div class="divide-y divide-border/30">
                <For each={filteredGroups()}>
                  {(group) => (
                    <GroupRow
                      group={group}
                      isExpanded={expandedGroups().has(group.id)}
                      onToggle={() => toggleGroup(group.id)}
                    />
                  )}
                </For>
              </div>
            </Show>
          </div>

          {/* Footer */}
          <Show when={results()}>
            <div class="px-5 py-2.5 border-t border-border/50 bg-bg-secondary/30 flex items-center justify-between">
              <span class="text-xs text-txt-muted">
                Analysis completed in {results()!.stats.elapsedMs}ms
              </span>
              <span class="text-xs text-txt-muted">
                {results()!.stats.uniqueFiles.toLocaleString()} unique files
              </span>
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
};

// =============================================================================
// Sub-components
// =============================================================================

/** Small stat box for the summary bar */
function StatBox(props: {
  label: string;
  value: string;
  accent?: boolean;
}) {
  return (
    <div class="flex flex-col items-center gap-0.5">
      <span
        class={`text-lg font-semibold ${props.accent ? "text-accent" : "text-txt"}`}
      >
        {props.value}
      </span>
      <span class="text-2xs text-txt-muted uppercase tracking-wider">
        {props.label}
      </span>
    </div>
  );
}

/** Expandable duplicate group row */
function GroupRow(props: {
  group: DuplicateGroup;
  isExpanded: boolean;
  onToggle: () => void;
}) {
  const g = props.group;

  return (
    <div class="group">
      {/* Group header */}
      <button
        class="w-full flex items-center gap-3 px-5 py-3 hover:bg-bg-hover transition-colors text-left"
        onClick={props.onToggle}
      >
        {/* Expand chevron */}
        <span class="text-txt-muted shrink-0">
          <Show
            when={props.isExpanded}
            fallback={<HiOutlineChevronRight class="w-3.5 h-3.5" />}
          >
            <HiOutlineChevronDown class="w-3.5 h-3.5" />
          </Show>
        </span>

        {/* File icon + name */}
        <HiOutlineDocumentText class="w-4 h-4 text-txt-muted shrink-0" />
        <div class="flex-1 min-w-0">
          <span class="text-sm text-txt font-medium truncate block">
            {g.representativeName}
          </span>
          <span class="text-2xs text-txt-muted">
            {g.fileCount} files &middot; {formatBytes(g.fileSize)} each
          </span>
        </div>

        {/* Match type badge */}
        <span
          class={`text-2xs font-medium px-2 py-0.5 rounded ${matchTypeBg(g.matchType)}`}
        >
          {matchTypeLabel(g.matchType)}
        </span>

        {/* Cross-container indicator */}
        <Show when={g.crossContainer}>
          <span class="text-2xs text-info flex items-center gap-1" title="Files span multiple containers">
            <HiOutlineArchiveBox class="w-3 h-3" />
            Cross
          </span>
        </Show>

        {/* Wasted space */}
        <span class="text-xs text-warning font-medium tabular-nums whitespace-nowrap">
          {formatBytes(g.wastedBytes)} wasted
        </span>
      </button>

      {/* Expanded file list */}
      <Show when={props.isExpanded}>
        <div class="px-5 pb-3">
          <div class="ml-8 border border-border/40 rounded-lg overflow-hidden bg-bg-secondary/20">
            <div class="grid grid-cols-[1fr_auto_auto] gap-2 px-3 py-1.5 text-2xs text-txt-muted uppercase tracking-wider border-b border-border/30 bg-bg-secondary/30">
              <span>File Path</span>
              <span>Container</span>
              <span>Hash</span>
            </div>
            <For each={g.files}>
              {(file) => (
                <div class="grid grid-cols-[1fr_auto_auto] gap-2 px-3 py-2 text-xs border-b border-border/20 last:border-b-0 hover:bg-bg-hover/50">
                  <div class="min-w-0">
                    <span class="text-txt truncate block" title={file.entryPath}>
                      {file.entryPath}
                    </span>
                  </div>
                  <span class="text-txt-muted text-2xs truncate max-w-[180px]" title={file.containerPath}>
                    {containerName(file.containerPath)}
                  </span>
                  <span class="font-mono text-compact text-txt-muted truncate max-w-[100px]" title={file.hash || "No hash"}>
                    {file.hash ? file.hash.slice(0, 12) + "..." : "—"}
                  </span>
                </div>
              )}
            </For>
          </div>
        </div>
      </Show>
    </div>
  );
}

// =============================================================================
// Helpers
// =============================================================================

/** Get background color class for match type badge */
function matchTypeBg(type: DuplicateMatchType): string {
  switch (type) {
    case "exactHash":
      return "bg-success/15 text-success";
    case "sizeAndName":
      return "bg-warning/15 text-warning";
    case "sizeOnly":
      return "bg-bg-secondary text-txt-muted";
    default:
      return "bg-bg-secondary text-txt-muted";
  }
}

/** Extract container filename from full path */
function containerName(path: string): string {
  const parts = path.replace(/\\/g, "/").split("/");
  return parts[parts.length - 1] || path;
}

export default DeduplicationPanel;
