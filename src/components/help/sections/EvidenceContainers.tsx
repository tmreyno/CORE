// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";

export const EvidenceContainersContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      CORE-FFX supports a wide range of forensic container formats. All operations are strictly read-only — source evidence is never modified.
    </p>

    <div class="overflow-x-auto">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-border">
            <th class="text-left py-2 pr-4 text-txt font-semibold">Format</th>
            <th class="text-left py-2 pr-4 text-txt font-semibold">Extensions</th>
            <th class="text-left py-2 text-txt font-semibold">Description</th>
          </tr>
        </thead>
        <tbody class="text-txt-secondary">
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-type-e01">E01/Ex01</td>
            <td class="py-2 pr-4"><code>.E01 .Ex01</code></td>
            <td class="py-2">EnCase Evidence File (EWF format) — physical and logical disk images with built-in compression and hash verification</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-type-ad1">AD1</td>
            <td class="py-2 pr-4"><code>.AD1</code></td>
            <td class="py-2">AccessData Logical Image — logical file collection format used by FTK Imager</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-type-l01">L01</td>
            <td class="py-2 pr-4"><code>.L01 .Lx01</code></td>
            <td class="py-2">EnCase Logical Evidence File — logical file collection in EWF format</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-type-ufed">UFED</td>
            <td class="py-2 pr-4"><code>.ufd .ufdr</code></td>
            <td class="py-2">Cellebrite UFED extraction — mobile device forensic data</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-type-raw">Raw Images</td>
            <td class="py-2 pr-4"><code>.dd .raw .img .iso .dmg</code></td>
            <td class="py-2">Raw disk images — bit-for-bit copies with optional filesystem parsing (NTFS, FAT, HFS+, APFS, ext2/3/4, exFAT)</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-type-archive">Archives</td>
            <td class="py-2 pr-4"><code>.zip .7z .tar .gz .rar</code></td>
            <td class="py-2">Common archive formats — browseable tree with inline extraction and nested container support</td>
          </tr>
          <tr>
            <td class="py-2 pr-4 font-medium text-txt-muted">Memory Dumps</td>
            <td class="py-2 pr-4"><code>.mem .dmp .vmem</code></td>
            <td class="py-2">Memory dump files — hex and binary analysis</td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="space-y-3">
      <h4 class="font-semibold text-txt text-sm">How to Use</h4>
      <div class="text-txt-secondary text-sm space-y-1">
        <p><strong>1. Scan for evidence:</strong> Set your evidence directory path in the toolbar and click <strong>Scan</strong>. CORE-FFX recursively discovers all supported containers.</p>
        <p><strong>2. Browse containers:</strong> Click the expand arrow next to any container in the evidence tree. E01 and raw images show filesystem partitions; AD1 and L01 show logical file trees; archives show their file listing.</p>
        <p><strong>3. View files:</strong> Click any file within a container to preview it in the center pane. The appropriate viewer is selected automatically.</p>
        <p><strong>4. Type filtering:</strong> Use the type filter buttons above the evidence tree to show only specific container types (e.g., only E01 files or only archives).</p>
      </div>
    </div>

    <div class="p-3 bg-bg-secondary rounded-lg border border-border/50 text-sm">
      <p class="font-medium text-txt mb-1">Nested Containers</p>
      <p class="text-txt-secondary">
        CORE-FFX supports containers inside containers — for example, a ZIP archive inside an E01 image, or an AD1 inside a 7z archive.
        These can be expanded inline in the evidence tree without manual extraction. Look for the nested container icon next to supported file types within the tree.
      </p>
    </div>

    <div class="p-3 bg-info/10 border border-info/20 rounded-lg text-sm">
      <p class="text-info font-medium mb-1">Best Practice</p>
      <p class="text-txt-secondary text-xs">
        Always verify container hashes before beginning analysis. For E01/L01 containers, use <strong>Verify Container</strong> (right-click) to check embedded hashes.
        For raw images, compute a fresh hash and compare against the acquisition hash from your evidence intake documentation.
      </p>
    </div>

    <div class="p-3 bg-warning/10 border border-warning/20 rounded-lg text-sm">
      <p class="text-warning font-medium mb-1">Read-Only Guarantee</p>
      <p class="text-txt-secondary text-xs">
        CORE-FFX <strong>never</strong> modifies source evidence files. All operations — browsing, viewing, hashing, exporting — are performed on read-only handles.
        Even nested container extraction uses temporary working copies, leaving the original container untouched.
      </p>
    </div>
  </div>
);
