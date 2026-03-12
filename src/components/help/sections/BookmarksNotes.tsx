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
          Attach notes to evidence files or entries. Notes support free-form text with
          title, priority levels, and tags. Create notes from the right-click context menu
          on files, or from the Notes tab in the Bookmarks & Notes sidebar panel. Use notes to record 
          observations, analysis results, or context that isn't captured in structured fields.
        </p>
      </div>
    </div>

    <div class="space-y-1">
      <h4 class="font-semibold text-txt text-sm">File-Level Bookmarks & Notes</h4>
      <div class="text-txt-secondary text-sm space-y-1 ml-1">
        <p><strong>1.</strong> Right-click a file in the evidence tree and choose <strong>📑 Bookmark</strong> or <strong>📝 Add Note</strong> from the context menu.</p>
        <p><strong>2.</strong> View and manage all bookmarks and notes in the <strong>Bookmarks & Notes</strong> sidebar panel (bookmark icon in the sidebar).</p>
        <p><strong>3.</strong> Use the <strong>Bookmarks</strong> and <strong>Notes</strong> sub-tabs to switch between the two. Notes include a full editor with title, content, priority, and tags.</p>
        <p><strong>4.</strong> Bookmarks and notes are included in the status bar counts and can be referenced when generating reports.</p>
      </div>
    </div>

    <div class="space-y-1">
      <h4 class="font-semibold text-txt text-sm">Text Selection Actions</h4>
      <p class="text-txt-secondary text-sm ml-1">
        Select text inside any document viewer (PDF, Office, email, text files, etc.), then right-click to access these actions:
      </p>
      <div class="text-txt-secondary text-sm space-y-1 ml-1">
        <p><strong>📑 Bookmark Selection</strong> — Creates a bookmark containing the selected text. The bookmark name shows a preview of the text, and the full selection is stored in the bookmark notes.</p>
        <p><strong>📝 Note from Selection</strong> — Creates a note with the selected text as the note content, automatically titled "Selection from [filename]".</p>
        <p><strong>🔍 Search for Selection</strong> — Opens the search panel pre-filled with the selected text, searching across all evidence containers for matching content.</p>
        <p><strong>📋 Copy</strong> — Copies the selected text to the clipboard.</p>
      </div>
    </div>

    <div class="p-3 bg-info/10 border border-info/20 rounded-lg text-sm">
      <p class="text-info font-medium mb-1">Tips</p>
      <ul class="text-txt-secondary text-xs space-y-0.5 list-disc ml-4">
        <li>Bookmark key evidence items <strong>before</strong> generating a report — the report wizard can reference bookmarked items.</li>
        <li>Notes are timestamped and attributed to the current user — useful for multi-examiner cases.</li>
        <li>Both bookmarks and notes are preserved when merging projects.</li>
        <li>Use <strong>Search for Selection</strong> to quickly find the same text across all documents in your evidence — great for identifying related files.</li>
        <li>When no text is selected, right-clicking in a viewer shows the browser's default context menu.</li>
      </ul>
    </div>
  </div>
);
