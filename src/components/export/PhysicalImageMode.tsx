// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// PhysicalImageMode - E01 / Raw disk image creation UI

import { Component, Show, For, Accessor, Setter, createMemo, createSignal } from "solid-js";
import {
  HiOutlineCircleStack,
  HiOutlineFingerPrint,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineCog6Tooth,
  HiOutlineInformationCircle,
  HiOutlineCheckBadge,
} from "../icons";
import { CaseMetadataSection } from "./CaseMetadataSection";
import { SplitSizeSelector } from "./SplitSizeSelector";

export type PhysicalFormat = "ewf" | "raw";

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
  // Physical imaging format selector
  physicalFormat: Accessor<PhysicalFormat>;
  setPhysicalFormat: Setter<PhysicalFormat>;
  // EWF-specific props (only used when physicalFormat === "ewf")
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
  // Verify after write (shared)
  ewfVerifyAfterWrite: Accessor<boolean>;
  setEwfVerifyAfterWrite: Setter<boolean>;
  rawVerifyAfterWrite: Accessor<boolean>;
  setRawVerifyAfterWrite: Setter<boolean>;
  // Hashes (EWF)
  computeMd5: Accessor<boolean>;
  setComputeMd5: Setter<boolean>;
  computeSha1: Accessor<boolean>;
  setComputeSha1: Setter<boolean>;
  // Case metadata (EWF)
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
  // Advanced (EWF)
  segmentSize: Accessor<number>;
  setSegmentSize: Setter<number>;
  showAdvanced: Accessor<boolean>;
  setShowAdvanced: Setter<boolean>;
  // Raw-specific props (only used when physicalFormat === "raw")
  rawImageName: Accessor<string>;
  setRawImageName: Setter<string>;
  rawComputeMd5: Accessor<boolean>;
  setRawComputeMd5: Setter<boolean>;
  rawComputeSha1: Accessor<boolean>;
  setRawComputeSha1: Setter<boolean>;
  rawComputeSha256: Accessor<boolean>;
  setRawComputeSha256: Setter<boolean>;
  rawSegmentSize: Accessor<number>;
  setRawSegmentSize: Setter<number>;
  rawCaseNumber: Accessor<string>;
  setRawCaseNumber: Setter<string>;
  rawEvidenceNumber: Accessor<string>;
  setRawEvidenceNumber: Setter<string>;
  rawExaminerName: Accessor<string>;
  setRawExaminerName: Setter<string>;
  rawDescription: Accessor<string>;
  setRawDescription: Setter<string>;
  rawNotes: Accessor<string>;
  setRawNotes: Setter<string>;
}

// --- Component ---

export const PhysicalImageMode: Component<PhysicalImageModeProps> = (props) => {
  const [showCaseMetadata, setShowCaseMetadata] = createSignal(false);

  const selectedFormat = createMemo(() =>
    EWF_FORMATS.find(f => f.id === props.format()) ?? EWF_FORMATS[0]
  );

  const isRaw = () => props.physicalFormat() === "raw";

  return (
    <div class="space-y-3">
      {/* Imaging Format Selector */}
      <div class="space-y-1">
        <label class="label flex items-center gap-1">
          <HiOutlineCircleStack class="w-3.5 h-3.5" />
          Imaging Format
        </label>
        <div class="flex gap-2" role="radiogroup" aria-label="Image format">
          <button
            class={`flex-1 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
              !isRaw()
                ? "bg-accent text-white"
                : "bg-bg-secondary text-txt-muted hover:text-txt"
            }`}
            role="radio"
            aria-checked={!isRaw()}
            onClick={() => props.setPhysicalFormat("ewf")}
          >
            E01 (Expert Witness)
          </button>
          <button
            class={`flex-1 px-3 py-1.5 rounded-lg text-xs font-medium transition-colors ${
              isRaw()
                ? "bg-accent text-white"
                : "bg-bg-secondary text-txt-muted hover:text-txt"
            }`}
            role="radio"
            aria-checked={isRaw()}
            onClick={() => props.setPhysicalFormat("raw")}
          >
            Raw (.dd)
          </button>
        </div>
      </div>

      {/* ===== E01 (EWF) Options ===== */}
      <Show when={!isRaw()}>
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

        {/* Embedded Hashes (EWF) */}
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

        {/* Verify After Write (EWF) */}
        <div class="flex items-center gap-2 pl-1">
          <label class="flex items-center gap-2 text-xs cursor-pointer">
            <input
              type="checkbox"
              checked={props.ewfVerifyAfterWrite()}
              onChange={(e) => props.setEwfVerifyAfterWrite(e.currentTarget.checked)}
              class="accent-accent"
            />
            <HiOutlineCheckBadge class="w-3.5 h-3.5 text-success" />
            <span class="text-txt">Verify after write</span>
          </label>
          <span class="text-xs text-txt-muted">Re-reads image and compares hashes</span>
        </div>

        {/* Case Metadata (EWF) */}
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

        {/* Advanced (EWF) */}
        <div class="space-y-2">
          <button
            class="flex items-center gap-1 text-xs text-txt-secondary hover:text-txt"
            onClick={() => props.setShowAdvanced(!props.showAdvanced())}
            aria-expanded={props.showAdvanced()}
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

        {/* Info Card (EWF) */}
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
      </Show>

      {/* ===== Raw (.dd) Options ===== */}
      <Show when={isRaw()}>
        {/* Image Name */}
        <div class="space-y-1">
          <label class="label">Image Name</label>
          <div class="flex items-center gap-2">
            <input
              class="input-sm flex-1"
              type="text"
              value={props.rawImageName()}
              onInput={(e) => props.setRawImageName(e.currentTarget.value)}
              placeholder="evidence"
            />
            <span class="text-sm text-txt-muted font-mono">.dd</span>
          </div>
        </div>

        {/* Verification Hashes */}
        <div class="space-y-2">
          <label class="label flex items-center gap-1">
            <HiOutlineFingerPrint class="w-3.5 h-3.5" />
            Verification Hashes
          </label>
          <div class="flex gap-4 pl-1">
            <label class="flex items-center gap-2 text-xs cursor-pointer">
              <input
                type="checkbox"
                checked={props.rawComputeMd5()}
                onChange={(e) => props.setRawComputeMd5(e.currentTarget.checked)}
                class="accent-accent"
              />
              <span class="text-txt">MD5</span>
            </label>
            <label class="flex items-center gap-2 text-xs cursor-pointer">
              <input
                type="checkbox"
                checked={props.rawComputeSha1()}
                onChange={(e) => props.setRawComputeSha1(e.currentTarget.checked)}
                class="accent-accent"
              />
              <span class="text-txt">SHA-1</span>
            </label>
            <label class="flex items-center gap-2 text-xs cursor-pointer">
              <input
                type="checkbox"
                checked={props.rawComputeSha256()}
                onChange={(e) => props.setRawComputeSha256(e.currentTarget.checked)}
                class="accent-accent"
              />
              <span class="text-txt">SHA-256</span>
            </label>
          </div>
        </div>

        {/* Verify After Write (Raw) */}
        <div class="flex items-center gap-2 pl-1">
          <label class="flex items-center gap-2 text-xs cursor-pointer">
            <input
              type="checkbox"
              checked={props.rawVerifyAfterWrite()}
              onChange={(e) => props.setRawVerifyAfterWrite(e.currentTarget.checked)}
              class="accent-accent"
            />
            <HiOutlineCheckBadge class="w-3.5 h-3.5 text-success" />
            <span class="text-txt">Verify after write</span>
          </label>
          <span class="text-xs text-txt-muted">Re-reads image and compares hashes</span>
        </div>

        {/* Case Metadata (Raw) */}
        <CaseMetadataSection
          isOpen={showCaseMetadata}
          setIsOpen={setShowCaseMetadata}
          caseNumber={props.rawCaseNumber}
          setCaseNumber={props.setRawCaseNumber}
          evidenceNumber={props.rawEvidenceNumber}
          setEvidenceNumber={props.setRawEvidenceNumber}
          examinerName={props.rawExaminerName}
          setExaminerName={props.setRawExaminerName}
          description={props.rawDescription}
          setDescription={props.setRawDescription}
          notes={props.rawNotes}
          setNotes={props.setRawNotes}
        />

        {/* Advanced (Raw) */}
        <div class="space-y-2">
          <button
            class="flex items-center gap-1 text-xs text-txt-secondary hover:text-txt"
            onClick={() => props.setShowAdvanced(!props.showAdvanced())}
            aria-expanded={props.showAdvanced()}
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
                valueMb={props.rawSegmentSize}
                setValueMb={props.setRawSegmentSize}
                label="Segment Size"
              />
            </div>
          </Show>
        </div>

        {/* Info Card (Raw) */}
        <div class="info-card">
          <div class="info-card-title">
            <HiOutlineInformationCircle class="w-4 h-4 text-info" />
            About Raw Images
          </div>
          <p class="text-xs text-txt-muted">
            Creates a bit-for-bit copy without any container format. Raw (.dd) images are
            universally compatible and can be mounted or analyzed by any forensic tool.
            No compression — output is the same size as the source. Verification hashes
            are computed alongside the copy to confirm integrity.
          </p>
        </div>
      </Show>
    </div>
  );
};
