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
            EWF containers (E01, L01) include embedded hash values. CORE-FFX verifies the computed hash
            against the stored hash and reports match/mismatch status.
          </p>
        </div>
      </div>

      <div class="p-3 bg-info/10 border border-info/20 rounded-lg text-sm">
        <p class="text-info font-medium mb-1">Stored Hashes</p>
        <p class="text-txt-secondary text-xs">
          When available, CORE-FFX uses hashes stored within the container (E01 embedded MD5/SHA-1)
          rather than recomputing — providing instant verification without re-reading the entire image.
        </p>
      </div>
    </div>
  </div>
);
