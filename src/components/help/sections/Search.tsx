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
          Click any result to navigate directly to that file in the evidence tree and open it in the center pane.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Full-Text Search</div>
        <p class="text-xs text-txt-muted mt-1">
          Search the project database using SQLite FTS (full-text search) for bookmarks, notes, and file metadata.
          Recent searches are saved and can be re-run quickly.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Deduplication</div>
        <p class="text-xs text-txt-muted mt-1">
          Identify duplicate files across evidence containers using hash-based comparison.
          Available via <strong>Tools → Deduplication</strong>.
          Especially useful for proving file movement between devices (e.g., laptop → USB drive).
        </p>
      </div>
    </div>

    <div class="space-y-3">
      <h4 class="font-semibold text-txt text-sm">How to Use Search</h4>
      <div class="text-txt-secondary text-sm space-y-1 ml-1">
        <p><strong>1.</strong> Press <Kbd keys="Cmd+F" muted /> to open the Search panel.</p>
        <p><strong>2.</strong> Type your search query — matching files appear in real time as you type.</p>
        <p><strong>3.</strong> Click any result to navigate to the file in the evidence tree and open it for viewing.</p>
        <p><strong>4.</strong> For project-wide metadata search (bookmarks, notes), switch to the <strong>Full-Text Search</strong> tab.</p>
      </div>

      <h4 class="font-semibold text-txt text-sm">How to Use Deduplication</h4>
      <div class="text-txt-secondary text-sm space-y-1 ml-1">
        <p><strong>1.</strong> First compute hashes for all evidence files (see Hash Verification section).</p>
        <p><strong>2.</strong> Open <strong>Tools → Deduplication</strong> from the sidebar (overlapping pages icon).</p>
        <p><strong>3.</strong> Results show groups of identical files across containers — each group shares the same hash.</p>
        <p><strong>4.</strong> Examine the locations to determine how data moved between devices or storage media.</p>
      </div>
    </div>

    <div class="p-3 bg-info/10 border border-info/20 rounded-lg text-sm">
      <p class="text-info font-medium mb-1">Forensic Use Case: Cross-Device File Tracking</p>
      <p class="text-txt-secondary text-xs">
        Deduplication is a powerful tool for insider threat and data theft investigations. By matching file hashes 
        across a suspect's laptop, USB drives, cloud backups, and email attachments, you can definitively prove 
        which files were copied to which devices — and the hash match makes the evidence mathematically certain.
      </p>
    </div>
  </div>
);
