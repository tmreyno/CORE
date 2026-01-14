// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, Show } from "solid-js";
import type { DiscoveredFile, ContainerInfo, HashHistoryEntry } from "../types";
import type { FileStatus, FileHashInfo } from "../hooks";
import { formatBytes } from "../utils";
import { FileRow } from "./FileRow";
import { TypeFilterBar } from "./TypeFilterBar";
import {
  HiOutlineFolderOpen,
  HiOutlineMagnifyingGlass,
} from "./icons";

interface FilePanelProps {
  discoveredFiles: DiscoveredFile[];
  filteredFiles: DiscoveredFile[];
  selectedFiles: Set<string>;
  activeFile: DiscoveredFile | null;
  hoveredFile: string | null;
  focusedFileIndex: number;
  typeFilter: string | null;
  containerStats: Record<string, number>;
  totalSize: number;
  fileInfoMap: Map<string, ContainerInfo>;
  fileStatusMap: Map<string, FileStatus>;
  fileHashMap: Map<string, FileHashInfo>;
  hashHistory: Map<string, HashHistoryEntry[]>;
  busy: boolean;
  allFilesSelected: boolean;
  onToggleTypeFilter: (type: string) => void;
  onClearTypeFilter: () => void;
  onToggleSelectAll: () => void;
  onSelectFile: (file: DiscoveredFile) => void;
  onToggleFileSelection: (path: string) => void;
  onHashFile: (file: DiscoveredFile) => void;
  onHover: (path: string | null) => void;
  onFocus: (index: number) => void;
  onKeyDown: (e: KeyboardEvent) => void;
  onContextMenu?: (file: DiscoveredFile, e: MouseEvent) => void;
}

export function FilePanel(props: FilePanelProps) {
  return (
    <aside class="flex flex-col bg-zinc-900/50 border-r border-zinc-700 min-w-[280px] max-w-[400px] w-[320px]" role="region" aria-label="Evidence files panel">
      <div class="flex items-center justify-between px-4 py-2 border-b border-zinc-700/50">
        <h3 class="text-sm font-semibold text-zinc-200 m-0">Evidence Files</h3>
        <Show when={props.discoveredFiles.length > 0}>
          <div class="flex items-center gap-3 text-xs text-zinc-400">
            <span>
              {props.filteredFiles.length}
              {props.typeFilter ? ` of ${props.discoveredFiles.length}` : ""} files
            </span>
            <span>{formatBytes(props.totalSize)}</span>
          </div>
        </Show>
      </div>
      
      {/* Type filter badges - shared component */}
      <TypeFilterBar
        containerStats={props.containerStats}
        totalCount={props.discoveredFiles.length}
        typeFilter={props.typeFilter}
        onToggleTypeFilter={props.onToggleTypeFilter}
        onClearTypeFilter={props.onClearTypeFilter}
        compact={false}
        class="px-3 py-2 border-zinc-700/50"
      />
      
      {/* Select all row */}
      <Show when={props.filteredFiles.length > 0}>
        <div class="row px-3 py-1.5 border-b border-zinc-700/50 bg-zinc-800/30">
          <label class="label-with-icon gap-2">
            <input 
              type="checkbox" 
              class="accent-cyan-500"
              checked={props.allFilesSelected} 
              onChange={props.onToggleSelectAll} 
            />
            <span>
              {props.allFilesSelected ? "Deselect All" : "Select All"}
              {props.typeFilter ? ` (${props.filteredFiles.length} shown)` : ""}
            </span>
          </label>
        </div>
      </Show>
      
      {/* File list */}
      <div 
        class="flex-1 overflow-y-auto focus:outline-none focus-visible:ring-2 focus-visible:ring-cyan-500 focus-visible:ring-inset"
        tabIndex={0} 
        onKeyDown={props.onKeyDown} 
        role="listbox" 
        aria-label="Evidence file list" 
        aria-activedescendant={props.focusedFileIndex >= 0 ? `file-row-${props.focusedFileIndex}` : undefined}
      >
        {/* Empty state - no files */}
        <Show when={props.discoveredFiles.length === 0}>
          <div class="flex flex-col items-center justify-center h-full text-center p-6">
            <HiOutlineFolderOpen class="w-10 h-10 mb-3 opacity-60" />
            <p class="text-sm text-zinc-400 mb-1">Open a directory to scan for evidence files</p>
            <p class="text-xs text-zinc-500">Supports AD1, E01, L01, Raw images</p>
          </div>
        </Show>
        
        {/* Empty state - filter has no results */}
        <Show when={props.discoveredFiles.length > 0 && props.filteredFiles.length === 0}>
          <div class="flex flex-col items-center justify-center h-full text-center p-6">
            <HiOutlineMagnifyingGlass class="w-10 h-10 mb-3 opacity-60" />
            <p class="text-sm text-zinc-400 mb-3">No {props.typeFilter} files found</p>
            <button 
              class="px-3 py-1.5 text-xs bg-zinc-700 text-zinc-200 rounded hover:bg-zinc-600 transition-colors"
              onClick={props.onClearTypeFilter}
            >
              Show all files
            </button>
          </div>
        </Show>
        
        {/* File rows */}
        <For each={props.filteredFiles}>
          {(file, index) => (
            <FileRow
              file={file}
              index={index()}
              isSelected={props.selectedFiles.has(file.path)}
              isActive={props.activeFile?.path === file.path}
              isFocused={props.focusedFileIndex === index()}
              isHovered={props.hoveredFile === file.path}
              fileStatus={props.fileStatusMap.get(file.path)}
              fileInfo={props.fileInfoMap.get(file.path)}
              fileHash={props.fileHashMap.get(file.path)}
              hashHistory={props.hashHistory.get(file.path) ?? []}
              busy={props.busy}
              onSelect={() => props.onSelectFile(file)}
              onToggleSelection={() => props.onToggleFileSelection(file.path)}
              onHash={() => props.onHashFile(file)}
              onMouseEnter={() => { props.onHover(file.path); props.onFocus(index()); }}
              onMouseLeave={() => props.onHover(null)}
              onContextMenu={(e: MouseEvent) => props.onContextMenu?.(file, e)}
            />
          )}
        </For>
      </div>
    </aside>
  );
}
