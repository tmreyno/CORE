// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * ProjectSummaryCard — A single project card in the Review step of MergeProjectsWizard.
 * Shows project metadata, owner selector, stats, and 5 expandable detail sections
 * (Examiners, Evidence Files, Collections, COC, Forms).
 */

import { Component, For, Show } from "solid-js";
import {
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineUserGroup,
  HiOutlineArchiveBox,
  HiOutlineArchiveBoxArrowDown,
  HiOutlineShieldCheck,
  HiOutlineClipboardDocumentList,
} from "../icons";
import type { ProjectMergeSummary, GlobalExaminer } from "./types";
import { formatDate, fmtBytes, roleBadgeClass, friendlyTemplateName } from "./helpers";

export interface ProjectSummaryCardProps {
  summary: ProjectMergeSummary;
  ownerOverride: string;
  globalExaminers: GlobalExaminer[];
  isCustomOwner: boolean;
  onToggleCustomOwner: () => void;
  onOwnerChange: (value: string) => void;
  isSectionExpanded: (sectionKey: string) => boolean;
  onToggleSection: (sectionKey: string) => void;
}

export const ProjectSummaryCard: Component<ProjectSummaryCardProps> = (props) => {
  const sKey = (section: string) => `${props.summary.cffxPath}:${section}`;

  return (
    <div class="p-3 rounded-lg bg-bg-secondary border border-border">
      {/* Header: name + date */}
      <div class="flex items-center justify-between mb-1">
        <span class="text-sm font-medium text-txt">{props.summary.name}</span>
        <span class="text-xs text-txt-muted">{formatDate(props.summary.createdAt)}</span>
      </div>

      {/* Owner/Examiner selector */}
      <div class="flex items-center gap-2 mb-2">
        <label class="text-xs text-txt-muted whitespace-nowrap">Owner:</label>
        <Show
          when={!props.isCustomOwner}
          fallback={
            <input
              class="input-xs flex-1"
              value={props.ownerOverride}
              onInput={(e) => props.onOwnerChange(e.currentTarget.value)}
              placeholder="Enter new examiner name"
              autofocus
            />
          }
        >
          <select
            class="input-xs flex-1"
            value={props.ownerOverride}
            onChange={(e) => {
              const val = e.currentTarget.value;
              if (val === "__custom__") {
                props.onToggleCustomOwner();
                return;
              }
              props.onOwnerChange(val);
            }}
          >
            <option value="">— Select examiner —</option>
            <For each={props.globalExaminers}>
              {(ex) => {
                const label = ex.displayName || ex.name;
                return (
                  <option value={label}>
                    {label} ({ex.role})
                  </option>
                );
              }}
            </For>
            <option value="__custom__">＋ Add new…</option>
          </select>
        </Show>
        <Show when={props.isCustomOwner}>
          <button
            class="icon-btn-sm text-txt-muted hover:text-accent"
            title="Switch back to dropdown"
            onClick={props.onToggleCustomOwner}
          >
            <HiOutlineUserGroup class="w-3.5 h-3.5" />
          </button>
        </Show>
      </div>

      {/* Quick stats rows */}
      <div class="grid grid-cols-4 gap-2 text-xs text-txt-muted">
        <span>Evidence: {props.summary.evidenceFileCount}</span>
        <span>Hashes: {props.summary.hashCount}</span>
        <span>Sessions: {props.summary.sessionCount}</span>
        <span>Activity: {props.summary.activityCount}</span>
      </div>
      <div class="grid grid-cols-4 gap-2 text-xs text-txt-muted mt-1">
        <span>Bookmarks: {props.summary.bookmarkCount}</span>
        <span>Notes: {props.summary.noteCount}</span>
        <span>Reports: {props.summary.reportCount}</span>
        <span>
          DB:{" "}
          <span classList={{ "text-success": props.summary.ffxdbExists, "text-warning": !props.summary.ffxdbExists }}>
            {props.summary.ffxdbExists ? "Yes" : "No"}
          </span>
        </span>
      </div>

      {/* === Expandable detail sections === */}
      <div class="mt-2 border-t border-border/50 pt-2 col gap-1">
        {/* --- Examiners --- */}
        <Show when={props.summary.examiners.length > 0}>
          <button
            class="flex items-center gap-1.5 text-xs text-txt-secondary hover:text-txt w-full text-left py-0.5"
            onClick={() => props.onToggleSection(sKey("examiners"))}
          >
            <Show when={props.isSectionExpanded(sKey("examiners"))} fallback={<HiOutlineChevronRight class="w-3 h-3" />}>
              <HiOutlineChevronDown class="w-3 h-3" />
            </Show>
            <HiOutlineUserGroup class="w-3.5 h-3.5" />
            <span>Examiners ({props.summary.examiners.length})</span>
          </button>
          <Show when={props.isSectionExpanded(sKey("examiners"))}>
            <div class="ml-5 col gap-1 text-xs">
              <For each={props.summary.examiners}>
                {(ex) => (
                  <div class="flex items-center gap-2">
                    <span class="text-txt font-medium">{ex.displayName || ex.name}</span>
                    <span class={roleBadgeClass(ex.role)} style="font-size: 10px; padding: 1px 5px;">
                      {ex.role}
                    </span>
                    <span class="text-txt-muted" style="font-size: 10px;">
                      ({ex.source})
                    </span>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </Show>

        {/* --- Evidence Files --- */}
        <Show when={props.summary.evidenceFiles.length > 0}>
          <button
            class="flex items-center gap-1.5 text-xs text-txt-secondary hover:text-txt w-full text-left py-0.5"
            onClick={() => props.onToggleSection(sKey("evidence"))}
          >
            <Show when={props.isSectionExpanded(sKey("evidence"))} fallback={<HiOutlineChevronRight class="w-3 h-3" />}>
              <HiOutlineChevronDown class="w-3 h-3" />
            </Show>
            <HiOutlineArchiveBox class="w-3.5 h-3.5" />
            <span>Evidence Files ({props.summary.evidenceFiles.length})</span>
          </button>
          <Show when={props.isSectionExpanded(sKey("evidence"))}>
            <div class="ml-5 col gap-1 text-xs max-h-40 overflow-y-auto">
              <For each={props.summary.evidenceFiles}>
                {(ef) => (
                  <div class="flex items-center gap-2">
                    <span class="text-txt truncate flex-1" title={ef.path}>
                      {ef.filename}
                    </span>
                    <Show when={ef.containerType}>
                      <span class="badge" style="font-size: 10px; padding: 1px 5px;">
                        {ef.containerType.toUpperCase()}
                      </span>
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
        <Show when={props.summary.collections.length > 0}>
          <button
            class="flex items-center gap-1.5 text-xs text-txt-secondary hover:text-txt w-full text-left py-0.5"
            onClick={() => props.onToggleSection(sKey("collections"))}
          >
            <Show when={props.isSectionExpanded(sKey("collections"))} fallback={<HiOutlineChevronRight class="w-3 h-3" />}>
              <HiOutlineChevronDown class="w-3 h-3" />
            </Show>
            <HiOutlineArchiveBoxArrowDown class="w-3.5 h-3.5" />
            <span>Collections ({props.summary.collections.length})</span>
          </button>
          <Show when={props.isSectionExpanded(sKey("collections"))}>
            <div class="ml-5 col gap-1.5 text-xs max-h-40 overflow-y-auto">
              <For each={props.summary.collections}>
                {(col) => (
                  <div class="p-1.5 rounded bg-bg/50">
                    <div class="flex items-center justify-between">
                      <span class="text-txt font-medium">{col.caseNumber || "No case #"}</span>
                      <span class="badge" style="font-size: 10px; padding: 1px 5px;">
                        {col.status}
                      </span>
                    </div>
                    <Show when={col.collectingOfficer}>
                      <div class="text-txt-muted">Officer: {col.collectingOfficer}</div>
                    </Show>
                    <div class="flex gap-3 text-txt-muted">
                      <Show when={col.collectionDate}>
                        <span>{col.collectionDate.slice(0, 10)}</span>
                      </Show>
                      <span>{col.itemCount} items</span>
                      <Show when={col.collectionLocation}>
                        <span>{col.collectionLocation}</span>
                      </Show>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </Show>

        {/* --- COC Items --- */}
        <Show when={props.summary.cocItems.length > 0}>
          <button
            class="flex items-center gap-1.5 text-xs text-txt-secondary hover:text-txt w-full text-left py-0.5"
            onClick={() => props.onToggleSection(sKey("coc"))}
          >
            <Show when={props.isSectionExpanded(sKey("coc"))} fallback={<HiOutlineChevronRight class="w-3 h-3" />}>
              <HiOutlineChevronDown class="w-3 h-3" />
            </Show>
            <HiOutlineShieldCheck class="w-3.5 h-3.5" />
            <span>Chain of Custody ({props.summary.cocItems.length})</span>
          </button>
          <Show when={props.isSectionExpanded(sKey("coc"))}>
            <div class="ml-5 col gap-1.5 text-xs max-h-40 overflow-y-auto">
              <For each={props.summary.cocItems}>
                {(coc) => (
                  <div class="p-1.5 rounded bg-bg/50">
                    <div class="flex items-center justify-between">
                      <span class="text-txt font-medium">{coc.cocNumber || coc.evidenceId || "—"}</span>
                      <span class="badge" style="font-size: 10px; padding: 1px 5px;">
                        {coc.status}
                      </span>
                    </div>
                    <Show when={coc.description}>
                      <div class="text-txt-muted truncate">{coc.description}</div>
                    </Show>
                    <div class="flex gap-3 text-txt-muted">
                      <Show when={coc.submittedBy}>
                        <span>From: {coc.submittedBy}</span>
                      </Show>
                      <Show when={coc.receivedBy}>
                        <span>To: {coc.receivedBy}</span>
                      </Show>
                    </div>
                  </div>
                )}
              </For>
            </div>
          </Show>
        </Show>

        {/* --- Form Submissions --- */}
        <Show when={props.summary.formSubmissions.length > 0}>
          <button
            class="flex items-center gap-1.5 text-xs text-txt-secondary hover:text-txt w-full text-left py-0.5"
            onClick={() => props.onToggleSection(sKey("forms"))}
          >
            <Show when={props.isSectionExpanded(sKey("forms"))} fallback={<HiOutlineChevronRight class="w-3 h-3" />}>
              <HiOutlineChevronDown class="w-3 h-3" />
            </Show>
            <HiOutlineClipboardDocumentList class="w-3.5 h-3.5" />
            <span>Forms &amp; Evidence Collections ({props.summary.formSubmissions.length})</span>
          </button>
          <Show when={props.isSectionExpanded(sKey("forms"))}>
            <div class="ml-5 col gap-1.5 text-xs max-h-40 overflow-y-auto">
              <For each={props.summary.formSubmissions}>
                {(form) => (
                  <div class="p-1.5 rounded bg-bg/50">
                    <div class="flex items-center justify-between">
                      <span class="text-txt font-medium">{friendlyTemplateName(form.templateId)}</span>
                      <span class="badge" style="font-size: 10px; padding: 1px 5px;">
                        {form.status}
                      </span>
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
};
