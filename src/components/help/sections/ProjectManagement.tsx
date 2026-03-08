// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";

export const ProjectManagementContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Projects (<code class="text-accent">.cffx</code>) store all case data — evidence paths, bookmarks,
      notes, hash results, chain of custody records, activity logs, and settings.
      A companion database (<code class="text-accent">.ffxdb</code>) provides SQL-backed persistence
      for high-volume data like bookmarks, search history, and export records.
    </p>

    <div class="space-y-2">
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Auto-Save</div>
        <p class="text-xs text-txt-muted mt-1">
          Enable auto-save via <strong>File → Toggle Auto-Save</strong> or the save dropdown in the toolbar.
          When enabled, the project is saved automatically after changes. The status bar shows 
          auto-save status — look for the "Auto-save ON" indicator.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Workspace Profiles</div>
        <p class="text-xs text-txt-muted mt-1">
          Configure reusable workspace profiles during project setup to preset evidence paths,
          export locations, and processed database directories. Profiles save time when you 
          frequently work with similar directory layouts (e.g., agency standard folder structures).
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Recovery</div>
        <p class="text-xs text-txt-muted mt-1">
          CORE-FFX supports project backup, versioning, and crash recovery.
          Use the project recovery check on startup to restore from auto-saved state.
          Manual backups can be created via the project menu.
        </p>
      </div>
    </div>

    <div class="space-y-1">
      <h4 class="font-semibold text-txt text-sm">Project File Structure</h4>
      <div class="text-txt-secondary text-xs space-y-0.5 ml-1 font-mono">
        <p>MyCase/</p>
        <p class="ml-4">MyCase.cffx <span class="text-txt-muted font-sans">— project file (JSON)</span></p>
        <p class="ml-4">MyCase.ffxdb <span class="text-txt-muted font-sans">— project database (SQLite + WAL)</span></p>
        <p class="ml-4">1.Evidence/ <span class="text-txt-muted font-sans">— evidence containers</span></p>
        <p class="ml-4">2.Processed.Database/ <span class="text-txt-muted font-sans">— third-party tool output</span></p>
        <p class="ml-4">4.Case.Documents/ <span class="text-txt-muted font-sans">— warrants, reports, notes</span></p>
      </div>
    </div>

    <div class="space-y-1">
      <h4 class="font-semibold text-txt text-sm">How to Set Up a New Project</h4>
      <div class="text-txt-secondary text-sm space-y-1 ml-1">
        <p><strong>1.</strong> <strong>File → New Project</strong> or use the Welcome Screen.</p>
        <p><strong>2.</strong> Choose a project name and location. Optionally select a workspace profile.</p>
        <p><strong>3.</strong> Configure evidence, export, and processed database paths.</p>
        <p><strong>4.</strong> Click <strong>Create</strong> — CORE-FFX creates the <code class="text-accent">.cffx</code> and <code class="text-accent">.ffxdb</code> files and begins scanning for evidence.</p>
      </div>
    </div>

    <div class="space-y-1">
      <h4 class="font-semibold text-txt text-sm">Merge Projects</h4>
      <p class="text-txt-secondary text-sm ml-1">
        Combine multiple projects via <strong>Tools → Merge Projects</strong>. The wizard identifies examiners 
        from both project files and databases, deduplicates evidence, and merges COC records, 
        bookmarks, notes, and form submissions into a single project.
      </p>
    </div>

    <div class="p-3 bg-info/10 border border-info/20 rounded-lg text-sm">
      <p class="text-info font-medium mb-1">WAL Database</p>
      <p class="text-txt-secondary text-xs">
        The <code class="text-accent">.ffxdb</code> uses SQLite WAL (Write-Ahead Logging) for performance. 
        Always save your project (<strong>Cmd+S</strong>) before copying or moving the project folder — this flushes 
        pending writes from the WAL file into the main database. If you see <code class="text-accent">.ffxdb-wal</code> and 
        <code class="text-accent">.ffxdb-shm</code> files, that's normal — they are automatically cleaned up on project close.
      </p>
    </div>
  </div>
);
