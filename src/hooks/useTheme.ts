// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Theme utilities and types
 * 
 * Theme is managed through the preferences system (createPreferences).
 * This file provides helper functions and types for theme-related operations.
 */

export type Theme = "light" | "light-macos" | "light-windows" | "dark" | "midnight" | "system";
export type ResolvedTheme = "light" | "light-macos" | "light-windows" | "dark" | "midnight";

/**
 * Detect operating system
 */
export function detectOS(): "macos" | "windows" | "linux" {
  if (typeof navigator === "undefined") return "windows";
  const platform = navigator.platform.toLowerCase();
  if (platform.includes("mac")) return "macos";
  if (platform.includes("win")) return "windows";
  return "linux";
}

/**
 * Get system theme preference
 */
export function getSystemTheme(): ResolvedTheme {
  if (typeof window === "undefined") return "dark";
  if (window.matchMedia("(prefers-color-scheme: light)").matches) {
    // Return OS-appropriate light theme
    const os = detectOS();
    return os === "macos" ? "light-macos" : os === "windows" ? "light-windows" : "light";
  }
  return "dark";
}

/**
 * Resolve theme to actual display theme
 */
export function resolveTheme(theme: Theme): ResolvedTheme {
  if (theme === "system") return getSystemTheme();
  if (theme === "light") {
    // "light" without OS suffix uses OS-appropriate variant
    const os = detectOS();
    return os === "macos" ? "light-macos" : os === "windows" ? "light-windows" : "light";
  }
  return theme;
}

/**
 * Apply theme to document
 */
export function applyTheme(theme: Theme): void {
  const resolved = resolveTheme(theme);
  document.documentElement.setAttribute("data-theme", resolved);
}

/**
 * Get next theme in cycle: dark -> light -> light-macos -> light-windows -> midnight -> system -> dark
 */
export function getNextTheme(current: Theme): Theme {
  switch (current) {
    case "dark": return "light";
    case "light": return "light-macos";
    case "light-macos": return "light-windows";
    case "light-windows": return "midnight";
    case "midnight": return "system";
    case "system": return "dark";
  }
}

/**
 * Get theme icon for display
 */
export function getThemeIcon(theme: Theme, resolved: ResolvedTheme): string {
  if (theme === "system") return "💻";
  if (theme === "midnight" || resolved === "midnight") return "✨";
  if (theme === "light-macos" || resolved === "light-macos") return "🍎";
  if (theme === "light-windows" || resolved === "light-windows") return "🪟";
  return resolved.startsWith("light") ? "☀️" : "🌙";
}

/**
 * Get theme label for display
 */
export function getThemeLabel(theme: Theme): string {
  switch (theme) {
    case "light": return "Light (Auto)";
    case "light-macos": return "Light (macOS)";
    case "light-windows": return "Light (Windows)";
    case "dark": return "Dark";
    case "midnight": return "Midnight";
    case "system": return "System";
  }
}

/**
 * Hook interface for components that need theme functionality
 * Used by ThemeSwitcher to interact with preferences
 */
export interface ThemeActions {
  theme: () => Theme;
  resolvedTheme: () => ResolvedTheme;
  setTheme: (theme: Theme) => void;
  cycleTheme: () => void;
  toggleTheme: () => void;
}

/**
 * Create theme actions from preferences
 * 
 * Usage:
 * ```tsx
 * const themeActions = createThemeActions(
 *   () => preferences.preferences().theme,
 *   (theme) => preferences.updatePreference("theme", theme)
 * );
 * ```
 */
export function createThemeActions(
  getTheme: () => Theme,
  setTheme: (theme: Theme) => void
): ThemeActions {
  return {
    theme: getTheme,
    resolvedTheme: () => resolveTheme(getTheme()),
    setTheme,
    cycleTheme: () => setTheme(getNextTheme(getTheme())),
    toggleTheme: () => {
      const resolved = resolveTheme(getTheme());
      setTheme(resolved === "light" ? "dark" : "light");
    },
  };
}
