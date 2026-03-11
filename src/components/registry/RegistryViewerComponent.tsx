// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * RegistryViewerComponent — slim render shell for the registry hive viewer.
 * All state and logic live in useRegistryData hook.
 */

import { Show, For } from "solid-js";
import { HiOutlineExclamationTriangle } from "../icons";
import { SearchIcon, ChevronDownIcon, ChevronRightIcon } from "../icons";
import { useRegistryData } from "./useRegistryData";
import { getDataTypeColor, formatSize } from "./helpers";
import type { RegistryViewerProps } from "./types";

export function RegistryViewer(props: RegistryViewerProps) {
  const reg = useRegistryData({
    path: () => props.path,
    onMetadata: props.onMetadata,
  });

  return (
    <div class={`flex flex-col h-full bg-bg ${props.class || ""}`}>
      {/* Loading */}
      <Show when={reg.loading()}>
        <div class="flex flex-col items-center justify-center h-full gap-4">
          <div class="relative w-12 h-12">
            <div class="absolute inset-0 rounded-full border-2 border-border opacity-30" />
            <div class="absolute inset-0 rounded-full border-2 border-accent border-t-transparent animate-spin" />
          </div>
          <p class="text-txt-secondary text-sm">Loading registry hive...</p>
        </div>
      </Show>

      {/* Error */}
      <Show when={reg.error()}>
        <div class="m-4 p-4 rounded-lg bg-error/10 border border-error/30">
          <div class="flex items-start gap-3">
            <HiOutlineExclamationTriangle class="w-5 h-5 text-error shrink-0 mt-0.5" />
            <div>
              <p class="text-error font-medium text-sm">Failed to load registry hive</p>
              <p class="text-txt-muted text-xs mt-1">{reg.error()}</p>
            </div>
          </div>
        </div>
      </Show>

      {/* Main Content */}
      <Show when={!reg.loading() && !reg.error() && reg.hiveInfo()}>
        {/* Header bar */}
        <div class="panel-header gap-3 border-b border-border">
          <span class="badge badge-warning text-xs">Registry</span>
          <span class="text-sm text-txt font-medium truncate">
            {reg.hiveInfo()!.rootKeyName}
          </span>
          <span class="text-xs text-txt-muted">
            {reg.hiveInfo()!.totalKeys.toLocaleString()} keys ·{" "}
            {reg.hiveInfo()!.totalValues.toLocaleString()} values
          </span>
          <Show when={reg.hiveInfo()!.rootTimestamp}>
            <span class="text-xs text-txt-muted ml-auto">
              {reg.hiveInfo()!.rootTimestamp}
            </span>
          </Show>
        </div>

        {/* Split: Tree + Values */}
        <div class="flex flex-1 min-h-0">
          {/* Left: Tree Navigation */}
          <div class="w-64 border-r border-border flex flex-col">
            <div class="p-2 text-xs text-txt-muted font-medium uppercase tracking-wider border-b border-border">
              Registry Keys
            </div>
            <div class="flex-1 overflow-y-auto p-1">
              <For each={reg.flatTree()}>
                {({ node, depth }) => (
                  <div
                    class={`flex items-center gap-1 px-1 py-0.5 rounded cursor-pointer text-xs hover:bg-bg-hover ${
                      reg.selectedPath() === node.key.path
                        ? "bg-accent/20 text-accent"
                        : "text-txt"
                    }`}
                    style={{ "padding-left": `${depth * 16 + 4}px` }}
                    onClick={() => reg.selectKey(node)}
                  >
                    <button
                      class="w-4 h-4 flex items-center justify-center shrink-0"
                      onClick={(e) => {
                        e.stopPropagation();
                        reg.toggleNode(node);
                      }}
                    >
                      <Show when={node.key.hasSubkeys}>
                        <Show
                          when={node.expanded}
                          fallback={<ChevronRightIcon class="w-3 h-3" />}
                        >
                          <ChevronDownIcon class="w-3 h-3" />
                        </Show>
                      </Show>
                    </button>
                    <svg
                      class="w-3.5 h-3.5 shrink-0 text-yellow-500"
                      fill="none"
                      viewBox="0 0 24 24"
                      stroke="currentColor"
                      stroke-width="2"
                    >
                      <path
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        d="M3 7v10a2 2 0 002 2h14a2 2 0 002-2V9a2 2 0 00-2-2h-6l-2-2H5a2 2 0 00-2 2z"
                      />
                    </svg>
                    <span class="truncate" title={node.key.path}>
                      {node.key.name}
                    </span>
                    <Show when={node.key.subkeyCount > 0}>
                      <span class="text-2xs text-txt-muted ml-auto">
                        {node.key.subkeyCount}
                      </span>
                    </Show>
                  </div>
                )}
              </For>
            </div>
          </div>

          {/* Right: Values Panel */}
          <div class="flex-1 flex flex-col min-w-0">
            <Show when={reg.selectedKeyInfo()}>
              <div class="px-3 py-2 border-b border-border bg-bg-secondary">
                <div class="flex items-center gap-2">
                  <span class="text-sm text-txt font-medium truncate">
                    {reg.selectedKeyInfo()!.prettyPath}
                  </span>
                  <Show when={reg.selectedKeyInfo()!.timestamp}>
                    <span class="text-xs text-txt-muted ml-auto shrink-0">
                      {reg.selectedKeyInfo()!.timestamp}
                    </span>
                  </Show>
                </div>
                <div class="flex items-center gap-3 mt-1">
                  <span class="text-xs text-txt-muted">
                    {reg.selectedKeyInfo()!.subkeyCount} subkeys
                  </span>
                  <span class="text-xs text-txt-muted">
                    {reg.selectedKeyInfo()!.valueCount} values
                  </span>
                </div>
              </div>

              {/* Search bar */}
              <div class="px-3 py-2 border-b border-border">
                <div class="relative">
                  <SearchIcon class="absolute left-2 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-txt-muted" />
                  <input
                    class="input-sm w-full pl-7"
                    placeholder="Filter values..."
                    value={reg.searchQuery()}
                    onInput={(e) => reg.setSearchQuery(e.currentTarget.value)}
                  />
                </div>
              </div>

              {/* Values table */}
              <div class="flex-1 overflow-auto">
                <Show when={reg.valuesLoading()}>
                  <div class="flex items-center justify-center py-8">
                    <div class="w-6 h-6 rounded-full border-2 border-accent border-t-transparent animate-spin" />
                  </div>
                </Show>
                <Show when={!reg.valuesLoading()}>
                  <table class="w-full text-xs">
                    <thead class="sticky top-0 bg-bg-panel z-10">
                      <tr class="text-left text-txt-muted border-b border-border">
                        <th class="px-3 py-1.5 font-medium">Name</th>
                        <th class="px-3 py-1.5 font-medium w-32">Type</th>
                        <th class="px-3 py-1.5 font-medium">Data</th>
                        <th class="px-3 py-1.5 font-medium w-16 text-right">Size</th>
                      </tr>
                    </thead>
                    <tbody>
                      <For
                        each={reg.filteredValues()}
                        fallback={
                          <tr>
                            <td
                              colspan="4"
                              class="px-3 py-4 text-center text-txt-muted italic"
                            >
                              {reg.searchQuery() ? "No matching values" : "No values"}
                            </td>
                          </tr>
                        }
                      >
                        {(val) => (
                          <tr class="border-b border-border/50 hover:bg-bg-hover">
                            <td
                              class="px-3 py-1.5 text-txt font-mono truncate max-w-[200px]"
                              title={val.name}
                            >
                              {val.name}
                            </td>
                            <td
                              class={`px-3 py-1.5 font-mono ${getDataTypeColor(val.dataType)}`}
                            >
                              {val.dataType}
                            </td>
                            <td
                              class="px-3 py-1.5 text-txt-secondary font-mono truncate max-w-[400px]"
                              title={val.data}
                            >
                              {val.data}
                            </td>
                            <td class="px-3 py-1.5 text-txt-muted text-right">
                              {formatSize(val.size)}
                            </td>
                          </tr>
                        )}
                      </For>
                    </tbody>
                  </table>
                </Show>
              </div>
            </Show>

            <Show when={!reg.selectedKeyInfo() && !reg.valuesLoading()}>
              <div class="flex items-center justify-center h-full text-txt-muted text-sm">
                Select a key from the tree to view its values
              </div>
            </Show>
          </div>
        </div>
      </Show>
    </div>
  );
}
