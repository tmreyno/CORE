// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component } from "solid-js";
import { Toggle, Slider } from "../ui";
import { SettingGroup, SettingRow, SettingsSelect } from "../settings";
import type { AppPreferences } from "../preferences";

interface PerformanceSettingsProps {
  preferences: AppPreferences;
  onUpdate: <K extends keyof AppPreferences>(key: K, value: AppPreferences[K]) => void;
}

export const PerformanceSettings: Component<PerformanceSettingsProps> = (props) => {
  const concurrentLabel = () =>
    props.preferences.maxConcurrentOperations === 0
      ? "Auto (all cores)"
      : String(props.preferences.maxConcurrentOperations);

  const workerLabel = () =>
    props.preferences.workerThreads === 0
      ? "Auto (all cores)"
      : String(props.preferences.workerThreads);

  return (
    <>
      <SettingGroup title="Loading" description="Control how data is loaded">
        <SettingRow label="Lazy Load Threshold" description="Items before lazy loading activates">
          <Slider
            value={props.preferences.lazyLoadThreshold}
            min={50}
            max={1000}
            step={50}
            onChange={(v) => props.onUpdate("lazyLoadThreshold", v)}
          />
        </SettingRow>

        <SettingRow label="Concurrent Operations" description={`Maximum parallel operations (${concurrentLabel()})`}>
          <Slider
            value={props.preferences.maxConcurrentOperations}
            min={0}
            max={32}
            onChange={(v) => props.onUpdate("maxConcurrentOperations", v)}
          />
        </SettingRow>

        <SettingRow label="Worker Threads" description={`Background worker threads (${workerLabel()})`}>
          <Slider
            value={props.preferences.workerThreads}
            min={0}
            max={32}
            onChange={(v) => props.onUpdate("workerThreads", v)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Memory" description="Memory and caching settings">
        <SettingRow label="Cache Size (MB)" description="Memory allocated for caching">
          <Slider
            value={props.preferences.cacheSizeMb}
            min={128}
            max={2048}
            step={128}
            onChange={(v) => props.onUpdate("cacheSizeMb", v)}
          />
        </SettingRow>

        <SettingRow label="Max Preview Size (MB)" description="Maximum file size for previews">
          <Slider
            value={props.preferences.maxPreviewSizeMb}
            min={5}
            max={200}
            step={5}
            onChange={(v) => props.onUpdate("maxPreviewSizeMb", v)}
          />
        </SettingRow>

        <SettingRow label="Chunk Size" description="Data chunk size for processing">
          <SettingsSelect
            value={String(props.preferences.chunkSizeKb)}
            options={[
              { value: "512", label: "512 KB" },
              { value: "1024", label: "1 MB" },
              { value: "2048", label: "2 MB" },
              { value: "4096", label: "4 MB" },
              { value: "8192", label: "8 MB" },
              { value: "16384", label: "16 MB" },
            ]}
            onChange={(v) => props.onUpdate("chunkSizeKb", Number(v))}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Advanced" description="Advanced performance options">
        <SettingRow label="Hardware Acceleration" description="Use GPU for rendering">
          <Toggle
            checked={props.preferences.useHardwareAcceleration}
            onChange={(v) => props.onUpdate("useHardwareAcceleration", v)}
          />
        </SettingRow>

        <SettingRow label="Enable Memory Mapping" description="Use mmap for large files">
          <Toggle
            checked={props.preferences.enableMmap}
            onChange={(v) => props.onUpdate("enableMmap", v)}
          />
        </SettingRow>
      </SettingGroup>
    </>
  );
};
