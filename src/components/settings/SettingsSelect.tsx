/**
 * SettingsSelect Component
 * Styled select dropdown for settings panels
 */

import { For } from "solid-js";
import type { JSX } from "solid-js";

interface SettingsSelectOption {
  value: string;
  label: string;
}

interface SettingsSelectProps {
  value: string;
  options: SettingsSelectOption[];
  onChange: (value: string) => void;
  class?: string;
}

/**
 * Styled select element for settings
 */
export function SettingsSelect(props: SettingsSelectProps) {
  const handleChange: JSX.EventHandler<HTMLSelectElement, Event> = (e) => {
    props.onChange(e.currentTarget.value);
  };

  return (
    <select
      value={props.value}
      onChange={handleChange}
      class={`px-3 py-1.5 text-sm bg-bg-secondary border border-border rounded focus:outline-none focus:ring-2 focus:ring-accent text-txt ${props.class || ""}`}
    >
      <For each={props.options}>
        {(option) => <option value={option.value}>{option.label}</option>}
      </For>
    </select>
  );
}
