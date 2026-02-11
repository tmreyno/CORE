// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Profile Helpers - Utility functions for workspace profiles
 */

import type { ProfileType } from "../../hooks/useWorkspaceProfiles";

/**
 * Get display label for profile type
 */
export const getProfileTypeLabel = (type: ProfileType): string => {
  const labels: Record<ProfileType, string> = {
    Investigation: "Investigation",
    Analysis: "Analysis",
    Review: "Review",
    Mobile: "Mobile",
    Computer: "Computer",
    Network: "Network",
    IncidentResponse: "Incident Response",
    Custom: "Custom",
  };
  return labels[type] || type;
};

/**
 * Get color class for profile type
 */
export const getProfileTypeColor = (type: ProfileType): string => {
  const colors: Record<ProfileType, string> = {
    Investigation: "text-type-ad1",
    Analysis: "text-type-e01",
    Review: "text-warning",
    Mobile: "text-type-ufed",
    Computer: "text-info",
    Network: "text-accent",
    IncidentResponse: "text-error",
    Custom: "text-txt-secondary",
  };
  return colors[type] || "text-txt-secondary";
};

/**
 * Profile type options for select dropdown
 */
export const PROFILE_TYPE_OPTIONS: Array<{ value: ProfileType; label: string }> = [
  { value: "Investigation", label: "Investigation" },
  { value: "Analysis", label: "Analysis" },
  { value: "Review", label: "Review" },
  { value: "Mobile", label: "Mobile Forensics" },
  { value: "Computer", label: "Computer Forensics" },
  { value: "Network", label: "Network Forensics" },
  { value: "IncidentResponse", label: "Incident Response" },
  { value: "Custom", label: "Custom" },
];
