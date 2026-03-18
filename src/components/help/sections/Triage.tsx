// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { Component } from "solid-js";

export const TriageContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Forensic triage rapidly collects key system artifacts and optionally scans for credentials and
      secrets. Access triage via the <strong>Export panel → Triage</strong> tab.
    </p>

    {/* Collection Profiles */}
    <div class="space-y-3">
      <h4 class="font-semibold text-txt text-sm">Collection Profiles</h4>
      <p class="text-txt-secondary text-xs">
        Profiles are preconfigured sets of artifact categories. Select a profile to auto-check
        the matching categories, or choose <strong>Custom Selection</strong> to pick individually.
      </p>
      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-border">
              <th class="text-left py-2 pr-4 text-txt font-semibold">Profile</th>
              <th class="text-left py-2 text-txt font-semibold">Description</th>
            </tr>
          </thead>
          <tbody class="text-txt-secondary">
            <tr class="border-b border-border/50">
              <td class="py-2 pr-4 font-medium text-txt">🔒 Security Audit</td>
              <td class="py-2">Registry hives, event logs, and system security databases — ideal for
                incident response and intrusion analysis.</td>
            </tr>
            <tr class="border-b border-border/50">
              <td class="py-2 pr-4 font-medium text-txt">🔑 Credential Collection</td>
              <td class="py-2">SSH keys, cloud provider tokens, browser password databases,
                keychains, and certificates.</td>
            </tr>
            <tr class="border-b border-border/50">
              <td class="py-2 pr-4 font-medium text-txt">👤 User Activity</td>
              <td class="py-2">Shell history, recent documents, application usage patterns, and
                browser history.</td>
            </tr>
            <tr class="border-b border-border/50">
              <td class="py-2 pr-4 font-medium text-txt">🌍 Network Config</td>
              <td class="py-2">WiFi profiles, DNS settings, firewall rules, hosts file, and
                network configuration.</td>
            </tr>
            <tr class="border-b border-border/50">
              <td class="py-2 pr-4 font-medium text-txt">📦 Full Collection</td>
              <td class="py-2">Comprehensive collection of ALL artifact categories — recommended
                for thorough forensic triage.</td>
            </tr>
            <tr>
              <td class="py-2 pr-4 font-medium text-txt">🪪 System Identification</td>
              <td class="py-2">Essential identifiers (UUID, serial number, hostname, OS version)
                for evidence forms and chain of custody.</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    {/* Artifact Categories */}
    <div class="space-y-3">
      <h4 class="font-semibold text-txt text-sm">Artifact Categories</h4>
      <p class="text-txt-secondary text-xs">
        Each category groups related system artifacts. Click the expand arrow on any category to see
        the individual artifact targets that will be collected.
      </p>
      <div class="overflow-x-auto">
        <table class="w-full text-sm">
          <thead>
            <tr class="border-b border-border">
              <th class="text-left py-2 pr-3 text-txt font-semibold w-8">&nbsp;</th>
              <th class="text-left py-2 pr-4 text-txt font-semibold">Category</th>
              <th class="text-left py-2 text-txt font-semibold">Artifacts Collected</th>
            </tr>
          </thead>
          <tbody class="text-txt-secondary text-xs">
            <tr class="border-b border-border/50">
              <td class="py-1.5 pr-3">🗃️</td>
              <td class="py-1.5 pr-4 font-medium text-txt">Registry</td>
              <td class="py-1.5">SAM, SECURITY, SYSTEM, SOFTWARE hives — user accounts, security
                policies, installed software, services</td>
            </tr>
            <tr class="border-b border-border/50">
              <td class="py-1.5 pr-3">📋</td>
              <td class="py-1.5 pr-4 font-medium text-txt">Event Logs</td>
              <td class="py-1.5">System, Security, Application, PowerShell, and forensic log sources
                (Sysmon, Task Scheduler)</td>
            </tr>
            <tr class="border-b border-border/50">
              <td class="py-1.5 pr-3">⚙️</td>
              <td class="py-1.5 pr-4 font-medium text-txt">System Config</td>
              <td class="py-1.5">OS-level configuration, scheduled tasks, launch agents/daemons,
                sudoers, security audit databases</td>
            </tr>
            <tr class="border-b border-border/50">
              <td class="py-1.5 pr-3">🔑</td>
              <td class="py-1.5 pr-4 font-medium text-txt">Credentials</td>
              <td class="py-1.5">SSH keys, cloud CLI credentials (AWS, Azure, GCP), keychains, GPG
                keys, API tokens, certificates</td>
            </tr>
            <tr class="border-b border-border/50">
              <td class="py-1.5 pr-3">🌐</td>
              <td class="py-1.5 pr-4 font-medium text-txt">Browser Data</td>
              <td class="py-1.5">Chrome, Firefox, Safari, Edge profiles — history, bookmarks,
                cookies, saved passwords, extensions</td>
            </tr>
            <tr class="border-b border-border/50">
              <td class="py-1.5 pr-3">👤</td>
              <td class="py-1.5 pr-4 font-medium text-txt">User Activity</td>
              <td class="py-1.5">Shell history (bash, zsh, fish, PowerShell), recent docs,
                trash/recycle bin, application usage</td>
            </tr>
            <tr class="border-b border-border/50">
              <td class="py-1.5 pr-3">🌍</td>
              <td class="py-1.5 pr-4 font-medium text-txt">Network</td>
              <td class="py-1.5">Hosts file, WiFi profiles, DNS configuration, firewall rules,
                VPN settings, network interfaces</td>
            </tr>
            <tr>
              <td class="py-1.5 pr-3">🖥️</td>
              <td class="py-1.5 pr-4 font-medium text-txt">System Info</td>
              <td class="py-1.5">Hardware UUID, serial number, hostname, OS version, platform
                profile — for evidence identification</td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>

    {/* How to Run Triage */}
    <div class="space-y-3">
      <h4 class="font-semibold text-txt text-sm">How to Run Triage</h4>
      <div class="text-txt-secondary text-sm space-y-1 ml-1">
        <p><strong>1.</strong> Open the <strong>Export panel</strong> (sidebar upload arrow or <strong>File → Acquire & Export</strong>).</p>
        <p><strong>2.</strong> Select the <strong>Triage</strong> tab.</p>
        <p><strong>3.</strong> Choose a <strong>Collection Profile</strong> or pick categories manually.</p>
        <p><strong>4.</strong> Optionally enable <strong>Scan for credentials & secrets</strong>.</p>
        <p><strong>5.</strong> Set the output destination and click <strong>Start Collection</strong>.</p>
        <p><strong>6.</strong> Review results — collected file count, size, duration, and any secret findings.</p>
      </div>
    </div>

    {/* Credential Scanning */}
    <div class="p-3 bg-warning/10 border border-warning/20 rounded-lg text-sm">
      <p class="text-warning font-medium mb-1">🔑 Credential & Secret Scanning</p>
      <p class="text-txt-secondary text-xs leading-relaxed">
        When enabled, all collected text files are scanned for credentials and secrets using
        <strong> 30+ pattern matchers</strong>. Detected types include: API keys, private keys (RSA, EC, PGP),
        bearer/JWT tokens, database connection strings, cloud provider credentials (AWS, Azure, GCP),
        passwords in config files, and encryption keys. Each finding is reported with a
        <strong> confidence level</strong> (high, medium, low) and a <strong>redacted preview</strong> so
        sensitive material is not fully exposed.
      </p>
    </div>

    {/* Platform-specific note */}
    <div class="p-3 bg-info/10 border border-info/20 rounded-lg text-sm">
      <p class="text-info font-medium mb-1">🖥️ Platform-Specific Artifacts</p>
      <p class="text-txt-secondary text-xs leading-relaxed">
        Artifact definitions are <strong>platform-specific</strong>. The triage engine automatically detects the
        operating system (macOS, Windows, Linux) and collects only the artifacts relevant to that
        platform. For example, registry hives are collected on Windows, keychains on macOS, and
        systemd journal logs on Linux.
      </p>
    </div>

    {/* Output */}
    <div class="p-3 bg-bg-secondary border border-border/30 rounded-lg text-sm">
      <p class="text-txt font-medium mb-1">📁 Output Structure</p>
      <p class="text-txt-secondary text-xs leading-relaxed">
        Collected artifacts are organized by category in the output directory. A companion file
        (<code>.ffx-companion.json</code>) is automatically created alongside the output with timing,
        hash, and case metadata. An evidence collection record is also created in the project database
        for chain of custody tracking.
      </p>
    </div>
  </div>
);
