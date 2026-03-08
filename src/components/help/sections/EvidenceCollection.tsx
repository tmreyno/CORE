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
          Each collection is stored per-project — all collections for the current project appear in the collection list.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Schema-Driven Forms</div>
        <p class="text-xs text-txt-muted mt-1">
          Collection forms are rendered from JSON schema templates. Fields include location, 
          date/time, authorization, collected items, and examiner details.
          Changes are auto-saved to the project database. Custom templates can define additional fields 
          specific to your agency or workflow.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Linked Data</div>
        <p class="text-xs text-txt-muted mt-1">
          The right panel shows a linked data tree with relationships between collected items, 
          COC records, and evidence files when a collection tab is active. This helps you trace 
          which items were collected, how they relate to COC entries, and which evidence files they correspond to.
        </p>
      </div>
    </div>

    <div class="space-y-1">
      <h4 class="font-semibold text-txt text-sm">How to Document a Collection</h4>
      <div class="text-txt-secondary text-sm space-y-1 ml-1">
        <p><strong>1.</strong> Open Evidence Collection via the command palette (<Kbd keys="Cmd+K" muted />) or sidebar context menu.</p>
        <p><strong>2.</strong> Fill in the case number, collection date, location, and authorization details.</p>
        <p><strong>3.</strong> Add each collected item — describe the device, serial number, and condition.</p>
        <p><strong>4.</strong> Assign the collecting officer and any witnesses.</p>
        <p><strong>5.</strong> Save — the form auto-saves, but you can also press <Kbd keys="Cmd+S" muted /> to force a project save.</p>
        <p><strong>6.</strong> Review the linked data tree in the right panel to verify relationships.</p>
      </div>
    </div>

    <div class="p-3 bg-info/10 border border-info/20 rounded-lg text-sm">
      <p class="text-info font-medium mb-1">Collection List</p>
      <p class="text-txt-secondary text-xs">
        Open the collection list via Command Palette → "Evidence Collection List" to browse all collections in the current project.
        Click any collection to open it. Collections can be opened in read-only mode for review without risk of accidental edits.
      </p>
    </div>
  </div>
);
