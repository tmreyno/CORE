// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";
import { Kbd } from "../../ui/Kbd";

export const EvidenceCollectionContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Evidence Collection is a standalone on-site acquisition form for documenting collected items.
      It is separate from the Report Wizard and opens as a center-pane tab.
    </p>

    <div class="space-y-2">
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Creating a Collection</div>
        <p class="text-xs text-txt-muted mt-1">
          Right-click the Report button in the sidebar → "Evidence Collection…", 
          or use <strong>Tools → Evidence Collection</strong>, 
          or open the command palette (<Kbd keys="Cmd+K" muted />) and search "Evidence Collection".
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Schema-Driven Forms</div>
        <p class="text-xs text-txt-muted mt-1">
          Collection forms are rendered from JSON schema templates. Fields include location, 
          date/time, authorization, collected items, and examiner details.
          Changes are auto-saved to the project database.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Linked Data</div>
        <p class="text-xs text-txt-muted mt-1">
          The right panel shows a linked data tree with relationships between collected items, 
          COC records, and evidence files when a collection tab is active.
        </p>
      </div>
    </div>
  </div>
);
