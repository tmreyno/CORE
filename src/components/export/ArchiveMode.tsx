// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For, createSignal } from "solid-js";
import {
  HiOutlineArchiveBox,
  HiOutlineInformationCircle,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineCog6Tooth,
  HiOutlineShieldCheck,
  HiOutlineFingerPrint,
  HiOutlineDocumentCheck,
  HiOutlineLockClosed,
  HiOutlineSparkles,
} from "../icons";
import { CompressionLevel, formatBytes, getCompressionRatio } from "../../api/archiveCreate";

// ─── Forensic Preset Definitions ─────────────────────────────────────────────

/** Forensic archive preset configuration */
export interface ForensicPreset {
  /** Unique preset identifier */
  id: string;
  /** Display name */
  name: string;
  /** Short description */
  description: string;
  /** Compression level */
  compressionLevel: number;
  /** Enable solid compression */
  solid: boolean;
  /** Split size in MB (0 = no split) */
  splitSizeMb: number;
  /** Generate forensic manifest */
  generateManifest: boolean;
  /** Verify archive after creation */
  verifyAfterCreate: boolean;
  /** Hash algorithm for manifest */
  hashAlgorithm: ForensicHashAlgorithm;
  /** Include examiner metadata in manifest */
  includeExaminerInfo: boolean;
}

/** Hash algorithms available for forensic manifests */
export type ForensicHashAlgorithm = "SHA-256" | "SHA-1" | "MD5" | "SHA-256+MD5";

/** Built-in forensic presets */
export const FORENSIC_PRESETS: ForensicPreset[] = [
  {
    id: "forensic-standard",
    name: "Forensic Standard",
    description: "Store mode, SHA-256 manifest, post-creation verification. Best for E01/AD1 containers.",
    compressionLevel: CompressionLevel.Store,
    solid: false,
    splitSizeMb: 2048,
    generateManifest: true,
    verifyAfterCreate: true,
    hashAlgorithm: "SHA-256",
    includeExaminerInfo: true,
  },
  {
    id: "forensic-court",
    name: "Court Submission",
    description: "Store mode, dual-hash (SHA-256 + MD5), verified, examiner metadata. Maximum chain-of-custody.",
    compressionLevel: CompressionLevel.Store,
    solid: false,
    splitSizeMb: 4700, // DVD size for physical media
    generateManifest: true,
    verifyAfterCreate: true,
    hashAlgorithm: "SHA-256+MD5",
    includeExaminerInfo: true,
  },
  {
    id: "forensic-transfer",
    name: "Secure Transfer",
    description: "Fast compression, SHA-256, AES-256 encryption. For sending evidence over network.",
    compressionLevel: CompressionLevel.Fastest,
    solid: false,
    splitSizeMb: 2048, // Cloud-friendly 2GB chunks
    generateManifest: true,
    verifyAfterCreate: true,
    hashAlgorithm: "SHA-256",
    includeExaminerInfo: false,
  },
  {
    id: "forensic-archive-long",
    name: "Long-Term Archive",
    description: "Store mode, SHA-256 manifest, no split. Optimized for archival storage.",
    compressionLevel: CompressionLevel.Store,
    solid: false,
    splitSizeMb: 0, // No split for archival
    generateManifest: true,
    verifyAfterCreate: true,
    hashAlgorithm: "SHA-256",
    includeExaminerInfo: true,
  },
  {
    id: "custom",
    name: "Custom",
    description: "Manual configuration of all archive settings.",
    compressionLevel: CompressionLevel.Store,
    solid: false,
    splitSizeMb: 2048,
    generateManifest: false,
    verifyAfterCreate: false,
    hashAlgorithm: "SHA-256",
    includeExaminerInfo: false,
  },
];

export interface ArchiveModeProps {
  archiveName: () => string;
  setArchiveName: (name: string) => void;
  compressionLevel: () => number;
  setCompressionLevel: (level: number) => void;
  estimatedUncompressed: () => number;
  estimatedCompressed: () => number;
  password: () => string;
  setPassword: (password: string) => void;
  showPassword: () => boolean;
  setShowPassword: (show: boolean) => void;
  showAdvanced: () => boolean;
  setShowAdvanced: (show: boolean) => void;
  solid: () => boolean;
  setSolid: (solid: boolean) => void;
  numThreads: () => number;
  setNumThreads: (threads: number) => void;
  splitSizeMb: () => number;
  setSplitSizeMb: (size: number) => void;
  // Forensic options
  generateManifest: () => boolean;
  setGenerateManifest: (value: boolean) => void;
  verifyAfterCreate: () => boolean;
  setVerifyAfterCreate: (value: boolean) => void;
  hashAlgorithm: () => ForensicHashAlgorithm;
  setHashAlgorithm: (algo: ForensicHashAlgorithm) => void;
  includeExaminerInfo: () => boolean;
  setIncludeExaminerInfo: (value: boolean) => void;
  examinerName: () => string;
  setExaminerName: (name: string) => void;
  caseNumber: () => string;
  setCaseNumber: (num: string) => void;
  evidenceDescription: () => string;
  setEvidenceDescription: (desc: string) => void;
}

export const ArchiveMode: Component<ArchiveModeProps> = (props) => {
  const [selectedPreset, setSelectedPreset] = createSignal<string>("forensic-standard");

  /** Apply a preset's settings to all the parent state */
  const applyPreset = (presetId: string) => {
    setSelectedPreset(presetId);
    const preset = FORENSIC_PRESETS.find((p) => p.id === presetId);
    if (!preset || preset.id === "custom") return;

    props.setCompressionLevel(preset.compressionLevel);
    props.setSolid(preset.solid);
    props.setSplitSizeMb(preset.splitSizeMb);
    props.setGenerateManifest(preset.generateManifest);
    props.setVerifyAfterCreate(preset.verifyAfterCreate);
    props.setHashAlgorithm(preset.hashAlgorithm);
    props.setIncludeExaminerInfo(preset.includeExaminerInfo);
  };

  /** Preset icon mapping */
  const presetIcon = (id: string) => {
    switch (id) {
      case "forensic-standard":
        return <HiOutlineShieldCheck class="w-4 h-4 text-success" />;
      case "forensic-court":
        return <HiOutlineFingerPrint class="w-4 h-4 text-accent" />;
      case "forensic-transfer":
        return <HiOutlineLockClosed class="w-4 h-4 text-info" />;
      case "forensic-archive-long":
        return <HiOutlineDocumentCheck class="w-4 h-4 text-warning" />;
      default:
        return <HiOutlineCog6Tooth class="w-4 h-4 text-txt-muted" />;
    }
  };

  return (
    <div class="space-y-4 p-4 bg-bg-secondary rounded-lg border border-border">
      <h3 class="text-sm font-semibold text-txt flex items-center gap-2">
        <HiOutlineArchiveBox class="w-4 h-4" />
        Archive Options
      </h3>

      {/* ── Forensic Preset Selector ──────────────────────────────── */}
      <div class="space-y-2">
        <label class="label text-xs flex items-center gap-1">
          <HiOutlineSparkles class="w-3 h-3" />
          Forensic Preset
        </label>
        <div class="grid grid-cols-1 gap-1.5">
          <For each={FORENSIC_PRESETS}>
            {(preset) => (
              <button
                class={`flex items-start gap-2 p-2 rounded-lg border text-left transition-colors ${
                  selectedPreset() === preset.id
                    ? "border-accent bg-accent/10 text-txt"
                    : "border-border bg-bg-panel text-txt-secondary hover:bg-bg-hover hover:border-border"
                }`}
                onClick={() => applyPreset(preset.id)}
              >
                <div class="mt-0.5 flex-shrink-0">{presetIcon(preset.id)}</div>
                <div class="min-w-0">
                  <div class="text-xs font-medium">{preset.name}</div>
                  <div class="text-[10px] text-txt-muted leading-tight mt-0.5">
                    {preset.description}
                  </div>
                </div>
              </button>
            )}
          </For>
        </div>
      </div>
      
      {/* Archive Name */}
      <div class="space-y-1">
        <label class="label text-xs">Archive Name</label>
        <input
          class="input input-sm"
          type="text"
          value={props.archiveName()}
          onInput={(e) => props.setArchiveName(e.currentTarget.value)}
          placeholder="evidence.7z"
        />
      </div>
      
      {/* Compression Level */}
      <div class="space-y-1">
        <label class="label text-xs">Compression Level</label>
        <select
          class="input input-sm"
          value={props.compressionLevel()}
          onChange={(e) => {
            props.setCompressionLevel(Number(e.currentTarget.value));
            setSelectedPreset("custom");
          }}
        >
          <option value={CompressionLevel.Store}>Store (~500+ MB/s) - Recommended for E01/AD1</option>
          <option value={CompressionLevel.Fastest}>Fastest (~180 MB/s)</option>
          <option value={CompressionLevel.Fast}>Fast (~80 MB/s)</option>
          <option value={CompressionLevel.Normal}>Normal (~22 MB/s)</option>
          <option value={CompressionLevel.Maximum}>Maximum (~12 MB/s)</option>
          <option value={CompressionLevel.Ultra}>Ultra (~9 MB/s)</option>
        </select>
      </div>
      
      {/* Size Estimate */}
      <Show when={props.estimatedUncompressed() > 0}>
        <div class="p-2 bg-bg-panel rounded border border-border">
          <div class="text-xs space-y-1">
            <div class="flex justify-between">
              <span class="text-txt-muted">Original:</span>
              <span class="text-txt font-medium">{formatBytes(props.estimatedUncompressed())}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-txt-muted">Estimated:</span>
              <span class="text-txt font-medium">{formatBytes(props.estimatedCompressed())}</span>
            </div>
            <div class="flex justify-between">
              <span class="text-txt-muted">Ratio:</span>
              <span class="text-accent font-medium">
                {getCompressionRatio(props.estimatedUncompressed(), props.estimatedCompressed()).toFixed(1)}%
              </span>
            </div>
          </div>
        </div>
      </Show>
      
      {/* Password */}
      <div class="space-y-1">
        <label class="label text-xs">Password (Optional)</label>
        <div class="flex gap-2">
          <input
            class="input input-sm flex-1"
            type={props.showPassword() ? "text" : "password"}
            value={props.password()}
            onInput={(e) => props.setPassword(e.currentTarget.value)}
            placeholder="AES-256 encryption password"
          />
          <button
            class="btn-sm"
            onClick={() => props.setShowPassword(!props.showPassword())}
          >
            {props.showPassword() ? "Hide" : "Show"}
          </button>
        </div>
        <Show when={props.password()}>
          <div class="flex items-start gap-1 text-xs text-warning">
            <HiOutlineInformationCircle class="w-3 h-3 mt-0.5 flex-shrink-0" />
            <span>Strong password recommended (12+ characters)</span>
          </div>
        </Show>
      </div>

      {/* ── Forensic Options ──────────────────────────────────────── */}
      <div class="border-t border-border pt-4 space-y-3">
        <h4 class="text-xs font-semibold text-txt flex items-center gap-1.5">
          <HiOutlineShieldCheck class="w-3.5 h-3.5" />
          Forensic Options
        </h4>

        {/* Generate Manifest */}
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={props.generateManifest()}
            onChange={(e) => {
              props.setGenerateManifest(e.currentTarget.checked);
              setSelectedPreset("custom");
            }}
            class="w-4 h-4"
          />
          <span class="text-xs text-txt">Generate forensic manifest (JSON)</span>
        </label>

        <Show when={props.generateManifest()}>
          {/* Hash Algorithm */}
          <div class="ml-6 space-y-2">
            <div class="space-y-1">
              <label class="label text-xs">Hash Algorithm</label>
              <select
                class="input input-sm"
                value={props.hashAlgorithm()}
                onChange={(e) => {
                  props.setHashAlgorithm(e.currentTarget.value as ForensicHashAlgorithm);
                  setSelectedPreset("custom");
                }}
              >
                <option value="SHA-256">SHA-256 (Recommended)</option>
                <option value="SHA-1">SHA-1 (Legacy compatibility)</option>
                <option value="MD5">MD5 (Legacy compatibility)</option>
                <option value="SHA-256+MD5">SHA-256 + MD5 (Dual hash - court submissions)</option>
              </select>
            </div>
          </div>
        </Show>

        {/* Verify After Create */}
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={props.verifyAfterCreate()}
            onChange={(e) => {
              props.setVerifyAfterCreate(e.currentTarget.checked);
              setSelectedPreset("custom");
            }}
            class="w-4 h-4"
          />
          <span class="text-xs text-txt">Verify archive after creation</span>
        </label>

        {/* Include Examiner Info */}
        <label class="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={props.includeExaminerInfo()}
            onChange={(e) => {
              props.setIncludeExaminerInfo(e.currentTarget.checked);
              setSelectedPreset("custom");
            }}
            class="w-4 h-4"
          />
          <span class="text-xs text-txt">Include examiner metadata</span>
        </label>

        <Show when={props.includeExaminerInfo()}>
          <div class="ml-6 space-y-2">
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
              <label class="label text-xs">Evidence Description</label>
              <input
                class="input input-sm"
                type="text"
                value={props.evidenceDescription()}
                onInput={(e) => props.setEvidenceDescription(e.currentTarget.value)}
                placeholder="e.g., Suspect laptop forensic image"
              />
            </div>
          </div>
        </Show>
      </div>
      
      {/* Advanced Options (Collapsible) */}
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
            {/* Solid Compression */}
            <div class="space-y-1">
              <label class="flex items-center gap-2 cursor-pointer">
                <input
                  type="checkbox"
                  checked={props.solid()}
                  onChange={(e) => {
                    props.setSolid(e.currentTarget.checked);
                    setSelectedPreset("custom");
                  }}
                  class="w-4 h-4"
                />
                <span class="text-xs text-txt">Solid compression</span>
              </label>
              <p class="text-[10px] text-txt-muted ml-6 leading-tight">
                Better compression ratio but slower processing and prevents extracting individual files. 
                Not recommended for large archives or archives that may need partial extraction.
              </p>
            </div>
            
            {/* Thread Count */}
            <div class="space-y-1">
              <label class="label text-xs">Threads (0 = auto)</label>
              <input
                class="input input-sm"
                type="number"
                min="0"
                max="16"
                value={props.numThreads()}
                onInput={(e) => props.setNumThreads(Number(e.currentTarget.value))}
              />
            </div>
            
            {/* Split Size */}
            <div class="space-y-1">
              <label class="label text-xs">Split Size</label>
              <select
                class="input input-sm"
                value={props.splitSizeMb()}
                onChange={(e) => {
                  props.setSplitSizeMb(Number(e.currentTarget.value));
                  setSelectedPreset("custom");
                }}
              >
                <option value={0}>No Split</option>
                <option value={700}>700 MB (CD)</option>
                <option value={2048}>2 GB (Cloud/USB) - Recommended</option>
                <option value={4700}>4.7 GB (DVD)</option>
                <option value={8500}>8.5 GB (DVD DL)</option>
                <option value={25000}>25 GB (Blu-ray)</option>
                <option value={50000}>50 GB (Blu-ray DL)</option>
              </select>
            </div>
          </div>
        </Show>
      </div>

      {/* Forensic info card */}
      <Show when={props.generateManifest()}>
        <div class="p-2 bg-bg-panel rounded border border-success/20 text-xs text-success">
          <HiOutlineShieldCheck class="w-3 h-3 inline mr-1" />
          A forensic manifest will be generated alongside the archive with file inventory, hashes, timestamps, and chain-of-custody metadata.
        </div>
      </Show>
    </div>
  );
};
