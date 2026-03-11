// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { Component, Show } from "solid-js";

// =============================================================================
// CHECKBOX COMPONENT
// =============================================================================

interface CheckboxProps {
  /** Checked state */
  checked: boolean;
  /** Change handler */
  onChange: (checked: boolean) => void;
  /** Label text */
  label?: string;
  /** Disabled state */
  disabled?: boolean;
  /** Additional class names */
  class?: string;
}

/** Styled checkbox component */
export const Checkbox: Component<CheckboxProps> = (props) => {
  return (
    <label class={`flex items-center gap-2 cursor-pointer ${props.disabled ? "opacity-50 cursor-not-allowed" : ""} ${props.class || ""}`}>
      <div 
        class={`w-5 h-5 rounded-md border-2 flex items-center justify-center transition-colors ${
          props.checked 
            ? "bg-accent border-accent" 
            : "border-border/50 hover:border-accent/50"
        }`}
        onClick={() => !props.disabled && props.onChange(!props.checked)}
      >
        {props.checked && (
          <span class="text-white text-xs font-bold">✓</span>
        )}
      </div>
      {props.label && (
        <span class="text-sm">{props.label}</span>
      )}
    </label>
  );
};

// =============================================================================
// TOGGLE COMPONENT
// =============================================================================

interface ToggleProps {
  checked: boolean;
  onChange: (checked: boolean) => void;
  disabled?: boolean;
  class?: string;
}

/** Toggle switch component */
export const Toggle: Component<ToggleProps> = (props) => {
  return (
    <button
      role="switch"
      aria-checked={props.checked}
      disabled={props.disabled}
      class={`relative w-10 h-6 rounded-full transition-colors ${
        props.checked ? "bg-accent" : "bg-bg-active"
      } ${props.disabled ? "opacity-50 cursor-not-allowed" : ""} ${props.class || ""}`}
      onClick={() => !props.disabled && props.onChange(!props.checked)}
    >
      <span
        class={`absolute top-1 left-1 w-4 h-4 rounded-full bg-white transition-transform ${
          props.checked ? "translate-x-4" : ""
        }`}
      />
    </button>
  );
};

// =============================================================================
// SLIDER COMPONENT
// =============================================================================

interface SliderProps {
  value: number;
  min: number;
  max: number;
  step?: number;
  onChange: (value: number) => void;
  showValue?: boolean;
  suffix?: string;
  class?: string;
}

/** Range slider with optional value display */
export const Slider: Component<SliderProps> = (props) => {
  return (
    <div class={`flex items-center gap-3 ${props.class || ""}`}>
      <input
        type="range"
        class="w-24 accent-accent"
        min={props.min}
        max={props.max}
        step={props.step ?? 1}
        value={props.value}
        onInput={(e) => props.onChange(Number(e.currentTarget.value))}
      />
      <Show when={props.showValue !== false}>
        <span class="text-sm text-txt-secondary w-10 text-right">{props.value}{props.suffix ?? ""}</span>
      </Show>
    </div>
  );
};
