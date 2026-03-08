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
      the filesystem to display files and directories. Detection uses boot sector magic bytes — 
      no manual configuration is required.
    </p>

    <div class="overflow-x-auto">
      <table class="w-full text-sm">
        <thead>
          <tr class="border-b border-border">
            <th class="text-left py-2 pr-4 text-txt font-semibold">Filesystem</th>
            <th class="text-left py-2 pr-4 text-txt font-semibold">Typical Source</th>
            <th class="text-left py-2 text-txt font-semibold">Forensic Notes</th>
          </tr>
        </thead>
        <tbody class="text-txt-secondary text-xs">
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-txt">NTFS</td>
            <td class="py-2 pr-4">Windows PCs, servers</td>
            <td class="py-2">MFT-based. Supports Alternate Data Streams (ADS), $MFT timestamps, SID permissions.</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-txt">FAT12/16/32</td>
            <td class="py-2 pr-4">USB drives, SD cards, legacy</td>
            <td class="py-2">No journaling. Deleted files may be recoverable from directory entries.</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-txt">exFAT</td>
            <td class="py-2 pr-4">Modern removable media</td>
            <td class="py-2">64-bit offsets, large file support. No ADS or permissions.</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-txt">HFS+ / HFSX</td>
            <td class="py-2 pr-4">macOS (pre-Catalina)</td>
            <td class="py-2">Journaled. Resource forks may contain hidden data. Case-sensitivity varies.</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-txt">APFS</td>
            <td class="py-2 pr-4">Modern macOS / iOS</td>
            <td class="py-2">Copy-on-write, snapshots, native encryption. Most current Apple devices.</td>
          </tr>
          <tr class="border-b border-border/50">
            <td class="py-2 pr-4 font-medium text-txt">ext2/3/4</td>
            <td class="py-2 pr-4">Linux systems</td>
            <td class="py-2">Inode-based. ext3/4 are journaled. Common in IoT and server forensics.</td>
          </tr>
          <tr>
            <td class="py-2 pr-4 font-medium text-txt">DMG</td>
            <td class="py-2 pr-4">Apple disk images</td>
            <td class="py-2">Container format — may contain HFS+, APFS, or FAT partitions inside.</td>
          </tr>
        </tbody>
      </table>
    </div>

    <div class="space-y-1">
      <h4 class="font-semibold text-txt text-sm">How Filesystem Browsing Works</h4>
      <div class="text-txt-secondary text-sm space-y-1 ml-1">
        <p><strong>1.</strong> Load an E01 or raw disk image into your project.</p>
        <p><strong>2.</strong> Expand the container in the evidence tree — CORE-FFX reads the partition table and detects filesystems.</p>
        <p><strong>3.</strong> Click a partition to browse its directory structure. Files can be viewed, hashed, and exported.</p>
        <p><strong>4.</strong> Nested containers (e.g., a ZIP file inside an NTFS partition in an E01) can be expanded inline.</p>
      </div>
    </div>

    <div class="p-3 bg-info/10 border border-info/20 rounded-lg text-sm">
      <p class="text-info font-medium mb-1">Read-Only Parsing</p>
      <p class="text-txt-secondary text-xs">
        All filesystem drivers are strictly read-only. CORE-FFX never writes to or modifies the source image.
        The VFS (Virtual Filesystem) layer abstracts the image format — you browse files the same way 
        regardless of whether the image is E01, raw, or split across multiple segments.
      </p>
    </div>
  </div>
);
