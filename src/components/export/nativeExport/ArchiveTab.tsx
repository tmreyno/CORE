// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show, For, createSignal } from "solid-js";
import {
  HiOutlineInformationCircle,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineCog6Tooth,
  HiOutlineShieldCheck,
  HiOutlineSparkles,
  HiOutlineLockClosed,
} from "../../icons";
import { formatBytes, getCompressionRatio } from "../../../api/archiveCreate";
import { SplitSizeSelector } from "../SplitSizeSelector";
import { FORENSIC_PRESETS, COMPRESSION_LEVELS } from "./constants";
import type { NativeExportModeProps, ForensicPreset, ForensicHashAlgorithm } from "./types";

/** 7z archive sub-tab: presets, compression, password, forensic options, advanced */
export const ArchiveTab: Component<
  Pick<
    NativeExportModeProps,
    | "archiveName"
    | "setArchiveName"
    | "compressionLevel"
    | "setCompressionLevel"
    | "estimatedUncompressed"
    | "estimatedCompressed"
    | "password"
    | "setPassword"
    | "showPassword"
    | "setShowPassword"
    | "showAdvanced"
    | "setShowAdvanced"
    | "solid"
    | "setSolid"
    | "numThreads"
    | "setNumThreads"
    | "splitSizeMb"
    | "setSplitSizeMb"
    | "generateManifest"
    | "setGenerateManifest"
    | "verifyAfterCreate"
    | "setVerifyAfterCreate"
    | "hashAlgorithm"
    | "setHashAlgorithm"
    | "includeExaminerInfo"
    | "setIncludeExaminerInfo"
    | "examinerName"
    | "setExaminerName"
    | "caseNumber"
    | "setCaseNumber"
    | "evidenceDescription"
    | "setEvidenceDescription"
  >
> = (props) => {
  const [activePreset, setActivePreset] = createSignal("forensic-standard");

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
    <>
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
          onInput={(e) => {
            // Sanitize: strip invalid filename characters
            const sanitized = e.currentTarget.value.replace(/[<>:"/\\|?*\x00-\x1f]/g, "").trim();
            props.setArchiveName(sanitized);
          }}
          onBlur={() => {
            if (!props.archiveName().trim()) {
              props.setArchiveName("evidence.7z");
            }
          }}
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
            {(level) => <option value={level.value}>{level.label}</option>}
          </For>
        </select>
      </div>

      {/* Size Estimate */}
      <Show when={props.estimatedUncompressed() > 0}>
        <div class="flex items-center justify-between text-xs text-txt-secondary bg-bg-secondary rounded-md px-3 py-2">
          <span>
            {formatBytes(props.estimatedUncompressed())} -&gt; ~
            {formatBytes(props.estimatedCompressed())}
          </span>
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
                onChange={(e) =>
                  props.setHashAlgorithm(e.currentTarget.value as ForensicHashAlgorithm)
                }
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
          <Show
            when={props.showAdvanced()}
            fallback={<HiOutlineChevronRight class="w-4 h-4" />}
          >
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

            <SplitSizeSelector valueMb={props.splitSizeMb} setValueMb={props.setSplitSizeMb} />
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
          Creates encrypted 7z archives using AES-256 encryption. Forensic presets configure
          hashing, manifests, and split volumes for evidence handling. Store mode preserves files
          without compression for fastest creation.
        </p>
      </div>
    </>
  );
};
