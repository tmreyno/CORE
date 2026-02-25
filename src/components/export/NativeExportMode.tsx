// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// NativeExportMode - Combined file export + 7z archive creation with sub-tabs

import { Component, Show, For, Accessor, Setter, createSignal } from "solid-js";
import {
  HiOutlineArchiveBox,
  HiOutlineArrowUpTray,
  HiOutlineInformationCircle,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineCog6Tooth,
  HiOutlineShieldCheck,
  HiOutlineSparkles,
  HiOutlineLockClosed,
} from "../icons";
import { CompressionLevel, formatBytes, getCompressionRatio } from "../../api/archiveCreate";
import { SplitSizeSelector } from "./SplitSizeSelector";

// --- Types ---

export type NativeExportTab = "files" | "archive";

export type ForensicHashAlgorithm = "SHA-256" | "SHA-1" | "MD5" | "SHA-256+MD5";

export interface ForensicPreset {
  id: string;
  name: string;
  description: string;
  compressionLevel: number;
  solid: boolean;
  splitSizeMb: number;
  generateManifest: boolean;
  verifyAfterCreate: boolean;
  hashAlgorithm: ForensicHashAlgorithm;
  includeExaminerInfo: boolean;
}

// --- Constants ---

const FORENSIC_PRESETS: ForensicPreset[] = [
  {
    id: "forensic-standard",
    name: "Standard",
    description: "SHA-256 hashing, manifest, and verification",
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
    name: "Court",
    description: "No compression, split for media, dual hashes",
    compressionLevel: CompressionLevel.Store,
    solid: false,
    splitSizeMb: 4096,
    generateManifest: true,
    verifyAfterCreate: true,
    hashAlgorithm: "SHA-256+MD5",
    includeExaminerInfo: true,
  },
  {
    id: "forensic-transfer",
    name: "Transfer",
    description: "No compression, split for USB/cloud",
    compressionLevel: CompressionLevel.Store,
    solid: false,
    splitSizeMb: 2048,
    generateManifest: true,
    verifyAfterCreate: true,
    hashAlgorithm: "SHA-256",
    includeExaminerInfo: false,
  },
  {
    id: "forensic-archive-long",
    name: "Long-term",
    description: "No compression, split for media, dual hashes",
    compressionLevel: CompressionLevel.Store,
    solid: false,
    splitSizeMb: 2048,
    generateManifest: true,
    verifyAfterCreate: true,
    hashAlgorithm: "SHA-256+MD5",
    includeExaminerInfo: true,
  },
  {
    id: "custom",
    name: "Custom",
    description: "Configure all settings manually",
    compressionLevel: CompressionLevel.Store,
    solid: false,
    splitSizeMb: 2048,
    generateManifest: true,
    verifyAfterCreate: false,
    hashAlgorithm: "SHA-256",
    includeExaminerInfo: false,
  },
];

const COMPRESSION_LEVELS: { value: number; label: string }[] = [
  { value: CompressionLevel.Store, label: "None (Store)" },
  { value: CompressionLevel.Fastest, label: "Fastest" },
  { value: CompressionLevel.Fast, label: "Fast" },
  { value: CompressionLevel.Normal, label: "Normal" },
  { value: CompressionLevel.Maximum, label: "Maximum" },
  { value: CompressionLevel.Ultra, label: "Ultra" },
];

// --- Props ---

export interface NativeExportModeProps {
  // Sub-tab
  activeTab: Accessor<NativeExportTab>;
  setActiveTab: Setter<NativeExportTab>;

  // === File Export props ===
  exportName: Accessor<string>;
  setExportName: Setter<string>;
  computeHashes: Accessor<boolean>;
  setComputeHashes: Setter<boolean>;
  verifyAfterCopy: Accessor<boolean>;
  setVerifyAfterCopy: Setter<boolean>;
  generateJsonManifest: Accessor<boolean>;
  setGenerateJsonManifest: Setter<boolean>;
  generateTxtReport: Accessor<boolean>;
  setGenerateTxtReport: Setter<boolean>;

  // === Archive (7z) props ===
  archiveName: Accessor<string>;
  setArchiveName: Setter<string>;
  compressionLevel: Accessor<number>;
  setCompressionLevel: Setter<number>;
  estimatedUncompressed: Accessor<number>;
  estimatedCompressed: Accessor<number>;
  password: Accessor<string>;
  setPassword: Setter<string>;
  showPassword: Accessor<boolean>;
  setShowPassword: Setter<boolean>;
  showAdvanced: Accessor<boolean>;
  setShowAdvanced: Setter<boolean>;
  solid: Accessor<boolean>;
  setSolid: Setter<boolean>;
  numThreads: Accessor<number>;
  setNumThreads: Setter<number>;
  splitSizeMb: Accessor<number>;
  setSplitSizeMb: Setter<number>;
  generateManifest: Accessor<boolean>;
  setGenerateManifest: Setter<boolean>;
  verifyAfterCreate: Accessor<boolean>;
  setVerifyAfterCreate: Setter<boolean>;
  hashAlgorithm: Accessor<ForensicHashAlgorithm>;
  setHashAlgorithm: Setter<ForensicHashAlgorithm>;
  includeExaminerInfo: Accessor<boolean>;
  setIncludeExaminerInfo: Setter<boolean>;
  examinerName: Accessor<string>;
  setExaminerName: Setter<string>;
  caseNumber: Accessor<string>;
  setCaseNumber: Setter<string>;
  evidenceDescription: Accessor<string>;
  setEvidenceDescription: Setter<string>;
}

// --- Component ---

export const NativeExportMode: Component<NativeExportModeProps> = (props) => {
  const [activePreset, setActivePreset] = createSignal("forensic-standard");

  const isFiles = () => props.activeTab() === "files";
  const isArchive = () => props.activeTab() === "archive";

  const applyPreset = (preset: ForensicPreset) => {
    setActivePreset(preset.id);
    if (preset.id === "custom") return;
    props.setCompressionLevel(preset.compressionLevel);
    props.setSolid(preset.solid);
    props.setSplitSizeMb(preset.splitSizeMb);
    props.setGenerateManifest(preset.generateManifest);
    props.setVerifyAfterCreate(preset.verifyAfterCreate);
    props.setHashAlgorithm(preset.hashAlgorithm);
    props.setIncludeExaminerInfo(preset.includeExaminerInfo);
  };

  return (
    <div class="space-y-4">
      {/* Sub-tab Toggle */}
      <div class="grid grid-cols-2 gap-2">
        <button
          class={`flex items-center justify-center gap-2 p-2.5 rounded-lg border transition-colors ${
            isFiles()
              ? "border-accent bg-accent/10 text-accent"
              : "border-border bg-bg-secondary text-txt-secondary hover:border-border-hover"
          }`}
          onClick={() => props.setActiveTab("files")}
        >
          <HiOutlineArrowUpTray class="w-4 h-4" />
          <span class="text-sm font-medium">File Export</span>
        </button>
        <button
          class={`flex items-center justify-center gap-2 p-2.5 rounded-lg border transition-colors ${
            isArchive()
              ? "border-accent bg-accent/10 text-accent"
              : "border-border bg-bg-secondary text-txt-secondary hover:border-border-hover"
          }`}
          onClick={() => props.setActiveTab("archive")}
        >
          <HiOutlineArchiveBox class="w-4 h-4" />
          <span class="text-sm font-medium">7z Archive</span>
        </button>
      </div>

      {/* ===== FILE EXPORT ===== */}
      <Show when={isFiles()}>
        <div class="space-y-4 p-4 bg-bg-secondary rounded-lg border border-border">
          <h3 class="text-sm font-semibold text-txt flex items-center gap-2">
            <HiOutlineArrowUpTray class="w-4 h-4" />
            Export Options
          </h3>

          {/* Export Name */}
          <div class="space-y-1">
            <label class="label text-xs">Export Name</label>
            <input
              class="input input-sm"
              type="text"
              value={props.exportName()}
              onInput={(e) => props.setExportName(e.currentTarget.value)}
              placeholder="forensic_export"
            />
            <p class="text-[10px] text-txt-muted leading-tight">
              Used for manifest and report filenames
            </p>
          </div>

          {/* Checkbox Options */}
          <div class="space-y-2">
            <label class="flex items-center gap-2 cursor-pointer">
              <input
                type="checkbox"
                checked={props.computeHashes()}
                onChange={(e) => props.setComputeHashes(e.currentTarget.checked)}
                class="w-4 h-4"
              />
              <span class="text-xs text-txt">Compute SHA-256 hashes</span>
            </label>

            <Show when={props.computeHashes()}>
              <label class="flex items-center gap-2 cursor-pointer ml-6">
                <input
                  type="checkbox"
                  checked={props.verifyAfterCopy()}
                  onChange={(e) => props.setVerifyAfterCopy(e.currentTarget.checked)}
                  class="w-4 h-4"
                />
                <span class="text-xs text-txt">Verify copied files</span>
              </label>

              <label class="flex items-center gap-2 cursor-pointer ml-6">
                <input
                  type="checkbox"
                  checked={props.generateJsonManifest()}
                  onChange={(e) => props.setGenerateJsonManifest(e.currentTarget.checked)}
                  class="w-4 h-4"
                />
                <span class="text-xs text-txt">Generate JSON manifest</span>
              </label>

              <label class="flex items-center gap-2 cursor-pointer ml-6">
                <input
                  type="checkbox"
                  checked={props.generateTxtReport()}
                  onChange={(e) => props.setGenerateTxtReport(e.currentTarget.checked)}
                  class="w-4 h-4"
                />
                <span class="text-xs text-txt">Generate TXT report</span>
              </label>
            </Show>
          </div>

          <div class="p-2 bg-bg-panel rounded border border-info/20 text-xs text-info">
            <HiOutlineInformationCircle class="w-3 h-3 inline mr-1" />
            Forensic export includes timestamps, hashes, and manifests for chain-of-custody
          </div>
        </div>
      </Show>

      {/* ===== 7z ARCHIVE ===== */}
      <Show when={isArchive()}>
        {/* Preset Selector */}
        <div class="space-y-2">
          <label class="label flex items-center gap-1">
            <HiOutlineSparkles class="w-4 h-4 text-accent" />
            Forensic Preset
          </label>
          <div class="grid grid-cols-5 gap-1.5">
            <For each={FORENSIC_PRESETS}>
              {(preset) => (
                <button
                  class={`px-2 py-1.5 text-xs rounded-md border transition-colors ${
                    activePreset() === preset.id
                      ? "border-accent bg-accent/10 text-accent font-medium"
                      : "border-border bg-bg-secondary text-txt-secondary hover:border-border-hover"
                  }`}
                  onClick={() => applyPreset(preset)}
                  title={preset.description}
                >
                  {preset.name}
                </button>
              )}
            </For>
          </div>
        </div>

        {/* Archive Name */}
        <div class="space-y-1">
          <label class="label">Archive Name</label>
          <input
            class="input"
            type="text"
            value={props.archiveName()}
            onInput={(e) => props.setArchiveName(e.currentTarget.value)}
            placeholder="evidence.7z"
          />
        </div>

        {/* Compression Level */}
        <div class="space-y-1">
          <label class="label">Compression</label>
          <select
            class="input"
            value={props.compressionLevel()}
            onChange={(e) => props.setCompressionLevel(Number(e.currentTarget.value))}
          >
            <For each={COMPRESSION_LEVELS}>
              {(level) => (
                <option value={level.value}>{level.label}</option>
              )}
            </For>
          </select>
        </div>

        {/* Size Estimate */}
        <Show when={props.estimatedUncompressed() > 0}>
          <div class="flex items-center justify-between text-xs text-txt-secondary bg-bg-secondary rounded-md px-3 py-2">
            <span>{formatBytes(props.estimatedUncompressed())} -&gt; ~{formatBytes(props.estimatedCompressed())}</span>
            <span class="text-accent font-medium">
              {getCompressionRatio(props.estimatedUncompressed(), props.estimatedCompressed())}
            </span>
          </div>
        </Show>

        {/* Password */}
        <div class="space-y-1">
          <label class="label flex items-center gap-1">
            <HiOutlineLockClosed class="w-3.5 h-3.5" />
            Password
          </label>
          <div class="flex gap-2">
            <input
              class="input flex-1"
              type={props.showPassword() ? "text" : "password"}
              value={props.password()}
              onInput={(e) => props.setPassword(e.currentTarget.value)}
              placeholder="Optional encryption password"
            />
            <button
              class="btn-sm"
              onClick={() => props.setShowPassword(!props.showPassword())}
            >
              {props.showPassword() ? "Hide" : "Show"}
            </button>
          </div>
        </div>

        {/* Forensic Options */}
        <div class="space-y-3">
          <label class="label flex items-center gap-1">
            <HiOutlineShieldCheck class="w-4 h-4 text-success" />
            Forensic Options
          </label>

          <div class="space-y-2 pl-1">
            <label class="flex items-center gap-2 text-sm cursor-pointer">
              <input
                type="checkbox"
                checked={props.generateManifest()}
                onChange={(e) => props.setGenerateManifest(e.currentTarget.checked)}
                class="accent-accent"
              />
              <span class="text-txt">Generate forensic manifest</span>
            </label>

            <Show when={props.generateManifest()}>
              <div class="pl-6 space-y-2">
                <label class="label text-xs">Hash Algorithm</label>
                <select
                  class="input-sm"
                  value={props.hashAlgorithm()}
                  onChange={(e) => props.setHashAlgorithm(e.currentTarget.value as ForensicHashAlgorithm)}
                >
                  <option value="SHA-256">SHA-256</option>
                  <option value="SHA-1">SHA-1</option>
                  <option value="MD5">MD5</option>
                  <option value="SHA-256+MD5">SHA-256 + MD5</option>
                </select>
              </div>
            </Show>

            <label class="flex items-center gap-2 text-sm cursor-pointer">
              <input
                type="checkbox"
                checked={props.verifyAfterCreate()}
                onChange={(e) => props.setVerifyAfterCreate(e.currentTarget.checked)}
                class="accent-accent"
              />
              <span class="text-txt">Verify archive after creation</span>
            </label>

            <label class="flex items-center gap-2 text-sm cursor-pointer">
              <input
                type="checkbox"
                checked={props.includeExaminerInfo()}
                onChange={(e) => props.setIncludeExaminerInfo(e.currentTarget.checked)}
                class="accent-accent"
              />
              <span class="text-txt">Include examiner info</span>
            </label>

            <Show when={props.includeExaminerInfo()}>
              <div class="pl-6 space-y-2">
                <input
                  class="input-sm"
                  type="text"
                  value={props.examinerName()}
                  onInput={(e) => props.setExaminerName(e.currentTarget.value)}
                  placeholder="Examiner name"
                />
                <input
                  class="input-sm"
                  type="text"
                  value={props.caseNumber()}
                  onInput={(e) => props.setCaseNumber(e.currentTarget.value)}
                  placeholder="Case number"
                />
                <input
                  class="input-sm"
                  type="text"
                  value={props.evidenceDescription()}
                  onInput={(e) => props.setEvidenceDescription(e.currentTarget.value)}
                  placeholder="Evidence description"
                />
              </div>
            </Show>
          </div>
        </div>

        {/* Advanced Options */}
        <div class="space-y-2">
          <button
            class="flex items-center gap-1 text-sm text-txt-secondary hover:text-txt"
            onClick={() => props.setShowAdvanced(!props.showAdvanced())}
          >
            <Show when={props.showAdvanced()} fallback={<HiOutlineChevronRight class="w-4 h-4" />}>
              <HiOutlineChevronDown class="w-4 h-4" />
            </Show>
            <HiOutlineCog6Tooth class="w-4 h-4" />
            Advanced
          </button>

          <Show when={props.showAdvanced()}>
            <div class="space-y-3 pl-5 pt-1">
              <label class="flex items-center gap-2 text-sm cursor-pointer">
                <input
                  type="checkbox"
                  checked={props.solid()}
                  onChange={(e) => props.setSolid(e.currentTarget.checked)}
                  class="accent-accent"
                />
                <span class="text-txt">Solid compression</span>
                <span class="text-xs text-txt-muted">(better ratio, slower)</span>
              </label>

              <div class="space-y-1">
                <label class="label text-xs">Threads</label>
                <input
                  class="input-sm w-20"
                  type="number"
                  min={0}
                  max={32}
                  value={props.numThreads()}
                  onInput={(e) => props.setNumThreads(Number(e.currentTarget.value))}
                />
                <span class="text-xs text-txt-muted ml-2">0 = auto</span>
              </div>

              <SplitSizeSelector
                valueMb={props.splitSizeMb}
                setValueMb={props.setSplitSizeMb}
              />
            </div>
          </Show>
        </div>

        {/* Info Card */}
        <div class="info-card">
          <div class="info-card-title">
            <HiOutlineInformationCircle class="w-4 h-4 text-info" />
            About 7z Archives
          </div>
          <p class="text-xs text-txt-muted">
            Creates encrypted 7z archives using AES-256 encryption. Forensic presets
            configure hashing, manifests, and split volumes for evidence handling.
            Store mode preserves files without compression for fastest creation.
          </p>
        </div>
      </Show>
    </div>
  );
};
