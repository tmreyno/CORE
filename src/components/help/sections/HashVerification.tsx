// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";

export const HashVerificationContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Hash verification ensures evidence integrity by computing cryptographic digests and comparing
      them against stored values. CORE-FFX supports MD5, SHA-1, SHA-256, and SHA-512.
    </p>

    <div class="space-y-3">
      <h4 class="font-semibold text-txt text-sm">Hashing Workflows</h4>
      
      <div class="space-y-2">
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">Hash Active File</div>
          <p class="text-xs text-txt-muted mt-1">
            Select an evidence file in the left panel, then use <strong>Tools → Hash Active File</strong> or the toolbar hash button.
            Results appear in the right panel.
          </p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">Hash Selected Files</div>
          <p class="text-xs text-txt-muted mt-1">
            Multi-select evidence files using checkboxes, then use <strong>Tools → Hash Selected Files</strong>.
            A batch queue processes files with progress tracking.
          </p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">Hash All Files</div>
          <p class="text-xs text-txt-muted mt-1">
            Use <strong>Tools → Hash All Files</strong> to compute hashes for every discovered evidence file.
            Results are saved to the project database.
          </p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">E01/L01 Verification</div>
          <p class="text-xs text-txt-muted mt-1">
            EWF containers (E01, L01) include embedded hash values from acquisition. Right-click any E01/L01 → <strong>Verify Container</strong> 
            to re-compute the hash over the entire image data and compare against the stored value.
            A match confirms the file has not been altered since acquisition.
          </p>
        </div>
      </div>

      <h4 class="font-semibold text-txt text-sm">Step-by-Step: Verify Evidence on Intake</h4>
      <div class="text-txt-secondary text-sm space-y-1 ml-1">
        <p><strong>1.</strong> Select the hash algorithm in the toolbar dropdown (SHA-256 recommended for new cases).</p>
        <p><strong>2.</strong> Click the hash button → <strong>Hash All Files</strong> to compute hashes for all evidence.</p>
        <p><strong>3.</strong> For E01 containers, also right-click → <strong>Verify Container</strong> to check the embedded acquisition hash.</p>
        <p><strong>4.</strong> Compare computed values against your intake documentation (chain of custody form, acquisition notes).</p>
        <p><strong>5.</strong> All hash results are automatically stored in the project database (<code class="text-accent">.ffxdb</code>).</p>
      </div>

      <div class="p-3 bg-info/10 border border-info/20 rounded-lg text-sm">
        <p class="text-info font-medium mb-1">Stored Hashes</p>
        <p class="text-txt-secondary text-xs">
          When available, CORE-FFX uses hashes stored within the container (E01 embedded MD5/SHA-1)
          rather than recomputing — providing instant verification without re-reading the entire image.
          UFED extractions also include their own integrity hashes.
        </p>
      </div>

      <div class="p-3 bg-warning/10 border border-warning/20 rounded-lg text-sm">
        <p class="text-warning font-medium mb-1">Forensic Best Practice</p>
        <p class="text-txt-secondary text-xs">
          Always hash evidence <strong>immediately upon intake</strong>, before any analysis begins. 
          Record the hash values in your case documentation and chain of custody records.
          Re-verify at the end of your examination to confirm nothing changed during analysis — 
          since CORE-FFX is read-only, these values should always match.
        </p>
      </div>
    </div>
  </div>
);
