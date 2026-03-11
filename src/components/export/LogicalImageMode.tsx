// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// LogicalImageMode - L01 logical evidence container creation UI

import { Component, Show, Accessor, Setter, createSignal } from "solid-js";
import {
  HiOutlineFingerPrint,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineCog6Tooth,
  HiOutlineInformationCircle,
} from "../icons";
import { CaseMetadataSection } from "./CaseMetadataSection";
import { SplitSizeSelector } from "./SplitSizeSelector";

// --- Props ---

export interface LogicalImageModeProps {
  // Image name
  imageName: Accessor<string>;
  setImageName: Setter<string>;
  // Compression
  compression: Accessor<string>;
  setCompression: Setter<string>;
  // Case metadata
  caseNumber: Accessor<string>;
  setCaseNumber: Setter<string>;
  evidenceNumber: Accessor<string>;
  setEvidenceNumber: Setter<string>;
  examinerName: Accessor<string>;
  setExaminerName: Setter<string>;
  description: Accessor<string>;
  setDescription: Setter<string>;
  notes: Accessor<string>;
  setNotes: Setter<string>;
  // Advanced
  segmentSize: Accessor<number>;
  setSegmentSize: Setter<number>;
  showAdvanced: Accessor<boolean>;
  setShowAdvanced: Setter<boolean>;
}

// --- Component ---

export const LogicalImageMode: Component<LogicalImageModeProps> = (props) => {
  const [showCaseMetadata, setShowCaseMetadata] = createSignal(false);

  return (
    <div class="space-y-3">
      {/* Compact info callout */}
      <div class="flex items-start gap-2 bg-bg-secondary border border-border rounded-lg p-2.5">
        <HiOutlineInformationCircle class="w-4 h-4 text-info mt-0.5 flex-shrink-0" />
        <p class="text-xs text-txt-muted leading-relaxed">
          Creates an <span class="text-txt-secondary font-medium">L01 logical evidence container</span> that
          packages files and folders into a compressed, hash-verified container with forensic
          case metadata. Compatible with EnCase, FTK Imager, and AXIOM.
        </p>
      </div>

      {/* Image Name + Compression */}
      <div class="grid grid-cols-2 gap-2">
        <div class="space-y-1">
          <label class="label">Image Name</label>
          <div class="flex items-center gap-2">
            <input
              class="input-sm flex-1"
              type="text"
              value={props.imageName()}
              onInput={(e) => props.setImageName(e.currentTarget.value)}
              placeholder="evidence"
            />
            <span class="text-sm text-txt-muted font-mono">.L01</span>
          </div>
        </div>
        <div class="space-y-1">
          <label class="label text-xs">Compression</label>
          <select
            class="input-sm"
            value={props.compression()}
            onChange={(e) => props.setCompression(e.currentTarget.value)}
          >
            <option value="none">None (Store)</option>
            <option value="fast">Fast (zlib default)</option>
            <option value="best">Best (zlib maximum)</option>
          </select>
        </div>
      </div>

      {/* Embedded Hashes - always both MD5 + SHA-1 */}
      <div class="space-y-2">
        <label class="label flex items-center gap-1">
          <HiOutlineFingerPrint class="w-3.5 h-3.5" />
          Embedded Hashes
        </label>
        <div class="flex gap-4 pl-1">
          <label class="flex items-center gap-2 text-xs cursor-default">
            <input
              type="checkbox"
              checked={true}
              disabled
              class="accent-accent"
            />
            <span class="text-txt">MD5</span>
          </label>
          <label class="flex items-center gap-2 text-xs cursor-default">
            <input
              type="checkbox"
              checked={true}
              disabled
              class="accent-accent"
            />
            <span class="text-txt">SHA-1</span>
          </label>
        </div>
        <div class="text-xs text-txt-muted pl-1">
          L01 always embeds both MD5 and SHA-1 (per-file and image-level)
        </div>
      </div>

      {/* Case Metadata */}
      <CaseMetadataSection
        isOpen={showCaseMetadata}
        setIsOpen={setShowCaseMetadata}
        caseNumber={props.caseNumber}
        setCaseNumber={props.setCaseNumber}
        evidenceNumber={props.evidenceNumber}
        setEvidenceNumber={props.setEvidenceNumber}
        examinerName={props.examinerName}
        setExaminerName={props.setExaminerName}
        description={props.description}
        setDescription={props.setDescription}
        notes={props.notes}
        setNotes={props.setNotes}
      />

      {/* Advanced */}
      <div class="space-y-2">
        <button
          class="flex items-center gap-1 text-xs text-txt-secondary hover:text-txt"
          onClick={() => props.setShowAdvanced(!props.showAdvanced())}
        >
          <Show when={props.showAdvanced()} fallback={<HiOutlineChevronRight class="w-3.5 h-3.5" />}>
            <HiOutlineChevronDown class="w-3.5 h-3.5" />
          </Show>
          <HiOutlineCog6Tooth class="w-3.5 h-3.5" />
          Advanced
        </button>

        <Show when={props.showAdvanced()}>
          <div class="space-y-3 pl-5 pt-1">
            <SplitSizeSelector
              valueMb={props.segmentSize}
              setValueMb={props.setSegmentSize}
              label="Segment Size"
            />
          </div>
        </Show>
      </div>

      {/* Info Card */}
      <div class="info-card">
        <div class="info-card-title">
          <HiOutlineInformationCircle class="w-4 h-4 text-info" />
          About L01 Containers
        </div>
        <p class="text-xs text-txt-muted">
          L01 is the logical evidence variant of the Expert Witness Format. Unlike E01
          (disk images), L01 stores individual files and directories with per-file
          MD5/SHA-1 hashes, timestamps, and full directory hierarchy. Compatible with
          EnCase, FTK Imager, AXIOM, and other forensic tools.
        </p>
      </div>
    </div>
  );
};
