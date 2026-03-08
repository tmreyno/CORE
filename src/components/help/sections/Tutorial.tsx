// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Tutorial — Step-by-step walkthrough of a simulated forensic case.
 *
 * "Case 2026-0042: Meridian Financial Group"
 *
 * Covers every major CORE-FFX feature in the order a forensic examiner would
 * use them during a real investigation.
 */

import type { Component } from "solid-js";
import { Kbd } from "../../ui/Kbd";

// =============================================================================
// Reusable sub-components for the tutorial
// =============================================================================

const Step: Component<{ num: number; title: string; children: any }> = (props) => (
  <div class="space-y-2">
    <h4 class="font-semibold text-txt flex items-center gap-2">
      <span class="flex items-center justify-center w-6 h-6 rounded-full bg-accent text-white text-xs font-bold flex-shrink-0">
        {props.num}
      </span>
      {props.title}
    </h4>
    <div class="ml-8 space-y-2 text-sm text-txt-secondary">{props.children}</div>
  </div>
);

const Scenario: Component<{ children: any }> = (props) => (
  <div class="p-3 bg-accent/5 border border-accent/20 rounded-lg text-sm">
    <span class="font-semibold text-accent text-xs uppercase tracking-wider">Scenario</span>
    <div class="mt-1 text-txt-secondary">{props.children}</div>
  </div>
);

const Action: Component<{ children: any }> = (props) => (
  <div class="p-2.5 bg-bg-secondary rounded-lg border border-border/50 text-sm">
    <span class="font-semibold text-success text-xs uppercase tracking-wider">Action</span>
    <div class="mt-1 text-txt-secondary">{props.children}</div>
  </div>
);

const Result: Component<{ children: any }> = (props) => (
  <div class="p-2.5 bg-info/5 border border-info/20 rounded-lg text-sm">
    <span class="font-semibold text-info text-xs uppercase tracking-wider">Result</span>
    <div class="mt-1 text-txt-secondary">{props.children}</div>
  </div>
);

const Tip: Component<{ children: any }> = (props) => (
  <div class="p-2.5 bg-warning/5 border border-warning/20 rounded-lg text-sm">
    <span class="font-semibold text-warning text-xs uppercase tracking-wider">Tip</span>
    <div class="mt-1 text-txt-secondary">{props.children}</div>
  </div>
);

// =============================================================================
// TutorialContent
// =============================================================================

export const TutorialContent: Component = () => (
  <div class="space-y-6">
    {/* Introduction */}
    <div class="space-y-3">
      <p class="text-txt-secondary leading-relaxed">
        This tutorial walks you through a complete forensic investigation using CORE-FFX. 
        Follow along with the simulated case below to learn every major feature in context.
      </p>

      <Scenario>
        <p class="font-medium text-txt">Case 2026-0042: Meridian Financial Group — Suspected Insider Data Theft</p>
        <p class="mt-1">
          You are a digital forensic examiner tasked with analyzing evidence from a corporate investigation.
          The subject, a former finance director, is suspected of exfiltrating proprietary financial models
          and client data before departing the company. Law enforcement has seized:
        </p>
        <ul class="mt-2 ml-4 list-disc space-y-1">
          <li><strong>Item 1:</strong> Dell Latitude laptop — imaged as <code class="text-accent">EV-001-Laptop.E01</code> (120 GB, EnCase format)</li>
          <li><strong>Item 2:</strong> SanDisk 64 GB USB drive — imaged as <code class="text-accent">EV-002-USB.dd</code> (raw image)</li>
          <li><strong>Item 3:</strong> iPhone 14 extraction — <code class="text-accent">EV-003-iPhone.ufdr</code> (Cellebrite UFED)</li>
          <li><strong>Item 4:</strong> Office workstation — logical collection as <code class="text-accent">EV-004-Workstation.AD1</code> (FTK Imager)</li>
          <li><strong>Item 5:</strong> Cloud backup archive — <code class="text-accent">EV-005-CloudBackup.7z</code></li>
        </ul>
        <p class="mt-2">
          Your supervisor has also provided processed database output from Magnet AXIOM 
          (<code class="text-accent">AXIOM-Case-2026-0042/</code>) run against the laptop image.
        </p>
      </Scenario>
    </div>

    <div class="border-t border-border/50 pt-4">
      <h3 class="text-sm font-bold text-txt uppercase tracking-wider mb-4">Phase 1 — Project Setup & Evidence Intake</h3>
    </div>

    {/* Step 1: Create Project */}
    <Step num={1} title="Create a New Project">
      <p>Every investigation begins with a CORE-FFX project that organizes all your case data.</p>
      <Action>
        <ol class="list-decimal ml-4 space-y-1">
          <li>Press <Kbd keys="Cmd+Shift+N" muted /> or go to <strong>File → New Project</strong></li>
          <li>The Project Setup Wizard opens. Enter:
            <ul class="ml-4 list-disc mt-1 space-y-0.5">
              <li><strong>Project Name:</strong> <code class="text-accent">2026-0042_Meridian_Financial</code></li>
              <li><strong>Save Location:</strong> Your case working directory (e.g., <code>/Cases/2026-0042/</code>)</li>
            </ul>
          </li>
          <li>Under <strong>Configure Locations</strong>, set:
            <ul class="ml-4 list-disc mt-1 space-y-0.5">
              <li><strong>Evidence Path:</strong> <code>/Cases/2026-0042/Evidence/</code> (where your images live)</li>
              <li><strong>Processed Database Path:</strong> <code>/Cases/2026-0042/ProcessedDB/</code></li>
              <li><strong>Case Documents Path:</strong> <code>/Cases/2026-0042/CaseDocs/</code></li>
            </ul>
          </li>
          <li>Optionally select a <strong>Workspace Profile</strong> if you have one configured for your lab</li>
          <li>Click <strong>Create Project</strong></li>
        </ol>
      </Action>
      <Result>
        <p>
          CORE-FFX creates two files: <code class="text-accent">2026-0042_Meridian_Financial.cffx</code> (project JSON)
          and <code class="text-accent">.ffxdb</code> (SQLite database). The dashboard opens in the center pane, 
          and an automatic scan of the evidence directory begins.
        </p>
      </Result>
      <Tip>
        <p>
          If you frequently use the same directory structure (e.g., your lab's standard folder layout),
          create a Workspace Profile during setup. Next time, just select that profile and all paths
          are pre-filled.
        </p>
      </Tip>
    </Step>

    {/* Step 2: Review Discovered Evidence */}
    <Step num={2} title="Review Discovered Evidence">
      <p>After scanning, review what CORE-FFX found in your evidence directory.</p>
      <Action>
        <ol class="list-decimal ml-4 space-y-1">
          <li>Click the <strong>Evidence Containers</strong> icon in the left sidebar (archive box icon)</li>
          <li>The evidence tree shows all discovered containers, color-coded by type:
            <ul class="ml-4 list-disc mt-1 space-y-0.5">
              <li class="text-type-e01">EV-001-Laptop.E01 (E01 — blue)</li>
              <li class="text-type-raw">EV-002-USB.dd (Raw — purple)</li>
              <li class="text-type-ufed">EV-003-iPhone.ufdr (UFED — orange)</li>
              <li class="text-type-ad1">EV-004-Workstation.AD1 (AD1 — green)</li>
              <li class="text-type-archive">EV-005-CloudBackup.7z (Archive — gray)</li>
            </ul>
          </li>
          <li>The toolbar shows the scan directory and a total count of discovered files</li>
        </ol>
      </Action>
      <Tip>
        <p>Use the type filter buttons above the evidence tree to show only specific container types.
        Click a type badge (e.g., "E01") to filter; click again to clear.</p>
      </Tip>
    </Step>

    {/* Step 3: Verify Evidence Integrity */}
    <Step num={3} title="Verify Evidence Integrity (Hashing)">
      <p>Before analysis begins, verify that all evidence is intact and unmodified since acquisition.</p>
      <Action>
        <ol class="list-decimal ml-4 space-y-1">
          <li>In the toolbar, set the hash algorithm to <strong>SHA-256</strong> using the dropdown</li>
          <li>Click the <strong>Hash</strong> button (fingerprint icon) → <strong>Hash All Files</strong></li>
          <li>A progress indicator appears in the status bar as files are processed</li>
          <li>For E01 files specifically: Right-click <code>EV-001-Laptop.E01</code> → <strong>Verify Container</strong>
            <p class="text-xs mt-0.5">This checks the embedded MD5/SHA-1 hash stored inside the E01 against a fresh computation of the image data</p>
          </li>
        </ol>
      </Action>
      <Result>
        <p>
          Hash results appear in the right panel for each file. E01 containers show a 
          <span class="badge badge-success ml-1">Verified</span> badge if the stored hash matches.
          All hash values are automatically saved to the project database for your records.
        </p>
      </Result>
      <Tip>
        <p>
          E01 and L01 containers store embedded hashes from acquisition. CORE-FFX reads these 
          first for instant comparison. For raw images (.dd), hashes must be computed fresh — 
          make sure you have the original acquisition hash for comparison.
        </p>
      </Tip>
    </Step>

    <div class="border-t border-border/50 pt-4">
      <h3 class="text-sm font-bold text-txt uppercase tracking-wider mb-4">Phase 2 — Evidence Browsing & Analysis</h3>
    </div>

    {/* Step 4: Browse Container Contents */}
    <Step num={4} title="Browse Container Contents">
      <p>Expand containers in the evidence tree to explore their file systems and contents.</p>
      <Action>
        <ol class="list-decimal ml-4 space-y-1">
          <li>Click the expand arrow next to <code class="text-type-e01">EV-001-Laptop.E01</code></li>
          <li>CORE-FFX automatically detects the NTFS filesystem and displays partitions</li>
          <li>Expand the partition to browse the directory structure — just like a file explorer</li>
          <li>Navigate to <code>C:\Users\JSmith\Documents\</code> to look for financial documents</li>
          <li>Click any file to preview it in the center pane</li>
        </ol>
      </Action>
      <Result>
        <p>
          The center pane shows the file content using the appropriate viewer (auto-detected).
          The right panel shows file metadata: size, timestamps, path, and any computed hashes.
          For images, EXIF data is extracted and displayed.
        </p>
      </Result>

      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <p class="font-medium text-txt text-sm mb-1">Filesystem Support per Container Type</p>
        <ul class="list-disc ml-4 space-y-0.5 text-xs text-txt-muted">
          <li><strong>E01 / Raw (.dd):</strong> Full filesystem parsing — NTFS, FAT, exFAT, HFS+, APFS, ext2/3/4</li>
          <li><strong>AD1:</strong> Logical file tree (directory + file hierarchy stored in the container)</li>
          <li><strong>UFED (.ufdr):</strong> Mobile extraction tree with app data, messages, photos</li>
          <li><strong>Archives (.7z, .zip):</strong> Archive file listing with synthesized directory tree</li>
        </ul>
      </div>
    </Step>

    {/* Step 5: Use File Viewers */}
    <Step num={5} title="View Files Inside Containers">
      <p>CORE-FFX provides specialized viewers for many file types — all without extracting from the container.</p>
      <Action>
        <ol class="list-decimal ml-4 space-y-1">
          <li>Navigate to a <strong>.xlsx</strong> file (e.g., <code>Q4_Revenue_Model.xlsx</code>) → auto-opens in the Spreadsheet Viewer</li>
          <li>Find a <strong>.pdf</strong> file (e.g., <code>Board_Presentation.pdf</code>) → opens in the embedded PDF Viewer</li>
          <li>Locate a <strong>.pst</strong> file (e.g., <code>Outlook.pst</code>) → opens the PST Viewer with folder tree and message list</li>
          <li>For any file, switch views using the keyboard:
            <ul class="ml-4 list-disc mt-1">
              <li><Kbd keys="Cmd+1" muted /> — Info / metadata view</li>
              <li><Kbd keys="Cmd+2" muted /> — Hex viewer (raw bytes)</li>
              <li><Kbd keys="Cmd+3" muted /> — Text viewer (decoded text)</li>
            </ul>
          </li>
        </ol>
      </Action>
      <Tip>
        <p>
          If CORE-FFX doesn't recognize a file extension, it uses <strong>magic byte detection</strong> to determine the
          file type. You can always fall back to the hex viewer to inspect raw data. The center pane supports 
          multiple open tabs — <Kbd keys="Cmd+W" muted /> closes the active tab.
        </p>
      </Tip>
    </Step>

    {/* Step 6: Nested Containers */}
    <Step num={6} title="Explore Nested Containers">
      <p>Evidence containers can contain other containers — CORE-FFX handles this transparently.</p>
      <Scenario>
        <p>
          Inside <code class="text-type-e01">EV-001-Laptop.E01</code> → <code>C:\Users\JSmith\Downloads\</code>,
          you find <code>backup_2025.zip</code>. This is a ZIP archive inside an E01 image.
        </p>
      </Scenario>
      <Action>
        <ol class="list-decimal ml-4 space-y-1">
          <li>In the evidence tree, look for the nested container icon (small archive badge) next to <code>backup_2025.zip</code></li>
          <li>Click the expand arrow — CORE-FFX extracts it to a temp directory and parses the archive</li>
          <li>Browse the ZIP contents inline, just like top-level containers</li>
          <li>Click files within the nested archive to view them</li>
        </ol>
      </Action>
      <Result>
        <p>
          Nested containers work identically to top-level ones. You can have archives inside E01 images, 
          AD1 files inside 7z archives, etc. There is no depth limit. Temporary extractions are cached 
          and cleaned up when you close the project.
        </p>
      </Result>
    </Step>

    {/* Step 7: Bookmark Findings */}
    <Step num={7} title="Bookmark Important Findings">
      <p>As you browse, bookmark files and entries of interest for later reference and reporting.</p>
      <Scenario>
        <p>
          You've found a suspicious file: <code>ClientList_CONFIDENTIAL.xlsx</code> in the user's Documents folder 
          and also on the USB drive — suggesting it was copied.
        </p>
      </Scenario>
      <Action>
        <ol class="list-decimal ml-4 space-y-1">
          <li>Select the file in the evidence tree</li>
          <li>Right-click → <strong>Bookmark</strong>, or use the bookmark button in the right panel</li>
          <li>Add a note: <em>"Confidential client list found on both laptop and USB — potential exfiltration"</em></li>
          <li>View all bookmarks by clicking the <strong>Bookmarks</strong> icon in the left sidebar</li>
        </ol>
      </Action>
      <Tip>
        <p>
          Bookmarks are searchable and appear on the Dashboard. You can also export all bookmarks 
          to JSON via right-click on the Bookmarks sidebar icon → <strong>Export Bookmarks</strong>. 
          Bookmark counts appear in the status bar at the bottom.
        </p>
      </Tip>
    </Step>

    {/* Step 8: Search */}
    <Step num={8} title="Search Across All Evidence">
      <p>Use search to find specific files or keywords across all loaded containers.</p>
      <Action>
        <ol class="list-decimal ml-4 space-y-1">
          <li>Press <Kbd keys="Cmd+F" muted /> to open the Search panel</li>
          <li>Search for <code>CONFIDENTIAL</code> — this searches file names across all containers</li>
          <li>Click any result to navigate directly to that file in the evidence tree</li>
          <li>For project-wide text searching (bookmarks, notes, metadata), use the <strong>Full-Text Search</strong> tab
            <p class="text-xs mt-0.5">This queries the project's SQLite FTS index</p>
          </li>
        </ol>
      </Action>
    </Step>

    {/* Step 9: Deduplication */}
    <Step num={9} title="Find Duplicate Files (Deduplication)">
      <p>Identify files that appear in multiple evidence containers — critical for proving data movement.</p>
      <Scenario>
        <p>
          You suspect files were copied from the laptop to the USB drive. Deduplication will confirm this
          by matching files by hash across all containers.
        </p>
      </Scenario>
      <Action>
        <ol class="list-decimal ml-4 space-y-1">
          <li>Click the <strong>Deduplication</strong> icon in the sidebar (overlapping pages icon)</li>
          <li>CORE-FFX compares files across all evidence containers using hash values</li>
          <li>Results show groups of identical files with their locations</li>
          <li>In our case, <code>ClientList_CONFIDENTIAL.xlsx</code> appears in both <code>EV-001-Laptop.E01</code> and <code>EV-002-USB.dd</code></li>
        </ol>
      </Action>
      <Result>
        <p>
          This confirms the file was copied. The hash match proves the USB copy is byte-identical 
          to the laptop original — strong evidence of intentional data transfer.
        </p>
      </Result>
    </Step>

    {/* Step 10: Processed Databases */}
    <Step num={10} title="Review Processed Database Results">
      <p>Import analysis results from other forensic tools to correlate findings.</p>
      <Action>
        <ol class="list-decimal ml-4 space-y-1">
          <li>Click <strong>Processed Databases</strong> in the left sidebar (chart icon)</li>
          <li>The AXIOM case directory should appear if your Processed DB path was set correctly</li>
          <li>Click the AXIOM case to open it — CORE-FFX parses the case info, evidence sources, and artifact categories</li>
          <li>Review artifact categories: Browser History, Cloud Storage, USB Activity, etc.</li>
        </ol>
      </Action>
      <Tip>
        <p>
          CORE-FFX supports three forensic tools: <strong>Magnet AXIOM</strong>, <strong>Cellebrite Physical Analyzer</strong>,
          and <strong>Autopsy</strong>. The tool type is auto-detected — just point to the output directory. 
          This is read-only analysis; original databases are never modified.
        </p>
      </Tip>
    </Step>

    <div class="border-t border-border/50 pt-4">
      <h3 class="text-sm font-bold text-txt uppercase tracking-wider mb-4">Phase 3 — Documentation & Chain of Custody</h3>
    </div>

    {/* Step 11: Evidence Collection */}
    <Step num={11} title="Document Evidence Collection">
      <p>Create a formal evidence collection record documenting what was seized, by whom, and under what authority.</p>
      <Action>
        <ol class="list-decimal ml-4 space-y-1">
          <li>Right-click the <strong>Report</strong> icon in the sidebar → <strong>Evidence Collection…</strong>
            <p class="text-xs mt-0.5">Alternative: Use <strong>Tools → Evidence Collection</strong> or <Kbd keys="Cmd+K" muted /> → "Evidence Collection"</p>
          </li>
          <li>A schema-driven form opens as a center pane tab. Fill in:
            <ul class="ml-4 list-disc mt-1 space-y-0.5">
              <li><strong>Case Number:</strong> 2026-0042</li>
              <li><strong>Collecting Officer:</strong> Det. Sarah Chen, Badge #4451</li>
              <li><strong>Collection Date:</strong> 2026-02-15</li>
              <li><strong>Authorization:</strong> Search Warrant #SW-2026-0042-A, signed by Judge M. Thompson</li>
              <li><strong>Location:</strong> Meridian Financial Group, 1200 Commerce Blvd, Suite 400</li>
            </ul>
          </li>
          <li>Under <strong>Collected Items</strong>, add each evidence item with its description, serial number, and condition</li>
          <li>The form auto-saves to the project database as you type</li>
        </ol>
      </Action>
      <Result>
        <p>
          The right panel shows a <strong>Linked Data Tree</strong> — a visual map of relationships between 
          this collection record and the evidence files, COC items, and collected items in your project.
        </p>
      </Result>
    </Step>

    {/* Step 12: Chain of Custody */}
    <Step num={12} title="Create Chain of Custody Records">
      <p>Every evidence transfer must be documented. COC records use an immutable lifecycle.</p>
      <Action>
        <ol class="list-decimal ml-4 space-y-1">
          <li>Open the Report Wizard: <Kbd keys="Cmd+P" muted /> or <strong>Tools → Generate Report</strong></li>
          <li>Navigate to the <strong>Chain of Custody</strong> section</li>
          <li>Click <strong>Add COC Item</strong> for each evidence transfer:
            <ul class="ml-4 list-disc mt-1 space-y-0.5">
              <li><strong>Item 1:</strong> EV-001 Laptop received from Det. Chen by Lab Tech A. Martinez on 2026-02-16</li>
              <li><strong>Item 2:</strong> EV-002 USB received from Det. Chen by Lab Tech A. Martinez on 2026-02-16</li>
            </ul>
          </li>
          <li>Fill in COC number, description, submitted by, received by, date/time, and location</li>
          <li>Each record starts in <span class="badge badge-success">Draft</span> status</li>
        </ol>
      </Action>

      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <p class="font-medium text-txt text-sm mb-1">COC Immutability Lifecycle</p>
        <ul class="list-disc ml-4 space-y-0.5 text-xs text-txt-muted">
          <li><span class="badge badge-success">Draft</span> — Freely editable, can be deleted</li>
          <li><span class="px-2 py-0.5 rounded bg-warning/20 text-warning text-xs font-medium">🔒 Locked</span> — Click "Lock" when record is final. After locking, any edit requires your <strong>initials</strong> and a <strong>reason</strong> — creating a formal amendment in the audit trail</li>
          <li><span class="badge badge-error">Voided</span> — Soft-deleted (record remains for audit) — requires initials and reason</li>
        </ul>
      </div>

      <Tip>
        <p>
          Lock COC records as soon as they are verified — this prevents accidental edits and creates 
          a defensible audit trail. If you need to correct a locked record, the amendment process 
          preserves the original value alongside the correction and your justification.
        </p>
      </Tip>
    </Step>

    <div class="border-t border-border/50 pt-4">
      <h3 class="text-sm font-bold text-txt uppercase tracking-wider mb-4">Phase 4 — Export & Reporting</h3>
    </div>

    {/* Step 13: Export Evidence */}
    <Step num={13} title="Export Evidence for Court / Sharing">
      <p>Export selected evidence in forensically sound formats for court submission or inter-agency sharing.</p>
      <Action>
        <ol class="list-decimal ml-4 space-y-1">
          <li>Click the <strong>Export</strong> icon in the sidebar (upload arrow icon) — the Export Panel opens as a center tab</li>
          <li>Choose your export mode:
            <ul class="ml-4 list-disc mt-1 space-y-1">
              <li><strong>Logical Image (L01):</strong> To create a portable evidence package of specific files
                <p class="text-xs text-txt-muted">Select files from the evidence tree, then use L01 mode to package them with metadata and hashes</p>
              </li>
              <li><strong>Physical Image (E01):</strong> To re-image a drive source
                <p class="text-xs text-txt-muted">Select a system drive as source — CORE-FFX can optionally remount it read-only for forensic integrity</p>
              </li>
              <li><strong>Native (7z / Copy):</strong> For quick file export or compressed archives
                <p class="text-xs text-txt-muted">Use forensic presets: Standard, Court, Transfer, or Long-term archival</p>
              </li>
            </ul>
          </li>
          <li>Fill in <strong>Case Metadata</strong> (expandable section): case number, evidence number, examiner, description</li>
          <li>Verify the defaults: <strong>No compression</strong>, <strong>2 GB split size</strong> (FAT32 compatible)</li>
          <li>Click <strong>Start Export</strong></li>
        </ol>
      </Action>

      <div class="p-3 bg-warning/5 border border-warning/20 rounded-lg text-sm">
        <span class="font-semibold text-warning text-xs uppercase tracking-wider">Important</span>
        <p class="mt-1 text-txt-secondary">
          All export modes default to <strong>no compression</strong> for bit-for-bit forensic integrity. 
          Change this only if you understand the implications. A JSON manifest and text report are 
          generated alongside every export for documentation.
        </p>
      </div>
    </Step>

    {/* Step 14: Generate Report */}
    <Step num={14} title="Generate the Forensic Examination Report">
      <p>Compile your findings into a formal report using the Report Wizard.</p>
      <Action>
        <ol class="list-decimal ml-4 space-y-1">
          <li>Press <Kbd keys="Cmd+P" muted /> or <strong>Tools → Generate Report</strong></li>
          <li>The wizard walks through each section:
            <ul class="ml-4 list-disc mt-1 space-y-0.5">
              <li><strong>Report Type:</strong> Select "Forensic Examination Report"</li>
              <li><strong>Case Information:</strong> Case #2026-0042, Examiner name, Agency, etc.</li>
              <li><strong>Evidence Items:</strong> List all examined items with hash values and acquisition details</li>
              <li><strong>Findings:</strong> Document your analysis — the suspicious file copy, cloud backup contents, etc.</li>
              <li><strong>Chain of Custody:</strong> COC records are pulled in automatically from the project database</li>
            </ul>
          </li>
          <li>Preview the report before export</li>
          <li>Export as PDF or the desired format</li>
        </ol>
      </Action>
      <Tip>
        <p>
          The report wizard automatically includes hash values, evidence file metadata, and bookmarked items 
          from your project. This saves significant manual data entry and reduces transcription errors.
        </p>
      </Tip>
    </Step>

    {/* Step 15: Save and Close */}
    <Step num={15} title="Save, Backup, and Close">
      <p>Ensure all work is preserved before closing the case.</p>
      <Action>
        <ol class="list-decimal ml-4 space-y-1">
          <li>Save the project: <Kbd keys="Cmd+S" muted /></li>
          <li>The status bar confirms the save and shows the project statistics:
            <ul class="ml-4 list-disc mt-1">
              <li>Total evidence files, bookmarks, notes, and activity events</li>
            </ul>
          </li>
          <li>Your project is fully self-contained in two files:
            <ul class="ml-4 list-disc mt-1 space-y-0.5">
              <li><code class="text-accent">.cffx</code> — Project state (JSON)</li>
              <li><code class="text-accent">.ffxdb</code> — Database (SQLite with WAL)</li>
            </ul>
          </li>
          <li>Copy both files to your case archive. The project can be reopened on any machine with CORE-FFX installed.</li>
        </ol>
      </Action>
      <Tip>
        <p>
          Enable <strong>Auto-Save</strong> (File → Toggle Auto-Save) for ongoing cases to prevent data loss.
          CORE-FFX also supports project backup (<strong>Tools → Create Backup</strong>) and crash recovery.
          If reconnecting later, use <Kbd keys="Cmd+O" muted /> to reopen the <code>.cffx</code> file — 
          all your tabs, tree state, and analysis are restored exactly as you left them.
        </p>
      </Tip>
    </Step>

    <div class="border-t border-border/50 pt-4">
      <h3 class="text-sm font-bold text-txt uppercase tracking-wider mb-4">Advanced: Multi-Examiner Workflow</h3>
    </div>

    {/* Step 16: Merge Projects */}
    <Step num={16} title="Merge Projects from Multiple Examiners">
      <Scenario>
        <p>
          Two examiners worked on this case — you analyzed the laptop and USB, while 
          a colleague analyzed the iPhone and workstation in a separate CORE-FFX project.
          Now you need to combine both projects for the final report.
        </p>
      </Scenario>
      <Action>
        <ol class="list-decimal ml-4 space-y-1">
          <li>Go to <strong>Tools → Merge Projects</strong></li>
          <li>Select both <code>.cffx</code> files</li>
          <li>The wizard shows a review step with per-project details:
            <ul class="ml-4 list-disc mt-1 space-y-0.5">
              <li><strong>Examiners:</strong> Both examiner names detected from project data, COC records, and session logs</li>
              <li><strong>Evidence Files:</strong> All containers from both projects</li>
              <li><strong>Collections, COC, Forms:</strong> Merged with deduplication</li>
            </ul>
          </li>
          <li>Set the merged project owner and output path</li>
          <li>Click <strong>Merge</strong> — databases are combined via INSERT OR IGNORE (no duplicates)</li>
        </ol>
      </Action>
      <Result>
        <p>
          A new merged project contains all evidence, bookmarks, notes, COC records, and activity 
          from both examiners. Provenance tracking shows which data came from which source project.
        </p>
      </Result>
    </Step>

    {/* Summary Checklist */}
    <div class="border-t border-border/50 pt-4">
      <h3 class="text-sm font-bold text-txt uppercase tracking-wider mb-3">Quick Reference Checklist</h3>
      <div class="p-4 bg-bg-secondary rounded-lg border border-border/50 text-sm space-y-2">
        <label class="flex items-start gap-2 text-txt-secondary">
          <span class="text-accent mt-0.5">☐</span>
          <span>Create project with correct evidence/processed DB/case docs paths</span>
        </label>
        <label class="flex items-start gap-2 text-txt-secondary">
          <span class="text-accent mt-0.5">☐</span>
          <span>Verify all evidence file hashes (SHA-256) before analysis</span>
        </label>
        <label class="flex items-start gap-2 text-txt-secondary">
          <span class="text-accent mt-0.5">☐</span>
          <span>E01 container verification — confirm stored hash matches</span>
        </label>
        <label class="flex items-start gap-2 text-txt-secondary">
          <span class="text-accent mt-0.5">☐</span>
          <span>Browse evidence containers and document findings with bookmarks</span>
        </label>
        <label class="flex items-start gap-2 text-txt-secondary">
          <span class="text-accent mt-0.5">☐</span>
          <span>Run deduplication to identify cross-container file copies</span>
        </label>
        <label class="flex items-start gap-2 text-txt-secondary">
          <span class="text-accent mt-0.5">☐</span>
          <span>Review processed database output (AXIOM / Cellebrite / Autopsy)</span>
        </label>
        <label class="flex items-start gap-2 text-txt-secondary">
          <span class="text-accent mt-0.5">☐</span>
          <span>Complete evidence collection form with authorization details</span>
        </label>
        <label class="flex items-start gap-2 text-txt-secondary">
          <span class="text-accent mt-0.5">☐</span>
          <span>Create and lock Chain of Custody records for all transfers</span>
        </label>
        <label class="flex items-start gap-2 text-txt-secondary">
          <span class="text-accent mt-0.5">☐</span>
          <span>Export evidence in forensic format (L01/E01/7z) with manifests</span>
        </label>
        <label class="flex items-start gap-2 text-txt-secondary">
          <span class="text-accent mt-0.5">☐</span>
          <span>Generate forensic examination report</span>
        </label>
        <label class="flex items-start gap-2 text-txt-secondary">
          <span class="text-accent mt-0.5">☐</span>
          <span>Save project and archive .cffx + .ffxdb files</span>
        </label>
      </div>
    </div>
  </div>
);
