// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";
import { Kbd } from "../../ui/Kbd";

export const GettingStartedContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      CORE-FFX is a forensic file explorer for analyzing digital evidence containers.
      It provides read-only access to forensic images with hash verification, 
      file preview, and chain of custody tracking.
    </p>

    <div class="space-y-3">
      <h4 class="font-semibold text-txt flex items-center gap-2">
        <span class="text-accent">1.</span> Create or Open a Project
      </h4>
      <p class="text-txt-secondary text-sm ml-5">
        Use <Kbd keys="Cmd+Shift+N" muted /> to create a new project or <Kbd keys="Cmd+O" muted /> to open an existing <code class="text-accent">.cffx</code> project file.
        Projects organize your case work — evidence paths, bookmarks, notes, hash results, and chain of custody records are all saved within the project.
      </p>

      <h4 class="font-semibold text-txt flex items-center gap-2">
        <span class="text-accent">2.</span> Point to Evidence
      </h4>
      <p class="text-txt-secondary text-sm ml-5">
        During project setup, specify your evidence directory. CORE-FFX will recursively scan for supported forensic containers (E01, AD1, L01, UFED, archives, raw images, and more).
      </p>

      <h4 class="font-semibold text-txt flex items-center gap-2">
        <span class="text-accent">3.</span> Browse and Analyze
      </h4>
      <p class="text-txt-secondary text-sm ml-5">
        Click any evidence file in the left panel to expand its contents in the evidence tree.
        Select files to view them in hex, text, or document preview mode. The right panel shows metadata, EXIF data, and hash information.
      </p>

      <h4 class="font-semibold text-txt flex items-center gap-2">
        <span class="text-accent">4.</span> Verify and Document
      </h4>
      <p class="text-txt-secondary text-sm ml-5">
        Compute hashes to verify evidence integrity. Use chain of custody records for audit trails.
        Generate reports and export evidence in forensically sound formats (E01, L01, 7z).
      </p>
    </div>
  </div>
);
