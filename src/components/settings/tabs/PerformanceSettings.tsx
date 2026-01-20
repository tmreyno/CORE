/**
 * PerformanceSettings Tab
 * Settings for threading, memory, and performance optimization
 */

import type { SettingsUpdateProps } from "../types";
import { SettingGroup } from "../SettingGroup";
import { SettingRow } from "../SettingRow";
import { SettingsSelect } from "../SettingsSelect";
import { Toggle, Slider } from "../../ui";

export function PerformanceSettings(props: SettingsUpdateProps) {
  return (
    <>
      <SettingGroup title="Threading" description="Parallel processing settings">
        <SettingRow label="Concurrent Operations" description="Maximum parallel operations">
          <Slider
            value={props.preferences.maxConcurrentOperations}
            min={1}
            max={16}
            onChange={(v) => props.onUpdate("maxConcurrentOperations", v)}
          />
        </SettingRow>

        <SettingRow label="Worker Threads" description="Background worker threads">
          <Slider
            value={props.preferences.workerThreads}
            min={1}
            max={16}
            onChange={(v) => props.onUpdate("workerThreads", v)}
          />
        </SettingRow>

        <SettingRow label="Lazy Load Threshold" description="Items before virtualizing">
          <Slider
            value={props.preferences.lazyLoadThreshold}
            min={50}
            max={500}
            step={50}
            onChange={(v) => props.onUpdate("lazyLoadThreshold", v)}
          />
        </SettingRow>
      </SettingGroup>

      <SettingGroup title="Memory" description="Memory and caching settings">
        <SettingRow label="Cache Size (MB)" description="Memory allocated for caching">
          <Slider
            value={props.preferences.cacheSizeMb}
            min={64}
            max={1024}
            step={64}
            onChange={(v) => props.onUpdate("cacheSizeMb", v)}
          />
        </SettingRow>

        <SettingRow label="Max Preview Size (MB)" description="Maximum file size for previews">
          <Slider
            value={props.preferences.maxPreviewSizeMb}
            min={1}
            max={100}
            onChange={(v) => props.onUpdate("maxPreviewSizeMb", v)}
          />
        </SettingRow>

        <SettingRow label="Chunk Size (KB)" description="Data chunk size for processing">
          <SettingsSelect
            value={String(props.preferences.chunkSizeKb)}
            options={[
              { value: "256", label: "256 KB" },
              { value: "512", label: "512 KB" },
              { value: "1024", label: "1 MB" },
              { value: "2048", label: "2 MB" },
              { value: "4096", label: "4 MB" },
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
}
