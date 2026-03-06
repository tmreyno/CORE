// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";

export const FilesystemsContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      When browsing raw or E01 disk images, CORE-FFX automatically detects and parses
      the filesystem to display files and directories.
    </p>

    <div class="overflow-x-auto">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-border">
            <th class="text-left py-2 pr-4 text-txt font-semibold">Filesystem</th>
            <th class="text-left py-2 text-txt font-semibold">Description</th>
          </tr>
        </thead>
        <tbody class="text-txt-secondary">
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-txt">NTFS</td>
            <td class="py-2">Windows — MFT-based, supports ADS, timestamps, permissions</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-txt">FAT12/16/32</td>
            <td class="py-2">Universal — USB drives, SD cards, legacy systems</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-txt">exFAT</td>
            <td class="py-2">Large file support — modern removable media</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-txt">HFS+ / HFSX</td>
            <td class="py-2">macOS — Journaled filesystem with resource forks</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-txt">APFS</td>
            <td class="py-2">Modern macOS/iOS — Copy-on-write, snapshots, encryption</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-txt">ext2/3/4</td>
            <td class="py-2">Linux — Journaled with inode-based structure</td>
          </tr>
          <tr>
            <td class="py-2 pr-4 font-medium text-txt">DMG</td>
            <td class="py-2">Apple Disk Image container</td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>
);
