// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, type JSX, type Accessor } from "solid-js";
import { getThemeLabel, type Theme, type ResolvedTheme } from "../hooks/useTheme";
import { HiOutlineSun, HiOutlineMoon, HiOutlineComputerDesktop, HiOutlineSparkles } from "./icons";

interface ThemeSwitcherProps {
  /** Compact mode - just icon button */
  compact?: boolean;
  /** Current theme accessor */
  theme: Accessor<Theme>;
  /** Resolved theme accessor */
  resolvedTheme: Accessor<ResolvedTheme>;
  /** Function to cycle to next theme */
  cycleTheme: () => void;
}

/**
 * Get theme icon as JSX element
 */
function ThemeIconElement(props: { theme: Theme; resolved: "light" | "dark" | "midnight"; class?: string }): JSX.Element {
  const iconClass = () => props.class || "w-5 h-5";
  
  if (props.theme === "system") {
    return <HiOutlineComputerDesktop class={iconClass()} />;
  }
  if (props.theme === "midnight" || props.resolved === "midnight") {
    return <HiOutlineSparkles class={iconClass()} />;
  }
  return props.resolved === "light" 
    ? <HiOutlineSun class={iconClass()} /> 
    : <HiOutlineMoon class={iconClass()} />;
}

/**
 * Theme switcher button/dropdown
 * 
 * Usage:
 * ```tsx
 * // In header
 * <ThemeSwitcher 
 *   theme={themeActions.theme} 
 *   resolvedTheme={themeActions.resolvedTheme}
 *   cycleTheme={themeActions.cycleTheme} 
 * />
 * 
 * // Compact icon-only
 * <ThemeSwitcher compact {...themeActions} />
 * ```
 */
export function ThemeSwitcher(props: ThemeSwitcherProps) {
  const label = () => getThemeLabel(props.theme());
  
  // Compact mode - just a cycle button
  if (props.compact) {
    return (
      <button
        class="p-2 rounded bg-bg-hover text-txt hover:bg-bg-active transition-colors focus:outline-none focus:ring-2 focus:ring-accent"
        onClick={props.cycleTheme}
        title={`Theme: ${label()} (click to change)`}
        aria-label={`Current theme: ${label()}. Click to cycle themes.`}
      >
        <ThemeIconElement theme={props.theme()} resolved={props.resolvedTheme()} />
      </button>
    );
  }
  
  // Full dropdown
  return (
    <div class="relative">
      <button
        class="flex items-center gap-2 px-3 py-2 text-sm font-medium rounded bg-bg-hover text-txt hover:bg-bg-active transition-colors focus:outline-none focus:ring-2 focus:ring-accent"
        onClick={props.cycleTheme}
        title={`Theme: ${label()}`}
      >
        <ThemeIconElement theme={props.theme()} resolved={props.resolvedTheme()} />
        <span>{label()}</span>
      </button>
    </div>
  );
}

interface ThemeSelectorProps {
  /** Current theme accessor */
  theme: Accessor<Theme>;
  /** Resolved theme accessor */
  resolvedTheme: Accessor<ResolvedTheme>;
  /** Function to set theme */
  setTheme: (theme: Theme) => void;
}

/**
 * Theme selector with all options visible
 */
export function ThemeSelector(props: ThemeSelectorProps) {
  const themes: Theme[] = ["light", "dark", "midnight", "system"];
  
  return (
    <div class="flex items-center gap-1 p-1 bg-bg-secondary rounded-lg">
      <For each={themes}>
        {(t) => (
          <button
            class={`px-3 py-1.5 text-sm rounded transition-colors focus:outline-none ${
              props.theme() === t 
                ? "bg-accent text-white" 
                : "text-txt-secondary hover:text-txt hover:bg-bg-hover"
            }`}
            onClick={() => props.setTheme(t)}
            title={getThemeLabel(t)}
          >
            <ThemeIconElement theme={t} resolved={props.resolvedTheme()} class="w-4 h-4" />
          </button>
        )}
      </For>
    </div>
  );
}
