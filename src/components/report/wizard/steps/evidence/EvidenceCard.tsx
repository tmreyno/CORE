// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EvidenceCard — individual evidence item card with selection, metadata expansion,
 * type selector, and notes input.
 */

import { Show, For } from "solid-js";
import { formatBytes } from "../../../../../utils";
import { getDirname } from "../../../../../utils/pathUtils";
import {
  HiOutlineServer,
  HiOutlineCalendarDays,
  HiOutlineCheckCircle,
  HiOutlineChevronDown,
  HiOutlineChevronUp,
  HiOutlineInformationCircle,
  HiOutlineTag,
  HiOutlineDocumentText,
} from "../../../../icons";
import { EVIDENCE_TYPES } from "../../../constants";
import type { EvidenceType } from "../../../types";
import { useWizard } from "../../WizardContext";
import { getDisplayName, getDisplaySize, getAcquisitionDate, detectEvidenceType } from "../../utils/evidenceUtils";
import { countEwfFields, countAd1Fields, countUfedFields } from "./fieldCounters";
import { ContainerMetadata } from "./ContainerMetadata";
import { StoredHashesDetail } from "./StoredHashesDetail";
import type { EvidenceCardProps } from "./types";

export function EvidenceCard(props: EvidenceCardProps) {
  const ctx = useWizard();
  const file = props.group.primaryFile;
  const info = () => ctx.props.fileInfoMap.get(file.path);
  const hashInfo = () => ctx.props.fileHashMap.get(file.path);
  const isSelected = () => ctx.selectedEvidence().has(file.path);

  const displayInfo = () => {
    const i = info();
    const totalSize = getDisplaySize(props.group, i);
    const acqDate = getAcquisitionDate(i);
    return { totalSize, acqDate };
  };

  const displayName = () => getDisplayName(props.group);

  // Auto-detect evidence type if not overridden
  const effectiveType = () =>
    props.evidenceType || detectEvidenceType(file);

  // Count metadata items available for the info badge
  const metadataCount = () => {
    const i = info();
    if (!i) return 0;
    let count = 0;
    if (i.e01) count += countEwfFields(i);
    if (i.l01) count += countEwfFields(i);
    if (i.ad1?.companion_log) count += countAd1Fields(i);
    if (i.ufed) count += countUfedFields(i);
    if (i.archive) count++;
    if (i.raw) count++;
    return count;
  };

  return (
    <div
      class={`rounded-xl border-2 transition-all duration-200 ${
        isSelected()
          ? 'border-accent bg-accent/5 shadow-sm shadow-accent/10'
          : 'border-border/30 bg-surface/30 hover:border-accent/30 hover:bg-surface/50'
      }`}
    >
      {/* Main card row */}
      <div
        class="flex items-start gap-3 p-3.5 cursor-pointer"
        onClick={() => ctx.toggleEvidence(file.path)}
      >
        <div class={`w-5 h-5 rounded-md border-2 flex items-center justify-center mt-0.5 transition-colors ${
          isSelected() ? 'bg-accent border-accent' : 'border-border/50'
        }`}>
          <Show when={isSelected()}>
            <span class="text-white text-xs font-bold">✓</span>
          </Show>
        </div>
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2 flex-wrap">
            <span class="font-medium text-sm truncate">{displayName()}</span>
            <span class="text-xs px-2 py-0.5 bg-accent/10 text-accent rounded-full font-medium">
              {file.container_type}
            </span>
            <Show when={props.group.segmentCount > 1}>
              <span class="text-xs px-2 py-0.5 bg-blue-500/10 text-blue-400 rounded-full font-medium">
                {props.group.segmentCount} segments
              </span>
            </Show>
          </div>
          <div class="text-xs text-txt/50 truncate mt-0.5">
            {getDirname(file.path) || ""}
          </div>
          <div class="flex items-center gap-3 mt-2 flex-wrap">
            <Show when={displayInfo()?.totalSize}>
              <span class="text-xs text-txt/60 flex items-center gap-1">
                <HiOutlineServer class="w-3 h-3" /> {formatBytes(displayInfo()!.totalSize!)}
                <Show when={props.group.segmentCount > 1}>
                  <span class="text-txt/40">(total)</span>
                </Show>
              </span>
            </Show>
            <Show when={displayInfo()?.acqDate}>
              <span class="text-xs text-txt/60 flex items-center gap-1">
                <HiOutlineCalendarDays class="w-3 h-3" /> {displayInfo()!.acqDate}
              </span>
            </Show>
            <Show when={hashInfo()}>
              <span class={`text-xs font-mono flex items-center gap-1 ${
                hashInfo()!.verified === true ? 'text-success' :
                hashInfo()!.verified === false ? 'text-error' : 'text-txt/60'
              }`}>
                <HiOutlineCheckCircle class="w-3 h-3" /> {hashInfo()!.algorithm}
                {hashInfo()!.verified === true && " ✓"}
                {hashInfo()!.verified === false && " ✗"}
              </span>
            </Show>
          </div>
        </div>

        {/* Expand metadata button */}
        <Show when={metadataCount() > 0 || isSelected()}>
          <button
            class="flex items-center gap-1 px-2 py-1 text-xs text-txt/50 hover:text-accent rounded-lg hover:bg-accent/10 transition-colors"
            onClick={(e) => props.onToggleExpand(e)}
            title={props.isExpanded ? "Hide details" : "Show container details"}
          >
            <HiOutlineInformationCircle class="w-3.5 h-3.5" />
            {props.isExpanded
              ? <HiOutlineChevronUp class="w-3 h-3" />
              : <HiOutlineChevronDown class="w-3 h-3" />
            }
          </button>
        </Show>
      </div>

      {/* Expanded metadata panel */}
      <Show when={props.isExpanded}>
        <div class="px-3.5 pb-3.5 space-y-3 border-t border-border/20 pt-3 ml-8">
          {/* Evidence Type & Notes */}
          <div class="grid grid-cols-2 gap-3">
            <div>
              <label class="block text-xs text-txt/50 mb-1">
                <HiOutlineTag class="w-3 h-3 inline mr-1" />Evidence Type
              </label>
              <select
                class="w-full px-2.5 py-1.5 bg-bg-panel border border-border/50 rounded-lg text-xs focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                value={effectiveType()}
                onChange={(e) => {
                  e.stopPropagation();
                  props.onTypeChange(e.currentTarget.value as EvidenceType);
                }}
                onClick={(e) => e.stopPropagation()}
              >
                <For each={EVIDENCE_TYPES}>
                  {(t) => <option value={t.value}>{t.label}</option>}
                </For>
              </select>
            </div>
            <div>
              <label class="block text-xs text-txt/50 mb-1">
                <HiOutlineDocumentText class="w-3 h-3 inline mr-1" />Notes
              </label>
              <input
                type="text"
                class="w-full px-2.5 py-1.5 bg-bg-panel border border-border/50 rounded-lg text-xs focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                value={props.notes}
                onInput={(e) => {
                  e.stopPropagation();
                  props.onNotesChange(e.currentTarget.value);
                }}
                onClick={(e) => e.stopPropagation()}
                placeholder="Additional notes for this evidence item..."
              />
            </div>
          </div>

          {/* Container-specific metadata */}
          <ContainerMetadata info={info()} />

          {/* Stored hashes detail */}
          <StoredHashesDetail info={info()} />
        </div>
      </Show>
    </div>
  );
}
