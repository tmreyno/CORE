// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { ActivityLogEntry } from "../../types/project";

/** Convert activity entries to CSV string */
export const activitiesToCsv = (entries: ActivityLogEntry[]): string => {
  const header = "Timestamp,Category,Action,Description,User,File Path";
  const rows = entries.map((e) => {
    const escapeCsv = (val: string | undefined) => {
      if (!val) return "";
      if (val.includes(",") || val.includes('"') || val.includes("\n")) {
        return `"${val.replace(/"/g, '""')}"`;
      }
      return val;
    };
    return [
      e.timestamp,
      e.category,
      e.action,
      escapeCsv(e.description),
      escapeCsv(e.user),
      escapeCsv(e.file_path),
    ].join(",");
  });
  return [header, ...rows].join("\n");
};

/** Convert activity entries to JSON string */
export const activitiesToJson = (entries: ActivityLogEntry[]): string => {
  return JSON.stringify(entries, null, 2);
};
