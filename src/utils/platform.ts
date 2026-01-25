// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Platform Detection Utility
 * 
 * Centralized platform detection for cross-platform functionality.
 * Use this instead of inline navigator.platform checks.
 * 
 * @example
 * ```tsx
 * import { platform, isMac } from '../utils/platform';
 * 
 * // Object access
 * const modifier = platform.isMac ? '⌘' : 'Ctrl';
 * 
 * // Direct import
 * if (isMac) {
 *   // macOS-specific code
 * }
 * ```
 */

// =============================================================================
// Platform Detection
// =============================================================================

/** Check if running in browser environment */
const isBrowser = typeof navigator !== 'undefined' && typeof navigator.platform === 'string';

/** User agent string (for additional detection) */
const userAgent = isBrowser ? navigator.userAgent.toLowerCase() : '';

/** Platform string */
const platformString = isBrowser ? navigator.platform : '';

// =============================================================================
// Platform Flags
// =============================================================================

/** Detect macOS (including iPad with desktop mode) */
export const isMac = isBrowser && /Mac|iPod|iPhone|iPad/.test(platformString);

/** Detect Windows */
export const isWindows = isBrowser && /Win/.test(platformString);

/** Detect Linux */
export const isLinux = isBrowser && /Linux/.test(platformString) && !userAgent.includes('android');

/** Detect iOS (iPhone, iPad, iPod) */
export const isIOS = isBrowser && /iPhone|iPad|iPod/.test(platformString);

/** Detect Android */
export const isAndroid = isBrowser && userAgent.includes('android');

/** Detect mobile device */
export const isMobile = isIOS || isAndroid;

/** Detect desktop (not mobile) */
export const isDesktop = !isMobile;

/** Detect Tauri environment */
export const isTauri = typeof window !== 'undefined' && '__TAURI__' in window;

// =============================================================================
// Platform Object (Alternative API)
// =============================================================================

/**
 * Platform detection object
 * Provides all platform flags in a single namespace
 */
export const platform = {
  /** macOS (including iPad with desktop mode) */
  isMac,
  /** Windows */
  isWindows,
  /** Linux (excluding Android) */
  isLinux,
  /** iOS (iPhone, iPad, iPod) */
  isIOS,
  /** Android */
  isAndroid,
  /** Mobile device (iOS or Android) */
  isMobile,
  /** Desktop (not mobile) */
  isDesktop,
  /** Running in Tauri */
  isTauri,
  /** Running in browser */
  isBrowser,
  
  /**
   * Get the platform name as a string
   */
  get name(): 'mac' | 'windows' | 'linux' | 'ios' | 'android' | 'unknown' {
    if (isMac && !isIOS) return 'mac';
    if (isWindows) return 'windows';
    if (isLinux) return 'linux';
    if (isIOS) return 'ios';
    if (isAndroid) return 'android';
    return 'unknown';
  },

  /**
   * Get the primary modifier key for the current platform
   * Returns '⌘' on Mac, 'Ctrl' on Windows/Linux
   */
  get modifierKey(): string {
    return isMac ? '⌘' : 'Ctrl';
  },

  /**
   * Get the modifier key symbol for display
   */
  get modifierSymbol(): string {
    return isMac ? '⌘' : 'Ctrl';
  },

  /**
   * Get the alt/option key name for the current platform
   */
  get altKey(): string {
    return isMac ? '⌥' : 'Alt';
  },

  /**
   * Get the shift key symbol for the current platform
   */
  get shiftKey(): string {
    return isMac ? '⇧' : 'Shift';
  },

  /**
   * Get the control key symbol for the current platform
   */
  get ctrlKey(): string {
    return isMac ? '⌃' : 'Ctrl';
  },
} as const;

// =============================================================================
// Keyboard Modifier Helpers
// =============================================================================

/**
 * Check if the primary modifier key is pressed in an event
 * Uses Cmd on Mac, Ctrl on Windows/Linux
 */
export function isPrimaryModifier(event: KeyboardEvent | MouseEvent): boolean {
  return isMac ? event.metaKey : event.ctrlKey;
}

/**
 * Get keyboard shortcut display string for current platform
 * @param modifiers Array of modifier names: 'mod' | 'shift' | 'alt' | 'ctrl'
 * @param key The main key
 */
export function formatShortcut(modifiers: string[], key: string): string {
  const parts = modifiers.map(mod => {
    switch (mod.toLowerCase()) {
      case 'mod':
      case 'cmd':
      case 'command':
        return platform.modifierSymbol;
      case 'shift':
        return platform.shiftKey;
      case 'alt':
      case 'option':
        return platform.altKey;
      case 'ctrl':
      case 'control':
        return platform.ctrlKey;
      default:
        return mod;
    }
  });
  
  parts.push(key.toUpperCase());
  return parts.join(isMac ? '' : '+');
}

export default platform;
