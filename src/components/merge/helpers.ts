// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Helpers for the MergeProjectsWizard: formatting, display name mapping, etc.
 */

/** Map raw template_id values to user-friendly display names */
export const TEMPLATE_DISPLAY_NAMES: Record<string, string> = {
  evidence_collection: "Evidence Collection",
  iar: "Investigative Activity Report",
  user_activity: "User Activity Log",
  chain_of_custody: "Chain of Custody",
  incident_report: "Incident Report",
  search_warrant: "Search Warrant",
  consent_to_search: "Consent to Search",
};

export const friendlyTemplateName = (templateId: string): string =>
  TEMPLATE_DISPLAY_NAMES[templateId] ||
  templateId.replace(/_/g, " ").replace(/\b\w/g, (c) => c.toUpperCase());

/** Extract filename from a full path */
export const basename = (path: string): string => path.split("/").pop() || path;

/** Format an ISO date for display */
export const formatDate = (iso: string): string => {
  try {
    return new Date(iso).toLocaleDateString(undefined, {
      year: "numeric",
      month: "short",
      day: "numeric",
    });
  } catch {
    return iso.slice(0, 10);
  }
};

/** Format bytes to human-readable size */
export const fmtBytes = (bytes: number): string => {
  if (bytes === 0) return "0 B";
  const units = ["B", "KB", "MB", "GB", "TB"];
  const i = Math.min(Math.floor(Math.log(bytes) / Math.log(1024)), units.length - 1);
  return `${(bytes / Math.pow(1024, i)).toFixed(i > 0 ? 1 : 0)} ${units[i]}`;
};

/** Get CSS class for a role badge */
export const roleBadgeClass = (role: string): string => {
  if (role === "project owner") return "badge badge-success";
  if (role === "session user") return "badge";
  if (role.includes("COC")) return "badge badge-warning";
  return "badge";
};
