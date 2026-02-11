// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ProcessedDatabaseToolbar - Toolbar for database panel
 */

import { Component, Show } from 'solid-js';
import {
  HiOutlineCircleStack,
  HiOutlineMagnifyingGlass,
  HiOutlinePlus,
  HiOutlineTrash,
} from '../icons';

interface ProcessedDatabaseToolbarProps {
  databaseCount: number;
  loading: boolean;
  onScan: () => void;
  onAdd: () => void;
  onClearAll: () => void;
}

export const ProcessedDatabaseToolbar: Component<ProcessedDatabaseToolbarProps> = (props) => {
  return (
    <div class="flex items-center justify-between px-2 py-1 bg-bg-card border-b border-border shrink-0">
      <h3 class={`m-0 text-[11px] leading-tight font-semibold text-txt flex items-center gap-1`}>
        <HiOutlineCircleStack class="w-3 h-3" /> Processed Databases
      </h3>
      <div class={`flex gap-1`}>
        <button 
          class={`bg-transparent border border-transparent rounded px-1 py-0.5 cursor-pointer text-[11px] leading-tight transition-all duration-150 hover:bg-bg-hover hover:border-border disabled:opacity-50 disabled:cursor-not-allowed flex items-center`}
          onClick={props.onScan} 
          disabled={props.loading}
          title="Scan folder for databases"
        >
          <HiOutlineMagnifyingGlass class="w-3 h-3" />
        </button>
        <button 
          class={`bg-transparent border border-transparent rounded px-1 py-0.5 cursor-pointer text-[11px] leading-tight transition-all duration-150 hover:bg-bg-hover hover:border-border disabled:opacity-50 disabled:cursor-not-allowed flex items-center`}
          onClick={props.onAdd} 
          disabled={props.loading}
          title="Add database file"
        >
          <HiOutlinePlus class="w-3 h-3" />
        </button>
        <Show when={props.databaseCount > 0}>
          <button 
            class={`bg-transparent border border-transparent rounded px-1 py-0.5 cursor-pointer text-[11px] leading-tight transition-all duration-150 hover:bg-bg-hover hover:border-border flex items-center`}
            onClick={props.onClearAll}
            title="Clear all"
          >
            <HiOutlineTrash class="w-3 h-3" />
          </button>
        </Show>
      </div>
    </div>
  );
};
