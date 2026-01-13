// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { For, type JSX } from "solid-js";
import { useTheme, getThemeLabel, type Theme } from "../hooks/useTheme";
import { HiOutlineSun, HiOutlineMoon, HiOutlineComputerDesktop } from "./icons";

interface ThemeSwitcherProps {
  /** Compact mode - just icon button */
  compact?: boolean;
}

/**
 * Get theme icon as JSX element
 */
function ThemeIconElement(props: { theme: Theme; resolved: "light" | "dark"; class?: string }): JSX.Element {
  const iconClass = () => props.class || "w-5 h-5";
  
  if (props.theme === "system") {
    return <HiOutlineComputerDesktop class={iconClass()} />;
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
 * <ThemeSwitcher />
 * 
 * // Compact icon-only
 * <ThemeSwitcher compact />
 * ```
 */
export function ThemeSwitcher(props: ThemeSwitcherProps) {
  const { theme, resolvedTheme, cycleTheme } = useTheme();
  
  const label = () => getThemeLabel(theme());
  
  // Compact mode - just a cycle button
  if (props.compact) {
    return (
      <button
        class="p-2 rounded bg-zinc-700 text-zinc-200 hover:bg-zinc-600 transition-colors focus:outline-none focus:ring-2 focus:ring-cyan-500"
        onClick={cycleTheme}
        title={`Theme: ${label()} (click to change)`}
        aria-label={`Current theme: ${label()}. Click to cycle themes.`}
      >
        <ThemeIconElement theme={theme()} resolved={resolvedTheme()} />
      </button>
    );
  }
  
  // Full dropdown
  return (
    <div class="relative">
      <button
        class="flex items-center gap-2 px-3 py-2 text-sm font-medium rounded bg-zinc-700 text-zinc-200 hover:bg-zinc-600 transition-colors focus:outline-none focus:ring-2 focus:ring-cyan-500"
        onClick={cycleTheme}
        title={`Theme: ${label()}`}
      >
        <ThemeIconElement theme={theme()} resolved={resolvedTheme()} />
        <span>{label()}</span>
      </button>
    </div>
  );
}

/**
 * Theme selector with all options visible
 */
export function ThemeSelector() {
  const { theme, setTheme, resolvedTheme } = useTheme();
  
  const themes: Theme[] = ["light", "dark", "system"];
  
  return (
    <div class="flex items-center gap-1 p-1 bg-zinc-800 rounded-lg">
      <For each={themes}>
        {(t) => (
          <button
            class={`px-3 py-1.5 text-sm rounded transition-colors focus:outline-none ${
              theme() === t 
                ? "bg-cyan-600 text-white" 
                : "text-zinc-400 hover:text-zinc-200 hover:bg-zinc-700"
            }`}
            onClick={() => setTheme(t)}
            title={getThemeLabel(t)}
          >
            <ThemeIconElement theme={t} resolved={resolvedTheme()} class="w-4 h-4" />
          </button>
        )}
      </For>
    </div>
  );
}
