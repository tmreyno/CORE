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
      cross-tool analysis without leaving the application.
    </p>

    <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50 text-center">
        <div class="font-medium text-txt text-sm">Magnet AXIOM</div>
        <p class="text-xs text-txt-muted mt-1">Case directories with artifact categories and evidence sources</p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50 text-center">
        <div class="font-medium text-txt text-sm">Cellebrite PA</div>
        <p class="text-xs text-txt-muted mt-1">report.xml + SQLite databases with data sources and artifacts</p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50 text-center">
        <div class="font-medium text-txt text-sm">Autopsy</div>
        <p class="text-xs text-txt-muted mt-1">.aut files + autopsy.db with data sources, artifacts, and tags</p>
      </div>
    </div>
  </div>
);
