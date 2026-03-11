// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EvidenceCollectionSummaryPanel — Right-panel component that shows evidence
 * collection form data when a container is selected in the evidence tree.
 *
 * Features:
 * - Loads collection data from the project database (.ffxdb)
 * - Matches collected items to the active evidence file via `evidenceFileId`
 * - Displays collection header + item details in a read-only summary layout
 * - "Save as Document" — exports the active collection + items as formatted text/HTML
 * - "Export All as Spreadsheet" — exports ALL collected items across all collections as CSV
 */

import {
  Show,
  For,
  createSignal,
  createEffect,
  createMemo,
  on,
  type Accessor,
} from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import { save } from "@tauri-apps/plugin-dialog";
import {
  HiOutlineArchiveBoxArrowDown,
  HiOutlineDocumentText,
  HiOutlineArrowDownTray,
  HiOutlinePrinter,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineTableCells,
} from "./icons";
import { OptionalMetadataRow, StatusBadge } from "./viewerMetadata/shared";
import { printDocument } from "./document/documentHelpers";
import { useToast } from "./Toast";
import { logger } from "../utils/logger";
import type { DbEvidenceCollection, DbCollectedItem } from "../types/projectDb";
import type { DiscoveredFile } from "../types";

const log = logger.scope("EvidenceCollectionSummary");

// =============================================================================
// Props
// =============================================================================

export interface EvidenceCollectionSummaryPanelProps {
  /** Currently active evidence file in the tree */
  activeFile: Accessor<DiscoveredFile | null>;
  /** Whether a project is loaded (required for DB queries) */
  hasProject: Accessor<boolean>;
}

// =============================================================================
// Component
// =============================================================================

export function EvidenceCollectionSummaryPanel(props: EvidenceCollectionSummaryPanelProps) {
  const toast = useToast();

  // ─── State ──────────────────────────────────────────────────────────
  const [collections, setCollections] = createSignal<DbEvidenceCollection[]>([]);
  const [allItems, setAllItems] = createSignal<DbCollectedItem[]>([]);
  const [loading, setLoading] = createSignal(false);

  // ─── Load all collections + items when project changes ─────────────
  const loadData = async () => {
    if (!props.hasProject()) {
      setCollections([]);
      setAllItems([]);
      return;
    }
    setLoading(true);
    try {
      const [cols, items] = await Promise.all([
        invoke<DbEvidenceCollection[]>("project_db_get_evidence_collections", {
          caseNumber: null,
        }),
        invoke<DbCollectedItem[]>("project_db_get_all_collected_items"),
      ]);
      setCollections(cols);
      setAllItems(items);
    } catch (e) {
      log.warn("Failed to load evidence collections:", e);
      setCollections([]);
      setAllItems([]);
    } finally {
      setLoading(false);
    }
  };

  // Reload when project status changes
  createEffect(on(() => props.hasProject(), loadData));

  // Also reload when active file changes (collections may have been modified)
  createEffect(
    on(
      () => props.activeFile()?.path,
      () => {
        if (props.hasProject()) loadData();
      },
    ),
  );

  // ─── Derived: items matching the active file ──────────────────────
  const activeFilePath = createMemo(() => props.activeFile()?.path ?? "");

  /** Collected items that reference this evidence file */
  const matchingItems = createMemo(() => {
    const path = activeFilePath();
    if (!path) return [];
    return allItems().filter((item) => item.evidenceFileId === path);
  });

  /** Collection IDs referenced by matching items */
  const matchingCollectionIds = createMemo(() => {
    const ids = new Set(matchingItems().map((i) => i.collectionId));
    return ids;
  });

  /** Collections that contain items referencing this evidence file */
  const matchingCollections = createMemo(() => {
    const ids = matchingCollectionIds();
    if (ids.size === 0) return [];
    return collections().filter((c) => ids.has(c.id));
  });

  /** Whether there's any data to show for this particular file */
  const hasDataForFile = createMemo(() => matchingItems().length > 0);

  /** Whether there's any collection data at all */
  const hasAnyData = createMemo(() => collections().length > 0);

  // ─── Items grouped by collection ───────────────────────────────────
  const itemsByCollection = createMemo(() => {
    const map = new Map<string, DbCollectedItem[]>();
    for (const item of matchingItems()) {
      const list = map.get(item.collectionId) ?? [];
      list.push(item);
      map.set(item.collectionId, list);
    }
    return map;
  });

  // ─── Export: Save as Document (formatted text for current file) ────
  const handleSaveAsDocument = async () => {
    const cols = matchingCollections();
    const items = matchingItems();
    if (cols.length === 0) return;

    try {
      const filename = props.activeFile()?.filename ?? "evidence";
      const path = await save({
        title: "Save Evidence Collection Summary",
        defaultPath: `${sanitizeFilename(filename)}_collection.txt`,
        filters: [
          { name: "Text File", extensions: ["txt"] },
          { name: "HTML File", extensions: ["html"] },
        ],
      });
      if (!path) return;

      const isHtml = path.endsWith(".html");
      const content = isHtml
        ? buildHtmlDocument(cols, items, itemsByCollection(), filename)
        : buildTextDocument(cols, items, itemsByCollection(), filename);

      await invoke("write_text_file", { path, content });
      toast.success("Document Saved", `Collection summary saved successfully`);
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      toast.error("Save Failed", msg);
    }
  };

  // ─── Export: Print document ────────────────────────────────────────
  const handlePrint = () => {
    const cols = matchingCollections();
    const items = matchingItems();
    if (cols.length === 0) return;
    const filename = props.activeFile()?.filename ?? "evidence";
    const html = buildHtmlDocument(cols, items, itemsByCollection(), filename);
    printDocument(html);
  };

  // ─── Export: All collections as CSV spreadsheet ────────────────────
  const handleExportAllCsv = async () => {
    const cols = collections();
    const items = allItems();
    if (cols.length === 0 && items.length === 0) return;

    try {
      const path = await save({
        title: "Export All Evidence Collections as Spreadsheet",
        defaultPath: "evidence_collections.csv",
        filters: [{ name: "CSV Spreadsheet", extensions: ["csv"] }],
      });
      if (!path) return;

      const content = buildCsvSpreadsheet(cols, items);
      await invoke("write_text_file", { path, content });
      toast.success(
        "Spreadsheet Exported",
        `${items.length} items across ${cols.length} collections exported`,
      );
    } catch (e) {
      const msg = e instanceof Error ? e.message : String(e);
      toast.error("Export Failed", msg);
    }
  };

  // ─── Render ────────────────────────────────────────────────────────
  return (
    <div class="flex flex-col h-full bg-bg">
      {/* Header */}
      <div class="flex items-center gap-1 border-b border-border bg-bg-secondary px-3 py-2">
        <HiOutlineArchiveBoxArrowDown class="w-4 h-4 text-accent shrink-0" />
        <span class="text-xs font-medium text-txt flex-1">Evidence Collection</span>

        {/* Action buttons */}
        <Show when={hasDataForFile()}>
          <button
            class="icon-btn-sm"
            title="Save as document"
            onClick={handleSaveAsDocument}
          >
            <HiOutlineDocumentText class="w-3.5 h-3.5" />
          </button>
          <button class="icon-btn-sm" title="Print" onClick={handlePrint}>
            <HiOutlinePrinter class="w-3.5 h-3.5" />
          </button>
        </Show>
        <Show when={hasAnyData()}>
          <button
            class="icon-btn-sm"
            title="Export all collections as CSV"
            onClick={handleExportAllCsv}
          >
            <HiOutlineTableCells class="w-3.5 h-3.5" />
          </button>
        </Show>
      </div>

      {/* Body */}
      <div class="flex-1 overflow-y-auto">
        <Show when={loading()}>
          <div class="flex items-center justify-center py-8 text-txt-muted text-xs">
            Loading…
          </div>
        </Show>

        <Show when={!loading()}>
          {/* File-specific collection data */}
          <Show
            when={hasDataForFile()}
            fallback={
              <div class="flex flex-col items-center justify-center py-8 text-txt-muted text-sm gap-2">
                <HiOutlineArchiveBoxArrowDown class="w-8 h-8 opacity-30" />
                <p>No collection data</p>
                <Show when={props.activeFile()}>
                  <p class="text-xs text-center px-4">
                    No evidence collection form has been created for this container yet.
                  </p>
                </Show>
                <Show when={hasAnyData()}>
                  <button
                    class="btn-text text-xs mt-2"
                    onClick={handleExportAllCsv}
                  >
                    <HiOutlineArrowDownTray class="w-3.5 h-3.5 inline mr-1" />
                    Export all {collections().length} collections as CSV
                  </button>
                </Show>
              </div>
            }
          >
            {/* Matching collections + items */}
            <For each={matchingCollections()}>
              {(col) => (
                <CollectionSection
                  collection={col}
                  items={itemsByCollection().get(col.id) ?? []}
                />
              )}
            </For>

            {/* Global export link */}
            <Show when={collections().length > matchingCollections().length}>
              <div class="border-t border-border px-3 py-2 text-xs text-txt-muted">
                <button class="btn-text text-xs" onClick={handleExportAllCsv}>
                  <HiOutlineArrowDownTray class="w-3.5 h-3.5 inline mr-1" />
                  Export all {collections().length} collections as CSV
                </button>
              </div>
            </Show>
          </Show>
        </Show>
      </div>
    </div>
  );
}

// =============================================================================
// CollectionSection — Collapsible section for one collection + its items
// =============================================================================

function CollectionSection(props: {
  collection: DbEvidenceCollection;
  items: DbCollectedItem[];
}) {
  const [open, setOpen] = createSignal(true);
  const col = () => props.collection;

  return (
    <div class="border-b border-border">
      {/* Collection header */}
      <button
        class="flex items-center gap-1.5 w-full px-3 py-2 text-left hover:bg-bg-hover transition-colors"
        onClick={() => setOpen(!open())}
      >
        <Show when={open()} fallback={<HiOutlineChevronRight class="w-3.5 h-3.5 text-txt-muted shrink-0" />}>
          <HiOutlineChevronDown class="w-3.5 h-3.5 text-txt-muted shrink-0" />
        </Show>
        <span class="text-xs font-medium text-txt flex-1 truncate">
          {col().caseNumber ? `Case ${col().caseNumber}` : "Collection"}
          {col().collectionDate ? ` — ${formatDate(col().collectionDate)}` : ""}
        </span>
        <StatusBadge status={col().status} />
      </button>

      <Show when={open()}>
        <div class="px-3 pb-2 flex flex-col gap-2">
          {/* Collection metadata */}
          <div class="flex flex-col gap-0.5 text-xs">
            <OptionalMetadataRow label="Officer" value={col().collectingOfficer} />
            <OptionalMetadataRow label="Location" value={col().collectionLocation} />
            <OptionalMetadataRow label="Authorization" value={col().authorization} />
            <OptionalMetadataRow label="Auth. Date" value={formatDate(col().authorizationDate)} />
            <OptionalMetadataRow label="Authority" value={col().authorizingAuthority} />
            <OptionalMetadataRow label="Conditions" value={col().conditions} />
            <OptionalMetadataRow label="Notes" value={col().documentationNotes} />
          </div>

          {/* Collected items */}
          <Show when={props.items.length > 0}>
            <div class="text-[10px] font-medium text-txt-muted uppercase tracking-wider mt-1">
              Collected Items ({props.items.length})
            </div>
            <For each={props.items}>
              {(item) => <CollectedItemCard item={item} />}
            </For>
          </Show>
        </div>
      </Show>
    </div>
  );
}

// =============================================================================
// CollectedItemCard — Compact card for one collected item
// =============================================================================

function CollectedItemCard(props: { item: DbCollectedItem }) {
  const [expanded, setExpanded] = createSignal(false);
  const item = () => props.item;

  return (
    <div class="border border-border rounded bg-bg-secondary">
      {/* Item header */}
      <button
        class="flex items-center gap-1.5 w-full px-2 py-1.5 text-left hover:bg-bg-hover transition-colors rounded"
        onClick={() => setExpanded(!expanded())}
      >
        <Show when={expanded()} fallback={<HiOutlineChevronRight class="w-3 h-3 text-txt-muted shrink-0" />}>
          <HiOutlineChevronDown class="w-3 h-3 text-txt-muted shrink-0" />
        </Show>
        <span class="text-xs text-txt flex-1 truncate">
          {item().itemNumber ? `#${item().itemNumber} — ` : ""}
          {item().description || "Unnamed item"}
        </span>
        <Show when={item().itemType}>
          <span class="text-[10px] text-txt-muted">{item().itemType}</span>
        </Show>
      </button>

      <Show when={expanded()}>
        <div class="px-2 pb-2 flex flex-col gap-0.5 text-xs border-t border-border">
          {/* Device identification */}
          <Show when={item().brand || item().make || item().model || item().serialNumber}>
            <div class="text-[10px] font-medium text-txt-muted uppercase tracking-wider mt-1">
              Device
            </div>
            <OptionalMetadataRow label="Brand" value={item().brand} />
            <OptionalMetadataRow label="Make" value={item().make} />
            <OptionalMetadataRow label="Model" value={item().model} />
            <OptionalMetadataRow label="Serial #" value={item().serialNumber} mono />
            <OptionalMetadataRow label="IMEI" value={item().imei} mono />
            <OptionalMetadataRow label="Device Type" value={item().deviceType} />
          </Show>

          {/* Forensic info */}
          <Show when={item().imageFormat || item().acquisitionMethod}>
            <div class="text-[10px] font-medium text-txt-muted uppercase tracking-wider mt-1">
              Forensic Acquisition
            </div>
            <OptionalMetadataRow label="Format" value={item().imageFormat} />
            <OptionalMetadataRow label="Method" value={item().acquisitionMethod} />
          </Show>

          {/* Location & condition */}
          <Show when={item().foundLocation || item().condition || item().packaging}>
            <div class="text-[10px] font-medium text-txt-muted uppercase tracking-wider mt-1">
              Collection Details
            </div>
            <OptionalMetadataRow label="Found" value={item().foundLocation} />
            <OptionalMetadataRow label="Condition" value={item().condition} />
            <OptionalMetadataRow label="Packaging" value={item().packaging} />
          </Show>

          {/* Notes */}
          <Show when={item().notes || item().storageNotes}>
            <div class="text-[10px] font-medium text-txt-muted uppercase tracking-wider mt-1">
              Notes
            </div>
            <Show when={item().notes}>
              <p class="text-xs text-txt-secondary whitespace-pre-wrap pl-2">{item().notes}</p>
            </Show>
            <Show when={item().storageNotes}>
              <OptionalMetadataRow label="Storage" value={item().storageNotes} />
            </Show>
          </Show>
        </div>
      </Show>
    </div>
  );
}

// =============================================================================
// Shared UI sub-components
// =============================================================================

// Local FieldRow and StatusBadge replaced with shared imports from
// viewerMetadata/shared.tsx for right-panel consistency.
// FieldRow → OptionalMetadataRow (w-20 left-aligned, text-xs, auto-hides when empty)
// StatusBadge → shared StatusBadge (consistent status coloring)

// =============================================================================
// Document builders
// =============================================================================

function buildTextDocument(
  cols: DbEvidenceCollection[],
  items: DbCollectedItem[],
  itemsByCol: Map<string, DbCollectedItem[]>,
  filename: string,
): string {
  const lines: string[] = [];
  lines.push("=" .repeat(72));
  lines.push("EVIDENCE COLLECTION SUMMARY");
  lines.push(`Evidence Container: ${filename}`);
  lines.push(`Generated: ${new Date().toLocaleString()}`);
  lines.push("=".repeat(72));
  lines.push("");

  for (const col of cols) {
    lines.push("-".repeat(72));
    lines.push(`COLLECTION: ${col.caseNumber || "(no case number)"}`);
    lines.push("-".repeat(72));
    if (col.collectionDate) lines.push(`  Date:          ${col.collectionDate}`);
    if (col.collectingOfficer) lines.push(`  Officer:       ${col.collectingOfficer}`);
    if (col.collectionLocation) lines.push(`  Location:      ${col.collectionLocation}`);
    if (col.authorization) lines.push(`  Authorization: ${col.authorization}`);
    if (col.authorizationDate) lines.push(`  Auth. Date:    ${col.authorizationDate}`);
    if (col.authorizingAuthority) lines.push(`  Authority:     ${col.authorizingAuthority}`);
    if (col.conditions) lines.push(`  Conditions:    ${col.conditions}`);
    if (col.documentationNotes) lines.push(`  Notes:         ${col.documentationNotes}`);
    lines.push(`  Status:        ${col.status || "draft"}`);
    lines.push("");

    const colItems = itemsByCol.get(col.id) ?? [];
    if (colItems.length > 0) {
      lines.push(`  COLLECTED ITEMS (${colItems.length}):`);
      lines.push("");
      for (const item of colItems) {
        lines.push(`    Item ${item.itemNumber || "—"}: ${item.description || "Unnamed"}`);
        if (item.itemType) lines.push(`      Type:          ${item.itemType}`);
        if (item.brand) lines.push(`      Brand:         ${item.brand}`);
        if (item.make) lines.push(`      Make:          ${item.make}`);
        if (item.model) lines.push(`      Model:         ${item.model}`);
        if (item.serialNumber) lines.push(`      Serial #:      ${item.serialNumber}`);
        if (item.imei) lines.push(`      IMEI:          ${item.imei}`);
        if (item.deviceType) lines.push(`      Device Type:   ${item.deviceType}`);
        if (item.imageFormat) lines.push(`      Image Format:  ${item.imageFormat}`);
        if (item.acquisitionMethod) lines.push(`      Acq. Method:   ${item.acquisitionMethod}`);
        if (item.foundLocation) lines.push(`      Found At:      ${item.foundLocation}`);
        if (item.condition) lines.push(`      Condition:     ${item.condition}`);
        if (item.packaging) lines.push(`      Packaging:     ${item.packaging}`);
        if (item.notes) lines.push(`      Notes:         ${item.notes}`);
        if (item.storageNotes) lines.push(`      Storage:       ${item.storageNotes}`);
        lines.push("");
      }
    }
  }

  lines.push("=".repeat(72));
  lines.push(`Total Collections: ${cols.length}`);
  lines.push(`Total Items: ${items.length}`);
  lines.push("=".repeat(72));
  return lines.join("\n");
}

function buildHtmlDocument(
  cols: DbEvidenceCollection[],
  items: DbCollectedItem[],
  itemsByCol: Map<string, DbCollectedItem[]>,
  filename: string,
): string {
  const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

  const itemRows = (colItems: DbCollectedItem[]) =>
    colItems
      .map(
        (item) => `
          <tr>
            <td>${esc(item.itemNumber || "—")}</td>
            <td>${esc(item.description || "")}</td>
            <td>${esc(item.itemType || "")}</td>
            <td>${esc([item.brand, item.make, item.model].filter(Boolean).join(" / ") || "")}</td>
            <td style="font-family:monospace;font-size:11px;">${esc(item.serialNumber || "")}</td>
            <td>${esc(item.imageFormat || "")}</td>
            <td>${esc(item.acquisitionMethod || "")}</td>
            <td>${esc(item.condition || "")}</td>
            <td>${esc(item.notes || "")}</td>
          </tr>`,
      )
      .join("");

  const collectionSections = cols
    .map(
      (col) => `
        <div class="collection">
          <h3>Collection: ${esc(col.caseNumber || "(no case number)")}</h3>
          <table class="meta">
            <tr><td class="label">Date</td><td>${esc(col.collectionDate || "—")}</td></tr>
            <tr><td class="label">Officer</td><td>${esc(col.collectingOfficer || "—")}</td></tr>
            <tr><td class="label">Location</td><td>${esc(col.collectionLocation || "—")}</td></tr>
            <tr><td class="label">Authorization</td><td>${esc(col.authorization || "—")}</td></tr>
            ${col.authorizingAuthority ? `<tr><td class="label">Authority</td><td>${esc(col.authorizingAuthority)}</td></tr>` : ""}
            ${col.conditions ? `<tr><td class="label">Conditions</td><td>${esc(col.conditions)}</td></tr>` : ""}
            <tr><td class="label">Status</td><td>${esc(col.status || "draft")}</td></tr>
          </table>
          ${
            (itemsByCol.get(col.id) ?? []).length > 0
              ? `
                <h4>Collected Items (${(itemsByCol.get(col.id) ?? []).length})</h4>
                <table class="items">
                  <thead><tr>
                    <th>#</th><th>Description</th><th>Type</th><th>Device</th>
                    <th>Serial #</th><th>Format</th><th>Method</th><th>Condition</th><th>Notes</th>
                  </tr></thead>
                  <tbody>${itemRows(itemsByCol.get(col.id) ?? [])}</tbody>
                </table>`
              : ""
          }
        </div>`,
    )
    .join("");

  return `<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <title>Evidence Collection Summary — ${esc(filename)}</title>
  <style>
    body { font-family: system-ui, -apple-system, sans-serif; margin: 24px; color: #1a1a1a; }
    h2 { margin: 0 0 4px; font-size: 18px; }
    .subtitle { color: #666; font-size: 12px; margin-bottom: 16px; }
    .collection { margin-bottom: 24px; border: 1px solid #ddd; border-radius: 8px; padding: 16px; }
    .collection h3 { margin: 0 0 8px; font-size: 15px; }
    .collection h4 { margin: 12px 0 6px; font-size: 13px; color: #444; }
    table.meta { border-collapse: collapse; margin-bottom: 8px; }
    table.meta td { padding: 2px 12px 2px 0; font-size: 12px; vertical-align: top; }
    table.meta .label { font-weight: 600; color: #555; min-width: 100px; }
    table.items { border-collapse: collapse; width: 100%; font-size: 11px; }
    table.items th, table.items td { border: 1px solid #ddd; padding: 4px 8px; text-align: left; }
    table.items th { background: #f5f5f5; font-weight: 600; }
    .footer { margin-top: 16px; font-size: 11px; color: #999; border-top: 1px solid #eee; padding-top: 8px; }
    @media print { body { margin: 12px; } .collection { break-inside: avoid; } }
  </style>
</head>
<body>
  <h2>Evidence Collection Summary</h2>
  <div class="subtitle">
    Evidence Container: ${esc(filename)}<br>
    Generated: ${new Date().toLocaleString()}
  </div>
  ${collectionSections}
  <div class="footer">
    Total Collections: ${cols.length} &bull; Total Items: ${items.length}
  </div>
</body>
</html>`;
}

// =============================================================================
// CSV spreadsheet builder (all collections)
// =============================================================================

const CSV_COLUMNS = [
  "Collection ID",
  "Case Number",
  "Collection Date",
  "Collecting Officer",
  "Collection Location",
  "Authorization",
  "Status",
  "Item Number",
  "Description",
  "Item Type",
  "Device Type",
  "Brand",
  "Make",
  "Model",
  "Serial Number",
  "IMEI",
  "Image Format",
  "Acquisition Method",
  "Found Location",
  "Condition",
  "Packaging",
  "Notes",
  "Storage Notes",
  "Evidence File ID",
];

function buildCsvSpreadsheet(
  cols: DbEvidenceCollection[],
  items: DbCollectedItem[],
): string {
  const colMap = new Map(cols.map((c) => [c.id, c]));
  const rows: string[] = [CSV_COLUMNS.join(",")];

  for (const item of items) {
    const col = colMap.get(item.collectionId);
    rows.push(
      [
        csvEsc(item.collectionId),
        csvEsc(col?.caseNumber ?? ""),
        csvEsc(col?.collectionDate ?? ""),
        csvEsc(col?.collectingOfficer ?? ""),
        csvEsc(col?.collectionLocation ?? ""),
        csvEsc(col?.authorization ?? ""),
        csvEsc(col?.status ?? ""),
        csvEsc(item.itemNumber),
        csvEsc(item.description),
        csvEsc(item.itemType),
        csvEsc(item.deviceType ?? ""),
        csvEsc(item.brand ?? ""),
        csvEsc(item.make ?? ""),
        csvEsc(item.model ?? ""),
        csvEsc(item.serialNumber ?? ""),
        csvEsc(item.imei ?? ""),
        csvEsc(item.imageFormat ?? ""),
        csvEsc(item.acquisitionMethod ?? ""),
        csvEsc(item.foundLocation),
        csvEsc(item.condition),
        csvEsc(item.packaging),
        csvEsc(item.notes ?? ""),
        csvEsc(item.storageNotes ?? ""),
        csvEsc(item.evidenceFileId ?? ""),
      ].join(","),
    );
  }

  return rows.join("\n");
}

// =============================================================================
// Helpers
// =============================================================================

function formatDate(iso?: string): string | undefined {
  if (!iso) return undefined;
  try {
    const d = new Date(iso);
    if (isNaN(d.getTime())) return iso;
    return d.toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  } catch {
    return iso;
  }
}

function csvEsc(s: string): string {
  if (s.includes(",") || s.includes('"') || s.includes("\n")) {
    return `"${s.replace(/"/g, '""')}"`;
  }
  return s;
}

function sanitizeFilename(name: string): string {
  return name.replace(/[^a-zA-Z0-9._-]/g, "_").replace(/_{2,}/g, "_");
}
