// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ImportAcquisitionsWizard — Modal for scanning a directory for
 * `.ffx-companion.json` sidecar files and importing the referenced
 * acquisitions into the current project.
 *
 * Single-view flow: directory picker → inline results → inline summary.
 * No step navigation — content appears progressively.
 */

import { Component, createSignal, createMemo, Show, For } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import {
  HiOutlineArchiveBoxArrowDown,
  HiOutlineXMark,
  HiOutlineFolderOpen,
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
  HiOutlineArrowPath,
  HiOutlineInformationCircle,
} from "../icons";
import { useImportAcquisitions } from "../../hooks/useImportAcquisitions";
import type { ImportAcquisitionsOptions } from "../../hooks/useImportAcquisitions";
import type { DiscoveredAcquisition } from "../../api/importAcquisitions";
import type { ImportResult } from "../../api/importAcquisitions";
import type { DiscoveredFile } from "../../types/container";
import { formatBytes } from "../../utils";
import { isTauri } from "../../utils/platform";

// ─── Types ──────────────────────────────────────────────────────────────────

export interface ImportAcquisitionsWizardProps {
  onClose: () => void;
  /** Called for each imported file (add to evidence tree) */
  onFileImported?: (file: DiscoveredFile) => void;
  /** Called after a successful import batch so callers can mark project state dirty. */
  onImportComplete?: (result: ImportResult) => void;
  /** Set of output paths already in the project (to detect duplicates) */
  knownPaths?: Set<string>;
}

// ─── Helpers ────────────────────────────────────────────────────────────────

const TYPE_BADGES: Record<string, { label: string; color: string }> = {
  e01: { label: "E01", color: "text-type-e01" },
  l01: { label: "L01", color: "text-type-l01" },
  aff4: { label: "AFF4", color: "text-accent" },
  raw: { label: "RAW", color: "text-type-raw" },
  archive: { label: "7z", color: "text-type-archive" },
  file_copy: { label: "Copy", color: "text-txt-muted" },
  memory: { label: "MEM", color: "text-warning" },
  triage: { label: "Triage", color: "text-info" },
};

const BROWSER_DIRECTORY_PICKER_MESSAGE =
  "Acquisition directory scanning is available in the desktop app.";

function getTypeBadge(acquisitionType: string) {
  return TYPE_BADGES[acquisitionType] || { label: acquisitionType, color: "text-txt-muted" };
}

function extractFilename(path: string): string {
  return path.split(/[/\\]/).pop() || path;
}

function formatDate(iso: string): string {
  if (!iso) return "—";
  try {
    return new Date(iso).toLocaleString();
  } catch {
    return iso;
  }
}

// ─── Component ──────────────────────────────────────────────────────────────

export const ImportAcquisitionsWizard: Component<ImportAcquisitionsWizardProps> = (props) => {
  const [scanDir, setScanDir] = createSignal("");
  const [hasScanned, setHasScanned] = createSignal(false);
  const [browseMessage, setBrowseMessage] = createSignal<string | null>(null);

  const importer = useImportAcquisitions();

  const selectedCount = createMemo(() => importer.selected().size);
  const totalCount = createMemo(() => importer.results().length);
  const hasResults = createMemo(() => totalCount() > 0);
  const isDone = createMemo(() => !!importer.importResult());

  const alreadyImported = createMemo(() => {
    const known = props.knownPaths || new Set<string>();
    return new Set(
      importer.results()
        .filter(a => known.has(a.companion.output.primaryPath))
        .map(a => a.companionPath),
    );
  });

  // ── Actions ──

  async function handleBrowse() {
    if (!isTauri) {
      setBrowseMessage(BROWSER_DIRECTORY_PICKER_MESSAGE);
      return;
    }

    const dir = await open({ directory: true, title: "Select acquisition directory" });
    if (typeof dir === "string") {
      setScanDir(dir);
      // Auto-scan on directory selection
      await doScan(dir);
    }
  }

  async function doScan(dir: string) {
    if (!dir) return;
    if (!isTauri) {
      setBrowseMessage(BROWSER_DIRECTORY_PICKER_MESSAGE);
      return;
    }
    setHasScanned(true);
    await importer.scan(dir);
  }

  async function handleScan() {
    await doScan(scanDir());
  }

  async function handleImport() {
    const known = props.knownPaths || new Set<string>();
    const options: ImportAcquisitionsOptions = {
      onFileImported: props.onFileImported,
    };
    const result = await importer.importSelected(known, options);
    if (result.imported > 0) {
      props.onImportComplete?.(result);
    }
  }

  function handleClose() {
    importer.reset();
    props.onClose();
  }

  return (
    <div class="modal-overlay" onClick={(e) => { if (e.target === e.currentTarget) handleClose(); }}>
      <div class="modal-content w-[680px] max-h-[85vh] flex flex-col">
        {/* ── Header ── */}
        <div class="modal-header">
          <div class="flex items-center gap-2">
            <HiOutlineArchiveBoxArrowDown class="w-icon-base h-icon-base text-accent" />
            <h2 class="text-lg font-medium text-txt">Import Acquisitions</h2>
          </div>
          <button class="icon-btn-sm" onClick={handleClose} title="Close">
            <HiOutlineXMark class="w-5 h-5" />
          </button>
        </div>

        {/* ── Body ── */}
        <div class="modal-body overflow-y-auto flex-1 space-y-4">

          {/* ── Directory picker (always visible unless done) ── */}
          <Show when={!isDone()}>
            <div class="space-y-2">
              <p class="text-sm text-txt-secondary">
                Select a directory to scan for acquisition companion files.
              </p>
              <div class="flex items-center gap-2">
                <input
                  class="input flex-1"
                  type="text"
                  value={scanDir()}
                  onInput={(e) => setScanDir(e.currentTarget.value)}
                  placeholder="Path to acquisition directory…"
                  onKeyDown={(e) => { if (e.key === "Enter") handleScan(); }}
                />
                <button class="btn btn-secondary" onClick={handleBrowse}>
                  <HiOutlineFolderOpen class="w-icon-sm h-icon-sm" />
                  Browse
                </button>
                <button
                  class="btn btn-secondary"
                  onClick={handleScan}
                  disabled={!scanDir() || importer.scanning()}
                  title="Scan directory"
                >
                  <Show when={importer.scanning()} fallback={<HiOutlineArrowPath class="w-icon-sm h-icon-sm" />}>
                    <span class="animate-spin"><HiOutlineArrowPath class="w-icon-sm h-icon-sm" /></span>
                  </Show>
                </button>
              </div>
            </div>
          </Show>

          {/* ── Error ── */}
          <Show when={browseMessage()}>
            {(message) => (
              <div class="flex items-center gap-2 p-3 rounded-lg bg-warning/10 text-warning text-sm">
                <HiOutlineInformationCircle class="w-icon-sm h-icon-sm shrink-0" />
                <span>{message()}</span>
              </div>
            )}
          </Show>

          <Show when={importer.error()}>
            <div class="flex items-center gap-2 p-3 rounded-lg bg-error/10 text-error text-sm">
              <HiOutlineExclamationTriangle class="w-icon-sm h-icon-sm shrink-0" />
              <span>{importer.error()}</span>
            </div>
          </Show>

          {/* ── Empty state (scanned but nothing found) ── */}
          <Show when={hasScanned() && !importer.scanning() && !hasResults() && !importer.error()}>
            <div class="flex flex-col items-center justify-center py-8 text-txt-muted text-sm gap-2">
              <HiOutlineArchiveBoxArrowDown class="w-8 h-8 opacity-30" />
              <span>No acquisitions found in this directory.</span>
            </div>
          </Show>

          {/* ── Results list (visible after scan finds items, hidden after import) ── */}
          <Show when={hasResults() && !isDone()}>
            <div class="space-y-3">
              {/* Selection bar */}
              <div class="flex items-center justify-between">
                <span class="text-xs text-txt-muted">
                  Found {totalCount()} — {selectedCount()} selected
                </span>
                <div class="flex items-center gap-2">
                  <button class="btn-text text-xs" onClick={() => importer.selectAll()}>
                    Select All
                  </button>
                  <button class="btn-text text-xs" onClick={() => importer.deselectAll()}>
                    Deselect All
                  </button>
                </div>
              </div>

              {/* Acquisition cards */}
              <div class="space-y-2">
                <For each={importer.results()}>
                  {(acq) => (
                    <AcquisitionCard
                      acquisition={acq}
                      isSelected={importer.selected().has(acq.companionPath)}
                      isAlreadyImported={alreadyImported().has(acq.companionPath)}
                      onToggle={() => importer.toggleSelect(acq.companionPath)}
                    />
                  )}
                </For>
              </div>
            </div>
          </Show>

          {/* ── Import summary (replaces results after import) ── */}
          <Show when={isDone()}>
            {(_) => {
              const result = importer.importResult()!;
              return (
                <div class="space-y-4">
                  <div class="flex items-center gap-3 p-4 rounded-lg bg-success/10">
                    <HiOutlineCheckCircle class="w-8 h-8 text-success shrink-0" />
                    <div>
                      <h3 class="text-base font-medium text-txt">Import Complete</h3>
                      <p class="text-sm text-txt-secondary mt-1">
                        {result.imported} acquisition{result.imported !== 1 ? "s" : ""} imported
                        {result.skipped > 0 && `, ${result.skipped} skipped`}
                      </p>
                    </div>
                  </div>

                  <Show when={result.errors.length > 0}>
                    <div class="space-y-2">
                      <h4 class="text-sm font-medium text-error">
                        {result.errors.length} error{result.errors.length !== 1 ? "s" : ""}
                      </h4>
                      <div class="space-y-1">
                        <For each={result.errors}>
                          {(err) => (
                            <div class="flex items-start gap-2 p-2 rounded bg-error/5 text-xs text-error">
                              <HiOutlineExclamationTriangle class="w-4 h-4 shrink-0 mt-0.5" />
                              <span class="break-all">{err}</span>
                            </div>
                          )}
                        </For>
                      </div>
                    </div>
                  </Show>
                </div>
              );
            }}
          </Show>
        </div>

        {/* ── Footer ── */}
        <div class="modal-footer justify-end">
          <Show when={isDone()} fallback={
            <>
              <button class="btn btn-secondary" onClick={handleClose}>Cancel</button>
              <Show when={hasResults()}>
                <button
                  class="btn btn-primary"
                  onClick={handleImport}
                  disabled={selectedCount() === 0 || importer.importing()}
                >
                  <Show when={importer.importing()} fallback={
                    <>Import {selectedCount()} Acquisition{selectedCount() !== 1 ? "s" : ""}</>
                  }>
                    Importing…
                  </Show>
                </button>
              </Show>
            </>
          }>
            <button class="btn btn-primary" onClick={handleClose}>Done</button>
          </Show>
        </div>
      </div>
    </div>
  );
};

// ─── Acquisition Card ───────────────────────────────────────────────────────

interface AcquisitionCardProps {
  acquisition: DiscoveredAcquisition;
  isSelected: boolean;
  isAlreadyImported: boolean;
  onToggle: () => void;
}

const AcquisitionCard: Component<AcquisitionCardProps> = (props) => {
  const c = () => props.acquisition.companion;
  const badge = () => getTypeBadge(c().acquisitionType);
  const filename = () => extractFilename(c().output.primaryPath);

  return (
    <div
      class="flex items-start gap-3 p-3 rounded-lg border transition-colors cursor-pointer"
      classList={{
        "border-accent/40 bg-accent/5": props.isSelected && !props.isAlreadyImported,
        "border-border bg-bg-secondary": !props.isSelected && !props.isAlreadyImported,
        "border-border/50 bg-bg-secondary opacity-60": props.isAlreadyImported,
      }}
      onClick={() => { if (!props.isAlreadyImported) props.onToggle(); }}
    >
      {/* Checkbox */}
      <div class="pt-0.5">
        <input
          type="checkbox"
          checked={props.isSelected}
          disabled={props.isAlreadyImported}
          onChange={() => props.onToggle()}
          onClick={(e) => e.stopPropagation()}
          class="accent-accent"
        />
      </div>

      {/* Content */}
      <div class="flex-1 min-w-0 space-y-1">
        {/* Row 1: filename + type badge + warnings */}
        <div class="flex items-center gap-2">
          <span class="text-sm font-medium text-txt truncate">{filename()}</span>
          <span class={`text-2xs font-semibold uppercase ${badge().color}`}>
            {badge().label}
          </span>
          <Show when={props.isAlreadyImported}>
            <span class="text-2xs text-warning font-medium">Already imported</span>
          </Show>
          <Show when={!props.acquisition.outputExists}>
            <span class="text-2xs text-error font-medium">File missing</span>
          </Show>
        </div>

        {/* Row 2: metadata chips */}
        <div class="flex items-center gap-3 text-xs text-txt-muted flex-wrap">
          <span>{formatBytes(c().output.totalBytes)}</span>
          <Show when={c().timing.completedAt}>
            <span>{formatDate(c().timing.completedAt)}</span>
          </Show>
          <Show when={c().case?.caseNumber}>
            <span>Case: {c().case.caseNumber}</span>
          </Show>
          <Show when={c().case?.examiner}>
            <span>By: {c().case.examiner}</span>
          </Show>
        </div>

        {/* Row 3: path */}
        <div class="text-2xs text-txt-muted font-mono truncate" title={c().output.primaryPath}>
          {c().output.primaryPath}
        </div>

        {/* Row 4: hashes */}
        <Show when={c().hashes.md5 || c().hashes.sha1 || c().hashes.sha256}>
          <div class="flex items-center gap-1 mt-0.5">
            <HiOutlineInformationCircle class="w-3 h-3 text-txt-muted shrink-0" />
            <span class="text-2xs text-txt-muted">
              Hashes:
              {c().hashes.md5 ? " MD5" : ""}
              {c().hashes.sha1 ? " SHA-1" : ""}
              {c().hashes.sha256 ? " SHA-256" : ""}
            </span>
          </div>
        </Show>
      </div>
    </div>
  );
};

export default ImportAcquisitionsWizard;
