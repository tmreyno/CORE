// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Report Constants - Static configuration options for report generation
 */

import type { Classification, Severity, EvidenceType } from "./types";

// =============================================================================
// CLASSIFICATION OPTIONS
// =============================================================================

export const CLASSIFICATIONS: { value: Classification; label: string; color: string }[] = [
  { value: "Public", label: "Public", color: "#22c55e" },
  { value: "Internal", label: "Internal", color: "#3b82f6" },
  { value: "Confidential", label: "Confidential", color: "#f97316" },
  { value: "Restricted", label: "Restricted", color: "#ef4444" },
  { value: "LawEnforcementSensitive", label: "Law Enforcement Sensitive", color: "#a855f7" },
];

// =============================================================================
// SEVERITY OPTIONS
// =============================================================================

export const SEVERITIES: { value: Severity; label: string; color: string }[] = [
  { value: "Critical", label: "Critical", color: "#dc2626" },
  { value: "High", label: "High", color: "#ea580c" },
  { value: "Medium", label: "Medium", color: "#ca8a04" },
  { value: "Low", label: "Low", color: "#16a34a" },
  { value: "Informational", label: "Info", color: "#6b7280" },
];

// =============================================================================
// EVIDENCE TYPE OPTIONS
// =============================================================================

/** Evidence type options for dropdowns - exported for extensions */
export const EVIDENCE_TYPES: { value: EvidenceType; label: string }[] = [
  { value: "HardDrive", label: "Hard Drive" },
  { value: "SSD", label: "SSD" },
  { value: "UsbDrive", label: "USB Drive" },
  { value: "ExternalDrive", label: "External Drive" },
  { value: "MemoryCard", label: "Memory Card" },
  { value: "MobilePhone", label: "Mobile Phone" },
  { value: "Tablet", label: "Tablet" },
  { value: "Computer", label: "Computer" },
  { value: "Laptop", label: "Laptop" },
  { value: "OpticalDisc", label: "Optical Disc" },
  { value: "NetworkCapture", label: "Network Capture" },
  { value: "CloudStorage", label: "Cloud Storage" },
  { value: "ForensicImage", label: "Forensic Image" },
  { value: "Other", label: "Other" },
];

// =============================================================================
// INVESTIGATION TYPES
// =============================================================================

export const INVESTIGATION_TYPES = [
  { value: "criminal", label: "Criminal Investigation" },
  { value: "civil", label: "Civil Litigation" },
  { value: "internal", label: "Internal Investigation" },
  { value: "compliance", label: "Compliance Audit" },
  { value: "incident_response", label: "Incident Response" },
  { value: "ediscovery", label: "eDiscovery" },
  { value: "research", label: "Research/Academic" },
  { value: "other", label: "Other" },
] as const;

// =============================================================================
// FINDING CATEGORIES
// =============================================================================

export const FINDING_CATEGORIES = [
  { value: "general", label: "General" },
  { value: "malware", label: "Malware/Malicious Software" },
  { value: "data_exfiltration", label: "Data Exfiltration" },
  { value: "unauthorized_access", label: "Unauthorized Access" },
  { value: "policy_violation", label: "Policy Violation" },
  { value: "evidence_destruction", label: "Evidence Destruction" },
  { value: "communication", label: "Communication/Messaging" },
  { value: "financial", label: "Financial Activity" },
  { value: "user_activity", label: "User Activity" },
  { value: "system_artifacts", label: "System Artifacts" },
  { value: "network", label: "Network Activity" },
  { value: "timeline", label: "Timeline Event" },
] as const;

// =============================================================================
// CHAIN OF CUSTODY ACTIONS
// =============================================================================

export const CUSTODY_ACTIONS = [
  { value: "received", label: "Received" },
  { value: "transferred", label: "Transferred" },
  { value: "analyzed", label: "Analyzed" },
  { value: "stored", label: "Stored" },
  { value: "released", label: "Released" },
  { value: "returned", label: "Returned" },
  { value: "other", label: "Other" },
] as const;
