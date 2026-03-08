// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";

export const ProcessedDatabasesContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      CORE-FFX can read case data from third-party forensic tool databases, enabling 
      cross-tool analysis without leaving the application. All databases are opened <strong>read-only</strong> — 
      original tool data is never modified.
    </p>

    <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50 text-center">
        <div class="font-medium text-txt text-sm">Magnet AXIOM</div>
        <p class="text-xs text-txt-muted mt-1">
          Case directories with artifact categories and evidence sources.
          Reads case info, examiner details, search results, and artifact data.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50 text-center">
        <div class="font-medium text-txt text-sm">Cellebrite PA</div>
        <p class="text-xs text-txt-muted mt-1">
          report.xml + SQLite databases with data sources and artifacts.
          Parses extraction reports, device info, and categorized artifacts.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50 text-center">
        <div class="font-medium text-txt text-sm">Autopsy</div>
        <p class="text-xs text-txt-muted mt-1">
          .aut files + autopsy.db with data sources, artifacts, and tags.
          Reads case metadata, ingest module results, and user-assigned tags.
        </p>
      </div>
    </div>

    <div class="space-y-1">
      <h4 class="font-semibold text-txt text-sm">How to Import Processed Databases</h4>
      <div class="text-txt-secondary text-sm space-y-1 ml-1">
        <p><strong>1.</strong> Set your processed database path in the toolbar <strong>Location Selector</strong> dropdown.</p>
        <p><strong>2.</strong> Navigate to <strong>View → Processed Databases</strong> from the menu bar, or use the sidebar.</p>
        <p><strong>3.</strong> CORE-FFX auto-detects the tool type (AXIOM, Cellebrite, or Autopsy) from file signatures.</p>
        <p><strong>4.</strong> Browse artifact categories, evidence sources, and case metadata within the Processed DB panel.</p>
      </div>
    </div>

    <div class="p-3 bg-info/10 border border-info/20 rounded-lg text-sm">
      <p class="text-info font-medium mb-1">Cross-Tool Correlation</p>
      <p class="text-txt-secondary text-xs">
        When working with multiple tool outputs (e.g., AXIOM for computer analysis and Cellebrite for mobile), 
        CORE-FFX lets you view both in the same project. Use search and deduplication to find matching artifacts 
        across tools.
      </p>
    </div>
  </div>
);
