// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";

export const AboutContent: Component = () => (
  <div class="space-y-4">
    <div class="text-center py-4">
      <div class="inline-flex items-center justify-center w-16 h-16 bg-accent/10 rounded-2xl mb-3">
        <span class="text-4xl">🔍</span>
      </div>
      <h3 class="text-xl font-bold text-txt">CORE-FFX</h3>
      <p class="text-txt-secondary text-sm">Forensic File Xplorer</p>
      <p class="text-txt-muted text-xs mt-1">© 2024–2026 CORE-FFX Project Contributors</p>
      <p class="text-txt-muted text-xs">Licensed under the MIT License</p>
    </div>

    <div class="space-y-2">
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Technology Stack</div>
        <p class="text-xs text-txt-muted mt-1">
          <strong>Backend:</strong> Rust + Tauri v2 — native performance with web-based UI<br />
          <strong>Frontend:</strong> SolidJS + TypeScript — reactive UI with fine-grained updates<br />
          <strong>Storage:</strong> SQLite (per-project .ffxdb) — reliable, portable data persistence
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Key Libraries</div>
        <p class="text-xs text-txt-muted mt-1">
          <strong>libewf</strong> — EWF image creation (E01 export)<br />
          <strong>LZMA SDK 24.09</strong> — 7z archive creation<br />
          <strong>Pure-Rust parsers</strong> — E01/L01 reading, AD1, UFED, filesystem drivers
        </p>
      </div>
    </div>
  </div>
);
