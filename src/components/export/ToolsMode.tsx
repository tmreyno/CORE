// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";
import { open } from "@tauri-apps/plugin-dialog";
import { HiOutlineInformationCircle } from "../icons";

export interface ToolsModeProps {
  toolsTab: () => "test" | "repair" | "validate" | "extract";
  setToolsTab: (tab: "test" | "repair" | "validate" | "extract") => void;
  testArchivePath: () => string;
  setTestArchivePath: (path: string) => void;
  repairCorruptedPath: () => string;
  setRepairCorruptedPath: (path: string) => void;
  repairOutputPath: () => string;
  setRepairOutputPath: (path: string) => void;
  validateArchivePath: () => string;
  setValidateArchivePath: (path: string) => void;
  extractFirstVolume: () => string;
  setExtractFirstVolume: (path: string) => void;
  extractOutputDir: () => string;
  setExtractOutputDir: (path: string) => void;
}

export const ToolsMode: Component<ToolsModeProps> = (props) => {
  return (
    <div class="space-y-4">
      {/* Tools Tab Selector */}
      <div class="flex gap-1 border-b border-border">
        <button
          class={`px-4 py-2 -mb-px border-b-2 transition-colors text-xs ${
            props.toolsTab() === "test"
              ? "border-accent text-accent"
              : "border-transparent text-txt-secondary hover:text-txt"
          }`}
          onClick={() => props.setToolsTab("test")}
        >
          Test
        </button>
        <button
          class={`px-4 py-2 -mb-px border-b-2 transition-colors text-xs ${
            props.toolsTab() === "repair"
              ? "border-accent text-accent"
              : "border-transparent text-txt-secondary hover:text-txt"
          }`}
          onClick={() => props.setToolsTab("repair")}
        >
          Repair
        </button>
        <button
          class={`px-4 py-2 -mb-px border-b-2 transition-colors text-xs ${
            props.toolsTab() === "validate"
              ? "border-accent text-accent"
              : "border-transparent text-txt-secondary hover:text-txt"
          }`}
          onClick={() => props.setToolsTab("validate")}
        >
          Validate
        </button>
        <button
          class={`px-4 py-2 -mb-px border-b-2 transition-colors text-xs ${
            props.toolsTab() === "extract"
              ? "border-accent text-accent"
              : "border-transparent text-txt-secondary hover:text-txt"
          }`}
          onClick={() => props.setToolsTab("extract")}
        >
          Extract Split
        </button>
      </div>
      
      {/* Test Tab */}
      <Show when={props.toolsTab() === "test"}>
        <div class="space-y-3">
          <div class="info-card">
            <HiOutlineInformationCircle class="w-5 h-5 text-info" />
            <div>
              <div class="font-medium text-txt">Test Archive Integrity</div>
              <div class="text-xs text-txt-muted mt-1">
                Verify archive contents without extraction. Checks CRC and structure.
              </div>
            </div>
          </div>
          
          <div class="space-y-2">
            <label class="label">Archive File</label>
            <div class="flex gap-2">
              <input
                type="text"
                class="input-inline"
                value={props.testArchivePath()}
                onInput={(e) => props.setTestArchivePath(e.currentTarget.value)}
                placeholder="Select archive to test..."
              />
              <button class="btn-sm" onClick={async () => {
                const selected = await open({ directory: false, multiple: false, filters: [{ name: 'Archives', extensions: ['7z', 'zip'] }] });
                if (selected) props.setTestArchivePath(selected as string);
              }}>
                Browse
              </button>
            </div>
          </div>
        </div>
      </Show>
      
      {/* Repair Tab */}
      <Show when={props.toolsTab() === "repair"}>
        <div class="space-y-3">
          <div class="info-card">
            <HiOutlineInformationCircle class="w-5 h-5 text-info" />
            <div>
              <div class="font-medium text-txt">Repair Corrupted Archive</div>
              <div class="text-xs text-txt-muted mt-1">
                Attempt to recover data from damaged archives.
              </div>
            </div>
          </div>
          
          <div class="space-y-2">
            <label class="label">Corrupted Archive</label>
            <div class="flex gap-2">
              <input
                type="text"
                class="input-inline"
                value={props.repairCorruptedPath()}
                onInput={(e) => props.setRepairCorruptedPath(e.currentTarget.value)}
                placeholder="Select corrupted archive..."
              />
              <button class="btn-sm" onClick={async () => {
                const selected = await open({ directory: false, multiple: false, filters: [{ name: 'Archives', extensions: ['7z'] }] });
                if (selected) props.setRepairCorruptedPath(selected as string);
              }}>
                Browse
              </button>
            </div>
          </div>
          
          <div class="space-y-2">
            <label class="label">Output Archive</label>
            <div class="flex gap-2">
              <input
                type="text"
                class="input-inline"
                value={props.repairOutputPath()}
                onInput={(e) => props.setRepairOutputPath(e.currentTarget.value)}
                placeholder="Output path for repaired archive..."
              />
              <button class="btn-sm" onClick={async () => {
                const selected = await open({ directory: false, multiple: false, filters: [{ name: '7z Archive', extensions: ['7z'] }] });
                if (selected) props.setRepairOutputPath(selected as string);
              }}>
                Browse
              </button>
            </div>
          </div>
        </div>
      </Show>
      
      {/* Validate Tab */}
      <Show when={props.toolsTab() === "validate"}>
        <div class="space-y-3">
          <div class="info-card">
            <HiOutlineInformationCircle class="w-5 h-5 text-info" />
            <div>
              <div class="font-medium text-txt">Validate Archive Structure</div>
              <div class="text-xs text-txt-muted mt-1">
                Deep validation of archive format and headers.
              </div>
            </div>
          </div>
          
          <div class="space-y-2">
            <label class="label">Archive File</label>
            <div class="flex gap-2">
              <input
                type="text"
                class="input-inline"
                value={props.validateArchivePath()}
                onInput={(e) => props.setValidateArchivePath(e.currentTarget.value)}
                placeholder="Select archive to validate..."
              />
              <button class="btn-sm" onClick={async () => {
                const selected = await open({ directory: false, multiple: false, filters: [{ name: 'Archives', extensions: ['7z'] }] });
                if (selected) props.setValidateArchivePath(selected as string);
              }}>
                Browse
              </button>
            </div>
          </div>
        </div>
      </Show>
      
      {/* Extract Split Tab */}
      <Show when={props.toolsTab() === "extract"}>
        <div class="space-y-3">
          <div class="info-card">
            <HiOutlineInformationCircle class="w-5 h-5 text-info" />
            <div>
              <div class="font-medium text-txt">Extract Split Archive</div>
              <div class="text-xs text-txt-muted mt-1">
                Extract multi-volume archives (*.7z.001, *.7z.002, etc.)
              </div>
            </div>
          </div>
          
          <div class="space-y-2">
            <label class="label">First Volume</label>
            <div class="flex gap-2">
              <input
                type="text"
                class="input-inline"
                value={props.extractFirstVolume()}
                onInput={(e) => props.setExtractFirstVolume(e.currentTarget.value)}
                placeholder="Select first volume (.001)..."
              />
              <button class="btn-sm" onClick={async () => {
                const selected = await open({ directory: false, multiple: false });
                if (selected) props.setExtractFirstVolume(selected as string);
              }}>
                Browse
              </button>
            </div>
          </div>
          
          <div class="space-y-2">
            <label class="label">Output Directory</label>
            <div class="flex gap-2">
              <input
                type="text"
                class="input-inline"
                value={props.extractOutputDir()}
                onInput={(e) => props.setExtractOutputDir(e.currentTarget.value)}
                placeholder="Extract to..."
              />
              <button class="btn-sm" onClick={async () => {
                const selected = await open({ directory: true, multiple: false });
                if (selected) props.setExtractOutputDir(selected as string);
              }}>
                Browse
              </button>
            </div>
          </div>
        </div>
      </Show>
    </div>
  );
};
