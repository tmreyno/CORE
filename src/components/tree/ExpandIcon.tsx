// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ExpandIcon - Expand/collapse chevron with loading spinner
 * 
 * Pure Tailwind CSS styling with consistent sizing and animations.
 */

import { Show } from 'solid-js';
import { HiOutlineChevronDown, HiOutlineChevronRight } from '../icons';

interface ExpandIconProps {
  /** Whether the node is currently loading */
  isLoading: boolean;
  /** Whether the node is expanded */
  isExpanded: boolean;
  /** Custom class names */
  class?: string;
}

export function ExpandIcon(props: ExpandIconProps) {
  const baseClass = 'w-2.5 h-2.5 text-zinc-500 transition-transform duration-150';
  
  return (
    <Show 
      when={!props.isLoading} 
      fallback={
        <svg 
          class={`${baseClass} animate-spin ${props.class || ''}`}
          viewBox="0 0 24 24" 
          fill="none"
        >
          <circle 
            class="opacity-25" 
            cx="12" 
            cy="12" 
            r="10" 
            stroke="currentColor" 
            stroke-width="3"
          />
          <path 
            class="opacity-75" 
            fill="currentColor" 
            d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
          />
        </svg>
      }
    >
      <Show 
        when={props.isExpanded}
        fallback={
          <HiOutlineChevronRight 
            class={`${baseClass} ${props.class || ''}`}
          />
        }
      >
        <HiOutlineChevronDown 
          class={`${baseClass} ${props.class || ''}`}
        />
      </Show>
    </Show>
  );
}
