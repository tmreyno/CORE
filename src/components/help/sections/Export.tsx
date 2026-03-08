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
            <td class="py-2">Bitstream disk images — sector-level acquisition via libewf. Supports adding folders as source.</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-type-l01">Logical Image (L01)</td>
            <td class="py-2 pr-4"><code>.L01</code></td>
            <td class="py-2">Logical file collection — selected files/folders preserved with metadata, timestamps, per-file MD5/SHA-1 hashes</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-type-archive">Native (7z / Copy)</td>
            <td class="py-2 pr-4"><code>.7z</code> or files</td>
            <td class="py-2">Archive creation or direct file export — forensic presets (Standard, Court, Transfer, Long-term)</td>
          </tr>
          <tr>
            <td class="py-2 pr-4 font-medium text-txt-muted">Tools</td>
            <td class="py-2 pr-4">—</td>
            <td class="py-2">Test, repair, and validate existing archives</td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="space-y-3">
      <h4 class="font-semibold text-txt text-sm">How to Export Evidence</h4>
      <div class="text-txt-secondary text-sm space-y-1 ml-1">
        <p><strong>1.</strong> Click the <strong>Export</strong> icon in the sidebar (upload arrow) — the Export Panel opens as a center tab.</p>
        <p><strong>2.</strong> Select your export mode (Physical, Logical, Native, or Tools).</p>
        <p><strong>3.</strong> Choose source files from the evidence tree or system drives.</p>
        <p><strong>4.</strong> Fill in case metadata (expandable section): case number, evidence number, examiner, description.</p>
        <p><strong>5.</strong> Verify split size and compression settings (defaults: no compression, 2 GB split).</p>
        <p><strong>6.</strong> Click <strong>Start Export</strong>. A JSON manifest and text report are generated alongside the output.</p>
      </div>

      <h4 class="font-semibold text-txt text-sm">Split Size Presets</h4>
      <div class="overflow-x-auto">
        <table class="w-full text-xs">
          <tbody class="text-txt-secondary">
            <tr class="border-b border-border/30">
              <td class="py-1 pr-3 font-medium">No splitting</td>
              <td class="py-1 pr-3">Single file</td>
              <td class="py-1 pr-3 font-medium">2 GB</td>
              <td class="py-1">FAT32 / FTK default</td>
            </tr>
            <tr class="border-b border-border/30">
              <td class="py-1 pr-3 font-medium">650 / 700 MB</td>
              <td class="py-1 pr-3">CD-ROM / CD-R</td>
              <td class="py-1 pr-3 font-medium">4 / 4.7 GB</td>
              <td class="py-1">DVD / FAT32 limit</td>
            </tr>
            <tr>
              <td class="py-1 pr-3 font-medium">1 GB</td>
              <td class="py-1 pr-3">General purpose</td>
              <td class="py-1 pr-3 font-medium">25 GB</td>
              <td class="py-1">Blu-ray</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    <div class="p-3 bg-warning/10 border border-warning/20 rounded-lg text-sm">
      <p class="text-warning font-medium mb-1">Forensic Defaults</p>
      <p class="text-txt-secondary text-xs">
        All export modes default to <strong>no compression</strong> and <strong>2 GB split size</strong> for
        forensic integrity (bit-for-bit fidelity) and broad compatibility (FAT32, FTK Imager). 
        Only change compression if you understand the implications — compressed exports may not 
        be accepted by some forensic tools or courts.
      </p>
    </div>

    <div class="p-3 bg-info/10 border border-info/20 rounded-lg text-sm">
      <p class="text-info font-medium mb-1">Drive Source & Read-Only Mounting</p>
      <p class="text-txt-secondary text-xs">
        When imaging system drives (Physical or Logical mode), CORE-FFX can enumerate connected drives and 
        optionally remount them as <strong>read-only</strong> for forensic integrity before imaging. 
        The boot volume is protected — it cannot be remounted read-only.
        After imaging, original mount states are automatically restored.
      </p>
    </div>
  </div>
);
