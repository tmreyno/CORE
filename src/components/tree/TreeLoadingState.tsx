// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * TreeLoadingState - Loading indicator for tree views
 */

export function TreeLoadingState(props: { depth?: number; message?: string }) {
  const paddingLeft = () => props.depth ? `${(props.depth + 1) * 16}px` : '32px';
  
  return (
    <div 
      class="flex items-center gap-2 py-2 text-zinc-500"
      style={{ 'padding-left': paddingLeft() }}
    >
      <svg 
        class="w-4 h-4 animate-spin" 
        viewBox="0 0 24 24" 
        fill="none"
      >
        <circle 
          class="opacity-25" 
          cx="12" cy="12" r="10" 
          stroke="currentColor" 
          stroke-width="3"
        />
        <path 
          class="opacity-75" 
          fill="currentColor" 
          d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"
        />
      </svg>
      <span class="text-sm">{props.message || 'Loading...'}</span>
    </div>
  );
}
