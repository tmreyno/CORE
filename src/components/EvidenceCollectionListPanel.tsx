// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EvidenceCollectionListPanel — Non-modal, tab-based browse view for evidence
 * collections. Replaces EvidenceCollectionListModal.
 *
 * Shows all evidence collections for the current case/project as cards with
 * summary info and action buttons: review, edit, export (multi-format), delete.
 */

import { createSignal, createMemo, onMount, Show, For, Component } from "solid-js";
import {
  HiOutlineArchiveBoxArrowDown,
  HiOutlinePlus,
  HiOutlineEye,
  HiOutlinePencil,
  HiOutlineTrash,
  HiOutlineClipboardDocumentList,
  HiOutlineCalendarDays,
  HiOutlineUser,
  HiOutlineLockClosed,
  HiOutlineCheckBadge,
  HiOutlineArrowUpTray,
  HiOutlineChevronDown,
  HiOutlineMagnifyingGlass,
  HiOutlinePrinter,
} from "./icons";
import {
  loadAllEvidenceCollections,
  deleteEvidenceCollection,
} from "./report/wizard/cocDbSync";
import { printDocument } from "./document/documentHelpers";
import type { EvidenceExportFormat } from "./report/wizard/cocDbSync";
import type { DbEvidenceCollection } from "../types/projectDb";
import { logger } from "../utils/logger";

const log = logger.scope("EvidenceCollectionListPanel");

/** Escape HTML entities */
function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");
}

export interface EvidenceCollectionListPanelProps {
  /** Case number for filtering */
  caseNumber?: string;
  /** Project name (shown in header) */
  projectName?: string;
  /** Called when user wants to open a collection for editing/review */
  onOpenCollection: (collectionId: string, readOnly: boolean) => void;
  /** Called when user wants to create a new collection */
  onNewCollection: () => void;
  /** Called when user wants to export a collection */
  onExport?: (collectionId: string, format: EvidenceExportFormat) => void;
}

/** Format an ISO date string for display */
function formatDate(iso?: string): string {
  if (!iso) return "—";
  try {
    const d = new Date(iso);
    return d.toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  } catch {
    return iso;
  }
}

/** Status badge component */
function StatusBadge(props: { status: string }) {
  switch (props.status) {
    case "complete":
      return (
        <span class="badge badge-success flex items-center gap-1">
          <HiOutlineCheckBadge class="w-3 h-3" />
          Complete
        </span>
      );
    case "locked":
      return (
        <span class="badge badge-warning flex items-center gap-1">
          <HiOutlineLockClosed class="w-3 h-3" />
          Locked
        </span>
      );
    default:
      return (
        <span
          class="badge flex items-center gap-1"
          style={{
            background: "var(--color-bg-hover)",
            color: "var(--color-txt-muted)",
          }}
        >
          Draft
        </span>
      );
  }
}

export const EvidenceCollectionListPanel: Component<EvidenceCollectionListPanelProps> = (props) => {
  const [collections, setCollections] = createSignal<DbEvidenceCollection[]>([]);
  const [loading, setLoading] = createSignal(true);
  const [deletingId, setDeletingId] = createSignal<string | null>(null);
  const [exportMenuId, setExportMenuId] = createSignal<string | null>(null);
  const [searchQuery, setSearchQuery] = createSignal("");

  // Filtered collections
  const filteredCollections = createMemo(() => {
    const q = searchQuery().toLowerCase().trim();
    if (!q) return collections();
    return collections().filter((c) => {
      return (
        (c.collectingOfficer || "").toLowerCase().includes(q) ||
        (c.caseNumber || "").toLowerCase().includes(q) ||
        (c.authorization || "").toLowerCase().includes(q) ||
        (c.status || "draft").toLowerCase().includes(q) ||
        formatDate(c.collectionDate).toLowerCase().includes(q)
      );
    });
  });

  const refresh = async () => {
    setLoading(true);
    try {
      const result = await loadAllEvidenceCollections(props.caseNumber);
      setCollections(result);
    } catch (e) {
      log.error("Failed to load evidence collections:", e);
    } finally {
      setLoading(false);
    }
  };

  onMount(refresh);

  const handleDelete = async (id: string) => {
    setDeletingId(id);
    const ok = await deleteEvidenceCollection(id);
    if (ok) {
      setCollections((prev) => prev.filter((c) => c.id !== id));
    }
    setDeletingId(null);
  };

  const handleExport = (col: DbEvidenceCollection, format: EvidenceExportFormat) => {
    setExportMenuId(null);
    if (props.onExport) {
      props.onExport(col.id, format);
    }
  };

  /** Build printable HTML summary of all visible collections */
  const handlePrintAll = () => {
    const items = filteredCollections();
    if (items.length === 0) return;

    const rows = items
      .map(
        (c) =>
          `<tr>
          <td style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">${esc(c.status || "draft")}</td>
          <td style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">${esc(formatDate(c.collectionDate))}</td>
          <td style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">${esc(c.collectingOfficer || "—")}</td>
          <td style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">${esc(c.caseNumber || "—")}</td>
          <td style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">${esc(c.authorization || "—")}</td>
          <td style="border:1px solid #ccc;padding:4px 8px;font-size:12px;text-align:center;">${c.itemCount ?? 0}</td>
          <td style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">${esc(formatDate(c.createdAt))}</td>
        </tr>`,
      )
      .join("");
    const html = `<!DOCTYPE html><html><head><title>Evidence Collections</title><style>body{font-family:system-ui,sans-serif;margin:20px}table{border-collapse:collapse;width:100%}th{background:#f5f5f5;text-align:left}@media print{body{margin:0}}</style></head><body><h2>Evidence Collections${props.projectName ? ` — ${esc(props.projectName)}` : ""}</h2><p style="font-size:13px;color:#666;">Printed: ${new Date().toLocaleString()} | ${items.length} collection(s)</p><table><thead><tr><th style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">Status</th><th style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">Date</th><th style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">Officer</th><th style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">Case #</th><th style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">Authorization</th><th style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">Items</th><th style="border:1px solid #ccc;padding:4px 8px;font-size:12px;">Created</th></tr></thead><tbody>${rows}</tbody></table></body></html>`;
    printDocument(html);
  };

  return (
    <div class="flex flex-col h-full overflow-hidden bg-bg">
      {/* Header toolbar */}
      <div class="flex items-center justify-between px-4 py-2 border-b border-border bg-bg-secondary shrink-0">
        <div class="flex items-center gap-2">
          <HiOutlineClipboardDocumentList class="w-4 h-4 text-accent" />
          <div>
            <h2 class="text-xs font-semibold">Evidence Collections</h2>
            <p class="text-[11px] text-txt-muted">
              Browse and manage collection records
              <Show when={props.projectName}>
                <span> — {props.projectName}</span>
              </Show>
            </p>
          </div>
        </div>
        <div class="flex items-center gap-2">
          <button
            class="icon-btn-sm"
            title="Print collections"
            disabled={filteredCollections().length === 0}
            onClick={handlePrintAll}
          >
            <HiOutlinePrinter class="w-4 h-4" />
          </button>
          <button class="btn btn-primary" onClick={() => props.onNewCollection()}>
            <HiOutlinePlus class="w-4 h-4" />
            New Collection
          </button>
        </div>
      </div>

      {/* Search bar */}
      <div class="px-4 py-2 border-b border-border bg-bg-secondary shrink-0">
        <div class="relative max-w-md">
          <HiOutlineMagnifyingGlass class="absolute left-2.5 top-1/2 -translate-y-1/2 w-4 h-4 text-txt-muted" />
          <input
            type="text"
            placeholder="Search by officer, case #, status, date…"
            value={searchQuery()}
            onInput={(e) => setSearchQuery(e.currentTarget.value)}
            class="input-sm w-full pl-8"
          />
        </div>
      </div>

      {/* Body */}
      <div class="flex-1 overflow-y-auto p-4">
        <Show
          when={!loading()}
          fallback={
            <div class="flex items-center justify-center py-8">
              <div class="animate-pulse-slow text-txt-muted text-sm">Loading…</div>
            </div>
          }
        >
          <Show
            when={filteredCollections().length > 0}
            fallback={
              <div class="flex flex-col items-center justify-center py-10 gap-3">
                <HiOutlineArchiveBoxArrowDown class="w-8 h-8 text-txt-muted opacity-30" />
                <Show when={searchQuery().trim()} fallback={
                  <>
                    <p class="text-txt-muted text-sm">No evidence collections yet</p>
                    <button class="btn btn-primary" onClick={() => props.onNewCollection()}>
                      <HiOutlinePlus class="w-4 h-4" />
                      Create First Collection
                    </button>
                  </>
                }>
                  <p class="text-txt-muted text-sm">No collections match "{searchQuery()}"</p>
                </Show>
              </div>
            }
          >
            <div class="flex flex-col gap-2 max-w-3xl mx-auto">
              <For each={filteredCollections()}>
                {(col) => (
                  <div class="card-interactive p-3">
                    <div class="flex items-start justify-between gap-3">
                      {/* Left: collection info */}
                      <div class="flex-1 min-w-0">
                        <div class="flex items-center gap-2 mb-1">
                          <StatusBadge status={col.status || "draft"} />
                          <span class="text-[11px] text-txt-muted font-mono">{col.id}</span>
                          <Show when={col.itemCount != null && col.itemCount > 0}>
                            <span class="text-[11px] text-txt-muted">
                              {col.itemCount} {col.itemCount === 1 ? "item" : "items"}
                            </span>
                          </Show>
                        </div>

                        <div class="flex items-center gap-3 text-xs text-txt-secondary">
                          <Show when={col.collectionDate}>
                            <span class="flex items-center gap-1">
                              <HiOutlineCalendarDays class="w-3.5 h-3.5 text-txt-muted" />
                              {formatDate(col.collectionDate)}
                            </span>
                          </Show>
                          <Show when={col.collectingOfficer}>
                            <span class="flex items-center gap-1">
                              <HiOutlineUser class="w-3.5 h-3.5 text-txt-muted" />
                              {col.collectingOfficer}
                            </span>
                          </Show>
                        </div>

                        <Show when={col.authorization}>
                          <p class="text-xs text-txt-muted mt-1 truncate">Auth: {col.authorization}</p>
                        </Show>
                      </div>

                      {/* Right: action buttons */}
                      <div class="flex items-center gap-1.5 flex-shrink-0">
                        {/* Review (read-only) */}
                        <button class="icon-btn-sm" title="Review" onClick={() => props.onOpenCollection(col.id, true)}>
                          <HiOutlineEye class="w-4 h-4" />
                        </button>

                        {/* Edit (if not locked) */}
                        <Show when={(col.status || "draft") !== "locked"}>
                          <button class="icon-btn-sm" title="Edit" onClick={() => props.onOpenCollection(col.id, false)}>
                            <HiOutlinePencil class="w-4 h-4" />
                          </button>
                        </Show>

                        {/* Export dropdown */}
                        <Show when={props.onExport}>
                          <div class="relative">
                            <button
                              class="icon-btn-sm"
                              title="Export"
                              onClick={() => setExportMenuId(exportMenuId() === col.id ? null : col.id)}
                            >
                              <HiOutlineArrowUpTray class="w-4 h-4" />
                              <HiOutlineChevronDown class="w-3 h-3" />
                            </button>
                            <Show when={exportMenuId() === col.id}>
                              <div class="absolute right-0 top-full mt-1 bg-bg-panel border border-border rounded-lg shadow-lg py-1 z-dropdown min-w-[160px]">
                                <button class="w-full text-left px-3 py-1.5 text-sm hover:bg-bg-hover" onClick={() => handleExport(col, 'pdf')}>PDF Document</button>
                                <button class="w-full text-left px-3 py-1.5 text-sm hover:bg-bg-hover" onClick={() => handleExport(col, 'xlsx')}>Excel Spreadsheet</button>
                                <button class="w-full text-left px-3 py-1.5 text-sm hover:bg-bg-hover" onClick={() => handleExport(col, 'csv')}>CSV File</button>
                                <button class="w-full text-left px-3 py-1.5 text-sm hover:bg-bg-hover" onClick={() => handleExport(col, 'html')}>HTML Report</button>
                              </div>
                            </Show>
                          </div>
                        </Show>

                        {/* Delete (draft only) */}
                        <Show when={(col.status || "draft") === "draft" && deletingId() !== col.id}>
                          <button
                            class="icon-btn-sm text-error hover:text-error"
                            title="Delete"
                            onClick={() => handleDelete(col.id)}
                          >
                            <HiOutlineTrash class="w-4 h-4" />
                          </button>
                        </Show>
                      </div>
                    </div>

                    {/* Footer: timestamps */}
                    <div class="flex items-center gap-4 mt-2 pt-2 border-t border-border text-xs text-txt-muted">
                      <span>Created: {formatDate(col.createdAt)}</span>
                      <span>Modified: {formatDate(col.modifiedAt)}</span>
                      <Show when={col.caseNumber}>
                        <span>Case: {col.caseNumber}</span>
                      </Show>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </Show>
      </div>

      {/* Footer */}
      <div class="flex items-center justify-between px-4 py-2 border-t border-border bg-bg-secondary shrink-0">
        <div class="text-xs text-txt-muted">
          <Show
            when={searchQuery().trim()}
            fallback={`${collections().length} ${collections().length === 1 ? "collection" : "collections"}`}
          >
            {filteredCollections().length} / {collections().length} collections
          </Show>
        </div>
      </div>
    </div>
  );
};
