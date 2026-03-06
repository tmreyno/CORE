// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Hex viewer constants — byte layout, load sizes, color mapping.
 */

import { getPreference } from "../preferences";

export const BYTES_PER_LINE = 16;
export const INITIAL_LOAD_SIZE = 65536; // 64KB initial load (4096 lines)
export const LOAD_MORE_SIZE = 32768; // 32KB per additional load
export const SCROLL_THRESHOLD = 200; // pixels from bottom to trigger load

/** Get max loaded bytes from preferences (convert MB to bytes) */
export const getMaxLoadedBytes = () => getPreference("maxPreviewSizeMb") * 1024 * 1024;

/** Map color classes to actual colors (with very light transparency) */
export const COLOR_MAP: Record<string, string> = {
  "region-signature": "rgba(239, 68, 68, 0.15)",
  "region-header": "rgba(249, 115, 22, 0.15)",
  "region-segment": "rgba(249, 115, 22, 0.15)",
  "region-metadata": "rgba(234, 179, 8, 0.15)",
  "region-data": "rgba(34, 197, 94, 0.15)",
  "region-checksum": "rgba(59, 130, 246, 0.15)",
  "region-reserved": "rgba(139, 92, 246, 0.15)",
  "region-footer": "rgba(236, 72, 153, 0.15)",
} as const;

export const NAVIGATED_COLOR = "rgba(34, 197, 94, 0.4)";

/** Memoized color lookup (used in hot path) */
export function getRegionColor(colorClass: string): string {
  return COLOR_MAP[colorClass as keyof typeof COLOR_MAP] || "#6a6a7a";
}
