// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";
import {
  HiOutlineBookmark,
  HiOutlineDocumentText,
} from "../../icons";

export const BookmarksNotesContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Bookmarks and notes help you organize findings during analysis. Both are saved 
      to the project database and persisted across sessions.
    </p>

    <div class="space-y-2">
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm flex items-center gap-2">
          <HiOutlineBookmark class="w-4 h-4 text-accent" /> Bookmarks
        </div>
        <p class="text-xs text-txt-muted mt-1">
          Mark files or entries of interest for quick access later. 
          Bookmarks appear in the sidebar Bookmarks panel and are searchable.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm flex items-center gap-2">
          <HiOutlineDocumentText class="w-4 h-4 text-accent" /> Notes
        </div>
        <p class="text-xs text-txt-muted mt-1">
          Attach notes to evidence files or entries. Notes support free-form text 
          and are displayed in the right panel alongside file metadata.
        </p>
      </div>
    </div>
  </div>
);
