// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/** Format keyboard shortcut for display (e.g., "cmd+k" → "⌘K") */
export function formatShortcut(shortcut: string): string {
  return shortcut
    .replace("cmd", "⌘")
    .replace("ctrl", "⌃")
    .replace("alt", "⌥")
    .replace("shift", "⇧")
    .replace(/\+/g, "")
    .toUpperCase();
}

/** Highlight fuzzy-matched characters in text */
export function highlightMatch(text: string, query: string): string {
  const q = query.toLowerCase();
  if (!q) return text;

  let result = "";
  let queryIdx = 0;

  for (const char of text) {
    if (queryIdx < q.length && char.toLowerCase() === q[queryIdx]) {
      result += `<mark class="bg-accent/30 text-accent-hover">${char}</mark>`;
      queryIdx++;
    } else {
      result += char;
    }
  }

  return result;
}
