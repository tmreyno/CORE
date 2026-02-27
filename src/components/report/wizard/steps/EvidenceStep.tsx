// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EvidenceStep - Third wizard step for evidence selection
 *
 * Features:
 * - Select/deselect evidence items for the report
 * - Expandable metadata panels showing container-specific details
 * - Editable evidence type per item
 * - Per-item notes for report annotations
 * - Chain of custody sub-section
 */

import { createSignal, For, Show } from "solid-js";
import { formatBytes } from "../../../../utils";
import { getDirname } from "../../../../utils/pathUtils";
import {
  HiOutlineCircleStack,
  HiOutlineXMark,
  HiOutlineServer,
  HiOutlineCalendarDays,
  HiOutlineCheckCircle,
  HiOutlineChevronDown,
  HiOutlineChevronUp,
  HiOutlineInformationCircle,
  HiOutlineTag,
  HiOutlineFingerPrint,
  HiOutlineShieldCheck,
  HiOutlineDevicePhoneMobile,
  HiOutlineCpuChip,
  HiOutlineDocumentText,
} from "../../../icons";
import { EVIDENCE_TYPES } from "../../constants";
import type { EvidenceType } from "../../types";
import { useWizard } from "../WizardContext";
import { getDisplayName, getDisplaySize, getAcquisitionDate, detectEvidenceType } from "../utils/evidenceUtils";
import type { ContainerInfo } from "../../../../types/containerInfo";
import type { EvidenceGroup } from "../types";

export function EvidenceStep() {
  const ctx = useWizard();

  // Track which evidence cards are expanded to show metadata
  const [expandedCards, setExpandedCards] = createSignal<Set<string>>(new Set());
  // Per-evidence notes (path → notes string)
  const [evidenceNotes, setEvidenceNotes] = createSignal<Map<string, string>>(new Map());
  // Per-evidence type overrides (path → EvidenceType)
  const [evidenceTypes, setEvidenceTypes] = createSignal<Map<string, EvidenceType>>(new Map());

  const toggleExpanded = (path: string, e?: MouseEvent) => {
    e?.stopPropagation();
    setExpandedCards((prev) => {
      const next = new Set(prev);
      if (next.has(path)) next.delete(path);
      else next.add(path);
      return next;
    });
  };

  const setNoteForEvidence = (path: string, note: string) => {
    setEvidenceNotes((prev) => {
      const next = new Map(prev);
      if (note) next.set(path, note);
      else next.delete(path);
      return next;
    });
  };

  const setTypeForEvidence = (path: string, type: EvidenceType) => {
    setEvidenceTypes((prev) => {
      const next = new Map(prev);
      next.set(path, type);
      return next;
    });
  };

  return (
    <div class="space-y-5">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-2">
          <HiOutlineCircleStack class="w-5 h-5 text-accent" />
          <h3 class="text-base font-semibold">Evidence Items</h3>
        </div>
        <div class="flex items-center gap-3">
          <span class="text-sm text-txt/60">
            <span class="font-medium text-accent">{ctx.selectedEvidence().size}</span> of {ctx.groupedEvidence().length} selected
          </span>
          <Show when={ctx.groupedEvidence().length > 0}>
            <button
              class="text-xs text-accent hover:underline"
              onClick={() => {
                if (ctx.selectedEvidence().size === ctx.groupedEvidence().length) {
                  ctx.setSelectedEvidence(new Set<string>());
                } else {
                  ctx.setSelectedEvidence(new Set<string>(ctx.groupedEvidence().map(g => g.primaryFile.path)));
                }
              }}
            >
              {ctx.selectedEvidence().size === ctx.groupedEvidence().length ? 'Deselect All' : 'Select All'}
            </button>
          </Show>
        </div>
      </div>

      <Show when={ctx.groupedEvidence().length === 0}>
        <div class="text-center py-12 bg-surface/30 rounded-xl border border-border/30">
          <div class="w-16 h-16 mx-auto mb-4 rounded-2xl bg-accent/10 flex items-center justify-center">
            <span class="text-3xl">📂</span>
          </div>
          <p class="font-medium text-txt/80">No evidence files discovered</p>
          <p class="text-sm text-txt/50 mt-1">Scan a directory first to discover forensic images</p>
        </div>
      </Show>

      <div class="space-y-2 max-h-[400px] overflow-y-auto pr-1">
        <For each={ctx.groupedEvidence()}>
          {(group) => (
            <EvidenceCard
              group={group}
              isExpanded={expandedCards().has(group.primaryFile.path)}
              onToggleExpand={(e) => toggleExpanded(group.primaryFile.path, e)}
              notes={evidenceNotes().get(group.primaryFile.path) || ""}
              onNotesChange={(n) => setNoteForEvidence(group.primaryFile.path, n)}
              evidenceType={evidenceTypes().get(group.primaryFile.path)}
              onTypeChange={(t) => setTypeForEvidence(group.primaryFile.path, t)}
            />
          )}
        </For>
      </div>

      {/* Chain of Custody Section */}
      <Show when={ctx.enabledSections().chainOfCustody}>
        <ChainOfCustodySection />
      </Show>
    </div>
  );
}

// =============================================================================
// EVIDENCE CARD
// =============================================================================

interface EvidenceCardProps {
  group: EvidenceGroup;
  isExpanded: boolean;
  onToggleExpand: (e?: MouseEvent) => void;
  notes: string;
  onNotesChange: (notes: string) => void;
  evidenceType?: EvidenceType;
  onTypeChange: (type: EvidenceType) => void;
}

function EvidenceCard(props: EvidenceCardProps) {
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

// =============================================================================
// CONTAINER METADATA PANEL
// =============================================================================

/** Renders a labeled metadata row */
function MetaRow(props: { label: string; value: string | number | undefined | null }) {
  return (
    <Show when={props.value != null && props.value !== ""}>
      <div class="flex items-baseline gap-2 text-xs">
        <span class="text-txt/40 min-w-[100px] shrink-0">{props.label}</span>
        <span class="text-txt/80 font-mono break-all">{String(props.value)}</span>
      </div>
    </Show>
  );
}

/** Renders container-specific metadata based on the container type */
function ContainerMetadata(props: { info: ContainerInfo | undefined }) {
  return (
    <Show when={props.info}>
      {(ci) => (
        <>
          {/* EWF (E01/L01) Metadata */}
          <Show when={ci().e01 || ci().l01}>
            {(_ewf) => {
              const ewf = () => ci().e01 || ci().l01;
              return (
                <div class="space-y-1.5 p-2.5 bg-bg/50 rounded-lg border border-border/20">
                  <div class="flex items-center gap-1.5 text-xs font-medium text-accent/80 mb-1.5">
                    <HiOutlineCpuChip class="w-3.5 h-3.5" />
                    <span>EWF Container Details</span>
                  </div>
                  <div class="grid grid-cols-2 gap-x-4 gap-y-1">
                    <MetaRow label="Format" value={ewf()!.format_version} />
                    <MetaRow label="Segments" value={ewf()!.segment_count} />
                    <MetaRow label="Compression" value={ewf()!.compression} />
                    <MetaRow label="Sector Size" value={ewf()!.bytes_per_sector ? `${ewf()!.bytes_per_sector} bytes` : undefined} />
                    <MetaRow label="Total Size" value={ewf()!.total_size ? formatBytes(ewf()!.total_size) : undefined} />
                    <MetaRow label="Chunk Count" value={ewf()!.chunk_count} />
                    <MetaRow label="Case Number" value={ewf()!.case_number} />
                    <MetaRow label="Evidence #" value={ewf()!.evidence_number} />
                    <MetaRow label="Examiner" value={ewf()!.examiner_name} />
                    <MetaRow label="Acquiry Date" value={ewf()!.acquiry_date} />
                    <MetaRow label="Model" value={ewf()!.model} />
                    <MetaRow label="Serial" value={ewf()!.serial_number} />
                  </div>
                  <Show when={ewf()!.description}>
                    <div class="mt-1">
                      <MetaRow label="Description" value={ewf()!.description} />
                    </div>
                  </Show>
                  <Show when={ewf()!.notes}>
                    <div class="mt-1">
                      <MetaRow label="Notes" value={ewf()!.notes} />
                    </div>
                  </Show>
                </div>
              );
            }}
          </Show>

          {/* AD1 Companion Log Metadata */}
          <Show when={ci().ad1?.companion_log}>
            {(log) => (
              <div class="space-y-1.5 p-2.5 bg-bg/50 rounded-lg border border-border/20">
                <div class="flex items-center gap-1.5 text-xs font-medium text-accent/80 mb-1.5">
                  <HiOutlineDocumentText class="w-3.5 h-3.5" />
                  <span>AD1 Companion Log</span>
                </div>
                <div class="grid grid-cols-2 gap-x-4 gap-y-1">
                  <MetaRow label="Case Number" value={log().case_number} />
                  <MetaRow label="Evidence #" value={log().evidence_number} />
                  <MetaRow label="Examiner" value={log().examiner} />
                  <MetaRow label="Organization" value={log().organization} />
                  <MetaRow label="Acq. Date" value={log().acquisition_date} />
                  <MetaRow label="Acq. Tool" value={log().acquisition_tool} />
                  <MetaRow label="Acq. Method" value={log().acquisition_method} />
                  <MetaRow label="Source Device" value={log().source_device} />
                  <MetaRow label="Source Path" value={log().source_path} />
                  <MetaRow label="Total Items" value={log().total_items} />
                  <MetaRow label="Total Size" value={log().total_size ? formatBytes(log().total_size!) : undefined} />
                </div>
                <Show when={log().notes}>
                  <div class="mt-1">
                    <MetaRow label="Notes" value={log().notes} />
                  </div>
                </Show>
              </div>
            )}
          </Show>

          {/* AD1 Segment Info */}
          <Show when={ci().ad1 && !ci().ad1!.companion_log}>
            {(_ad1) => {
              const ad1 = () => ci().ad1!;
              return (
                <div class="space-y-1.5 p-2.5 bg-bg/50 rounded-lg border border-border/20">
                  <div class="flex items-center gap-1.5 text-xs font-medium text-accent/80 mb-1.5">
                    <HiOutlineDocumentText class="w-3.5 h-3.5" />
                    <span>AD1 Container</span>
                  </div>
                  <div class="grid grid-cols-2 gap-x-4 gap-y-1">
                    <MetaRow label="Item Count" value={ad1().item_count} />
                    <MetaRow label="Total Size" value={ad1().total_size ? formatBytes(ad1().total_size!) : undefined} />
                    <Show when={ad1().segment_files}>
                      <MetaRow label="Segments" value={ad1().segment_files!.length} />
                    </Show>
                    <Show when={ad1().missing_segments && ad1().missing_segments!.length > 0}>
                      <MetaRow label="Missing" value={ad1().missing_segments!.join(", ")} />
                    </Show>
                  </div>
                </div>
              );
            }}
          </Show>

          {/* UFED Metadata */}
          <Show when={ci().ufed}>
            {(ufed) => (
              <div class="space-y-1.5 p-2.5 bg-bg/50 rounded-lg border border-border/20">
                <div class="flex items-center gap-1.5 text-xs font-medium text-accent/80 mb-1.5">
                  <HiOutlineDevicePhoneMobile class="w-3.5 h-3.5" />
                  <span>UFED / Cellebrite</span>
                </div>
                <div class="grid grid-cols-2 gap-x-4 gap-y-1">
                  <MetaRow label="Format" value={ufed().format} />
                  <MetaRow label="Size" value={formatBytes(ufed().size)} />
                  <Show when={ufed().device_info}>
                    <MetaRow label="Device" value={ufed().device_info!.full_name || `${ufed().device_info!.vendor || ""} ${ufed().device_info!.model || ""}`.trim()} />
                    <MetaRow label="IMEI" value={ufed().device_info!.imei} />
                    <MetaRow label="OS Version" value={ufed().device_info!.os_version} />
                    <MetaRow label="Serial" value={ufed().device_info!.serial_number} />
                  </Show>
                  <Show when={ufed().case_info}>
                    <MetaRow label="Case ID" value={ufed().case_info!.case_identifier} />
                    <MetaRow label="Crime Type" value={ufed().case_info!.crime_type} />
                    <MetaRow label="Department" value={ufed().case_info!.department} />
                    <MetaRow label="Examiner" value={ufed().case_info!.examiner_name} />
                  </Show>
                  <Show when={ufed().extraction_info}>
                    <MetaRow label="Acq. Tool" value={ufed().extraction_info!.acquisition_tool} />
                    <MetaRow label="Tool Version" value={ufed().extraction_info!.tool_version} />
                    <MetaRow label="Extraction" value={ufed().extraction_info!.extraction_type} />
                    <MetaRow label="Started" value={ufed().extraction_info!.start_time} />
                    <MetaRow label="Ended" value={ufed().extraction_info!.end_time} />
                  </Show>
                </div>
              </div>
            )}
          </Show>

          {/* Archive Metadata */}
          <Show when={ci().archive}>
            {(arch) => (
              <div class="space-y-1.5 p-2.5 bg-bg/50 rounded-lg border border-border/20">
                <div class="flex items-center gap-1.5 text-xs font-medium text-accent/80 mb-1.5">
                  <HiOutlineServer class="w-3.5 h-3.5" />
                  <span>Archive Details</span>
                </div>
                <div class="grid grid-cols-2 gap-x-4 gap-y-1">
                  <MetaRow label="Format" value={arch().format} />
                  <MetaRow label="Total Size" value={formatBytes(arch().total_size)} />
                  <MetaRow label="Entries" value={arch().entry_count} />
                  <MetaRow label="Encrypted" value={arch().aes_encrypted ? "Yes (AES)" : arch().encrypted_headers ? "Yes (Headers)" : "No"} />
                  <Show when={arch().is_multipart}>
                    <MetaRow label="Segments" value={arch().segment_count} />
                  </Show>
                  <Show when={arch().version}>
                    <MetaRow label="Version" value={arch().version} />
                  </Show>
                  <Show when={arch().cellebrite_detected}>
                    <MetaRow label="Cellebrite" value={`Detected (${arch().cellebrite_files?.length || 0} files)`} />
                  </Show>
                </div>
              </div>
            )}
          </Show>

          {/* Raw Image Metadata */}
          <Show when={ci().raw}>
            {(raw) => (
              <div class="space-y-1.5 p-2.5 bg-bg/50 rounded-lg border border-border/20">
                <div class="flex items-center gap-1.5 text-xs font-medium text-accent/80 mb-1.5">
                  <HiOutlineServer class="w-3.5 h-3.5" />
                  <span>Raw Image</span>
                </div>
                <div class="grid grid-cols-2 gap-x-4 gap-y-1">
                  <MetaRow label="Segments" value={raw().segment_count} />
                  <MetaRow label="Total Size" value={formatBytes(raw().total_size)} />
                  <MetaRow label="First" value={raw().first_segment} />
                  <MetaRow label="Last" value={raw().last_segment} />
                </div>
              </div>
            )}
          </Show>

          {/* Companion Log (E01/other) */}
          <Show when={ci().companion_log}>
            {(log) => (
              <div class="space-y-1.5 p-2.5 bg-bg/50 rounded-lg border border-border/20">
                <div class="flex items-center gap-1.5 text-xs font-medium text-accent/80 mb-1.5">
                  <HiOutlineDocumentText class="w-3.5 h-3.5" />
                  <span>Companion Log</span>
                </div>
                <div class="grid grid-cols-2 gap-x-4 gap-y-1">
                  <MetaRow label="Case Number" value={log().case_number} />
                  <MetaRow label="Evidence #" value={log().evidence_number} />
                  <MetaRow label="Examiner" value={log().examiner} />
                  <MetaRow label="Description" value={log().unique_description} />
                  <MetaRow label="Acq. Started" value={log().acquisition_started} />
                  <MetaRow label="Acq. Finished" value={log().acquisition_finished} />
                </div>
              </div>
            )}
          </Show>
        </>
      )}
    </Show>
  );
}

// =============================================================================
// STORED HASHES DETAIL
// =============================================================================

/** Shows all stored hashes from the container metadata */
function StoredHashesDetail(props: { info: ContainerInfo | undefined }) {
  const storedHashes = () => {
    const ci = props.info;
    if (!ci) return [];

    const hashes: { algorithm: string; hash: string; verified?: boolean | null; source: string }[] = [];

    // EWF stored hashes
    const ewf = ci.e01 || ci.l01;
    if (ewf?.stored_hashes) {
      for (const sh of ewf.stored_hashes) {
        hashes.push({ algorithm: sh.algorithm, hash: sh.hash, verified: sh.verified, source: "EWF" });
      }
    }

    // Companion log hashes
    if (ci.companion_log?.stored_hashes) {
      for (const sh of ci.companion_log.stored_hashes) {
        hashes.push({ algorithm: sh.algorithm, hash: sh.hash, verified: sh.verified, source: "Log" });
      }
    }

    // AD1 companion log hashes
    const ad1Log = ci.ad1?.companion_log;
    if (ad1Log) {
      if (ad1Log.md5_hash) hashes.push({ algorithm: "MD5", hash: ad1Log.md5_hash, source: "AD1 Log" });
      if (ad1Log.sha1_hash) hashes.push({ algorithm: "SHA-1", hash: ad1Log.sha1_hash, source: "AD1 Log" });
      if (ad1Log.sha256_hash) hashes.push({ algorithm: "SHA-256", hash: ad1Log.sha256_hash, source: "AD1 Log" });
    }

    // UFED stored hashes
    if (ci.ufed?.stored_hashes) {
      for (const sh of ci.ufed.stored_hashes) {
        hashes.push({ algorithm: sh.algorithm, hash: sh.hash, source: "UFED" });
      }
    }

    return hashes;
  };

  return (
    <Show when={storedHashes().length > 0}>
      <div class="space-y-1.5 p-2.5 bg-bg/50 rounded-lg border border-border/20">
        <div class="flex items-center gap-1.5 text-xs font-medium text-accent/80 mb-1.5">
          <HiOutlineFingerPrint class="w-3.5 h-3.5" />
          <span>Stored Hashes</span>
          <span class="text-txt/40">({storedHashes().length})</span>
        </div>
        <div class="space-y-1">
          <For each={storedHashes()}>
            {(h) => (
              <div class="flex items-center gap-2 text-xs">
                <span class={`inline-flex items-center gap-1 ${
                  h.verified === true ? 'text-success' :
                  h.verified === false ? 'text-error' : 'text-txt/60'
                }`}>
                  <Show when={h.verified != null}>
                    <HiOutlineShieldCheck class="w-3 h-3" />
                  </Show>
                  <span class="min-w-[50px] text-txt/50">{h.algorithm}</span>
                </span>
                <span class="font-mono text-txt/70 break-all">{h.hash}</span>
                <span class="text-txt/30 text-[10px] ml-auto shrink-0">{h.source}</span>
              </div>
            )}
          </For>
        </div>
      </div>
    </Show>
  );
}

// =============================================================================
// METADATA FIELD COUNTERS (for info badge)
// =============================================================================

function countEwfFields(ci: ContainerInfo): number {
  const ewf = ci.e01 || ci.l01;
  if (!ewf) return 0;
  let count = 3; // format, compression, size always present
  if (ewf.case_number) count++;
  if (ewf.examiner_name) count++;
  if (ewf.evidence_number) count++;
  if (ewf.description) count++;
  if (ewf.model) count++;
  if (ewf.serial_number) count++;
  if (ewf.stored_hashes?.length) count++;
  return count;
}

function countAd1Fields(ci: ContainerInfo): number {
  const log = ci.ad1?.companion_log;
  if (!log) return 0;
  let count = 0;
  if (log.case_number) count++;
  if (log.examiner) count++;
  if (log.evidence_number) count++;
  if (log.acquisition_date) count++;
  if (log.source_device) count++;
  if (log.acquisition_tool) count++;
  return count;
}

function countUfedFields(ci: ContainerInfo): number {
  const ufed = ci.ufed;
  if (!ufed) return 0;
  let count = 2; // format, size always present
  if (ufed.device_info) count += 3;
  if (ufed.case_info) count += 2;
  if (ufed.extraction_info) count += 3;
  return count;
}

// =============================================================================
// CHAIN OF CUSTODY
// =============================================================================

/**
 * Chain of Custody sub-section
 */
function ChainOfCustodySection() {
  const ctx = useWizard();

  return (
    <div class="mt-6 pt-5 border-t border-border/30">
      <div class="flex items-center justify-between mb-4">
        <div class="flex items-center gap-2">
          <span class="text-lg">🔗</span>
          <h4 class="text-sm font-semibold">Chain of Custody</h4>
          <span class="text-xs px-2 py-0.5 bg-accent/10 text-accent rounded-full font-medium">
            {ctx.chainOfCustody().length} records
          </span>
        </div>
        <button
          type="button"
          class="px-3 py-1.5 rounded-lg text-sm font-medium bg-accent/10 text-accent hover:bg-accent/20 transition-colors"
          onClick={ctx.addCustodyRecord}
        >
          + Add Record
        </button>
      </div>

      <Show when={ctx.chainOfCustody().length === 0}>
        <div class="text-center py-8 bg-surface/30 rounded-xl border-2 border-dashed border-border/30">
          <div class="w-12 h-12 mx-auto mb-3 rounded-xl bg-accent/10 flex items-center justify-center">
            <span class="text-2xl">📋</span>
          </div>
          <p class="text-sm font-medium text-txt/70">No chain of custody records</p>
          <p class="text-xs text-txt/50 mt-1">Add records to document evidence handling</p>
        </div>
      </Show>

      <div class="space-y-3">
        <For each={ctx.chainOfCustody()}>
          {(record, index) => (
            <div class="p-4 bg-surface/50 border border-border/30 rounded-xl">
              <div class="grid grid-cols-4 gap-3">
                <div>
                  <label class="block text-xs text-txt/50 mb-1.5">Date/Time</label>
                  <input
                    type="datetime-local"
                    class="w-full px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                    value={record.timestamp.slice(0, 16)}
                    onInput={(e) => ctx.updateCustodyRecord(index(), { timestamp: new Date(e.currentTarget.value).toISOString() })}
                  />
                </div>
                <div>
                  <label class="block text-xs text-txt/50 mb-1.5">Action</label>
                  <select
                    class="w-full px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                    value={record.action}
                    onChange={(e) => ctx.updateCustodyRecord(index(), { action: e.currentTarget.value })}
                  >
                    <option value="Received">Received</option>
                    <option value="Transferred">Transferred</option>
                    <option value="Imaged">Imaged</option>
                    <option value="Analyzed">Analyzed</option>
                    <option value="Stored">Stored</option>
                    <option value="Released">Released</option>
                    <option value="Returned">Returned</option>
                    <option value="Destroyed">Destroyed</option>
                  </select>
                </div>
                <div>
                  <label class="block text-xs text-txt/50 mb-1.5">Handler</label>
                  <input
                    type="text"
                    class="w-full px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                    value={record.handler}
                    onInput={(e) => ctx.updateCustodyRecord(index(), { handler: e.currentTarget.value })}
                    placeholder="Name of handler"
                  />
                </div>
                <div>
                  <label class="block text-xs text-txt/50 mb-1.5">Location</label>
                  <input
                    type="text"
                    class="w-full px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                    value={record.location || ""}
                    onInput={(e) => ctx.updateCustodyRecord(index(), { location: e.currentTarget.value || undefined })}
                    placeholder="Storage location"
                  />
                </div>
              </div>
              <div class="mt-3 flex gap-2">
                <input
                  type="text"
                  class="flex-1 px-2.5 py-2 bg-bg-panel border border-border/50 rounded-lg text-sm focus:outline-none focus:border-accent focus:ring-1 focus:ring-accent/20 transition-all"
                  value={record.notes || ""}
                  onInput={(e) => ctx.updateCustodyRecord(index(), { notes: e.currentTarget.value || undefined })}
                  placeholder="Additional notes..."
                />
                <button
                  type="button"
                  class="p-2 text-error/70 hover:text-error hover:bg-error/10 rounded-lg transition-colors"
                  onClick={() => ctx.removeCustodyRecord(index())}
                  title="Remove custody record"
                >
                  <HiOutlineXMark class="w-4 h-4" />
                </button>
              </div>
            </div>
          )}
        </For>
      </div>
    </div>
  );
}
