// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// TriageMode — Forensic triage collection + credential scanning UI

import { Show, For, onMount, createMemo } from "solid-js";
import type { Accessor, Setter } from "solid-js";
import {
  HiOutlineShieldCheck,
  HiOutlineShieldExclamation,
  HiOutlineKey,
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
  HiOutlineFolder,
  HiOutlineFolderOpen,
  HiOutlineMagnifyingGlass,
  HiOutlineTag,
} from "../icons";
import type {
  TriageProfile,
  TriageCategory,
  TriageProgress,
  TriageResult,
  SecretFinding,
} from "../../api/triage";
import { systemCommands } from "../../api/commands";

// --- Props ---

export interface TriageModeProps {
  triageProfiles: Accessor<TriageProfile[]>;
  triageCategories: Accessor<TriageCategory[]>;
  triageProfilesLoading: Accessor<boolean>;
  selectedTriageProfile: Accessor<string>;
  setSelectedTriageProfile: (profileId: string) => void;
  selectedTriageCategories: Accessor<string[]>;
  toggleTriageCategory: (categoryId: string) => void;
  triageScanForSecrets: Accessor<boolean>;
  setTriageScanForSecrets: Setter<boolean>;
  triageProgress: Accessor<TriageProgress | null>;
  triageResult: Accessor<TriageResult | null>;
  isCollecting: Accessor<boolean>;
  onLoadProfiles: () => void;
}

// --- Helpers ---

function formatSize(bytes: number): string {
  if (bytes >= 1024 * 1024 * 1024) {
    return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GB`;
  }
  if (bytes >= 1024 * 1024) {
    return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
  }
  if (bytes >= 1024) {
    return `${(bytes / 1024).toFixed(1)} KB`;
  }
  return `${bytes} B`;
}

function confidenceColor(confidence: string): string {
  switch (confidence) {
    case "high":
      return "text-error";
    case "medium":
      return "text-warning";
    default:
      return "text-txt-muted";
  }
}

function confidenceBg(confidence: string): string {
  switch (confidence) {
    case "high":
      return "bg-red-500/10 border-red-500/30";
    case "medium":
      return "bg-amber-500/10 border-amber-500/30";
    default:
      return "bg-bg-secondary border-border/30";
  }
}

const CATEGORY_ICONS: Record<string, string> = {
  registry: "🗃️",
  eventlogs: "📋",
  system: "⚙️",
  credentials: "🔑",
  browser: "🌐",
  useractivity: "👤",
  network: "🌍",
  systeminfo: "🖥️",
};

// --- Component ---

export function TriageMode(props: TriageModeProps) {
  onMount(() => {
    if (props.triageProfiles().length === 0) {
      props.onLoadProfiles();
    }
  });

  const progress = createMemo(() => props.triageProgress());
  const result = createMemo(() => props.triageResult());
  const selectedCats = createMemo(() => props.selectedTriageCategories());
  const isCollecting = createMemo(() => props.isCollecting());

  return (
    <div class="space-y-3">
      {/* Active Status — shown at TOP so it's always visible */}

      {/* Initializing indicator: collecting started but no progress event yet */}
      <Show when={isCollecting() && !progress() && !result()}>
        <div class="card border border-accent/30">
          <div class="flex items-center gap-2">
            <div class="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin" />
            <span class="text-sm font-medium text-txt">Initializing triage collection...</span>
          </div>
          <div class="text-2xs text-txt-muted mt-1">
            Enumerating system artifacts in {selectedCats().length} categor{selectedCats().length !== 1 ? "ies" : "y"}
          </div>
        </div>
      </Show>

      {/* Progress — shown while collecting */}
      <Show when={progress()}>
        {(_) => {
          const p = () => props.triageProgress()!;
          return (
            <div class="card border border-accent/30">
              <div class="flex items-center justify-between mb-2">
                <span class="text-xs font-medium text-txt">
                  {p().phase === "collecting"
                    ? `Collecting ${p().currentCategory}...`
                    : p().phase === "scanning"
                      ? "Scanning for secrets..."
                      : p().phase === "complete"
                        ? "Finalizing..."
                        : p().phase}
                </span>
                <span class="text-xs text-txt-muted">{p().percent.toFixed(1)}%</span>
              </div>
              <div class="w-full h-2 bg-bg-secondary rounded-full overflow-hidden" role="progressbar" aria-valuenow={Math.round(Math.min(p().percent, 100))} aria-valuemin={0} aria-valuemax={100} aria-label="Triage collection progress">
                <div
                  class="h-full bg-accent rounded-full transition-all duration-200"
                  style={{ width: `${Math.min(p().percent, 100)}%` }}
                />
              </div>
              <div class="flex justify-between mt-1 text-2xs text-txt-muted">
                <span>{p().filesCollected} / {p().filesTotal} files collected</span>
                <span>{formatSize(p().bytesCollected)}</span>
              </div>
              <Show when={p().currentFile}>
                <div class="text-2xs text-txt-muted mt-1 truncate" title={p().currentFile}>
                  {p().currentFile}
                </div>
              </Show>
            </div>
          );
        }}
      </Show>

      {/* Result — shown at top after completion */}
      <Show when={result()}>
        {(_) => {
          const r = () => props.triageResult()!;
          return (
            <div class="space-y-3">
              {/* Collection Report Header */}
              <div class={`card border ${r().cancelled ? "border-amber-500/30" : "border-green-500/30"}`}>
                <div class="flex items-center justify-between mb-3">
                  <div class="flex items-center gap-2">
                    <Show
                      when={!r().cancelled}
                      fallback={
                        <>
                          <HiOutlineExclamationTriangle class="w-icon-sm h-icon-sm text-warning" />
                          <span class="text-sm font-medium text-warning">Triage Cancelled</span>
                        </>
                      }
                    >
                      <HiOutlineCheckCircle class="w-icon-sm h-icon-sm text-success" />
                      <span class="text-sm font-medium text-success">Collection Report</span>
                    </Show>
                  </div>
                  <button
                    class="btn-sm flex items-center gap-1.5"
                    onClick={() => systemCommands.openPath(r().outputDir)}
                    title="Open output folder in Finder"
                  >
                    <HiOutlineFolderOpen class="w-4 h-4" />
                    <span>Open Folder</span>
                  </button>
                </div>

                {/* Stats grid */}
                <div class="grid grid-cols-2 gap-2 mb-3">
                  <div class="stat-box">
                    <div class="text-txt-muted text-xs">Files Collected</div>
                    <div class="text-lg font-semibold text-txt">{r().filesCollected}</div>
                  </div>
                  <div class="stat-box">
                    <div class="text-txt-muted text-xs">Total Size</div>
                    <div class="text-lg font-semibold text-txt">{formatSize(r().bytesCollected)}</div>
                  </div>
                  <div class="stat-box">
                    <div class="text-txt-muted text-xs">Duration</div>
                    <div class="text-lg font-semibold text-txt">
                      {r().durationSecs < 60
                        ? `${r().durationSecs.toFixed(1)}s`
                        : `${Math.floor(r().durationSecs / 60)}m ${Math.floor(r().durationSecs % 60)}s`}
                    </div>
                  </div>
                  <div class="stat-box">
                    <div class="text-txt-muted text-xs">Secrets Found</div>
                    <div class={`text-lg font-semibold ${r().secretFindings.length > 0 ? "text-warning" : "text-txt"}`}>
                      {r().secretFindings.length}
                    </div>
                  </div>
                </div>

                {/* Output location */}
                <div class="text-xs text-txt-muted bg-bg-secondary rounded p-2 mb-2">
                  <span class="font-medium text-txt-muted">Output: </span>
                  <span class="font-mono text-compact break-all">{r().outputDir}</span>
                </div>

                {/* Categories collected */}
                <Show when={r().categoriesCollected.length > 0}>
                  <div class="flex flex-wrap items-center gap-1.5">
                    <HiOutlineTag class="w-3.5 h-3.5 text-txt-muted shrink-0" />
                    <For each={r().categoriesCollected}>
                      {(cat) => (
                        <span class="badge badge-success text-2xs">
                          {CATEGORY_ICONS[cat] || "📁"} {cat}
                        </span>
                      )}
                    </For>
                  </div>
                </Show>

                {/* Skipped/failed counts */}
                <Show when={r().filesSkipped > 0 || r().filesFailed > 0}>
                  <div class="flex gap-3 mt-2 text-xs text-txt-muted">
                    <Show when={r().filesSkipped > 0}>
                      <span>{r().filesSkipped} skipped</span>
                    </Show>
                    <Show when={r().filesFailed > 0}>
                      <span class="text-error">{r().filesFailed} failed</span>
                    </Show>
                  </div>
                </Show>
              </div>

              {/* Secret Findings */}
              <Show when={r().secretFindings.length > 0}>
                <div class="card border border-amber-500/30">
                  <div class="flex items-center gap-2 mb-3">
                    <HiOutlineShieldExclamation class="w-icon-sm h-icon-sm text-warning" />
                    <span class="text-sm font-medium text-warning">
                      Credential & Secret Findings ({r().secretFindings.length})
                    </span>
                  </div>
                  <div class="space-y-2 max-h-64 overflow-y-auto">
                    <For each={r().secretFindings}>
                      {(finding) => <SecretFindingRow finding={finding} />}
                    </For>
                  </div>
                </div>
              </Show>

              <Show when={r().secretFindings.length === 0 && !r().cancelled}>
                <div class="text-xs text-txt-muted p-2 bg-bg-secondary rounded text-center">
                  No credentials or secrets detected in collected files
                </div>
              </Show>
            </div>
          );
        }}
      </Show>
      {/* Profile Selection */}
      <div class="card">
        <div class="flex items-center gap-2 mb-3">
          <HiOutlineShieldCheck class="w-icon-sm h-icon-sm text-accent" />
          <span class="text-sm font-medium text-txt">Collection Profile</span>
        </div>

        <Show when={props.triageProfilesLoading()}>
          <div class="text-xs text-txt-muted animate-pulse-slow">
            Loading triage profiles...
          </div>
        </Show>

        <Show when={!props.triageProfilesLoading() && props.triageProfiles().length > 0}>
          <div class="space-y-2">
            <select
              class="input-sm w-full"
              value={props.selectedTriageProfile()}
              onChange={(e) => props.setSelectedTriageProfile(e.currentTarget.value)}
            >
              <For each={props.triageProfiles()}>
                {(profile) => (
                  <option value={profile.id}>{profile.name}</option>
                )}
              </For>
              <option value="custom">Custom Selection</option>
            </select>

            {/* Profile description */}
            <Show when={props.selectedTriageProfile() !== "custom"}>
              {(() => {
                const profile = createMemo(() =>
                  props.triageProfiles().find((p) => p.id === props.selectedTriageProfile()),
                );
                return (
                  <Show when={profile()}>
                    <div class="text-xs text-txt-muted p-2 bg-bg-secondary rounded">
                      {profile()!.description}
                    </div>
                  </Show>
                );
              })()}
            </Show>
          </div>
        </Show>
      </div>

      {/* Category Selection */}
      <Show when={props.triageCategories().length > 0}>
        <div class="card">
          <div class="flex items-center gap-2 mb-3">
            <HiOutlineFolder class="w-icon-sm h-icon-sm text-txt-muted" />
            <span class="text-sm font-medium text-txt">Artifact Categories</span>
            <span class="text-2xs text-txt-muted ml-auto">
              {selectedCats().length}/{props.triageCategories().length} selected
            </span>
          </div>

          <div class="space-y-1">
            <For each={props.triageCategories()}>
              {(cat) => (
                <label class="flex items-center gap-2 p-2 rounded hover:bg-bg-hover cursor-pointer">
                  <input
                    type="checkbox"
                    checked={selectedCats().includes(cat.id)}
                    onChange={() => props.toggleTriageCategory(cat.id)}
                    class="rounded border-border"
                  />
                  <span class="text-sm">{CATEGORY_ICONS[cat.id] || "📁"}</span>
                  <div class="flex-1 min-w-0">
                    <div class="text-sm text-txt">{cat.name}</div>
                    <div class="text-2xs text-txt-muted truncate">
                      {cat.description} · {cat.artifactCount} artifact{cat.artifactCount !== 1 ? "s" : ""}
                    </div>
                  </div>
                </label>
              )}
            </For>
          </div>
        </div>
      </Show>

      {/* Scan Options */}
      <div class="card">
        <div class="flex items-center gap-2 mb-3">
          <HiOutlineMagnifyingGlass class="w-icon-sm h-icon-sm text-txt-muted" />
          <span class="text-sm font-medium text-txt">Scanning Options</span>
        </div>

        <label class="flex items-start gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={props.triageScanForSecrets()}
            onChange={(e) => props.setTriageScanForSecrets(e.currentTarget.checked)}
            class="rounded border-border mt-0.5"
          />
          <div>
            <div class="flex items-center gap-1.5">
              <HiOutlineKey class="w-4 h-4 text-txt-muted" />
              <span class="text-sm text-txt">Scan for credentials & secrets</span>
            </div>
            <div class="text-2xs text-txt-muted mt-0.5">
              Searches collected files for API keys, private keys, tokens, connection strings, and encryption keys
            </div>
          </div>
        </label>
      </div>
    </div>
  );
}

// --- Sub-component ---

function SecretFindingRow(props: { finding: SecretFinding }) {
  const f = () => props.finding;

  return (
    <div class={`p-2 rounded border ${confidenceBg(f().confidence)}`}>
      <div class="flex items-center gap-2 mb-1">
        <span class={`text-2xs font-medium uppercase ${confidenceColor(f().confidence)}`}>
          {f().confidence}
        </span>
        <span class="text-xs font-medium text-txt">{f().secretType}</span>
      </div>
      <div class="text-2xs text-txt-muted">{f().description}</div>
      <div class="font-mono text-compact text-txt-muted mt-1 truncate" title={f().preview}>
        {f().preview}
      </div>
      <div class="text-2xs text-txt-muted mt-1 truncate" title={f().filePath}>
        {f().filePath}:{f().lineNumber}
      </div>
    </div>
  );
}
