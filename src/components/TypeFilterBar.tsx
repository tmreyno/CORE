// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * TypeFilterBar - Shared type filter component for evidence panels
 * 
 * Used by both EvidenceTree and FilePanel to display container type filter badges.
 * Provides consistent UI for filtering evidence by container type (AD1, E01, ZIP, etc.)
 */

import { For, Show, JSX } from "solid-js";
import { HiOutlineXMark } from "./icons";
import { getContainerTypeIcon } from "./tree";

export interface TypeFilterBarProps {
  /** Container type statistics - { "AD1": 5, "E01": 3, ... } */
  containerStats: Record<string, number>;
  /** Total number of discovered files (for "All" count) */
  totalCount: number;
  /** Currently active type filter (null = show all) */
  typeFilter: string | null;
  /** Callback when a type filter is toggled */
  onToggleTypeFilter: (type: string) => void;
  /** Callback to clear the type filter */
  onClearTypeFilter: () => void;
  /** Optional className for the container */
  class?: string;
  /** Whether to show compact badges (smaller text) */
  compact?: boolean;
}

/**
 * Type filter bar component with badge-style buttons
 */
export function TypeFilterBar(props: TypeFilterBarProps): JSX.Element {
  const isCompact = () => props.compact ?? true;
  
  // Base classes for buttons
  const buttonBaseClass = () => isCompact()
    ? "flex items-center gap-0.5 px-1 py-0.5 text-[10px] rounded transition-colors"
    : "flex items-center gap-1 px-1.5 py-0.5 text-xs rounded transition-colors";
  
  const iconClass = () => isCompact() ? "w-[10px] h-[10px]" : "w-3 h-3";
  
  return (
    <Show when={Object.keys(props.containerStats).length > 0}>
      <div class={`flex flex-wrap items-center gap-0.5 px-1.5 py-0.5 border-b border-zinc-700 bg-zinc-800/50 ${props.class ?? ''}`}>
        {/* All button */}
        <button
          class={`${buttonBaseClass()} ${!props.typeFilter ? 'bg-cyan-600 text-white' : 'text-zinc-400 hover:bg-zinc-700 hover:text-zinc-200'}`}
          onClick={props.onClearTypeFilter}
          title="Show all containers"
        >
          <span>All:</span>
          <span>{props.totalCount}</span>
        </button>
        
        {/* Type filter badges */}
        <For each={Object.entries(props.containerStats)}>
          {([type, count]) => {
            const IconComponent = getContainerTypeIcon(type);
            const isActive = props.typeFilter === type;
            return (
              <button
                class={`${buttonBaseClass()} ${isActive ? 'bg-cyan-600 text-white' : 'text-zinc-400 hover:bg-zinc-700 hover:text-zinc-200'}`}
                onClick={() => props.onToggleTypeFilter(type)}
                title={`Filter by ${type} (${count} files)${isActive ? ' - click to clear' : ''}`}
              >
                <IconComponent class={iconClass()} />
                <span>{type}:</span>
                <span>{count}</span>
                <Show when={isActive}>
                  <HiOutlineXMark class={`${iconClass()} ml-0.5 opacity-70 hover:opacity-100`} />
                </Show>
              </button>
            );
          }}
        </For>
      </div>
    </Show>
  );
}

export default TypeFilterBar;
