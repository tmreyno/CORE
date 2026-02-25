// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// SplitSizeSelector - Shared split/segment size picker for all export modes
// Provides standard preset sizes familiar from 7-Zip, FTK Imager, and EnCase.
// All values are stored in MB internally and converted to bytes by the consumer.

import { Component, Show, Accessor, Setter, createSignal, createMemo } from "solid-js";

// --- Types ---

export interface SplitSizeOption {
  label: string;
  valueMb: number;
  description: string;
}

// --- Constants ---

/** Standard split size presets — matches 7-Zip / FTK Imager / EnCase conventions */
export const SPLIT_SIZE_OPTIONS: SplitSizeOption[] = [
  { label: "No splitting", valueMb: 0, description: "Single output file" },
  { label: "100 MB", valueMb: 100, description: "Small transfers" },
  { label: "650 MB", valueMb: 650, description: "CD-ROM" },
  { label: "700 MB", valueMb: 700, description: "CD-R" },
  { label: "1 GB", valueMb: 1024, description: "General purpose" },
  { label: "2 GB", valueMb: 2048, description: "FAT32 / FTK default" },
  { label: "4 GB", valueMb: 4096, description: "DVD / FAT32 limit" },
  { label: "4.7 GB", valueMb: 4700, description: "DVD single-layer" },
  { label: "25 GB", valueMb: 25600, description: "Blu-ray" },
];

// --- Props ---

export interface SplitSizeSelectorProps {
  /** Current value in MB (0 = no split) */
  valueMb: Accessor<number>;
  /** Setter — receives value in MB */
  setValueMb: Setter<number>;
  /** Label text (default: "Split Size") */
  label?: string;
}

// --- Component ---

export const SplitSizeSelector: Component<SplitSizeSelectorProps> = (props) => {
  const [customMode, setCustomMode] = createSignal(false);

  const label = () => props.label ?? "Split Size";

  /** Whether the current value matches a preset */
  const isPreset = createMemo(() =>
    SPLIT_SIZE_OPTIONS.some((opt) => opt.valueMb === props.valueMb())
  );

  /** Currently selected preset value as string for the <select> */
  const selectValue = createMemo(() => {
    if (customMode()) return "custom";
    if (isPreset()) return String(props.valueMb());
    // Non-preset value → show custom
    return "custom";
  });

  /** Human-readable description of the current value */
  const currentDescription = createMemo(() => {
    const match = SPLIT_SIZE_OPTIONS.find((opt) => opt.valueMb === props.valueMb());
    if (match) return match.description;
    if (props.valueMb() > 0) {
      const gb = props.valueMb() / 1024;
      return gb >= 1 ? `${gb.toFixed(1)} GB` : `${props.valueMb()} MB`;
    }
    return "Single output file";
  });

  const handleSelectChange = (value: string) => {
    if (value === "custom") {
      setCustomMode(true);
    } else {
      setCustomMode(false);
      props.setValueMb(Number(value));
    }
  };

  return (
    <div class="space-y-1">
      <label class="label text-xs">{label()}</label>
      <div class="flex items-center gap-2">
        <select
          class="input-sm flex-1"
          value={selectValue()}
          onChange={(e) => handleSelectChange(e.currentTarget.value)}
        >
          {SPLIT_SIZE_OPTIONS.map((opt) => (
            <option value={String(opt.valueMb)}>
              {opt.label} — {opt.description}
            </option>
          ))}
          <option value="custom">Custom...</option>
        </select>
      </div>

      <Show when={customMode() || (!isPreset() && props.valueMb() > 0)}>
        <div class="flex items-center gap-2 pt-1">
          <input
            class="input-sm w-28"
            type="number"
            min={0}
            step={100}
            value={props.valueMb()}
            onInput={(e) => props.setValueMb(Number(e.currentTarget.value))}
          />
          <span class="text-xs text-txt-muted">MB</span>
          <span class="text-xs text-txt-muted">({currentDescription()})</span>
        </div>
      </Show>
    </div>
  );
};
