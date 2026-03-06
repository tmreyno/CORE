// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/** Format CPU usage — show cores used (e.g., "4.9/8") instead of raw percent */
export function formatCpuUsage(
  cpuPercent: number | undefined,
  cores: number | undefined,
): string {
  if (cpuPercent === undefined || cores === undefined) return "—";
  const coresUsed = cpuPercent / 100;
  if (coresUsed >= 1) {
    return `${coresUsed.toFixed(1)}/${cores}`;
  }
  return `${cpuPercent.toFixed(0)}%`;
}
