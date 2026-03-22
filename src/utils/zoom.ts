// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * App-wide zoom control using CSS zoom property.
 * Supports Cmd+/Cmd-/Cmd+0 via native menu and keyboard shortcuts.
 */

const ZOOM_STEP = 0.1;
const ZOOM_MIN = 0.5;
const ZOOM_MAX = 2.0;
const ZOOM_DEFAULT = 1.0;
const STORAGE_KEY = "ffx-zoom-level";

let currentZoom = ZOOM_DEFAULT;

function applyZoom(level: number): void {
  currentZoom = Math.round(Math.max(ZOOM_MIN, Math.min(ZOOM_MAX, level)) * 100) / 100;
  document.documentElement.style.zoom = String(currentZoom);
  try {
    localStorage.setItem(STORAGE_KEY, String(currentZoom));
  } catch {
    // localStorage may be unavailable
  }
}

export function zoomIn(): void {
  applyZoom(currentZoom + ZOOM_STEP);
}

export function zoomOut(): void {
  applyZoom(currentZoom - ZOOM_STEP);
}

export function zoomReset(): void {
  applyZoom(ZOOM_DEFAULT);
}

export function restoreZoom(): void {
  try {
    const stored = localStorage.getItem(STORAGE_KEY);
    if (stored) {
      const level = parseFloat(stored);
      if (!isNaN(level)) applyZoom(level);
    }
  } catch {
    // localStorage may be unavailable
  }
}
