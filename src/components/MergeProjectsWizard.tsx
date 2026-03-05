// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * MergeProjectsWizard — Modal wizard for merging multiple .cffx projects.
 *
 * Steps:
 * 1. Select .cffx files to merge
 * 2. Review project summaries and configure merge settings
 * 3. Execute merge and show results
 */

import { Component, createSignal, For, Show, createMemo } from "solid-js";
import { open, save } from "@tauri-apps/plugin-dialog";
import {
  HiOutlineDocumentDuplicate,
  HiOutlinePlus,
  HiOutlineTrash,
  HiOutlineCheckCircle,
  HiOutlineExclamationTriangle,
  HiOutlineXMark,
  HiOutlineArchiveBox,
  HiOutlineFolder,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineUserGroup,
  HiOutlineArchiveBoxArrowDown,
  HiOutlineShieldCheck,
  HiOutlineClipboardDocumentList,
} from "../components/icons";
import type { MergeExaminerInfo } from "../api/projectMerge";
import {
  analyzeProjects,
  executeMerge,
  type ProjectMergeSummary,
  type MergeResult,
  type MergeSourceAssignment,
} from "../api/projectMerge";

interface MergeProjectsWizardProps {
  onClose: () => void;
  /** Callback after successful merge — load the merged project */
  onMergeComplete?: (cffxPath: string) => void;
}

type WizardStep = "select" | "review" | "merging" | "complete";

/** Map raw template_id values to user-friendly display names */
const TEMPLATE_DISPLAY_NAMES: Record<string, string> = {
  evidence_collection: "Evidence Collection",
  iar: "Investigative Activity Report",
  user_activity: "User Activity Log",
  chain_of_custody: "Chain of Custody",
  incident_report: "Incident Report",
  search_warrant: "Search Warrant",
  consent_to_search: "Consent to Search",
};

const friendlyTemplateName = (templateId: string): string =>
  TEMPLATE_DISPLAY_NAMES[templateId] || templateId.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());

const MergeProjectsWizard: Component<MergeProjectsWizardProps> = (props) => {
  // State
  const [step, setStep] = createSignal<WizardStep>("select");
  const [cffxPaths, setCffxPaths] = createSignal<string[]>([]);
  const [summaries, setSummaries] = createSignal<ProjectMergeSummary[]>([]);
  /** Owner name overrides per project (cffxPath -> ownerName) */
  const [ownerOverrides, setOwnerOverrides] = createSignal<Record<string, string>>({});
  const [mergedName, setMergedName] = createSignal("Merged Project");
  const [outputPath, setOutputPath] = createSignal("");
  const [mergeResult, setMergeResult] = createSignal<MergeResult | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [analyzing, setAnalyzing] = createSignal(false);
  /** Tracks which collapsible sections are expanded per project: "cffxPath:section" */
  const [expandedSections, setExpandedSections] = createSignal<Set<string>>(new Set());

  const toggleSection = (key: string) => {
    setExpandedSections((prev) => {
      const next = new Set(prev);
      if (next.has(key)) next.delete(key);
      else next.add(key);
      return next;
    });
  };
  const isSectionExpanded = (key: string) => expandedSections().has(key);
  /** Tracks which projects have custom (free-text) owner entry toggled on */
  const [customOwnerMode, setCustomOwnerMode] = createSignal<Set<string>>(new Set());
  const toggleCustomOwner = (cffxPath: string) => {
    setCustomOwnerMode((prev) => {
      const next = new Set(prev);
      if (next.has(cffxPath)) next.delete(cffxPath);
      else next.add(cffxPath);
      return next;
    });
  };
  const isCustomOwner = (cffxPath: string) => customOwnerMode().has(cffxPath);

  // Derived state
  const canProceedToReview = createMemo(() => cffxPaths().length >= 2);

  /** All unique examiners across every project (deduped, case-insensitive) */
  const globalExaminers = createMemo(() => {
    const seen = new Map<string, MergeExaminerInfo & { projects: string[] }>();
    for (const s of summaries()) {
      for (const ex of s.examiners) {
        const key = (ex.displayName || ex.name).toLowerCase();
        const existing = seen.get(key);
        if (existing) {
          if (!existing.projects.includes(s.name)) existing.projects.push(s.name);
        } else {
          seen.set(key, { ...ex, projects: [s.name] });
        }
      }
    }
    return [...seen.values()];
  });
  const totalStats = createMemo(() => {
    const s = summaries();
    return {
      evidence: s.reduce((a, p) => a + p.evidenceFileCount, 0),
      hashes: s.reduce((a, p) => a + p.hashCount, 0),
      sessions: s.reduce((a, p) => a + p.sessionCount, 0),
      activity: s.reduce((a, p) => a + p.activityCount, 0),
      bookmarks: s.reduce((a, p) => a + p.bookmarkCount, 0),
      notes: s.reduce((a, p) => a + p.noteCount, 0),
      reports: s.reduce((a, p) => a + p.reportCount, 0),
    };
  });

  // --- Step 1: Add project files ---
  const handleAddProjects = async () => {
    const selected = await open({
      multiple: true,
      filters: [{ name: "CORE-FFX Projects", extensions: ["cffx"] }],
      title: "Select .cffx project files to merge",
    });

    if (selected) {
      const paths = Array.isArray(selected) ? selected : [selected];
      const newPaths = [...cffxPaths()];
      for (const p of paths) {
        if (!newPaths.includes(p)) {
          newPaths.push(p);
        }
      }
      setCffxPaths(newPaths);
    }
  };

  const handleRemoveProject = (path: string) => {
    setCffxPaths((prev) => prev.filter((p) => p !== path));
    setSummaries((prev) => prev.filter((s) => s.cffxPath !== path));
  };

  // --- Step 1 → 2: Analyze projects ---
  const handleAnalyze = async () => {
    setAnalyzing(true);
    setError(null);
    try {
      const results = await analyzeProjects(cffxPaths());
      setSummaries(results);

      // Initialize owner overrides from project owner_name or best examiner match
      const initialOwners: Record<string, string> = {};
      for (const s of results) {
        if (s.ownerName) {
          initialOwners[s.cffxPath] = s.ownerName;
        } else if (s.examiners.length > 0) {
          // Auto-suggest: prioritize "project owner" → "session user" → first examiner
          const owner = s.examiners.find((e) => e.role === "project owner")
            || s.examiners.find((e) => e.role === "session user")
            || s.examiners[0];
          initialOwners[s.cffxPath] = owner.displayName || owner.name;
        } else {
          initialOwners[s.cffxPath] = "";
        }
      }
      setOwnerOverrides(initialOwners);

      // Auto-generate merged name from first project
      if (results.length > 0 && mergedName() === "Merged Project") {
        setMergedName(results[0].name);
      }

      // Auto-suggest output path: same directory as first project
      if (results.length > 0 && !outputPath()) {
        const firstPath = results[0].cffxPath;
        const dir = firstPath.substring(0, firstPath.lastIndexOf("/"));
        setOutputPath(`${dir}/${results[0].name}_merged.cffx`);
      }

      setStep("review");
    } catch (e) {
      setError(`Failed to analyze projects: ${e}`);
    } finally {
      setAnalyzing(false);
    }
  };

  // --- Step 2: Choose output path ---
  const handleChooseOutput = async () => {
    const selected = await save({
      filters: [{ name: "CORE-FFX Project", extensions: ["cffx"] }],
      title: "Save merged project as",
      defaultPath: outputPath() || undefined,
    });
    if (selected) {
      setOutputPath(selected);
      // Update merged name from filename
      const basename = selected.split("/").pop()?.replace(".cffx", "") || mergedName();
      setMergedName(basename);
    }
  };

  // --- Step 2 → 3: Execute merge ---
  const handleMerge = async () => {
    if (!outputPath()) {
      setError("Please choose an output path");
      return;
    }

    setStep("merging");
    setError(null);
    try {
      // Build owner assignments from overrides
      const overrides = ownerOverrides();
      const ownerAssignments: MergeSourceAssignment[] = cffxPaths().map((path) => ({
        cffxPath: path,
        ownerName: overrides[path] || "",
      }));

      const result = await executeMerge(
        cffxPaths(),
        outputPath(),
        mergedName(),
        undefined,
        ownerAssignments,
      );
      setMergeResult(result);
      if (result.success) {
        setStep("complete");
      } else {
        setError(result.error || "Unknown merge error");
        setStep("review");
      }
    } catch (e) {
      setError(`Merge failed: ${e}`);
      setStep("review");
    }
  };

  // --- Step 4: Load merged project ---
  const handleLoadMerged = () => {
    const result = mergeResult();
    if (result?.cffxPath) {
      props.onMergeComplete?.(result.cffxPath);
    }
    props.onClose();
  };

  // Helper: get filename from path
  const basename = (path: string) => path.split("/").pop() || path;

  // Helper: format date
  const formatDate = (iso: string) => {
    try {
      return new Date(iso).toLocaleDateString(undefined, {
        year: "numeric",
        month: "short",
        day: "numeric",
      });
    } catch {
      return iso.slice(0, 10);
    }
  };

  return (
    <div class="modal-overlay" onClick={(e) => e.target === e.currentTarget && props.onClose()}>
      <div class="modal-content w-[680px] max-h-[85vh] flex flex-col">
        {/* Header */}
        <div class="modal-header">
          <div class="flex items-center gap-2">
            <HiOutlineDocumentDuplicate class="w-5 h-5 text-accent" />
            <h2 class="text-lg font-semibold text-txt">Merge Projects</h2>
          </div>
          <button class="icon-btn-sm" onClick={props.onClose}>
            <HiOutlineXMark class="w-4 h-4" />
          </button>
        </div>

        {/* Step indicator */}
        <div class="px-5 pt-3 pb-2 flex items-center gap-2 text-xs text-txt-muted border-b border-border">
          <span classList={{ "text-accent font-semibold": step() === "select" }}>1. Select</span>
          <span>→</span>
          <span classList={{ "text-accent font-semibold": step() === "review" }}>2. Review</span>
          <span>→</span>
          <span classList={{ "text-accent font-semibold": step() === "merging" || step() === "complete" }}>3. Merge</span>
        </div>

        {/* Body */}
        <div class="modal-body overflow-y-auto flex-1">
          {/* Error display */}
          <Show when={error()}>
            <div class="flex items-center gap-2 p-3 rounded-lg bg-error/10 text-error text-sm mb-4">
              <HiOutlineExclamationTriangle class="w-4 h-4 shrink-0" />
              <span>{error()}</span>
            </div>
          </Show>

          {/* Step 1: Select projects */}
          <Show when={step() === "select"}>
            <div class="col gap-4">
              <p class="text-sm text-txt-secondary">
                Select two or more .cffx project files to merge into a single project.
                All evidence data, sessions, hashes, bookmarks, and activity logs will be combined.
              </p>

              {/* Project list */}
              <div class="col gap-2">
                <For each={cffxPaths()}>
                  {(path) => (
                    <div class="flex items-center gap-2 p-2.5 rounded-lg bg-bg-secondary border border-border">
                      <HiOutlineArchiveBox class="w-4 h-4 text-accent shrink-0" />
                      <div class="flex-1 min-w-0">
                        <div class="text-sm font-medium text-txt truncate">{basename(path)}</div>
                        <div class="text-xs text-txt-muted truncate">{path}</div>
                      </div>
                      <button
                        class="icon-btn-sm text-txt-muted hover:text-error"
                        onClick={() => handleRemoveProject(path)}
                        title="Remove"
                      >
                        <HiOutlineTrash class="w-3.5 h-3.5" />
                      </button>
                    </div>
                  )}
                </For>
              </div>

              {/* Add button */}
              <button
                class="btn btn-secondary flex items-center gap-2"
                onClick={handleAddProjects}
              >
                <HiOutlinePlus class="w-4 h-4" />
                Add Project Files
              </button>

              <Show when={cffxPaths().length > 0 && cffxPaths().length < 2}>
                <p class="text-xs text-warning">Select at least 2 projects to merge.</p>
              </Show>
            </div>
          </Show>

          {/* Step 2: Review & configure */}
          <Show when={step() === "review"}>
            <div class="col gap-4">
              {/* Project summaries */}
              <div>
                <h3 class="text-sm font-semibold text-txt mb-2">Projects to Merge</h3>
                <div class="col gap-3">
                  <For each={summaries()}>
                    {(summary) => {
                      const sKey = (section: string) => `${summary.cffxPath}:${section}`;

                      /** Role badge color */
                      const roleBadgeClass = (role: string) => {
                        if (role === "project owner") return "badge badge-success";
                        if (role === "session user") return "badge";
                        if (role.includes("COC")) return "badge badge-warning";
                        return "badge";
                      };

                      /** Format bytes */
                      const fmtBytes = (bytes: number) => {
                        if (bytes === 0) return "0 B";
                        const units = ["B", "KB", "MB", "GB", "TB"];
                        const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
                        return `${(bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
                      };

                      return (
                        <div class="p-3 rounded-lg bg-bg-secondary border border-border">
                          {/* Header: name + date */}
                          <div class="flex items-center justify-between mb-1">
                            <span class="text-sm font-medium text-txt">{summary.name}</span>
                            <span class="text-xs text-txt-muted">{formatDate(summary.createdAt)}</span>
                          </div>

                          {/* Owner/Examiner selector */}
                          <div class="flex items-center gap-2 mb-2">
                            <label class="text-xs text-txt-muted whitespace-nowrap">Owner:</label>
                            <Show when={!isCustomOwner(summary.cffxPath)}
                              fallback={
                                <input
                                  class="input-xs flex-1"
                                  value={ownerOverrides()[summary.cffxPath] || ""}
                                  onInput={(e) => {
                                    const newVal = e.currentTarget.value;
                                    setOwnerOverrides((prev) => ({ ...prev, [summary.cffxPath]: newVal }));
                                  }}
                                  placeholder="Enter new examiner name"
                                  autofocus
                                />
                              }
                            >
                              <select
                                class="input-xs flex-1"
                                value={ownerOverrides()[summary.cffxPath] || ""}
                                onChange={(e) => {
                                  const val = e.currentTarget.value;
                                  if (val === "__custom__") {
                                    toggleCustomOwner(summary.cffxPath);
                                    return;
                                  }
                                  setOwnerOverrides((prev) => ({ ...prev, [summary.cffxPath]: val }));
                                }}
                              >
                                <option value="">— Select examiner —</option>
                                <For each={globalExaminers()}>
                                  {(ex) => {
                                    const label = ex.displayName || ex.name;
                                    return <option value={label}>{label} ({ex.role})</option>;
                                  }}
                                </For>
                                <option value="__custom__">＋ Add new…</option>
                              </select>
                            </Show>
                            <Show when={isCustomOwner(summary.cffxPath)}>
                              <button
                                class="icon-btn-sm text-txt-muted hover:text-accent"
                                title="Switch back to dropdown"
                                onClick={() => toggleCustomOwner(summary.cffxPath)}
                              >
                                <HiOutlineUserGroup class="w-3.5 h-3.5" />
                              </button>
                            </Show>
                          </div>

                          {/* Quick stats row */}
                          <div class="grid grid-cols-4 gap-2 text-xs text-txt-muted">
                            <span>Evidence: {summary.evidenceFileCount}</span>
                            <span>Hashes: {summary.hashCount}</span>
                            <span>Sessions: {summary.sessionCount}</span>
                            <span>Activity: {summary.activityCount}</span>
                          </div>
                          <div class="grid grid-cols-4 gap-2 text-xs text-txt-muted mt-1">
                            <span>Bookmarks: {summary.bookmarkCount}</span>
                            <span>Notes: {summary.noteCount}</span>
                            <span>Reports: {summary.reportCount}</span>
                            <span>
                              DB:{" "}
                              <span classList={{ "text-success": summary.ffxdbExists, "text-warning": !summary.ffxdbExists }}>
                                {summary.ffxdbExists ? "Yes" : "No"}
                              </span>
                            </span>
                          </div>

                          {/* === Expandable detail sections === */}
                          <div class="mt-2 border-t border-border/50 pt-2 col gap-1">

                            {/* --- Examiners --- */}
                            <Show when={summary.examiners.length > 0}>
                              <button
                                class="flex items-center gap-1.5 text-xs text-txt-secondary hover:text-txt w-full text-left py-0.5"
                                onClick={() => toggleSection(sKey("examiners"))}
                              >
                                <Show when={isSectionExpanded(sKey("examiners"))} fallback={<HiOutlineChevronRight class="w-3 h-3" />}>
                                  <HiOutlineChevronDown class="w-3 h-3" />
                                </Show>
                                <HiOutlineUserGroup class="w-3.5 h-3.5" />
                                <span>Examiners ({summary.examiners.length})</span>
                              </button>
                              <Show when={isSectionExpanded(sKey("examiners"))}>
                                <div class="ml-5 col gap-1 text-xs">
                                  <For each={summary.examiners}>
                                    {(ex) => (
                                      <div class="flex items-center gap-2">
                                        <span class="text-txt font-medium">{ex.displayName || ex.name}</span>
                                        <span class={roleBadgeClass(ex.role)} style="font-size: 10px; padding: 1px 5px;">{ex.role}</span>
                                        <span class="text-txt-muted" style="font-size: 10px;">({ex.source})</span>
                                      </div>
                                    )}
                                  </For>
                                </div>
                              </Show>
                            </Show>

                            {/* --- Evidence Files --- */}
                            <Show when={summary.evidenceFiles.length > 0}>
                              <button
                                class="flex items-center gap-1.5 text-xs text-txt-secondary hover:text-txt w-full text-left py-0.5"
                                onClick={() => toggleSection(sKey("evidence"))}
                              >
                                <Show when={isSectionExpanded(sKey("evidence"))} fallback={<HiOutlineChevronRight class="w-3 h-3" />}>
                                  <HiOutlineChevronDown class="w-3 h-3" />
                                </Show>
                                <HiOutlineArchiveBox class="w-3.5 h-3.5" />
                                <span>Evidence Files ({summary.evidenceFiles.length})</span>
                              </button>
                              <Show when={isSectionExpanded(sKey("evidence"))}>
                                <div class="ml-5 col gap-1 text-xs max-h-40 overflow-y-auto">
                                  <For each={summary.evidenceFiles}>
                                    {(ef) => (
                                      <div class="flex items-center gap-2">
                                        <span class="text-txt truncate flex-1" title={ef.path}>{ef.filename}</span>
                                        <Show when={ef.containerType}>
                                          <span class="badge" style="font-size: 10px; padding: 1px 5px;">{ef.containerType.toUpperCase()}</span>
                                        </Show>
                                        <Show when={ef.totalSize > 0}>
                                          <span class="text-txt-muted whitespace-nowrap">{fmtBytes(ef.totalSize)}</span>
                                        </Show>
                                      </div>
                                    )}
                                  </For>
                                </div>
                              </Show>
                            </Show>

                            {/* --- Evidence Collections --- */}
                            <Show when={summary.collections.length > 0}>
                              <button
                                class="flex items-center gap-1.5 text-xs text-txt-secondary hover:text-txt w-full text-left py-0.5"
                                onClick={() => toggleSection(sKey("collections"))}
                              >
                                <Show when={isSectionExpanded(sKey("collections"))} fallback={<HiOutlineChevronRight class="w-3 h-3" />}>
                                  <HiOutlineChevronDown class="w-3 h-3" />
                                </Show>
                                <HiOutlineArchiveBoxArrowDown class="w-3.5 h-3.5" />
                                <span>Collections ({summary.collections.length})</span>
                              </button>
                              <Show when={isSectionExpanded(sKey("collections"))}>
                                <div class="ml-5 col gap-1.5 text-xs max-h-40 overflow-y-auto">
                                  <For each={summary.collections}>
                                    {(col) => (
                                      <div class="p-1.5 rounded bg-bg/50">
                                        <div class="flex items-center justify-between">
                                          <span class="text-txt font-medium">{col.caseNumber || "No case #"}</span>
                                          <span class="badge" style="font-size: 10px; padding: 1px 5px;">{col.status}</span>
                                        </div>
                                        <Show when={col.collectingOfficer}>
                                          <div class="text-txt-muted">Officer: {col.collectingOfficer}</div>
                                        </Show>
                                        <div class="flex gap-3 text-txt-muted">
                                          <Show when={col.collectionDate}><span>{col.collectionDate.slice(0, 10)}</span></Show>
                                          <span>{col.itemCount} items</span>
                                          <Show when={col.collectionLocation}><span>{col.collectionLocation}</span></Show>
                                        </div>
                                      </div>
                                    )}
                                  </For>
                                </div>
                              </Show>
                            </Show>

                            {/* --- COC Items --- */}
                            <Show when={summary.cocItems.length > 0}>
                              <button
                                class="flex items-center gap-1.5 text-xs text-txt-secondary hover:text-txt w-full text-left py-0.5"
                                onClick={() => toggleSection(sKey("coc"))}
                              >
                                <Show when={isSectionExpanded(sKey("coc"))} fallback={<HiOutlineChevronRight class="w-3 h-3" />}>
                                  <HiOutlineChevronDown class="w-3 h-3" />
                                </Show>
                                <HiOutlineShieldCheck class="w-3.5 h-3.5" />
                                <span>Chain of Custody ({summary.cocItems.length})</span>
                              </button>
                              <Show when={isSectionExpanded(sKey("coc"))}>
                                <div class="ml-5 col gap-1.5 text-xs max-h-40 overflow-y-auto">
                                  <For each={summary.cocItems}>
                                    {(coc) => (
                                      <div class="p-1.5 rounded bg-bg/50">
                                        <div class="flex items-center justify-between">
                                          <span class="text-txt font-medium">{coc.cocNumber || coc.evidenceId || "—"}</span>
                                          <span class="badge" style="font-size: 10px; padding: 1px 5px;">{coc.status}</span>
                                        </div>
                                        <Show when={coc.description}>
                                          <div class="text-txt-muted truncate">{coc.description}</div>
                                        </Show>
                                        <div class="flex gap-3 text-txt-muted">
                                          <Show when={coc.submittedBy}><span>From: {coc.submittedBy}</span></Show>
                                          <Show when={coc.receivedBy}><span>To: {coc.receivedBy}</span></Show>
                                        </div>
                                      </div>
                                    )}
                                  </For>
                                </div>
                              </Show>
                            </Show>

                            {/* --- Form Submissions --- */}
                            <Show when={summary.formSubmissions.length > 0}>
                              <button
                                class="flex items-center gap-1.5 text-xs text-txt-secondary hover:text-txt w-full text-left py-0.5"
                                onClick={() => toggleSection(sKey("forms"))}
                              >
                                <Show when={isSectionExpanded(sKey("forms"))} fallback={<HiOutlineChevronRight class="w-3 h-3" />}>
                                  <HiOutlineChevronDown class="w-3 h-3" />
                                </Show>
                                <HiOutlineClipboardDocumentList class="w-3.5 h-3.5" />
                                <span>Forms &amp; Evidence Collections ({summary.formSubmissions.length})</span>
                              </button>
                              <Show when={isSectionExpanded(sKey("forms"))}>
                                <div class="ml-5 col gap-1.5 text-xs max-h-40 overflow-y-auto">
                                  <For each={summary.formSubmissions}>
                                    {(form) => (
                                      <div class="p-1.5 rounded bg-bg/50">
                                        <div class="flex items-center justify-between">
                                          <span class="text-txt font-medium">{friendlyTemplateName(form.templateId)}</span>
                                          <span class="badge" style="font-size: 10px; padding: 1px 5px;">{form.status}</span>
                                        </div>
                                        <div class="flex gap-3 text-txt-muted">
                                          <Show when={form.caseNumber}>
                                            <span>Case #{form.caseNumber}</span>
                                          </Show>
                                          <Show when={form.collectingOfficer}>
                                            <span>Officer: {form.collectingOfficer}</span>
                                          </Show>
                                          <Show when={form.leadExaminer}>
                                            <span>Examiner: {form.leadExaminer}</span>
                                          </Show>
                                          <Show when={form.collectionLocation}>
                                            <span>{form.collectionLocation}</span>
                                          </Show>
                                          <span class="ml-auto">{form.createdAt.slice(0, 10)}</span>
                                        </div>
                                      </div>
                                    )}
                                  </For>
                                </div>
                              </Show>
                            </Show>
                          </div>
                        </div>
                      );
                    }}
                  </For>
                </div>
              </div>

              {/* All DB Users & Examiners (global) */}
              <Show when={globalExaminers().length > 0}>
                <div class="p-3 rounded-lg bg-bg-panel border border-border">
                  <button
                    class="flex items-center gap-2 text-sm font-semibold text-txt w-full text-left"
                    onClick={() => toggleSection("__global_examiners")}
                  >
                    <Show when={isSectionExpanded("__global_examiners")} fallback={<HiOutlineChevronRight class="w-3.5 h-3.5" />}>
                      <HiOutlineChevronDown class="w-3.5 h-3.5" />
                    </Show>
                    <HiOutlineUserGroup class="w-4 h-4 text-accent" />
                    All DB Users &amp; Examiners ({globalExaminers().length})
                  </button>
                  <Show when={isSectionExpanded("__global_examiners")}>
                    <div class="mt-2 col gap-1.5">
                      <For each={globalExaminers()}>
                        {(ex) => {
                          const label = ex.displayName || ex.name;
                          const roleClass = ex.role === "project owner" ? "badge badge-success"
                            : ex.role.includes("COC") ? "badge badge-warning"
                            : "badge";
                          return (
                            <div class="flex items-center gap-2 text-xs">
                              <span class="text-txt font-medium">{label}</span>
                              <span class={roleClass} style="font-size: 10px; padding: 1px 5px;">{ex.role}</span>
                              <span class="text-txt-muted" style="font-size: 10px;">({ex.source})</span>
                              <span class="text-txt-muted ml-auto" style="font-size: 10px;" title={ex.projects.join(", ")}>
                                {ex.projects.length > 1 ? `${ex.projects.length} projects` : ex.projects[0]}
                              </span>
                            </div>
                          );
                        }}
                      </For>
                    </div>
                  </Show>
                </div>
              </Show>

              {/* Merge totals */}
              <div class="p-3 rounded-lg bg-bg-panel border border-border">
                <h3 class="text-sm font-semibold text-txt mb-2">Merge Summary (before dedup)</h3>
                <div class="grid grid-cols-4 gap-2 text-xs text-txt-secondary">
                  <span>Evidence: {totalStats().evidence}</span>
                  <span>Hashes: {totalStats().hashes}</span>
                  <span>Sessions: {totalStats().sessions}</span>
                  <span>Activity: {totalStats().activity}</span>
                  <span>Bookmarks: {totalStats().bookmarks}</span>
                  <span>Notes: {totalStats().notes}</span>
                  <span>Reports: {totalStats().reports}</span>
                  <span>Projects: {summaries().length}</span>
                </div>
              </div>

              {/* Merged project settings */}
              <div>
                <h3 class="text-sm font-semibold text-txt mb-2">Merged Project</h3>
                <div class="col gap-3">
                  <div class="form-group">
                    <label class="label">Project Name</label>
                    <input
                      class="input"
                      value={mergedName()}
                      onInput={(e) => setMergedName(e.currentTarget.value)}
                      placeholder="Merged project name"
                    />
                  </div>

                  <div class="form-group">
                    <label class="label">Output Path</label>
                    <div class="flex items-center gap-2">
                      <input
                        class="input flex-1"
                        value={outputPath()}
                        onInput={(e) => setOutputPath(e.currentTarget.value)}
                        placeholder="Path for the merged .cffx file"
                      />
                      <button class="btn btn-secondary" onClick={handleChooseOutput}>
                        <HiOutlineFolder class="w-4 h-4" />
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            </div>
          </Show>

          {/* Step 3: Merging */}
          <Show when={step() === "merging"}>
            <div class="flex flex-col items-center justify-center py-12 gap-4">
              <div class="w-8 h-8 border-2 border-accent border-t-transparent rounded-full animate-spin" />
              <p class="text-sm text-txt-secondary">Merging {summaries().length} projects…</p>
              <p class="text-xs text-txt-muted">This may take a moment for large databases.</p>
            </div>
          </Show>

          {/* Step 4: Complete */}
          <Show when={step() === "complete"}>
            <div class="col gap-4">
              <div class="flex flex-col items-center py-6 gap-3">
                <HiOutlineCheckCircle class="w-12 h-12 text-success" />
                <h3 class="text-lg font-semibold text-txt">Merge Complete!</h3>
              </div>

              <Show when={mergeResult()}>
                {(result) => (
                  <div class="col gap-3">
                    <div class="p-3 rounded-lg bg-bg-secondary border border-border">
                      <div class="text-sm text-txt-secondary mb-1">Merged project:</div>
                      <div class="text-sm font-medium text-txt truncate">{result().cffxPath}</div>
                    </div>

                    <Show when={result().stats}>
                      {(stats) => (
                        <div class="p-3 rounded-lg bg-bg-panel border border-border">
                          <h4 class="text-sm font-semibold text-txt mb-2">Merge Statistics</h4>
                          <div class="grid grid-cols-3 gap-2 text-xs text-txt-secondary">
                            <span>Users: {stats().usersMerged}</span>
                            <span>Sessions: {stats().sessionsMerged}</span>
                            <span>Activity: {stats().activityEntriesMerged}</span>
                            <span>Evidence: {stats().evidenceFilesMerged}</span>
                            <span>Hashes: {stats().hashesMerged}</span>
                            <span>Bookmarks: {stats().bookmarksMerged}</span>
                            <span>Notes: {stats().notesMerged}</span>
                            <span>Reports: {stats().reportsMerged}</span>
                            <span>Tags: {stats().tagsMerged}</span>
                            <span>Searches: {stats().searchesMerged}</span>
                            <span>DB Tables: {stats().ffxdbTablesMerged}</span>
                          </div>
                        </div>
                      )}
                    </Show>

                    {/* Source provenance */}
                    <Show when={result().sources && result().sources!.length > 0}>
                      <div class="p-3 rounded-lg bg-bg-secondary border border-border">
                        <h4 class="text-sm font-semibold text-txt mb-2">Source Projects</h4>
                        <div class="col gap-2">
                          <For each={result().sources!}>
                            {(source) => (
                              <div class="text-xs text-txt-secondary flex items-center justify-between">
                                <div>
                                  <span class="font-medium text-txt">{source.sourceProjectName}</span>
                                  <Show when={source.ownerName}>
                                    <span class="text-txt-muted"> — {source.ownerName}</span>
                                  </Show>
                                </div>
                                <span class="text-txt-muted">{source.evidenceFileCount} evidence</span>
                              </div>
                            )}
                          </For>
                        </div>
                      </div>
                    </Show>
                  </div>
                )}
              </Show>
            </div>
          </Show>
        </div>

        {/* Footer */}
        <div class="modal-footer justify-end">
          <Show when={step() === "select"}>
            <button class="btn btn-secondary" onClick={props.onClose}>Cancel</button>
            <button
              class="btn btn-primary"
              disabled={!canProceedToReview() || analyzing()}
              onClick={handleAnalyze}
            >
              {analyzing() ? "Analyzing…" : "Review & Merge"}
            </button>
          </Show>

          <Show when={step() === "review"}>
            <button class="btn btn-secondary" onClick={() => setStep("select")}>Back</button>
            <button
              class="btn btn-primary"
              disabled={!outputPath() || !mergedName()}
              onClick={handleMerge}
            >
              Merge Projects
            </button>
          </Show>

          <Show when={step() === "complete"}>
            <button class="btn btn-secondary" onClick={props.onClose}>Close</button>
            <Show when={mergeResult()?.cffxPath}>
              <button class="btn btn-primary" onClick={handleLoadMerged}>
                Open Merged Project
              </button>
            </Show>
          </Show>
        </div>
      </div>
    </div>
  );
};

export default MergeProjectsWizard;
