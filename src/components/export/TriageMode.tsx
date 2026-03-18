// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// TriageMode — Forensic triage collection + credential scanning UI
//
// Provides a compact, organized interface for selecting triage profiles,
// artifact categories (with expandable subcategory details), and
// credential/secret scanning options. Each category shows its individual
// artifacts so examiners know exactly what will be collected.

import { Show, For, onMount, createMemo, createSignal } from "solid-js";
import type { Accessor, Setter } from "solid-js";
import {
  HiOutlineShieldCheck,
  HiOutlineShieldExclamation,
  HiOutlineKey,
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
  HiOutlineFolderOpen,
  HiOutlineMagnifyingGlass,
  HiOutlineTag,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineInformationCircle,
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
      return "bg-error/10 border-error/30";
    case "medium":
      return "bg-warning/10 border-warning/30";
    default:
      return "bg-bg-secondary border-border/30";
  }
}

/** Category icons + short tooltip descriptions for quick reference. */
const CATEGORY_META: Record<string, { icon: string; tip: string }> = {
  registry: { icon: "🗃️", tip: "Windows registry hives (SAM, SECURITY, SYSTEM, SOFTWARE)" },
  eventlogs: { icon: "📋", tip: "System event logs, audit trails, and forensic log sources" },
  system: { icon: "⚙️", tip: "OS-level config files, scheduled tasks, security databases" },
  credentials: { icon: "🔑", tip: "SSH keys, cloud credentials, keychains, API tokens" },
  browser: { icon: "🌐", tip: "Browser profiles, history, cookies, saved passwords" },
  useractivity: { icon: "👤", tip: "Shell history, recent docs, application usage" },
  network: { icon: "🌍", tip: "Hosts file, WiFi profiles, firewall rules, DNS config" },
  systeminfo: { icon: "🖥️", tip: "Hardware UUID, serial number, hostname, OS version" },
};

/** Profile descriptions for tooltip display when hovering profile selector options. */
const PROFILE_TIPS: Record<string, string> = {
  security: "Collects registry hives, event logs, and system security databases — ideal for incident response and intrusion analysis.",
  credentials: "Targets authentication material: SSH keys, cloud provider tokens, browser password databases, and certificates.",
  useractivity: "Captures user behavior artifacts: shell history, recent documents, browser history, and application usage patterns.",
  network: "Network configuration artifacts: WiFi profiles, DNS settings, hosts file, and firewall rules.",
  full: "Comprehensive collection of ALL artifact categories — recommended for thorough forensic triage.",
  identification: "Essential system identifiers (UUID, serial number, hostname, OS version) for evidence forms and chain of custody.",
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

  // Track which categories have their artifact details expanded
  const [expandedCats, setExpandedCats] = createSignal<Set<string>>(new Set());
  const toggleExpanded = (catId: string) => {
    setExpandedCats((prev) => {
      const next = new Set(prev);
      if (next.has(catId)) next.delete(catId);
      else next.add(catId);
      return next;
    });
  };

  return (
    <div class="space-y-3">
      {/* Active Status — shown at TOP so it's always visible */}

      {/* Initializing indicator */}
      <Show when={isCollecting() && !progress() && !result()}>
        <div class="card border border-accent/30">
          <div class="flex items-center gap-2">
            <div class="w-4 h-4 border-2 border-accent border-t-transparent rounded-full animate-spin" />
            <span class="text-sm font-medium text-txt">Initializing triage collection...</span>
          </div>
          <div class="text-xs text-txt-muted mt-1">
            Enumerating system artifacts in {selectedCats().length} categor{selectedCats().length !== 1 ? "ies" : "y"}
          </div>
        </div>
      </Show>

      {/* Progress */}
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
              <div class="flex justify-between mt-1 text-xs text-txt-muted">
                <span>{p().filesCollected} / {p().filesTotal} files</span>
                <span>{formatSize(p().bytesCollected)}</span>
              </div>
              <Show when={p().currentFile}>
                <div class="text-xs text-txt-muted mt-1 truncate" title={p().currentFile}>
                  {p().currentFile}
                </div>
              </Show>
            </div>
          );
        }}
      </Show>

      {/* Result */}
      <Show when={result()}>
        {(_) => {
          const r = () => props.triageResult()!;
          return (
            <div class="space-y-3">
              <div class={`card border ${r().cancelled ? "border-warning/30" : "border-success/30"}`}>
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
                      <span class="text-sm font-medium text-success">Collection Complete</span>
                    </Show>
                  </div>
                  <button
                    class="btn-sm flex items-center gap-1.5"
                    onClick={() => systemCommands.openPath(r().outputDir)}
                    title="Open the output folder containing collected artifacts"
                  >
                    <HiOutlineFolderOpen class="w-4 h-4" />
                    <span>Open</span>
                  </button>
                </div>

                {/* Compact stats row */}
                <div class="grid grid-cols-4 gap-2 mb-3">
                  <div class="stat-box">
                    <div class="text-txt-muted text-2xs">Files</div>
                    <div class="text-sm font-semibold text-txt">{r().filesCollected}</div>
                  </div>
                  <div class="stat-box">
                    <div class="text-txt-muted text-2xs">Size</div>
                    <div class="text-sm font-semibold text-txt">{formatSize(r().bytesCollected)}</div>
                  </div>
                  <div class="stat-box">
                    <div class="text-txt-muted text-2xs">Time</div>
                    <div class="text-sm font-semibold text-txt">
                      {r().durationSecs < 60
                        ? `${r().durationSecs.toFixed(1)}s`
                        : `${Math.floor(r().durationSecs / 60)}m ${Math.floor(r().durationSecs % 60)}s`}
                    </div>
                  </div>
                  <div class="stat-box">
                    <div class="text-txt-muted text-2xs">Secrets</div>
                    <div class={`text-sm font-semibold ${r().secretFindings.length > 0 ? "text-warning" : "text-txt"}`}>
                      {r().secretFindings.length}
                    </div>
                  </div>
                </div>

                {/* Output location */}
                <div class="text-xs text-txt-muted bg-bg-secondary rounded p-1.5 mb-2">
                  <span class="font-medium">Output: </span>
                  <span class="font-mono text-compact break-all">{r().outputDir}</span>
                </div>

                {/* Categories collected */}
                <Show when={r().categoriesCollected.length > 0}>
                  <div class="flex flex-wrap items-center gap-1">
                    <HiOutlineTag class="w-3 h-3 text-txt-muted shrink-0" />
                    <For each={r().categoriesCollected}>
                      {(cat) => (
                        <span class="badge badge-success text-2xs">
                          {CATEGORY_META[cat]?.icon || "📁"} {cat}
                        </span>
                      )}
                    </For>
                  </div>
                </Show>

                <Show when={r().filesSkipped > 0 || r().filesFailed > 0}>
                  <div class="flex gap-3 mt-1.5 text-xs text-txt-muted">
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
                <div class="card border border-warning/30">
                  <div class="flex items-center gap-2 mb-2">
                    <HiOutlineShieldExclamation class="w-icon-sm h-icon-sm text-warning" />
                    <span class="text-xs font-medium text-warning">
                      Credential & Secret Findings ({r().secretFindings.length})
                    </span>
                  </div>
                  <div class="space-y-1.5 max-h-48 overflow-y-auto">
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
        <div class="flex items-center gap-2 mb-2">
          <HiOutlineShieldCheck class="w-icon-sm h-icon-sm text-accent" />
          <span class="text-xs font-medium text-txt">Collection Profile</span>
          <span
            class="ml-auto cursor-help"
            title="Profiles are preconfigured sets of artifact categories. Select a profile to auto-check the matching categories below, or choose 'Custom Selection' to pick individually."
          >
            <HiOutlineInformationCircle class="w-3.5 h-3.5 text-txt-muted" />
          </span>
        </div>

        <Show when={props.triageProfilesLoading()}>
          <div class="text-xs text-txt-muted animate-pulse-slow py-2">
            Detecting platform artifacts...
          </div>
        </Show>

        <Show when={!props.triageProfilesLoading() && props.triageProfiles().length > 0}>
          <select
            class="input-sm w-full text-xs"
            value={props.selectedTriageProfile()}
            onChange={(e) => props.setSelectedTriageProfile(e.currentTarget.value)}
            title="Select a preconfigured collection profile or choose Custom to pick categories manually"
          >
            <For each={props.triageProfiles()}>
              {(profile) => (
                <option value={profile.id} title={PROFILE_TIPS[profile.id] || profile.description}>
                  {profile.name}
                </option>
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
                  <div class="text-xs text-txt-muted p-1.5 bg-bg-secondary rounded mt-1.5">
                    {PROFILE_TIPS[profile()!.id] || profile()!.description}
                  </div>
                </Show>
              );
            })()}
          </Show>
        </Show>
      </div>

      {/* Artifact Categories — Compact 2-column grid with expandable subcategories */}
      <Show when={props.triageCategories().length > 0}>
        <div class="card">
          <div class="flex items-center gap-2 mb-2">
            <span class="text-xs font-medium text-txt">Artifact Categories</span>
            <span class="text-2xs text-txt-muted ml-auto">
              {selectedCats().length}/{props.triageCategories().length}
            </span>
          </div>

          <div class="grid grid-cols-2 gap-1">
            <For each={props.triageCategories()}>
              {(cat) => {
                const meta = CATEGORY_META[cat.id] || { icon: "📁", tip: cat.description };
                const isSelected = () => selectedCats().includes(cat.id);
                const isExpanded = () => expandedCats().has(cat.id);
                const hasArtifacts = () => cat.artifacts && cat.artifacts.length > 0;

                return (
                  <div
                    class="rounded border transition-colors"
                    classList={{
                      "border-accent/40 bg-accent/5": isSelected(),
                      "border-border/30 bg-bg-secondary/50": !isSelected(),
                    }}
                  >
                    {/* Category header row */}
                    <div class="flex items-center gap-1.5 p-1.5">
                      <input
                        type="checkbox"
                        checked={isSelected()}
                        onChange={() => props.toggleTriageCategory(cat.id)}
                        class="rounded border-border shrink-0"
                        title={`Toggle collection of ${cat.name}: ${meta.tip}`}
                      />
                      <span class="text-xs leading-none">{meta.icon}</span>
                      <span
                        class="text-xs text-txt truncate flex-1 cursor-default"
                        title={meta.tip}
                      >
                        {cat.name}
                      </span>
                      {/* Expand/collapse for artifact details */}
                      <Show when={hasArtifacts()}>
                        <button
                          class="p-0 text-txt-muted hover:text-txt shrink-0"
                          onClick={(e) => {
                            e.stopPropagation();
                            toggleExpanded(cat.id);
                          }}
                          title={isExpanded() ? `Hide ${cat.name} artifact details` : `Show ${cat.artifactCount} artifacts in ${cat.name}`}
                        >
                          <Show when={isExpanded()} fallback={<HiOutlineChevronRight class="w-3 h-3" />}>
                            <HiOutlineChevronDown class="w-3 h-3" />
                          </Show>
                        </button>
                      </Show>
                      <span class="text-2xs text-txt-muted tabular-nums shrink-0" title={`${cat.artifactCount} artifact target${cat.artifactCount !== 1 ? "s" : ""} in this category`}>
                        {cat.artifactCount}
                      </span>
                    </div>

                    {/* Expanded artifact subcategory list */}
                    <Show when={isExpanded() && hasArtifacts()}>
                      <div class="px-1.5 pb-1.5 pt-0">
                        <div class="border-t border-border/20 pt-1 space-y-0.5">
                          <For each={cat.artifacts}>
                            {(artifact) => (
                              <div class="flex items-center gap-1 pl-5" title={artifact}>
                                <span class="w-1 h-1 rounded-full bg-txt-muted/40 shrink-0" />
                                <span class="text-xs text-txt-muted truncate">{artifact}</span>
                              </div>
                            )}
                          </For>
                        </div>
                      </div>
                    </Show>
                  </div>
                );
              }}
            </For>
          </div>
        </div>
      </Show>

      {/* Credential & Secret Scanning */}
      <div class="card">
        <label class="flex items-start gap-2 cursor-pointer" title="When enabled, all collected text files are scanned for credentials, API keys, private keys, tokens, connection strings, and encryption keys using pattern matching. Findings are reported with confidence levels and redacted previews.">
          <input
            type="checkbox"
            checked={props.triageScanForSecrets()}
            onChange={(e) => props.setTriageScanForSecrets(e.currentTarget.checked)}
            class="rounded border-border mt-0.5 shrink-0"
          />
          <div class="min-w-0">
            <div class="flex items-center gap-1.5">
              <HiOutlineKey class="w-3.5 h-3.5 text-txt-muted shrink-0" />
              <span class="text-xs text-txt font-medium">Scan for credentials & secrets</span>
            </div>
            <div class="text-xs text-txt-muted mt-0.5 leading-relaxed">
              Searches collected files for API keys, private keys, tokens, connection strings,
              passwords, and encryption material. Detects 30+ secret patterns with confidence scoring.
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
    <div class={`p-1.5 rounded border ${confidenceBg(f().confidence)}`}>
      <div class="flex items-center gap-2 mb-0.5">
        <span class={`text-2xs font-medium uppercase ${confidenceColor(f().confidence)}`}>
          {f().confidence}
        </span>
        <span class="text-xs font-medium text-txt">{f().secretType}</span>
      </div>
      <div class="text-xs text-txt-muted">{f().description}</div>
      <div class="font-mono text-compact text-txt-muted mt-0.5 truncate" title={f().preview}>
        {f().preview}
      </div>
      <div class="text-xs text-txt-muted mt-0.5 truncate" title={f().filePath}>
        {f().filePath}:{f().lineNumber}
      </div>
    </div>
  );
}
