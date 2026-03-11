// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// PhysicalImageMode - E01 disk image creation UI

import { Component, Show, For, Accessor, Setter, createMemo, createSignal } from "solid-js";
import {
  HiOutlineCircleStack,
  HiOutlineFingerPrint,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineCog6Tooth,
  HiOutlineInformationCircle,
} from "../icons";
import { CaseMetadataSection } from "./CaseMetadataSection";
import { SplitSizeSelector } from "./SplitSizeSelector";

// --- Types ---

export type EwfFormatId = "e01" | "encase6" | "encase7" | "v2encase7" | "ftk";

interface FormatInfo {
  id: EwfFormatId;
  name: string;
  extension: string;
  description: string;
  supportsV2: boolean;
}

// --- Constants ---

const EWF_FORMATS: FormatInfo[] = [
  { id: "e01", name: "EnCase 5 (.E01)", extension: ".E01", description: "Most compatible format", supportsV2: false },
  { id: "encase6", name: "EnCase 6 (.E01)", extension: ".E01", description: "Improved compression", supportsV2: false },
  { id: "encase7", name: "EnCase 7 (.E01)", extension: ".E01", description: "LZMA support", supportsV2: false },
  { id: "v2encase7", name: "EnCase 7 v2 (.Ex01)", extension: ".Ex01", description: "EWF v2 - supports BZIP2", supportsV2: true },
  { id: "ftk", name: "FTK (.E01)", extension: ".E01", description: "AccessData FTK compatible", supportsV2: false },
];

// --- Props ---

export interface PhysicalImageModeProps {
  // Image name
  imageName: Accessor<string>;
  setImageName: Setter<string>;
  // EWF format
  format: Accessor<string>;
  setFormat: Setter<string>;
  // Compression
  compression: Accessor<string>;
  setCompression: Setter<string>;
  compressionMethod: Accessor<string>;
  setCompressionMethod: Setter<string>;
  // Hashes
  computeMd5: Accessor<boolean>;
  setComputeMd5: Setter<boolean>;
  computeSha1: Accessor<boolean>;
  setComputeSha1: Setter<boolean>;
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

export const PhysicalImageMode: Component<PhysicalImageModeProps> = (props) => {
  const [showCaseMetadata, setShowCaseMetadata] = createSignal(false);

  const selectedFormat = createMemo(() =>
    EWF_FORMATS.find(f => f.id === props.format()) ?? EWF_FORMATS[0]
  );

  return (
    <div class="space-y-3">
      {/* Image Name + EWF Format */}
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
            <span class="text-sm text-txt-muted font-mono">
              {selectedFormat().extension}
            </span>
          </div>
        </div>
        <div class="space-y-1">
          <label class="label flex items-center gap-1">
            <HiOutlineCircleStack class="w-3.5 h-3.5" />
            EWF Format
          </label>
          <select
            class="input-sm"
            value={props.format()}
            onChange={(e) => props.setFormat(e.currentTarget.value)}
          >
            <For each={EWF_FORMATS}>
              {(fmt) => (
                <option value={fmt.id}>{fmt.name}</option>
              )}
            </For>
          </select>
          <div class="text-xs text-txt-muted mt-0.5">{selectedFormat().description}</div>
        </div>
      </div>

      {/* Compression - Level + Method side by side */}
      <div class="grid grid-cols-2 gap-2">
        <div class="space-y-1">
          <label class="label text-xs">Compression Level</label>
          <select
            class="input-sm"
            value={props.compression()}
            onChange={(e) => props.setCompression(e.currentTarget.value)}
          >
            <option value="none">None</option>
            <option value="fast">Fast</option>
            <option value="best">Best</option>
          </select>
        </div>
        <div class="space-y-1">
          <label class="label text-xs">Method</label>
          <select
            class="input-sm"
            value={props.compressionMethod()}
            onChange={(e) => props.setCompressionMethod(e.currentTarget.value)}
          >
            <option value="deflate">Deflate (zlib)</option>
            <option value="bzip2" disabled={!selectedFormat().supportsV2}>
              BZIP2 {!selectedFormat().supportsV2 ? "(V2 only)" : ""}
            </option>
          </select>
          <Show when={props.compressionMethod() === "bzip2" && !selectedFormat().supportsV2}>
            <div class="text-xs text-warning mt-0.5">
              BZIP2 requires V2 format (Ex01)
            </div>
          </Show>
        </div>
      </div>

      {/* Embedded Hashes */}
      <div class="space-y-2">
        <label class="label flex items-center gap-1">
          <HiOutlineFingerPrint class="w-3.5 h-3.5" />
          Embedded Hashes
        </label>
        <div class="flex gap-4 pl-1">
          <label class="flex items-center gap-2 text-xs cursor-pointer">
            <input
              type="checkbox"
              checked={props.computeMd5()}
              onChange={(e) => props.setComputeMd5(e.currentTarget.checked)}
              class="accent-accent"
            />
            <span class="text-txt">MD5</span>
          </label>
          <label class="flex items-center gap-2 text-xs cursor-pointer">
            <input
              type="checkbox"
              checked={props.computeSha1()}
              onChange={(e) => props.setComputeSha1(e.currentTarget.checked)}
              class="accent-accent"
            />
            <span class="text-txt">SHA-1</span>
          </label>
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
          About E01 Images
        </div>
        <p class="text-xs text-txt-muted">
          Wraps raw evidence files into the Expert Witness Format with case metadata,
          compression, and integrity hashes. EnCase 5 (.E01) is recommended for maximum
          compatibility. Ex01 v2 supports BZIP2 but may not work with older tools.
        </p>
      </div>
    </div>
  );
};
