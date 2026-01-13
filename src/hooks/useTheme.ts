// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { createSignal, createEffect, onMount } from "solid-js";

export type Theme = "light" | "dark" | "system";

const THEME_STORAGE_KEY = "ffx-theme";

/**
 * Hook for managing theme (light/dark mode)
 * 
 * Usage:
 * ```tsx
 * const { theme, setTheme, resolvedTheme, toggleTheme } = useTheme();
 * 
 * // Toggle between light and dark
 * <button onClick={toggleTheme}>🌙</button>
 * 
 * // Set specific theme
 * setTheme("light");
 * setTheme("dark");
 * setTheme("system"); // follow OS preference
 * ```
 */
export function useTheme() {
  // Get initial theme from localStorage or default to "system"
  const getInitialTheme = (): Theme => {
    if (typeof window === "undefined") return "system";
    const stored = localStorage.getItem(THEME_STORAGE_KEY);
    if (stored === "light" || stored === "dark" || stored === "system") {
      return stored;
    }
    return "system";
  };

  const [theme, setThemeSignal] = createSignal<Theme>(getInitialTheme());
  const [resolvedTheme, setResolvedTheme] = createSignal<"light" | "dark">("dark");

  // Get system preference
  const getSystemTheme = (): "light" | "dark" => {
    if (typeof window === "undefined") return "dark";
    return window.matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark";
  };

  // Apply theme to document
  const applyTheme = (t: Theme) => {
    const resolved = t === "system" ? getSystemTheme() : t;
    setResolvedTheme(resolved);

    // Apply to document
    if (resolved === "light") {
      document.documentElement.setAttribute("data-theme", "light");
    } else {
      document.documentElement.removeAttribute("data-theme");
    }
  };

  // Set theme and persist
  const setTheme = (newTheme: Theme) => {
    setThemeSignal(newTheme);
    localStorage.setItem(THEME_STORAGE_KEY, newTheme);
    applyTheme(newTheme);
  };

  // Toggle between light and dark (ignores system)
  const toggleTheme = () => {
    const current = resolvedTheme();
    setTheme(current === "light" ? "dark" : "light");
  };

  // Cycle through all themes: light -> dark -> system
  const cycleTheme = () => {
    const current = theme();
    if (current === "light") setTheme("dark");
    else if (current === "dark") setTheme("system");
    else setTheme("light");
  };

  // Listen for system preference changes
  onMount(() => {
    applyTheme(theme());

    const mediaQuery = window.matchMedia("(prefers-color-scheme: light)");
    const handleChange = () => {
      if (theme() === "system") {
        applyTheme("system");
      }
    };

    mediaQuery.addEventListener("change", handleChange);
    return () => mediaQuery.removeEventListener("change", handleChange);
  });

  // Re-apply when theme changes
  createEffect(() => {
    applyTheme(theme());
  });

  return {
    /** Current theme setting (light/dark/system) */
    theme,
    /** Set theme to light, dark, or system */
    setTheme,
    /** Resolved theme (what's actually displayed - light or dark) */
    resolvedTheme,
    /** Toggle between light and dark */
    toggleTheme,
    /** Cycle through light -> dark -> system */
    cycleTheme,
  };
}

/**
 * Get theme icon for display
 */
export function getThemeIcon(theme: Theme, resolved: "light" | "dark"): string {
  if (theme === "system") return "💻";
  return resolved === "light" ? "☀️" : "🌙";
}

/**
 * Get theme label for display
 */
export function getThemeLabel(theme: Theme): string {
  switch (theme) {
    case "light": return "Light";
    case "dark": return "Dark";
    case "system": return "System";
  }
}
