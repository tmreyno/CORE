// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ViewerMetadataPanel - Right panel metadata display for specialized viewers
 *
 * Shows tabbed metadata when a viewer (image, registry, database, binary,
 * email, plist, document, spreadsheet) is active. Displays:
 * - File Info tab (always present)
 * - Viewer-specific metadata tabs based on the active viewer type
 *
 * Replaces the TreePanel default view in RightPanel when viewer metadata
 * is available.
 */

import { Show, For, createSignal, createMemo, type JSX } from "solid-js";
import {
  ChevronDownIcon,
  ChevronRightIcon,
} from "./icons";
import { formatBytes } from "../utils";
import type {
  ViewerMetadata,
  ViewerMetadataSection,
  ExifMetadataSection,
  RegistryMetadataSection,
  DatabaseMetadataSection,
  BinaryMetadataSection,
  EmailMetadataSection,
  PlistMetadataSection,
  DocumentMetadataSection,
  SpreadsheetMetadataSection,
} from "../types/viewerMetadata";

// =============================================================================
// Props
// =============================================================================

export interface ViewerMetadataPanelProps {
  /** Viewer metadata to display */
  metadata: ViewerMetadata;
}

// =============================================================================
// Tab definitions
// =============================================================================

type MetadataTabId = "file" | "viewer";

// =============================================================================
// Component
// =============================================================================

export function ViewerMetadataPanel(props: ViewerMetadataPanelProps) {
  const [activeTab, setActiveTab] = createSignal<MetadataTabId>("viewer");

  /** Get a human-readable label for the viewer type */
  const viewerLabel = createMemo(() => {
    const sections = props.metadata.sections;
    if (sections.length === 0) return "Details";
    const first = sections[0];
    switch (first.kind) {
      case "exif": return "EXIF";
      case "registry": return "Registry";
      case "database": return "Database";
      case "binary": return "Binary";
      case "email": return "Email";
      case "plist": return "Plist";
      case "document": return "Document";
      case "spreadsheet": return "Spreadsheet";
      default: return "Details";
    }
  });

  /** Switch to viewer tab if sections are available, otherwise file tab */
  const effectiveTab = createMemo(() => {
    if (activeTab() === "viewer" && props.metadata.sections.length > 0) return "viewer";
    return "file";
  });

  return (
    <div class="flex flex-col h-full bg-bg">
      {/* Tab header */}
      <div class="flex items-center border-b border-border bg-bg-secondary">
        <button
          class={`px-3 py-2 text-xs font-medium transition-colors ${
            effectiveTab() === "file"
              ? "text-accent border-b-2 border-accent"
              : "text-txt-muted hover:text-txt"
          }`}
          onClick={() => setActiveTab("file")}
        >
          File Info
        </button>
        <Show when={props.metadata.sections.length > 0}>
          <button
            class={`px-3 py-2 text-xs font-medium transition-colors ${
              effectiveTab() === "viewer"
                ? "text-accent border-b-2 border-accent"
                : "text-txt-muted hover:text-txt"
            }`}
            onClick={() => setActiveTab("viewer")}
          >
            {viewerLabel()}
          </button>
        </Show>
      </div>

      {/* Tab content */}
      <div class="flex-1 overflow-y-auto">
        <Show when={effectiveTab() === "file"}>
          <FileInfoTab metadata={props.metadata} />
        </Show>
        <Show when={effectiveTab() === "viewer"}>
          <For each={props.metadata.sections}>
            {(section) => <MetadataSectionRenderer section={section} />}
          </For>
        </Show>
      </div>
    </div>
  );
}

// =============================================================================
// File Info Tab
// =============================================================================

function FileInfoTab(props: { metadata: ViewerMetadata }) {
  const info = () => props.metadata.fileInfo;

  return (
    <div class="p-3 space-y-3">
      {/* File name */}
      <div class="space-y-1">
        <div class="text-[10px] uppercase tracking-wider text-txt-muted font-medium">File</div>
        <div class="text-sm text-txt font-medium truncate" title={info().name}>{info().name}</div>
      </div>

      {/* Path */}
      <MetadataRow label="Path" value={info().path} truncate />

      {/* Size */}
      <MetadataRow label="Size" value={formatBytes(info().size)} />

      {/* Extension */}
      <Show when={info().extension}>
        <MetadataRow label="Extension" value={info().extension!} />
      </Show>

      {/* Container info */}
      <Show when={info().containerPath}>
        <div class="pt-2 border-t border-border/50">
          <div class="text-[10px] uppercase tracking-wider text-txt-muted font-medium mb-2">Container</div>
          <MetadataRow label="Path" value={info().containerPath!} truncate />
          <Show when={info().containerType}>
            <MetadataRow label="Type" value={info().containerType!} />
          </Show>
        </div>
      </Show>

      {/* Source type badges */}
      <div class="flex flex-wrap gap-1 pt-1">
        <Show when={info().isDiskFile}>
          <span class="badge badge-neutral">Disk File</span>
        </Show>
        <Show when={info().isVfsEntry}>
          <span class="badge badge-info">VFS Entry</span>
        </Show>
        <Show when={info().isArchiveEntry}>
          <span class="badge badge-warning">Archive Entry</span>
        </Show>
      </div>

      {/* Viewer type */}
      <div class="pt-2 border-t border-border/50">
        <MetadataRow label="Viewer" value={props.metadata.viewerType} />
      </div>
    </div>
  );
}

// =============================================================================
// Section Renderers
// =============================================================================

function MetadataSectionRenderer(props: { section: ViewerMetadataSection }) {
  switch (props.section.kind) {
    case "exif": return <ExifSection data={props.section} />;
    case "registry": return <RegistrySection data={props.section} />;
    case "database": return <DatabaseSection data={props.section} />;
    case "binary": return <BinarySection data={props.section} />;
    case "email": return <EmailSection data={props.section} />;
    case "plist": return <PlistSection data={props.section} />;
    case "document": return <DocumentSection data={props.section} />;
    case "spreadsheet": return <SpreadsheetSection data={props.section} />;
    default: return null;
  }
}

// =============================================================================
// EXIF Section
// =============================================================================

function ExifSection(props: { data: ExifMetadataSection }) {
  return (
    <div class="p-3 space-y-3">
      {/* Camera */}
      <Show when={props.data.make || props.data.model}>
        <CollapsibleGroup title="Camera" defaultOpen>
          <Show when={props.data.make}><MetadataRow label="Make" value={props.data.make!} /></Show>
          <Show when={props.data.model}><MetadataRow label="Model" value={props.data.model!} /></Show>
          <Show when={props.data.lensModel}><MetadataRow label="Lens" value={props.data.lensModel!} /></Show>
          <Show when={props.data.software}><MetadataRow label="Software" value={props.data.software!} /></Show>
        </CollapsibleGroup>
      </Show>

      {/* Capture Settings */}
      <Show when={props.data.exposureTime || props.data.fNumber || props.data.iso}>
        <CollapsibleGroup title="Capture Settings" defaultOpen>
          <Show when={props.data.exposureTime}><MetadataRow label="Exposure" value={props.data.exposureTime!} /></Show>
          <Show when={props.data.fNumber}><MetadataRow label="Aperture" value={`f/${props.data.fNumber}`} /></Show>
          <Show when={props.data.iso}><MetadataRow label="ISO" value={String(props.data.iso)} /></Show>
          <Show when={props.data.focalLength}><MetadataRow label="Focal Length" value={props.data.focalLength!} /></Show>
          <Show when={props.data.flash}><MetadataRow label="Flash" value={props.data.flash!} /></Show>
        </CollapsibleGroup>
      </Show>

      {/* Timestamps (forensically critical) */}
      <Show when={props.data.dateTimeOriginal || props.data.dateTime}>
        <CollapsibleGroup title="Timestamps" defaultOpen>
          <Show when={props.data.dateTimeOriginal}><MetadataRow label="Original" value={props.data.dateTimeOriginal!} highlight /></Show>
          <Show when={props.data.dateTimeDigitized}><MetadataRow label="Digitized" value={props.data.dateTimeDigitized!} /></Show>
          <Show when={props.data.dateTime}><MetadataRow label="Modified" value={props.data.dateTime!} /></Show>
          <Show when={props.data.gpsTimestamp}><MetadataRow label="GPS Time" value={props.data.gpsTimestamp!} highlight /></Show>
        </CollapsibleGroup>
      </Show>

      {/* GPS */}
      <Show when={props.data.gps}>
        <CollapsibleGroup title="GPS Location" defaultOpen>
          <MetadataRow label="Latitude" value={`${props.data.gps!.latitude.toFixed(6)}° ${props.data.gps!.latitudeRef}`} highlight />
          <MetadataRow label="Longitude" value={`${props.data.gps!.longitude.toFixed(6)}° ${props.data.gps!.longitudeRef}`} highlight />
          <Show when={props.data.gps!.altitude != null}>
            <MetadataRow label="Altitude" value={`${props.data.gps!.altitude!.toFixed(1)}m`} />
          </Show>
          <a
            href={`https://www.google.com/maps?q=${props.data.gps!.latitude},${props.data.gps!.longitude}`}
            target="_blank"
            rel="noopener noreferrer"
            class="block mt-1 text-xs text-accent hover:text-accent-hover underline"
          >
            Open in Google Maps ↗
          </a>
        </CollapsibleGroup>
      </Show>

      {/* Image Dimensions */}
      <Show when={props.data.width || props.data.height}>
        <CollapsibleGroup title="Image" defaultOpen={false}>
          <Show when={props.data.width && props.data.height}>
            <MetadataRow label="Dimensions" value={`${props.data.width} × ${props.data.height}`} />
          </Show>
          <Show when={props.data.orientation}><MetadataRow label="Orientation" value={String(props.data.orientation)} /></Show>
          <Show when={props.data.colorSpace}><MetadataRow label="Color Space" value={props.data.colorSpace!} /></Show>
        </CollapsibleGroup>
      </Show>

      {/* Forensic Identifiers */}
      <Show when={props.data.imageUniqueId || props.data.serialNumber || props.data.ownerName}>
        <CollapsibleGroup title="Forensic" defaultOpen>
          <Show when={props.data.imageUniqueId}><MetadataRow label="Unique ID" value={props.data.imageUniqueId!} highlight mono /></Show>
          <Show when={props.data.serialNumber}><MetadataRow label="Serial #" value={props.data.serialNumber!} highlight mono /></Show>
          <Show when={props.data.ownerName}><MetadataRow label="Owner" value={props.data.ownerName!} /></Show>
        </CollapsibleGroup>
      </Show>

      {/* Raw tag count */}
      <Show when={props.data.rawTagCount}>
        <div class="text-[10px] text-txt-muted pt-1 border-t border-border/50">
          {props.data.rawTagCount} raw EXIF tags available
        </div>
      </Show>
    </div>
  );
}

// =============================================================================
// Registry Section
// =============================================================================

function RegistrySection(props: { data: RegistryMetadataSection }) {
  return (
    <div class="p-3 space-y-3">
      <CollapsibleGroup title="Hive Info" defaultOpen>
        <MetadataRow label="Hive Name" value={props.data.hiveName} />
        <MetadataRow label="Type" value={props.data.hiveType} />
        <MetadataRow label="Root Key" value={props.data.rootKeyName} mono />
        <MetadataRow label="Total Keys" value={String(props.data.totalKeys)} />
        <MetadataRow label="Total Values" value={String(props.data.totalValues)} />
        <Show when={props.data.lastModified}>
          <MetadataRow label="Modified" value={props.data.lastModified!} />
        </Show>
      </CollapsibleGroup>

      <Show when={props.data.selectedKeyPath}>
        <CollapsibleGroup title="Selected Key" defaultOpen>
          <MetadataRow label="Path" value={props.data.selectedKeyPath!} mono truncate />
          <Show when={props.data.selectedKeyInfo}>
            <MetadataRow label="Subkeys" value={String(props.data.selectedKeyInfo!.subkeyCount)} />
            <MetadataRow label="Values" value={String(props.data.selectedKeyInfo!.valueCount)} />
            <Show when={props.data.selectedKeyInfo!.lastModified}>
              <MetadataRow label="Modified" value={props.data.selectedKeyInfo!.lastModified!} />
            </Show>
            <Show when={props.data.selectedKeyInfo!.className}>
              <MetadataRow label="Class" value={props.data.selectedKeyInfo!.className!} />
            </Show>
          </Show>
        </CollapsibleGroup>
      </Show>
    </div>
  );
}

// =============================================================================
// Database Section
// =============================================================================

function DatabaseSection(props: { data: DatabaseMetadataSection }) {
  const userTables = createMemo(() => props.data.tables.filter(t => !t.isSystem));
  const systemTables = createMemo(() => props.data.tables.filter(t => t.isSystem));

  return (
    <div class="p-3 space-y-3">
      <CollapsibleGroup title="Database Info" defaultOpen>
        <MetadataRow label="Page Size" value={formatBytes(props.data.pageSize)} />
        <MetadataRow label="Pages" value={String(props.data.pageCount)} />
        <MetadataRow label="Size" value={formatBytes(props.data.sizeBytes)} />
        <MetadataRow label="Tables" value={String(props.data.tableCount)} />
      </CollapsibleGroup>

      <Show when={props.data.selectedTable}>
        <CollapsibleGroup title="Selected Table" defaultOpen>
          <MetadataRow label="Name" value={props.data.selectedTable!} mono />
          {(() => {
            const table = props.data.tables.find(t => t.name === props.data.selectedTable);
            if (!table) return null;
            return (
              <>
                <MetadataRow label="Rows" value={String(table.rowCount)} />
                <MetadataRow label="Columns" value={String(table.columnCount)} />
              </>
            );
          })()}
        </CollapsibleGroup>
      </Show>

      <Show when={userTables().length > 0}>
        <CollapsibleGroup title={`Tables (${userTables().length})`} defaultOpen={false}>
          <div class="space-y-0.5">
            <For each={userTables()}>
              {(table) => (
                <div class="flex items-center justify-between text-xs py-0.5">
                  <span class="text-txt truncate font-mono" title={table.name}>{table.name}</span>
                  <span class="text-txt-muted shrink-0 ml-2">{table.rowCount} rows</span>
                </div>
              )}
            </For>
          </div>
        </CollapsibleGroup>
      </Show>

      <Show when={systemTables().length > 0}>
        <CollapsibleGroup title={`System Tables (${systemTables().length})`} defaultOpen={false}>
          <div class="space-y-0.5">
            <For each={systemTables()}>
              {(table) => (
                <div class="flex items-center justify-between text-xs py-0.5">
                  <span class="text-txt-muted truncate font-mono" title={table.name}>{table.name}</span>
                  <span class="text-txt-muted shrink-0 ml-2">{table.rowCount}</span>
                </div>
              )}
            </For>
          </div>
        </CollapsibleGroup>
      </Show>
    </div>
  );
}

// =============================================================================
// Binary Section
// =============================================================================

function BinarySection(props: { data: BinaryMetadataSection }) {
  return (
    <div class="p-3 space-y-3">
      <CollapsibleGroup title="Binary Info" defaultOpen>
        <MetadataRow label="Format" value={props.data.format} />
        <Show when={props.data.architecture}><MetadataRow label="Architecture" value={props.data.architecture!} /></Show>
        <Show when={props.data.entryPoint}><MetadataRow label="Entry Point" value={props.data.entryPoint!} mono /></Show>
        <Show when={props.data.subsystem}><MetadataRow label="Subsystem" value={props.data.subsystem!} /></Show>
        <Show when={props.data.compiler}><MetadataRow label="Compiler" value={props.data.compiler!} /></Show>
        <Show when={props.data.compiledDate}><MetadataRow label="Compiled" value={props.data.compiledDate!} highlight /></Show>
      </CollapsibleGroup>

      <CollapsibleGroup title="Structure" defaultOpen>
        <Show when={props.data.sectionCount != null}><MetadataRow label="Sections" value={String(props.data.sectionCount)} /></Show>
        <Show when={props.data.importCount != null}><MetadataRow label="Imports" value={String(props.data.importCount)} /></Show>
        <Show when={props.data.exportCount != null}><MetadataRow label="Exports" value={String(props.data.exportCount)} /></Show>
        <MetadataRow label="Stripped" value={props.data.isStripped ? "Yes" : "No"} />
        <MetadataRow label="Dynamic" value={props.data.isDynamic ? "Yes" : "No"} />
      </CollapsibleGroup>

      <Show when={props.data.characteristics && props.data.characteristics.length > 0}>
        <CollapsibleGroup title="Characteristics" defaultOpen={false}>
          <div class="flex flex-wrap gap-1">
            <For each={props.data.characteristics!}>
              {(char) => (
                <span class="px-1.5 py-0.5 text-[10px] bg-bg-hover text-txt-secondary rounded">{char}</span>
              )}
            </For>
          </div>
        </CollapsibleGroup>
      </Show>
    </div>
  );
}

// =============================================================================
// Email Section
// =============================================================================

function EmailSection(props: { data: EmailMetadataSection }) {
  return (
    <div class="p-3 space-y-3">
      <CollapsibleGroup title="Email Info" defaultOpen>
        <Show when={props.data.subject}><MetadataRow label="Subject" value={props.data.subject!} /></Show>
        <Show when={props.data.from}><MetadataRow label="From" value={props.data.from!} highlight /></Show>
        <Show when={props.data.to && props.data.to.length > 0}>
          <MetadataRow label="To" value={props.data.to!.join(", ")} />
        </Show>
        <Show when={props.data.cc && props.data.cc.length > 0}>
          <MetadataRow label="CC" value={props.data.cc!.join(", ")} />
        </Show>
        <Show when={props.data.date}><MetadataRow label="Date" value={props.data.date!} highlight /></Show>
      </CollapsibleGroup>

      <CollapsibleGroup title="Technical" defaultOpen={false}>
        <Show when={props.data.messageId}><MetadataRow label="Message-ID" value={props.data.messageId!} mono truncate /></Show>
        <Show when={props.data.inReplyTo}><MetadataRow label="In-Reply-To" value={props.data.inReplyTo!} mono truncate /></Show>
        <Show when={props.data.contentType}><MetadataRow label="Content-Type" value={props.data.contentType!} /></Show>
        <Show when={props.data.attachmentCount != null}>
          <MetadataRow label="Attachments" value={String(props.data.attachmentCount)} />
        </Show>
      </CollapsibleGroup>

      <Show when={props.data.messageCount != null}>
        <CollapsibleGroup title="MBOX" defaultOpen>
          <MetadataRow label="Messages" value={String(props.data.messageCount)} />
          <Show when={props.data.selectedMessageIndex != null}>
            <MetadataRow label="Viewing" value={`Message ${props.data.selectedMessageIndex! + 1}`} />
          </Show>
        </CollapsibleGroup>
      </Show>
    </div>
  );
}

// =============================================================================
// Plist Section
// =============================================================================

function PlistSection(props: { data: PlistMetadataSection }) {
  return (
    <div class="p-3 space-y-3">
      <CollapsibleGroup title="Property List" defaultOpen>
        <MetadataRow label="Format" value={props.data.format} />
        <MetadataRow label="Root Type" value={props.data.rootType} />
        <MetadataRow label="Entries" value={String(props.data.entryCount)} />
      </CollapsibleGroup>

      <Show when={props.data.notableKeys && props.data.notableKeys.length > 0}>
        <CollapsibleGroup title="Notable Keys" defaultOpen>
          <For each={props.data.notableKeys!}>
            {(entry) => (
              <MetadataRow label={entry.key} value={entry.value} mono />
            )}
          </For>
        </CollapsibleGroup>
      </Show>
    </div>
  );
}

// =============================================================================
// Document Section
// =============================================================================

function DocumentSection(props: { data: DocumentMetadataSection }) {
  return (
    <div class="p-3 space-y-3">
      <CollapsibleGroup title="Document Info" defaultOpen>
        <MetadataRow label="Format" value={props.data.format} />
        <Show when={props.data.title}><MetadataRow label="Title" value={props.data.title!} /></Show>
        <Show when={props.data.author}><MetadataRow label="Author" value={props.data.author!} /></Show>
        <Show when={props.data.creator}><MetadataRow label="Creator" value={props.data.creator!} /></Show>
        <Show when={props.data.producer}><MetadataRow label="Producer" value={props.data.producer!} /></Show>
        <Show when={props.data.pageCount != null}><MetadataRow label="Pages" value={String(props.data.pageCount)} /></Show>
        <Show when={props.data.wordCount != null}><MetadataRow label="Words" value={String(props.data.wordCount)} /></Show>
        <Show when={props.data.encrypted != null}><MetadataRow label="Encrypted" value={props.data.encrypted ? "Yes" : "No"} /></Show>
      </CollapsibleGroup>

      <Show when={props.data.creationDate || props.data.modificationDate}>
        <CollapsibleGroup title="Dates" defaultOpen>
          <Show when={props.data.creationDate}><MetadataRow label="Created" value={props.data.creationDate!} highlight /></Show>
          <Show when={props.data.modificationDate}><MetadataRow label="Modified" value={props.data.modificationDate!} highlight /></Show>
        </CollapsibleGroup>
      </Show>

      <Show when={props.data.keywords && props.data.keywords.length > 0}>
        <CollapsibleGroup title="Keywords" defaultOpen={false}>
          <div class="flex flex-wrap gap-1">
            <For each={props.data.keywords!}>
              {(kw) => (
                <span class="px-1.5 py-0.5 text-[10px] bg-bg-hover text-txt-secondary rounded">{kw}</span>
              )}
            </For>
          </div>
        </CollapsibleGroup>
      </Show>
    </div>
  );
}

// =============================================================================
// Spreadsheet Section
// =============================================================================

function SpreadsheetSection(props: { data: SpreadsheetMetadataSection }) {
  return (
    <div class="p-3 space-y-3">
      <CollapsibleGroup title="Spreadsheet Info" defaultOpen>
        <MetadataRow label="Format" value={props.data.format} />
        <MetadataRow label="Sheets" value={String(props.data.sheetCount)} />
        <Show when={props.data.selectedSheet}>
          <MetadataRow label="Active Sheet" value={props.data.selectedSheet!} />
        </Show>
      </CollapsibleGroup>

      <CollapsibleGroup title={`Sheets (${props.data.sheets.length})`} defaultOpen>
        <For each={props.data.sheets}>
          {(sheet) => (
            <div class="flex items-center justify-between text-xs py-0.5">
              <span class="text-txt truncate" title={sheet.name}>{sheet.name}</span>
              <span class="text-txt-muted shrink-0 ml-2">{sheet.rowCount}×{sheet.columnCount}</span>
            </div>
          )}
        </For>
      </CollapsibleGroup>
    </div>
  );
}

// =============================================================================
// Shared UI Helpers
// =============================================================================

/** Collapsible group with title */
function CollapsibleGroup(props: {
  title: string;
  defaultOpen?: boolean;
  children: JSX.Element;
}) {
  const [open, setOpen] = createSignal(props.defaultOpen !== false);

  return (
    <div class="border-b border-border/30 pb-2">
      <button
        class="flex items-center gap-1 w-full text-left py-1 group"
        onClick={() => setOpen(!open())}
      >
        <Show when={open()} fallback={<ChevronRightIcon class="w-3 h-3 text-txt-muted" />}>
          <ChevronDownIcon class="w-3 h-3 text-txt-muted" />
        </Show>
        <span class="text-[10px] uppercase tracking-wider text-txt-muted font-medium group-hover:text-txt-secondary">
          {props.title}
        </span>
      </button>
      <Show when={open()}>
        <div class="pl-4 space-y-1 mt-1">
          {props.children}
        </div>
      </Show>
    </div>
  );
}

/** Single metadata key-value row */
function MetadataRow(props: {
  label: string;
  value: string;
  highlight?: boolean;
  mono?: boolean;
  truncate?: boolean;
}) {
  return (
    <div class="flex items-baseline gap-2 text-xs py-0.5">
      <span class="text-txt-muted shrink-0 w-20">{props.label}</span>
      <span
        class={`text-txt min-w-0 ${props.highlight ? "text-accent" : ""} ${props.mono ? "font-mono text-[11px]" : ""} ${props.truncate ? "truncate" : "break-all"}`}
        title={props.value}
      >
        {props.value}
      </span>
    </div>
  );
}
