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
      <div class="text-txt-secondary text-sm ml-5 space-y-1">
        <p>
          Use <Kbd keys="Cmd+Shift+N" muted /> to create a new project or <Kbd keys="Cmd+O" muted /> to open an existing <code class="text-accent">.cffx</code> project file.
          Projects organize your case work — evidence paths, bookmarks, notes, hash results, and chain of custody records are all saved within the project.
        </p>
        <p class="text-xs text-txt-muted">
          Each project creates two files: a <code class="text-accent">.cffx</code> (JSON state) and a <code class="text-accent">.ffxdb</code> (SQLite database).
          Copy both files to transfer or archive a project.
        </p>
      </div>

      <h4 class="font-semibold text-txt flex items-center gap-2">
        <span class="text-accent">2.</span> Point to Evidence
      </h4>
      <div class="text-txt-secondary text-sm ml-5 space-y-1">
        <p>
          During project setup, specify your evidence directory. CORE-FFX will recursively scan for supported forensic containers (E01, AD1, L01, UFED, archives, raw images, and more).
        </p>
        <p class="text-xs text-txt-muted">
          You can also set a Processed Database path (for AXIOM/Cellebrite/Autopsy output) and a Case Documents path during setup.
          These paths appear in the toolbar dropdown for quick switching.
        </p>
      </div>

      <h4 class="font-semibold text-txt flex items-center gap-2">
        <span class="text-accent">3.</span> Browse and Analyze
      </h4>
      <div class="text-txt-secondary text-sm ml-5 space-y-1">
        <p>
          Click any evidence file in the left panel to expand its contents in the evidence tree.
          Select files to view them in hex, text, or document preview mode. The right panel shows metadata, EXIF data, and hash information.
        </p>
        <p class="text-xs text-txt-muted">
          CORE-FFX auto-detects file types using extension matching and magic-byte analysis (file signature detection).
          12 specialized viewers handle PDFs, images, Office documents, spreadsheets, email, PST archives, databases, registries, and more.
        </p>
      </div>

      <h4 class="font-semibold text-txt flex items-center gap-2">
        <span class="text-accent">4.</span> Verify and Document
      </h4>
      <div class="text-txt-secondary text-sm ml-5 space-y-1">
        <p>
          Compute hashes to verify evidence integrity. Use chain of custody records for audit trails.
          Generate reports and export evidence in forensically sound formats (E01, L01, 7z).
        </p>
        <p class="text-xs text-txt-muted">
          Hash algorithms supported: MD5, SHA-1, SHA-256, SHA-512. E01 and L01 containers include embedded hashes that are verified automatically.
        </p>
      </div>
    </div>

    <div class="p-3 bg-info/10 border border-info/20 rounded-lg text-sm">
      <p class="text-info font-medium mb-1">Quick Start Workflow</p>
      <p class="text-txt-secondary text-xs">
        <strong>New case?</strong> <Kbd keys="Cmd+Shift+N" muted /> → Set paths → Scan → Hash all → Browse → Bookmark findings → Generate report.<br />
        <strong>Returning to a case?</strong> <Kbd keys="Cmd+O" muted /> → All tabs, tree state, and analysis are restored.
      </p>
    </div>

    <div class="p-3 bg-accent/5 border border-accent/20 rounded-lg text-sm">
      <p class="text-accent font-medium mb-1">New to CORE-FFX?</p>
      <p class="text-txt-secondary text-xs">
        Check the <strong>Tutorial: Example Case</strong> section for a complete step-by-step walkthrough 
        using a simulated forensic investigation. It covers every major feature from project creation to final report.
      </p>
    </div>
  </div>
);
