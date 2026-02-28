// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * HelpPanel - Comprehensive in-app help and documentation
 *
 * Opens as a center-pane tab. Covers all major features, supported formats,
 * keyboard shortcuts, and forensic workflows.
 */

import { Component, createSignal, createMemo, For, Show } from "solid-js";
import {
  HiOutlineQuestionMarkCircle,
  HiOutlineArchiveBox,
  HiOutlineFingerPrint,
  HiOutlineArrowUpTray,
  HiOutlineClipboardDocumentList,
  HiOutlineMagnifyingGlass,
  HiOutlineCodeBracket,
  HiOutlineDocumentText,
  HiOutlineCog6Tooth,
  HiOutlineFolder,
  HiOutlineArchiveBoxArrowDown,
  HiOutlineRectangleGroup,
  HiOutlineCommandLine,
  HiOutlineLockClosed,
  HiOutlineEye,
  HiOutlinePhoto,
  HiOutlineEnvelope,
  HiOutlineCircleStack,
  HiOutlineBookmark,
  HiOutlineChartBar,
  HiOutlineBars3BottomLeft,
  HiOutlineBars3BottomRight,
  HiOutlineChevronUp,
  HiOutlineChevronDown,
} from "./icons";
import { Kbd } from "./ui/Kbd";

// =============================================================================
// Types
// =============================================================================

interface HelpSection {
  id: string;
  title: string;
  icon: Component<{ class?: string }>;
  content: Component;
}

// =============================================================================
// Section Content Components
// =============================================================================

const GettingStartedContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      CORE-FFX is a forensic file explorer for analyzing digital evidence containers.
      It provides read-only access to forensic images with hash verification, 
      file preview, and chain of custody tracking.
    </p>

    <div class="space-y-3">
      <h4 class="font-semibold text-txt flex items-center gap-2">
        <span class="text-accent">1.</span> Create or Open a Project
      </h4>
      <p class="text-txt-secondary text-sm ml-5">
        Use <Kbd keys="Cmd+Shift+N" muted /> to create a new project or <Kbd keys="Cmd+O" muted /> to open an existing <code class="text-accent">.cffx</code> project file.
        Projects organize your case work — evidence paths, bookmarks, notes, hash results, and chain of custody records are all saved within the project.
      </p>

      <h4 class="font-semibold text-txt flex items-center gap-2">
        <span class="text-accent">2.</span> Point to Evidence
      </h4>
      <p class="text-txt-secondary text-sm ml-5">
        During project setup, specify your evidence directory. CORE-FFX will recursively scan for supported forensic containers (E01, AD1, L01, UFED, archives, raw images, and more).
      </p>

      <h4 class="font-semibold text-txt flex items-center gap-2">
        <span class="text-accent">3.</span> Browse and Analyze
      </h4>
      <p class="text-txt-secondary text-sm ml-5">
        Click any evidence file in the left panel to expand its contents in the evidence tree.
        Select files to view them in hex, text, or document preview mode. The right panel shows metadata, EXIF data, and hash information.
      </p>

      <h4 class="font-semibold text-txt flex items-center gap-2">
        <span class="text-accent">4.</span> Verify and Document
      </h4>
      <p class="text-txt-secondary text-sm ml-5">
        Compute hashes to verify evidence integrity. Use chain of custody records for audit trails.
        Generate reports and export evidence in forensically sound formats (E01, L01, 7z).
      </p>
    </div>
  </div>
);

const EvidenceContainersContent: Component = () => (
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

    <div class="p-3 bg-bg-secondary rounded-lg border border-border/50 text-sm">
      <p class="font-medium text-txt mb-1">Nested Containers</p>
      <p class="text-txt-secondary">
        CORE-FFX supports containers inside containers — for example, a ZIP archive inside an E01 image, or an AD1 inside a 7z archive.
        These can be expanded inline in the evidence tree without manual extraction.
      </p>
    </div>
  </div>
);

const FileViewersContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Files inside evidence containers can be viewed directly without extraction.
      CORE-FFX auto-detects the best viewer based on file extension and content magic bytes.
    </p>

    <div class="grid grid-cols-1 md:grid-cols-2 gap-3">
      <ViewerCard
        icon={HiOutlineCodeBracket}
        title="Hex Viewer"
        desc="Raw hexadecimal byte display with ASCII sidebar, offset navigation, and header analysis"
        shortcut="Cmd+2"
      />
      <ViewerCard
        icon={HiOutlineDocumentText}
        title="Text Viewer"
        desc="Plain text and source code with syntax detection"
        shortcut="Cmd+3"
      />
      <ViewerCard
        icon={HiOutlineEye}
        title="PDF Viewer"
        desc="Embedded PDF rendering with page navigation"
      />
      <ViewerCard
        icon={HiOutlinePhoto}
        title="Image Viewer"
        desc="Image preview with EXIF metadata extraction (JPEG, PNG, TIFF, BMP, GIF, WebP)"
      />
      <ViewerCard
        icon={HiOutlineDocumentText}
        title="Office Documents"
        desc="DOCX, DOC, PPTX, PPT, ODT, ODP, RTF text extraction and preview"
      />
      <ViewerCard
        icon={HiOutlineChartBar}
        title="Spreadsheets"
        desc="Excel (XLSX/XLS), CSV, and ODS with tabular display"
      />
      <ViewerCard
        icon={HiOutlineEnvelope}
        title="Email Viewer"
        desc="EML and MBOX email parsing — headers, body, attachments"
      />
      <ViewerCard
        icon={HiOutlineEnvelope}
        title="PST Viewer"
        desc="Outlook PST file browsing — folder tree, message list, message preview"
      />
      <ViewerCard
        icon={HiOutlineCog6Tooth}
        title="Plist Viewer"
        desc="Apple property list (binary and XML) with tree display"
      />
      <ViewerCard
        icon={HiOutlineCodeBracket}
        title="Binary Viewer"
        desc="PE, ELF, and Mach-O executable analysis — headers, sections, imports, exports"
      />
      <ViewerCard
        icon={HiOutlineLockClosed}
        title="Registry Viewer"
        desc="Windows Registry hive browsing — keys, values, and types"
      />
      <ViewerCard
        icon={HiOutlineCircleStack}
        title="Database Viewer"
        desc="SQLite database browsing — tables, schemas, and data preview"
      />
    </div>
  </div>
);

const ViewerCard: Component<{ icon: Component<{ class?: string }>; title: string; desc: string; shortcut?: string }> = (props) => (
  <div class="p-3 bg-bg-secondary rounded-lg border border-border/50 flex gap-3">
    <div class="p-2 bg-accent/10 rounded-lg text-accent flex-shrink-0 h-fit">
      <props.icon class="w-5 h-5" />
    </div>
    <div class="min-w-0">
      <div class="font-medium text-txt text-sm flex items-center gap-2">
        {props.title}
        <Show when={props.shortcut}>
          <Kbd keys={props.shortcut!} muted />
        </Show>
      </div>
      <div class="text-xs text-txt-muted mt-0.5">{props.desc}</div>
    </div>
  </div>
);

const HashVerificationContent: Component = () => (
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

const SearchContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Search across evidence files and their contents. Open the search panel with <Kbd keys="Cmd+F" muted />.
    </p>

    <div class="space-y-2">
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">File Name Search</div>
        <p class="text-xs text-txt-muted mt-1">
          Search by filename across all loaded evidence containers. Supports partial matches and extension filtering.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Full-Text Search</div>
        <p class="text-xs text-txt-muted mt-1">
          Search the project database using SQLite FTS (full-text search) for bookmarks, notes, and file metadata.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Deduplication</div>
        <p class="text-xs text-txt-muted mt-1">
          Identify duplicate files across evidence containers using hash-based comparison.
          Available via <strong>Tools → Deduplication</strong>.
        </p>
      </div>
    </div>
  </div>
);

const ExportContent: Component = () => (
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

const ReportsContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Generate forensic examination reports via the Report Wizard. 
      Open with <Kbd keys="Cmd+P" muted /> or <strong>Tools → Generate Report</strong>.
    </p>

    <div class="space-y-3">
      <h4 class="font-semibold text-txt text-sm">Report Sections</h4>
      <div class="grid grid-cols-1 md:grid-cols-2 gap-2">
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">Case Information</div>
          <p class="text-xs text-txt-muted mt-1">Case number, examiner, agency, dates, and authority</p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">Evidence Items</div>
          <p class="text-xs text-txt-muted mt-1">Collected items with serial numbers, descriptions, and hash values</p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">Findings & Analysis</div>
          <p class="text-xs text-txt-muted mt-1">Detailed findings from examination with supporting evidence</p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm">Chain of Custody</div>
          <p class="text-xs text-txt-muted mt-1">Immutable COC records with lock/amend/void lifecycle</p>
        </div>
      </div>
    </div>
  </div>
);

const ChainOfCustodyContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Chain of Custody (COC) records use an append-only immutability model to ensure forensic integrity
      and maintain a complete audit trail for all evidence handling.
    </p>

    <div class="space-y-3">
      <h4 class="font-semibold text-txt text-sm">Status Lifecycle</h4>
      <div class="flex items-center gap-2 flex-wrap text-sm">
        <span class="badge badge-success">Draft</span>
        <span class="text-txt-muted">→ freely editable →</span>
        <span class="px-2 py-0.5 rounded bg-warning/20 text-warning text-xs font-medium">🔒 Locked</span>
        <span class="text-txt-muted">→ amend with initials + reason →</span>
        <span class="badge badge-error">Voided</span>
      </div>

      <div class="space-y-2 mt-3">
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm flex items-center gap-2">
            <span class="badge badge-success">Draft</span>
          </div>
          <p class="text-xs text-txt-muted mt-1">
            Freely editable. All fields can be modified and the record can be deleted (removed).
          </p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm flex items-center gap-2">
            <span class="px-2 py-0.5 rounded bg-warning/20 text-warning text-xs font-medium">🔒 Locked</span>
          </div>
          <p class="text-xs text-txt-muted mt-1">
            Immutable. Editing requires examiner's initials and a reason for the change — 
            this creates a formal amendment record in the audit trail. The original value is preserved.
          </p>
        </div>
        <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
          <div class="font-medium text-txt text-sm flex items-center gap-2">
            <span class="badge badge-error">Voided</span>
          </div>
          <p class="text-xs text-txt-muted mt-1">
            Soft-deleted. The record is hidden from active views but persists in the database for 
            the audit trail. Requires examiner's initials and a reason for voiding.
          </p>
        </div>
      </div>
    </div>
  </div>
);

const EvidenceCollectionContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Evidence Collection is a standalone on-site acquisition form for documenting collected items.
      It is separate from the Report Wizard and opens as a center-pane tab.
    </p>

    <div class="space-y-2">
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Creating a Collection</div>
        <p class="text-xs text-txt-muted mt-1">
          Right-click the Report button in the sidebar → "Evidence Collection…", 
          or use <strong>Tools → Evidence Collection</strong>, 
          or open the command palette (<Kbd keys="Cmd+K" muted />) and search "Evidence Collection".
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Schema-Driven Forms</div>
        <p class="text-xs text-txt-muted mt-1">
          Collection forms are rendered from JSON schema templates. Fields include location, 
          date/time, authorization, collected items, and examiner details.
          Changes are auto-saved to the project database.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Linked Data</div>
        <p class="text-xs text-txt-muted mt-1">
          The right panel shows a linked data tree with relationships between collected items, 
          COC records, and evidence files when a collection tab is active.
        </p>
      </div>
    </div>
  </div>
);

const ProcessedDatabasesContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      CORE-FFX can read case data from third-party forensic tool databases, enabling 
      cross-tool analysis without leaving the application.
    </p>

    <div class="grid grid-cols-1 md:grid-cols-3 gap-3">
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50 text-center">
        <div class="font-medium text-txt text-sm">Magnet AXIOM</div>
        <p class="text-xs text-txt-muted mt-1">Case directories with artifact categories and evidence sources</p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50 text-center">
        <div class="font-medium text-txt text-sm">Cellebrite PA</div>
        <p class="text-xs text-txt-muted mt-1">report.xml + SQLite databases with data sources and artifacts</p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50 text-center">
        <div class="font-medium text-txt text-sm">Autopsy</div>
        <p class="text-xs text-txt-muted mt-1">.aut files + autopsy.db with data sources, artifacts, and tags</p>
      </div>
    </div>
  </div>
);

const BookmarksNotesContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Bookmarks and notes help you organize findings during analysis. Both are saved 
      to the project database and persisted across sessions.
    </p>

    <div class="space-y-2">
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm flex items-center gap-2">
          <HiOutlineBookmark class="w-4 h-4 text-accent" /> Bookmarks
        </div>
        <p class="text-xs text-txt-muted mt-1">
          Mark files or entries of interest for quick access later. 
          Bookmarks appear in the sidebar Bookmarks panel and are searchable.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm flex items-center gap-2">
          <HiOutlineDocumentText class="w-4 h-4 text-accent" /> Notes
        </div>
        <p class="text-xs text-txt-muted mt-1">
          Attach notes to evidence files or entries. Notes support free-form text 
          and are displayed in the right panel alongside file metadata.
        </p>
      </div>
    </div>
  </div>
);

const KeyboardShortcutsContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Press <Kbd keys="?" muted /> at any time to open the full keyboard shortcuts reference.
      Here are the most important shortcuts:
    </p>

    <div class="space-y-4">
      <ShortcutGroup title="General" shortcuts={[
        { keys: "Cmd+K", desc: "Open command palette" },
        { keys: "Cmd+,", desc: "Open settings" },
        { keys: "?", desc: "Show keyboard shortcuts" },
        { keys: "Esc", desc: "Close dialog / clear filter" },
      ]} />
      <ShortcutGroup title="Project" shortcuts={[
        { keys: "Cmd+Shift+N", desc: "New project" },
        { keys: "Cmd+O", desc: "Open project" },
        { keys: "Cmd+S", desc: "Save project" },
        { keys: "Cmd+Shift+S", desc: "Save project as…" },
      ]} />
      <ShortcutGroup title="View" shortcuts={[
        { keys: "Cmd+1", desc: "Info view" },
        { keys: "Cmd+2", desc: "Hex view" },
        { keys: "Cmd+3", desc: "Text view" },
        { keys: "Cmd+B", desc: "Toggle left panel" },
        { keys: "Cmd+Shift+B", desc: "Toggle right panel" },
      ]} />
      <ShortcutGroup title="Actions" shortcuts={[
        { keys: "Cmd+F", desc: "Search files" },
        { keys: "Cmd+P", desc: "Generate report" },
        { keys: "Cmd+H", desc: "Compute hash" },
        { keys: "Cmd+E", desc: "Export" },
      ]} />
    </div>
  </div>
);

const ShortcutGroup: Component<{ title: string; shortcuts: { keys: string; desc: string }[] }> = (props) => (
  <div>
    <h4 class="text-xs font-semibold text-txt-muted uppercase tracking-wider mb-2">{props.title}</h4>
    <div class="space-y-1">
      <For each={props.shortcuts}>
        {(s) => (
          <div class="flex items-center justify-between py-1.5 px-3 bg-bg-secondary rounded border border-border/30">
            <span class="text-txt text-sm">{s.desc}</span>
            <Kbd keys={s.keys} muted />
          </div>
        )}
      </For>
    </div>
  </div>
);

const ProjectManagementContent: Component = () => (
  <div class="space-y-4">
    <p class="text-txt-secondary leading-relaxed">
      Projects (<code class="text-accent">.cffx</code>) store all case data — evidence paths, bookmarks, 
      notes, hash results, chain of custody records, activity logs, and settings.
      A companion database (<code class="text-accent">.ffxdb</code>) provides SQL-backed persistence.
    </p>

    <div class="space-y-2">
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Auto-Save</div>
        <p class="text-xs text-txt-muted mt-1">
          Enable auto-save via <strong>File → Toggle Auto-Save</strong> or the save dropdown in the toolbar.
          When enabled, the project is saved automatically after changes.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Workspace Profiles</div>
        <p class="text-xs text-txt-muted mt-1">
          Configure reusable workspace profiles during project setup to preset evidence paths, 
          export locations, and processed database directories.
        </p>
      </div>
      <div class="p-3 bg-bg-secondary rounded-lg border border-border/50">
        <div class="font-medium text-txt text-sm">Recovery</div>
        <p class="text-xs text-txt-muted mt-1">
          CORE-FFX supports project backup, versioning, and crash recovery. 
          Use the project recovery check on startup to restore from auto-saved state.
        </p>
      </div>
    </div>
  </div>
);

const FilesystemsContent: Component = () => (
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

const AboutContent: Component = () => (
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

// =============================================================================
// Section Registry
// =============================================================================

const HELP_SECTIONS: HelpSection[] = [
  { id: "getting-started", title: "Getting Started", icon: HiOutlineRectangleGroup, content: GettingStartedContent },
  { id: "evidence-containers", title: "Evidence Containers", icon: HiOutlineArchiveBox, content: EvidenceContainersContent },
  { id: "file-viewers", title: "File Viewers", icon: HiOutlineEye, content: FileViewersContent },
  { id: "hash-verification", title: "Hash Verification", icon: HiOutlineFingerPrint, content: HashVerificationContent },
  { id: "search", title: "Search & Deduplication", icon: HiOutlineMagnifyingGlass, content: SearchContent },
  { id: "export", title: "Export Formats", icon: HiOutlineArrowUpTray, content: ExportContent },
  { id: "reports", title: "Reports", icon: HiOutlineClipboardDocumentList, content: ReportsContent },
  { id: "chain-of-custody", title: "Chain of Custody", icon: HiOutlineLockClosed, content: ChainOfCustodyContent },
  { id: "evidence-collection", title: "Evidence Collection", icon: HiOutlineArchiveBoxArrowDown, content: EvidenceCollectionContent },
  { id: "processed-databases", title: "Processed Databases", icon: HiOutlineChartBar, content: ProcessedDatabasesContent },
  { id: "project-management", title: "Project Management", icon: HiOutlineFolder, content: ProjectManagementContent },
  { id: "filesystems", title: "Filesystem Drivers", icon: HiOutlineCircleStack, content: FilesystemsContent },
  { id: "bookmarks-notes", title: "Bookmarks & Notes", icon: HiOutlineBookmark, content: BookmarksNotesContent },
  { id: "keyboard-shortcuts", title: "Keyboard Shortcuts", icon: HiOutlineCommandLine, content: KeyboardShortcutsContent },
  { id: "about", title: "About CORE-FFX", icon: HiOutlineQuestionMarkCircle, content: AboutContent },
];

// =============================================================================
// Main HelpPanel Component
// =============================================================================

export const HelpPanel: Component = () => {
  const [activeSection, setActiveSection] = createSignal("getting-started");
  const [searchQuery, setSearchQuery] = createSignal("");
  const [tocSide, setTocSide] = createSignal<"left" | "right">("left");

  const filteredSections = createMemo(() => {
    const query = searchQuery().toLowerCase().trim();
    if (!query) return HELP_SECTIONS;
    return HELP_SECTIONS.filter(s => s.title.toLowerCase().includes(query));
  });

  const toggleSection = (id: string) => {
    setActiveSection(id);
  };

  const toggleTocSide = () => {
    setTocSide(prev => prev === "left" ? "right" : "left");
  };

  const activeSectionIndex = createMemo(() =>
    HELP_SECTIONS.findIndex(s => s.id === activeSection())
  );

  const goToPrev = () => {
    const idx = activeSectionIndex();
    if (idx > 0) setActiveSection(HELP_SECTIONS[idx - 1].id);
  };

  const goToNext = () => {
    const idx = activeSectionIndex();
    if (idx < HELP_SECTIONS.length - 1) setActiveSection(HELP_SECTIONS[idx + 1].id);
  };

  // Sidebar Navigation (rendered on left or right)
  const Sidebar = () => (
    <div
      class="w-56 flex-shrink-0 bg-bg-panel overflow-y-auto"
      classList={{
        "border-r border-border": tocSide() === "left",
        "border-l border-border": tocSide() === "right",
      }}
    >
      <div class="p-3">
        <div class="flex items-center gap-2 mb-3">
          <HiOutlineQuestionMarkCircle class="w-5 h-5 text-accent" />
          <h2 class="font-semibold text-txt text-sm flex-1">Help & Docs</h2>
          <button
            class="icon-btn-sm"
            onClick={goToPrev}
            disabled={activeSectionIndex() <= 0}
            title="Previous section"
          >
            <HiOutlineChevronUp class="w-4 h-4" />
          </button>
          <button
            class="icon-btn-sm"
            onClick={goToNext}
            disabled={activeSectionIndex() >= HELP_SECTIONS.length - 1}
            title="Next section"
          >
            <HiOutlineChevronDown class="w-4 h-4" />
          </button>
          <button
            class="icon-btn-sm"
            onClick={toggleTocSide}
            title={`Move contents to ${tocSide() === "left" ? "right" : "left"}`}
          >
            <Show when={tocSide() === "left"}
              fallback={<HiOutlineBars3BottomLeft class="w-4 h-4" />}
            >
              <HiOutlineBars3BottomRight class="w-4 h-4" />
            </Show>
          </button>
        </div>

        {/* Search */}
        <input
          type="text"
          placeholder="Search topics…"
          class="input-sm w-full mb-3"
          value={searchQuery()}
          onInput={(e) => setSearchQuery(e.currentTarget.value)}
        />

        {/* Section List */}
        <nav class="space-y-0.5">
          <For each={filteredSections()}>
            {(section) => (
              <button
                class="w-full flex items-center gap-2 px-2.5 py-2 rounded-lg text-sm transition-colors duration-150"
                classList={{
                  "bg-accent/10 text-accent font-medium": activeSection() === section.id,
                  "text-txt-secondary hover:bg-bg-hover hover:text-txt": activeSection() !== section.id,
                }}
                onClick={() => toggleSection(section.id)}
              >
                <section.icon class="w-4 h-4 flex-shrink-0" />
                <span class="truncate text-left">{section.title}</span>
              </button>
            )}
          </For>
        </nav>
      </div>
    </div>
  );

  // Content Area
  const Content = () => (
    <div class="flex-1 overflow-y-auto">
      <div class="max-w-3xl mx-auto p-6">
        <For each={HELP_SECTIONS}>
          {(section) => (
            <Show when={activeSection() === section.id}>
              <div class="animate-fade-in">
                <div class="flex items-center gap-3 mb-5">
                  <div class="p-2 bg-accent/10 rounded-xl text-accent">
                    <section.icon class="w-6 h-6" />
                  </div>
                  <div>
                    <h2 class="text-xl font-bold text-txt">{section.title}</h2>
                  </div>
                </div>
                <section.content />
              </div>
            </Show>
          )}
        </For>
      </div>
    </div>
  );

  return (
    <div class="flex h-full overflow-hidden bg-bg">
      <Show when={tocSide() === "left"}>
        <Sidebar />
      </Show>
      <Content />
      <Show when={tocSide() === "right"}>
        <Sidebar />
      </Show>
    </div>
  );
};
