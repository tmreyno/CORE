import { Component, Show, For, createSignal, createMemo } from "solid-js";
import { useDeduplication } from "../hooks/useDeduplication";
import { HiOutlineDocumentDuplicate, HiOutlineArrowDownTray, HiOutlineArrowPath } from "solid-icons/hi";
import type { DuplicateGroup } from "../hooks/useDeduplication";

export const DeduplicationPanel: Component = () => {
  const dedup = useDeduplication();
  const [sortBy, setSortBy] = createSignal<"size" | "count" | "waste">("waste");
  const [filterMinSize, setFilterMinSize] = createSignal(0);

  // Filtered and sorted duplicate groups
  const filteredGroups = createMemo(() => {
    const groups = dedup.duplicateGroups();
    const minSize = filterMinSize();
    
    let filtered = minSize > 0
      ? groups.filter(g => g.wastedSpace >= minSize)
      : groups;

    // Sort
    const sort = sortBy();
    filtered = [...filtered].sort((a, b) => {
      switch (sort) {
        case "size":
          return b.totalSize - a.totalSize;
        case "count":
          return b.fileCount - a.fileCount;
        case "waste":
        default:
          return b.wastedSpace - a.wastedSpace;
      }
    });

    return filtered;
  });

  const handleExport = async () => {
    const report = await dedup.exportReport();
    if (report) {
      // Create download
      const blob = new Blob([report], { type: "application/json" });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `deduplication-report-${Date.now()}.json`;
      a.click();
      URL.revokeObjectURL(url);
    }
  };

  return (
    <div class="flex flex-col h-full bg-bg">
      {/* Header */}
      <div class="flex items-center justify-between p-4 border-b border-border">
        <div class="flex items-center gap-small">
          <HiOutlineDocumentDuplicate class="w-icon-lg h-icon-lg text-accent" />
          <h2 class="text-lg font-semibold text-txt">File Deduplication</h2>
        </div>
        
        <div class="flex items-center gap-small">
          <Show when={dedup.stats()}>
            <button
              onClick={handleExport}
              class="px-3 py-1.5 bg-bg-secondary text-txt hover:bg-bg-hover rounded-md border border-border flex items-center gap-compact"
            >
              <HiOutlineArrowDownTray class="w-icon-sm h-icon-sm" />
              Export Report
            </button>
          </Show>
          
          <button
            onClick={() => dedup.clear()}
            class="px-3 py-1.5 bg-bg-secondary text-txt hover:bg-bg-hover rounded-md border border-border flex items-center gap-compact"
          >
            <HiOutlineArrowPath class="w-icon-sm h-icon-sm" />
            Clear
          </button>
        </div>
      </div>

      {/* Error Display */}
      <Show when={dedup.error()}>
        <div class="m-4 p-3 bg-error/10 border border-error/20 rounded-md">
          <p class="text-error text-sm">{dedup.error()}</p>
        </div>
      </Show>

      {/* Statistics Grid */}
      <Show when={dedup.stats()}>
        {(stats) => (
          <div class="grid grid-cols-4 gap-4 p-4 border-b border-border">
            <div class="bg-bg-panel rounded-md p-3 border border-border">
              <div class="text-xs text-txt-muted uppercase tracking-wide mb-1">Total Files</div>
              <div class="text-2xl font-bold text-txt">{stats().totalFiles.toLocaleString()}</div>
              <div class="text-xs text-txt-secondary mt-1">{dedup.formatBytes(stats().totalSize)}</div>
            </div>

            <div class="bg-bg-panel rounded-md p-3 border border-border">
              <div class="text-xs text-txt-muted uppercase tracking-wide mb-1">Unique Files</div>
              <div class="text-2xl font-bold text-success">{stats().uniqueFiles.toLocaleString()}</div>
              <div class="text-xs text-txt-secondary mt-1">
                {dedup.formatPercent((stats().uniqueFiles / stats().totalFiles) * 100)}
              </div>
            </div>

            <div class="bg-bg-panel rounded-md p-3 border border-border">
              <div class="text-xs text-txt-muted uppercase tracking-wide mb-1">Duplicates</div>
              <div class="text-2xl font-bold text-warning">{stats().duplicateFiles.toLocaleString()}</div>
              <div class="text-xs text-txt-secondary mt-1">
                {stats().duplicateGroups} groups
              </div>
            </div>

            <div class="bg-bg-panel rounded-md p-3 border border-border">
              <div class="text-xs text-txt-muted uppercase tracking-wide mb-1">Wasted Space</div>
              <div class="text-2xl font-bold text-error">{dedup.formatBytes(stats().wastedSpace)}</div>
              <div class="text-xs text-txt-secondary mt-1">
                {dedup.formatPercent(stats().spaceSavingsPercent)} savings
              </div>
            </div>
          </div>
        )}
      </Show>

      {/* Progress Bar */}
      <Show when={dedup.scanning() && dedup.progress()}>
        {(progress) => (
          <div class="p-4 border-b border-border">
            <div class="flex items-center justify-between mb-2">
              <span class="text-sm text-txt">
                Scanning... {progress().filesProcessed} / {progress().totalFiles}
              </span>
              <span class="text-sm text-txt-secondary">
                {dedup.formatPercent(progress().percentComplete)}
              </span>
            </div>
            <div class="w-full bg-bg-secondary rounded-full h-2">
              <div
                class="bg-accent h-2 rounded-full transition-all duration-300"
                style={{ width: `${progress().percentComplete}%` }}
              />
            </div>
            <Show when={progress().throughputMbps > 0}>
              <div class="text-xs text-txt-muted mt-1">
                Throughput: {progress().throughputMbps.toFixed(2)} MB/s
              </div>
            </Show>
          </div>
        )}
      </Show>

      {/* Filters and Sorting */}
      <Show when={dedup.duplicateGroups().length > 0}>
        <div class="flex items-center gap-4 p-4 border-b border-border">
          <div class="flex items-center gap-small">
            <label class="text-sm text-txt-secondary">Sort by:</label>
            <select
              value={sortBy()}
              onChange={(e) => {
                const value = e.currentTarget.value as "size" | "count" | "waste";
                setSortBy(value);
              }}
              class="px-2 py-1 bg-bg-secondary text-txt border border-border rounded-md text-sm"
            >
              <option value="waste">Wasted Space</option>
              <option value="size">File Size</option>
              <option value="count">File Count</option>
            </select>
          </div>

          <div class="flex items-center gap-small">
            <label class="text-sm text-txt-secondary">Min Waste:</label>
            <input
              type="number"
              value={filterMinSize()}
              onInput={(e) => setFilterMinSize(Number(e.currentTarget.value))}
              placeholder="0"
              class="w-32 px-2 py-1 bg-bg-secondary text-txt border border-border rounded-md text-sm"
            />
          </div>

          <div class="text-sm text-txt-muted ml-auto">
            Showing {filteredGroups().length} of {dedup.duplicateGroups().length} groups
          </div>
        </div>
      </Show>

      {/* Duplicate Groups List */}
      <div class="flex-1 overflow-y-auto p-4">
        <Show
          when={filteredGroups().length > 0}
          fallback={
            <div class="flex flex-col items-center justify-center h-full text-txt-muted">
              <HiOutlineDocumentDuplicate class="w-16 h-16 mb-4 opacity-50" />
              <p class="text-lg">No duplicate files found</p>
              <p class="text-sm mt-2">Scan files to detect duplicates</p>
            </div>
          }
        >
          <div class="space-y-3">
            <For each={filteredGroups()}>
              {(group) => <DuplicateGroupCard group={group} dedup={dedup} />}
            </For>
          </div>
        </Show>
      </div>
    </div>
  );
};

// Separate component for duplicate group card
const DuplicateGroupCard: Component<{ group: DuplicateGroup; dedup: ReturnType<typeof useDeduplication> }> = (props) => {
  const [expanded, setExpanded] = createSignal(false);

  return (
    <div class="bg-bg-panel rounded-md border border-border overflow-hidden">
      {/* Group Header */}
      <div
        class="flex items-center justify-between p-3 cursor-pointer hover:bg-bg-hover"
        onClick={() => setExpanded(!expanded())}
      >
        <div class="flex-1">
          <div class="flex items-center gap-small mb-1">
            <span class="text-sm font-medium text-txt">
              {props.group.fileCount} files
            </span>
            <span class="text-xs text-txt-muted">•</span>
            <span class="text-xs text-txt-secondary font-mono">
              {props.group.hash.slice(0, 16)}...
            </span>
          </div>
          <div class="flex items-center gap-4 text-xs">
            <span class="text-txt-secondary">
              Size: {props.dedup.formatBytes(props.group.totalSize / props.group.fileCount)}
            </span>
            <span class="text-warning">
              Wasted: {props.dedup.formatBytes(props.group.wastedSpace)}
            </span>
          </div>
        </div>

        <div class="text-txt-muted">
          {expanded() ? "▼" : "▶"}
        </div>
      </div>

      {/* Expanded File List */}
      <Show when={expanded()}>
        <div class="border-t border-border bg-bg">
          <For each={props.group.files}>
            {(file, index) => (
              <div
                class="flex items-center justify-between p-2 px-3 text-xs border-b border-border last:border-b-0"
                classList={{ "bg-success/5": index() === 0 }}
              >
                <div class="flex-1 font-mono text-txt-secondary truncate">
                  {file.path}
                </div>
                <Show when={index() === 0}>
                  <span class="ml-2 px-2 py-0.5 bg-success/20 text-success rounded text-[10px] font-medium">
                    ORIGINAL
                  </span>
                </Show>
              </div>
            )}
          </For>
        </div>
      </Show>
    </div>
  );
};
