// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For } from "solid-js";
import { useIndexCache } from "../hooks/useIndexCache";

export const IndexCacheManager: Component = () => {
  const indexCache = useIndexCache();

  const handleClearCache = async () => {
    if (confirm("Are you sure you want to clear the entire index cache?")) {
      await indexCache.clearCache();
    }
  };

  return (
    <div class="flex flex-col gap-base p-4 bg-bg-panel rounded-lg border border-border">
      <div class="flex items-center justify-between">
        <h3 class="text-lg font-semibold text-txt">Index Cache</h3>
        <button
          onClick={indexCache.refreshStats}
          class="px-3 py-1 text-sm hover:bg-bg-hover rounded-md transition-colors text-txt-secondary"
        >
          Refresh
        </button>
      </div>

      <Show when={indexCache.initialized()} fallback={<div class="text-txt-muted">Initializing...</div>}>
        <Show when={indexCache.stats()}>
          {(stats) => (
            <div class="grid grid-cols-2 gap-4">
              <div class="bg-bg rounded-md p-3 border border-border">
                <div class="text-txt-muted text-sm">Containers</div>
                <div class="text-2xl font-semibold text-txt">{stats().totalContainers}</div>
              </div>
              <div class="bg-bg rounded-md p-3 border border-border">
                <div class="text-txt-muted text-sm">Total Entries</div>
                <div class="text-2xl font-semibold text-txt">{stats().totalEntries.toLocaleString()}</div>
              </div>
              <div class="bg-bg rounded-md p-3 border border-border col-span-2">
                <div class="text-txt-muted text-sm">Cache Size</div>
                <div class="text-2xl font-semibold text-txt">{indexCache.formattedCacheSize()}</div>
              </div>
            </div>
          )}
        </Show>

        <div class="border-t border-border pt-4">
          <h4 class="text-sm font-medium text-txt mb-2">Active Indexing</h4>
          <Show
            when={indexCache.activeWorkers().length > 0}
            fallback={<div class="text-txt-muted text-sm">No active indexing operations</div>}
          >
            <div class="space-y-2">
              <For each={indexCache.activeWorkers()}>
                {(worker) => {
                  const progress = indexCache.getProgress(worker.containerPath);
                  return (
                    <div class="bg-bg rounded-md p-3 border border-border">
                      <div class="flex items-center justify-between mb-2">
                        <span class="text-sm text-txt truncate max-w-xs">
                          {worker.containerPath.split("/").pop()}
                        </span>
                        <button
                          onClick={() => indexCache.cancelIndexing(worker.containerPath)}
                          class="px-2 py-1 text-xs hover:bg-bg-hover rounded-md transition-colors text-error"
                        >
                          Cancel
                        </button>
                      </div>
                      <Show when={progress}>
                        {(prog) => (
                          <>
                            <div class="w-full bg-bg-secondary rounded-full h-2 mb-1">
                              <div
                                class="bg-accent h-2 rounded-full transition-all duration-300"
                                style={{ width: `${prog().percent}%` }}
                              />
                            </div>
                            <div class="text-xs text-txt-muted">
                              {prog().status} ({prog().current}/{prog().total})
                            </div>
                          </>
                        )}
                      </Show>
                    </div>
                  );
                }}
              </For>
            </div>
          </Show>
        </div>

        <div class="border-t border-border pt-4">
          <h4 class="text-sm font-medium text-txt mb-2">Actions</h4>
          <button
            onClick={handleClearCache}
            class="px-3 py-2 bg-error/10 hover:bg-error/20 text-error rounded-md transition-colors"
          >
            Clear Cache
          </button>
        </div>
      </Show>
    </div>
  );
};
