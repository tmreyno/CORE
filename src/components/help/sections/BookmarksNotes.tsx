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
          Use bookmarks to flag evidence items for follow-up, inclusion in reports, or peer review.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm flex items-center gap-2">
          <HiOutlineDocumentText class="w-4 h-4 text-accent" /> Notes
        </div>
        <p class="text-xs text-txt-muted mt-1">
          Attach notes to evidence files or entries. Notes support free-form text 
          and are displayed in the right panel alongside file metadata. Use notes to record 
          observations, analysis results, or context that isn't captured in structured fields.
        </p>
      </div>
    </div>

    <div class="space-y-1">
      <h4 class="font-semibold text-txt text-sm">How to Use Bookmarks & Notes</h4>
      <div class="text-txt-secondary text-sm space-y-1 ml-1">
        <p><strong>1.</strong> Select a file or entry in the evidence tree.</p>
        <p><strong>2.</strong> In the right panel, click the <strong>Bookmark</strong> icon to flag the item, or use the <strong>Notes</strong> tab to write an annotation.</p>
        <p><strong>3.</strong> View all bookmarks via <strong>View → Bookmarks</strong> in the sidebar.</p>
        <p><strong>4.</strong> Bookmarks and notes are included in the status bar counts and can be referenced when generating reports.</p>
      </div>
    </div>

    <div class="p-3 bg-info/10 border border-info/20 rounded-lg text-sm">
      <p class="text-info font-medium mb-1">Tips</p>
      <ul class="text-txt-secondary text-xs space-y-0.5 list-disc ml-4">
        <li>Bookmark key evidence items <strong>before</strong> generating a report — the report wizard can reference bookmarked items.</li>
        <li>Notes are timestamped and attributed to the current user — useful for multi-examiner cases.</li>
        <li>Both bookmarks and notes are preserved when merging projects.</li>
      </ul>
    </div>
  </div>
);
