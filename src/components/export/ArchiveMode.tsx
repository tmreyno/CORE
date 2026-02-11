// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import {
  HiOutlineArchiveBox,
  HiOutlineInformationCircle,
  HiOutlineChevronDown,
  HiOutlineChevronRight,
  HiOutlineCog6Tooth,
} from "../icons";
import { CompressionLevel, formatBytes, getCompressionRatio } from "../../api/archiveCreate";

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
}

export const ArchiveMode: Component<ArchiveModeProps> = (props) => {
  return (
    <div class="space-y-4 p-4 bg-bg-secondary rounded-lg border border-border">
      <h3 class="text-sm font-semibold text-txt flex items-center gap-2">
        <HiOutlineArchiveBox class="w-4 h-4" />
        Archive Options
      </h3>
      
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
          onChange={(e) => props.setCompressionLevel(Number(e.currentTarget.value))}
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
                  onChange={(e) => props.setSolid(e.currentTarget.checked)}
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
                onChange={(e) => props.setSplitSizeMb(Number(e.currentTarget.value))}
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
    </div>
  );
};
