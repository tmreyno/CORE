// =============================================================================
// usePreferenceEffects - Apply preferences to DOM
// =============================================================================

import { createEffect, onMount, Accessor } from "solid-js";
import { makeEventListener } from "@solid-primitives/event-listener";
import type { AppPreferences } from "../components/preferences";
import { resolveTheme } from "./useTheme";

/**
 * Hook to apply preference settings to the DOM
 * Handles theme, accent color, font size, animations, density, and sidebar position
 */
export function usePreferenceEffects(preferences: Accessor<AppPreferences>): void {
  // Apply theme preference
  createEffect(() => {
    const prefs = preferences();
    const theme = prefs.theme;
    
    // Resolve theme (handles system detection and OS-specific light variants)
    const resolved = resolveTheme(theme);
    document.documentElement.setAttribute("data-theme", resolved);
  });
  
  // Apply accent color preference
  createEffect(() => {
    const prefs = preferences();
    const accentColor = prefs.accentColor;
    
    // Remove existing accent classes
    const accentClasses = ["accent-cyan", "accent-blue", "accent-green", "accent-purple", "accent-orange", "accent-red"];
    document.documentElement.classList.remove(...accentClasses);
    
    // Add the selected accent class
    document.documentElement.classList.add(`accent-${accentColor}`);
  });
  
  // Apply font size preference
  // The fontSize preference represents the "base" size (text-base).
  // All other tokens are offset from it to match the standardized scale:
  //   2xs = base-4, compact = base-3, xs = base-2, sm = base-1,
  //   base = base, lg = base+2, xl = base+4, 2xl = base+8
  // At default (14): 10, 11, 12, 13, 14, 16, 18, 22 — matches tailwind.config.js
  createEffect(() => {
    const prefs = preferences();
    const base = prefs.fontSize;
    
    document.documentElement.style.setProperty("--app-font-size-2xs", `${base - 4}px`);
    document.documentElement.style.setProperty("--app-font-size-compact", `${base - 3}px`);
    document.documentElement.style.setProperty("--app-font-size-xs", `${base - 2}px`);
    document.documentElement.style.setProperty("--app-font-size-sm", `${base - 1}px`);
    document.documentElement.style.setProperty("--app-font-size", `${base}px`);
    document.documentElement.style.setProperty("--app-font-size-lg", `${base + 2}px`);
    document.documentElement.style.setProperty("--app-font-size-xl", `${base + 4}px`);
    document.documentElement.style.setProperty("--app-font-size-2xl", `${base + 8}px`);
  });
  
  // Apply animations preference
  createEffect(() => {
    const prefs = preferences();
    document.documentElement.classList.toggle("reduce-motion", !prefs.animationsEnabled);
  });
  
  // Apply tree density preference
  createEffect(() => {
    const prefs = preferences();
    const density = prefs.treeDensity;
    
    // Remove existing density classes
    document.documentElement.classList.remove("density-compact", "density-comfortable", "density-spacious");
    document.documentElement.classList.add(`density-${density}`);
  });
  
  // Apply sidebar position preference
  createEffect(() => {
    const prefs = preferences();
    document.documentElement.classList.toggle("sidebar-right", prefs.sidebarPosition === "right");
  });
  
  // Apply icon set preference
  createEffect(() => {
    const prefs = preferences();
    const iconSet = prefs.iconSet;
    
    // Remove existing icon set classes
    document.documentElement.classList.remove("icons-outlined", "icons-solid", "icons-mini");
    document.documentElement.classList.add(`icons-${iconSet}`);
  });
  
  // Apply show status bar preference
  createEffect(() => {
    const prefs = preferences();
    document.documentElement.classList.toggle("hide-status-bar", !prefs.showStatusBar);
  });
  
  // Apply show line numbers preference (hide when false)
  createEffect(() => {
    const prefs = preferences();
    document.documentElement.classList.toggle("hide-line-numbers", !prefs.showLineNumbers);
  });
  
  // Apply show file sizes preference (hide when false)
  createEffect(() => {
    const prefs = preferences();
    document.documentElement.classList.toggle("hide-file-sizes", !prefs.showFileSizes);
  });
  
  // Listen for system theme changes (when theme is set to "system" or "light")
  onMount(() => {
    const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");
    const handleThemeChange = () => {
      const theme = preferences().theme;
      // Re-resolve theme when system preference changes
      if (theme === "system" || theme === "light") {
        const resolved = resolveTheme(theme);
        document.documentElement.setAttribute("data-theme", resolved);
      }
    };
    // makeEventListener auto-cleans up on component unmount
    makeEventListener(mediaQuery, "change", handleThemeChange);
  });
}
