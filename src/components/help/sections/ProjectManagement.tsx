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
      A companion database (<code class="text-accent">.ffxdb</code>) provides SQL-backed persistence.
    </p>

    <div class="space-y-2">
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Auto-Save</div>
        <p class="text-xs text-txt-muted mt-1">
          Enable auto-save via <strong>File → Toggle Auto-Save</strong> or the save dropdown in the toolbar.
          When enabled, the project is saved automatically after changes.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Workspace Profiles</div>
        <p class="text-xs text-txt-muted mt-1">
          Configure reusable workspace profiles during project setup to preset evidence paths,
          export locations, and processed database directories.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Recovery</div>
        <p class="text-xs text-txt-muted mt-1">
          CORE-FFX supports project backup, versioning, and crash recovery.
          Use the project recovery check on startup to restore from auto-saved state.
        </p>
      </div>
    </div>
  </div>
);
