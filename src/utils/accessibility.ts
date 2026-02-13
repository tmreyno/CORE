// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Accessibility utilities for CORE-FFX.
 * Provides screen reader announcements and focus management.
 */

export type AriaLive = "polite" | "assertive" | "off";

// ============================================================================
// Screen Reader Announcements
// ============================================================================

let announcer: HTMLElement | null = null;

/**
 * Initialize the ARIA live announcer element.
 * Call once on app mount. Creates a visually-hidden live region
 * that screen readers monitor for dynamic content updates.
 */
export function initAnnouncer(): void {
  if (announcer && announcer.isConnected) return;

  announcer = document.createElement("div");
  announcer.setAttribute("role", "status");
  announcer.setAttribute("aria-live", "polite");
  announcer.setAttribute("aria-atomic", "true");
  announcer.style.cssText =
    "position:absolute;width:1px;height:1px;padding:0;margin:-1px;overflow:hidden;clip:rect(0,0,0,0);white-space:nowrap;border:0;";
  document.body.appendChild(announcer);
}

/**
 * Announce a message to screen readers via the ARIA live region.
 */
export function announce(
  message: string,
  priority: AriaLive = "polite"
): void {
  if (!announcer || !announcer.isConnected) {
    announcer = null;
    initAnnouncer();
  }
  if (!announcer) return;

  announcer.setAttribute("aria-live", priority);
  // Clear + re-set to force screen readers to re-read
  announcer.textContent = "";
  requestAnimationFrame(() => {
    if (announcer) announcer.textContent = message;
  });
}

// ============================================================================
// Focus Management
// ============================================================================

const FOCUSABLE_SELECTORS = [
  'a[href]',
  'button:not([disabled])',
  'input:not([disabled])',
  'select:not([disabled])',
  'textarea:not([disabled])',
  '[tabindex]:not([tabindex="-1"])',
  '[contenteditable="true"]',
].join(", ");

/**
 * Get all focusable elements within a container.
 */
export function getFocusableElements(container: HTMLElement): HTMLElement[] {
  return Array.from(container.querySelectorAll<HTMLElement>(FOCUSABLE_SELECTORS));
}
