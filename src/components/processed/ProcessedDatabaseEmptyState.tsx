// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ProcessedDatabaseEmptyState - Empty state display for database panel
 */

import { Component } from 'solid-js';

export const ProcessedDatabaseEmptyState: Component = () => {
  return (
    <div class={`flex flex-col items-center justify-center px-2 py-4 text-center text-txt-muted flex-1 text-[11px] leading-tight`}>
      <p class="my-0.5">No processed databases loaded</p>
      <p class="text-txt-faint">Click scan to find databases or add files</p>
      <p class="text-txt-faint">Supports: AXIOM, Cellebrite PA, X-Ways, Autopsy, EnCase, FTK</p>
    </div>
  );
};
