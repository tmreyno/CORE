// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";

export const ExportContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Export evidence in multiple forensically sound formats. Open the export panel via the sidebar or <strong>File → Export</strong>.
    </p>

    <div class="overflow-x-auto">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-border">
            <th class="text-left py-2 pr-4 text-txt font-semibold">Mode</th>
            <th class="text-left py-2 pr-4 text-txt font-semibold">Output</th>
            <th class="text-left py-2 text-txt font-semibold">Use Case</th>
          </tr>
        </thead>
        <tbody class="text-txt-secondary">
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-type-e01">Physical Image (E01)</td>
            <td class="py-2 pr-4"><code>.E01</code></td>
            <td class="py-2">Bitstream disk images — sector-level acquisition via libewf</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-type-l01">Logical Image (L01)</td>
            <td class="py-2 pr-4"><code>.L01</code></td>
            <td class="py-2">Logical file collection — selected files/folders preserved with metadata and hashes</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-type-archive">Native (7z / Copy)</td>
            <td class="py-2 pr-4"><code>.7z</code> or files</td>
            <td class="py-2">Archive creation or direct file export — forensic presets with no compression by default</td>
          </tr>
          <tr>
            <td class="py-2 pr-4 font-medium text-txt-muted">Tools</td>
            <td class="py-2 pr-4">—</td>
            <td class="py-2">Test, repair, and validate existing archives</td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="p-3 bg-warning/10 border border-warning/20 rounded-lg text-sm">
      <p class="text-warning font-medium mb-1">Forensic Defaults</p>
      <p class="text-txt-secondary text-xs">
        All export modes default to <strong>no compression</strong> and <strong>2 GB split size</strong> for
        forensic integrity (bit-for-bit fidelity) and broad compatibility (FAT32, FTK Imager). 
        Split sizes can be adjusted using the preset dropdown (CD, DVD, Blu-ray, custom).
      </p>
    </div>
  </div>
);
