// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * EwfExportMode — E01/Ex01/L01 forensic image creation UI
 *
 * Provides the UI for creating EWF forensic images via libewf-ffi.
 * Supports EnCase 5/6/7 and V2 (Ex01/Lx01) formats with deflate/bzip2
 * compression, full case metadata, and MD5/SHA1 hash embedding.
 */

import { Component, Show, createSignal } from "solid-js";
import {
  HiOutlineCircleStack,
  HiOutlineInformationCircle,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineCog6Tooth,
  HiOutlineShieldCheck,
  HiOutlineFingerPrint,
} from "../icons";

// ─── Format & Compression Types ─────────────────────────────────────────────

/** EWF output format identifiers (match backend parse_format()) */
export type EwfFormatId =
  | "e01"
  | "encase6"
  | "encase7"
  | "v2encase7"
  | "l01"
  | "l01v6"
  | "l01v7"
  | "lx01"
  | "ftk";

/** Format display info */
interface FormatInfo {
  id: EwfFormatId;
  name: string;
  extension: string;
  description: string;
  supportsV2: boolean;
}

/** Available EWF formats */
const EWF_FORMATS: FormatInfo[] = [
  {
    id: "e01",
    name: "EnCase 5",
    extension: ".E01",
    description: "Most compatible. Deflate compression, MD5 hash.",
    supportsV2: false,
  },
  {
    id: "encase6",
    name: "EnCase 6",
    extension: ".E01",
    description: "Supports larger images and SHA1 hash.",
    supportsV2: false,
  },
  {
    id: "encase7",
    name: "EnCase 7",
    extension: ".E01",
    description: "EWF1 segment type with SHA1 support.",
    supportsV2: false,
  },
  {
    id: "v2encase7",
    name: "EnCase 7 V2",
    extension: ".Ex01",
    description: "EWF2 format. Supports BZIP2 compression.",
    supportsV2: true,
  },
  {
    id: "l01",
    name: "Logical EnCase 5",
    extension: ".L01",
    description: "Logical evidence file. Most compatible.",
    supportsV2: false,
  },
  {
    id: "lx01",
    name: "Logical EnCase 7 V2",
    extension: ".Lx01",
    description: "EWF2 logical evidence. Supports BZIP2.",
    supportsV2: true,
  },
  {
    id: "ftk",
    name: "FTK Imager",
    extension: ".E01",
    description: "FTK Imager compatible format.",
    supportsV2: false,
  },
];

// ─── Props ──────────────────────────────────────────────────────────────────

export interface EwfExportModeProps {
  imageName: () => string;
  setImageName: (name: string) => void;
  format: () => string;
  setFormat: (format: string) => void;
  compression: () => string;
  setCompression: (compression: string) => void;
  compressionMethod: () => string;
  setCompressionMethod: (method: string) => void;
  computeMd5: () => boolean;
  setComputeMd5: (v: boolean) => void;
  computeSha1: () => boolean;
  setComputeSha1: (v: boolean) => void;
  showAdvanced: () => boolean;
  setShowAdvanced: (v: boolean) => void;
  segmentSize: () => number;
  setSegmentSize: (size: number) => void;
  // Case metadata
  caseNumber: () => string;
  setCaseNumber: (v: string) => void;
  evidenceNumber: () => string;
  setEvidenceNumber: (v: string) => void;
  examinerName: () => string;
  setExaminerName: (v: string) => void;
  description: () => string;
  setDescription: (v: string) => void;
  notes: () => string;
  setNotes: (v: string) => void;
}

// ─── Component ──────────────────────────────────────────────────────────────

export const EwfExportMode: Component<EwfExportModeProps> = (props) => {
  const [showCaseInfo, setShowCaseInfo] = createSignal(true);

  /** Whether the selected format supports BZIP2 */
  const isV2Format = () => {
    const fmt = EWF_FORMATS.find((f) => f.id === props.format());
    return fmt?.supportsV2 ?? false;
  };

  /** Derive file extension from selected format */
  const formatExtension = () => {
    const fmt = EWF_FORMATS.find((f) => f.id === props.format());
    return fmt?.extension ?? ".E01";
  };

  return (
    <div class="space-y-4 p-4 bg-bg-secondary rounded-lg border border-border">
      <h3 class="text-sm font-semibold text-txt flex items-center gap-2">
        <HiOutlineCircleStack class="w-4 h-4" />
        E01 Forensic Image Options
      </h3>

      {/* Image Name */}
      <div class="space-y-1">
        <label class="label text-xs">Image Name</label>
        <div class="flex items-center gap-2">
          <input
            class="input input-sm flex-1"
            type="text"
            value={props.imageName()}
            onInput={(e) => props.setImageName(e.currentTarget.value)}
            placeholder="evidence"
          />
          <span class="text-xs text-txt-muted font-mono">{formatExtension()}</span>
        </div>
      </div>

      {/* Output Format */}
      <div class="space-y-1">
        <label class="label text-xs">Format</label>
        <select
          class="input input-sm"
          value={props.format()}
          onChange={(e) => {
            const newFormat = e.currentTarget.value;
            props.setFormat(newFormat);
            // Reset compression method if switching from V2 to non-V2
            const fmt = EWF_FORMATS.find((f) => f.id === newFormat);
            if (!fmt?.supportsV2 && props.compressionMethod() === "bzip2") {
              props.setCompressionMethod("deflate");
            }
          }}
        >
          <optgroup label="Physical Image">
            <option value="e01">EnCase 5 (.E01) — Most compatible</option>
            <option value="encase6">EnCase 6 (.E01) — Larger images + SHA1</option>
            <option value="encase7">EnCase 7 (.E01) — EWF1</option>
            <option value="v2encase7">EnCase 7 V2 (.Ex01) — EWF2 + BZIP2</option>
            <option value="ftk">FTK Imager (.E01)</option>
          </optgroup>
          <optgroup label="Logical Image">
            <option value="l01">Logical EnCase 5 (.L01)</option>
            <option value="lx01">Logical EnCase 7 V2 (.Lx01)</option>
          </optgroup>
        </select>

        {/* Format description */}
        <div class="text-[10px] text-txt-muted mt-0.5">
          {EWF_FORMATS.find((f) => f.id === props.format())?.description}
        </div>
      </div>

      {/* Compression Level */}
      <div class="space-y-1">
        <label class="label text-xs">Compression</label>
        <select
          class="input input-sm"
          value={props.compression()}
          onChange={(e) => props.setCompression(e.currentTarget.value)}
        >
          <option value="none">None — Fastest, largest output</option>
          <option value="fast">Fast — Good speed, reasonable compression</option>
          <option value="best">Best — Smallest output, slowest</option>
        </select>
      </div>

      {/* Compression Method (visible when compression != "none") */}
      <Show when={props.compression() !== "none"}>
        <div class="space-y-1">
          <label class="label text-xs">Compression Method</label>
          <select
            class="input input-sm"
            value={props.compressionMethod()}
            onChange={(e) => props.setCompressionMethod(e.currentTarget.value)}
          >
            <option value="deflate">Deflate (zlib) — Standard, widely supported</option>
            <option value="bzip2" disabled={!isV2Format()}>
              BZIP2 — Better ratio {!isV2Format() ? "(requires V2 format)" : ""}
            </option>
          </select>
          <Show when={!isV2Format() && props.compressionMethod() === "bzip2"}>
            <div class="flex items-start gap-1 text-xs text-warning">
              <HiOutlineInformationCircle class="w-3 h-3 mt-0.5 flex-shrink-0" />
              <span>BZIP2 requires a V2 format (EnCase 7 V2 / .Ex01 or .Lx01)</span>
            </div>
          </Show>
        </div>
      </Show>

      {/* Hash Options */}
      <div class="space-y-2">
        <label class="label text-xs flex items-center gap-1">
          <HiOutlineFingerPrint class="w-3 h-3" />
          Hash Verification
        </label>
        <div class="space-y-1.5 ml-1">
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={props.computeMd5()}
              onChange={(e) => props.setComputeMd5(e.currentTarget.checked)}
              class="w-4 h-4"
            />
            <span class="text-xs text-txt">Compute & embed MD5 hash</span>
          </label>
          <label class="flex items-center gap-2 cursor-pointer">
            <input
              type="checkbox"
              checked={props.computeSha1()}
              onChange={(e) => props.setComputeSha1(e.currentTarget.checked)}
              class="w-4 h-4"
            />
            <span class="text-xs text-txt">Compute & embed SHA1 hash</span>
          </label>
        </div>
        <Show when={!props.computeMd5() && !props.computeSha1()}>
          <div class="flex items-start gap-1 text-xs text-warning">
            <HiOutlineInformationCircle class="w-3 h-3 mt-0.5 flex-shrink-0" />
            <span>No hash selected — image integrity cannot be verified later</span>
          </div>
        </Show>
      </div>

      {/* ── Case Metadata ─────────────────────────────────────────── */}
      <div class="border-t border-border pt-4 space-y-3">
        <button
          class="flex items-center gap-2 text-sm text-txt-secondary hover:text-txt transition-colors w-full"
          onClick={() => setShowCaseInfo(!showCaseInfo())}
        >
          <Show when={showCaseInfo()} fallback={<HiOutlineChevronRight class="w-4 h-4" />}>
            <HiOutlineChevronDown class="w-4 h-4" />
          </Show>
          <HiOutlineShieldCheck class="w-4 h-4" />
          <span>Case Metadata</span>
          <span class="text-[10px] text-txt-muted ml-auto">Embedded in E01 header</span>
        </button>

        <Show when={showCaseInfo()}>
          <div class="space-y-2 ml-1">
            <div class="space-y-1">
              <label class="label text-xs">Case Number</label>
              <input
                class="input input-sm"
                type="text"
                value={props.caseNumber()}
                onInput={(e) => props.setCaseNumber(e.currentTarget.value)}
                placeholder="e.g., 24-001234"
              />
            </div>
            <div class="space-y-1">
              <label class="label text-xs">Evidence Number</label>
              <input
                class="input input-sm"
                type="text"
                value={props.evidenceNumber()}
                onInput={(e) => props.setEvidenceNumber(e.currentTarget.value)}
                placeholder="e.g., EV-001"
              />
            </div>
            <div class="space-y-1">
              <label class="label text-xs">Examiner Name</label>
              <input
                class="input input-sm"
                type="text"
                value={props.examinerName()}
                onInput={(e) => props.setExaminerName(e.currentTarget.value)}
                placeholder="e.g., Det. Smith"
              />
            </div>
            <div class="space-y-1">
              <label class="label text-xs">Description</label>
              <input
                class="input input-sm"
                type="text"
                value={props.description()}
                onInput={(e) => props.setDescription(e.currentTarget.value)}
                placeholder="e.g., Suspect laptop hard drive image"
              />
            </div>
            <div class="space-y-1">
              <label class="label text-xs">Notes</label>
              <textarea
                class="textarea text-xs"
                rows="2"
                value={props.notes()}
                onInput={(e) => props.setNotes(e.currentTarget.value)}
                placeholder="Additional notes for chain-of-custody..."
              />
            </div>
          </div>
        </Show>
      </div>

      {/* ── Advanced Options ──────────────────────────────────────── */}
      <div class="border-t border-border pt-4">
        <button
          class="flex items-center gap-2 text-sm text-txt-secondary hover:text-txt transition-colors"
          onClick={() => props.setShowAdvanced(!props.showAdvanced())}
        >
          <Show when={props.showAdvanced()} fallback={<HiOutlineChevronRight class="w-4 h-4" />}>
            <HiOutlineChevronDown class="w-4 h-4" />
          </Show>
          <HiOutlineCog6Tooth class="w-4 h-4" />
          <span>Advanced Options</span>
        </button>

        <Show when={props.showAdvanced()}>
          <div class="mt-3 space-y-3">
            {/* Segment Size */}
            <div class="space-y-1">
              <label class="label text-xs">Segment File Size</label>
              <select
                class="input input-sm"
                value={props.segmentSize()}
                onChange={(e) => props.setSegmentSize(Number(e.currentTarget.value))}
              >
                <option value={0}>Default (~1.5 GB per segment)</option>
                <option value={681574400}>650 MB (CD)</option>
                <option value={734003200}>700 MB (CD)</option>
                <option value={2147483648}>2 GB</option>
                <option value={4700372992}>4.7 GB (DVD)</option>
              </select>
              <p class="text-[10px] text-txt-muted leading-tight">
                Large images are split into segment files (.E01, .E02, .E03, etc.)
              </p>
            </div>
          </div>
        </Show>
      </div>

      {/* Forensic info card */}
      <Show when={props.computeMd5() || props.computeSha1()}>
        <div class="p-2 bg-bg-panel rounded border border-success/20 text-xs text-success">
          <HiOutlineShieldCheck class="w-3 h-3 inline mr-1" />
          Hash{props.computeMd5() && props.computeSha1() ? "es" : ""} ({[
            props.computeMd5() && "MD5",
            props.computeSha1() && "SHA1",
          ].filter(Boolean).join(" + ")}) will be computed during write and embedded in the image for verification.
        </div>
      </Show>
    </div>
  );
};
