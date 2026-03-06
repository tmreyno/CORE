// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";
import { Kbd } from "../../ui/Kbd";

export const SearchContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Search across evidence files and their contents. Open the search panel with <Kbd keys="Cmd+F" muted />.
    </p>

    <div class="space-y-2">
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">File Name Search</div>
        <p class="text-xs text-txt-muted mt-1">
          Search by filename across all loaded evidence containers. Supports partial matches and extension filtering.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Full-Text Search</div>
        <p class="text-xs text-txt-muted mt-1">
          Search the project database using SQLite FTS (full-text search) for bookmarks, notes, and file metadata.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Deduplication</div>
        <p class="text-xs text-txt-muted mt-1">
          Identify duplicate files across evidence containers using hash-based comparison.
          Available via <strong>Tools → Deduplication</strong>.
        </p>
      </div>
    </div>
  </div>
);
