// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Report Constants - Static configuration options for report generation
 */

import type { Classification, Severity, EvidenceType, ReportType, IAREventCategory } from "./types";

// =============================================================================
// REPORT PRESET TYPE
// =============================================================================

/** Preset configuration profiles that pre-fill report defaults */
export type ReportPreset = "law_enforcement" | "corporate" | "academic" | "minimal" | "custom";

/** Section visibility defaults for a preset */
export interface ReportPresetSections {
  executiveSummary: boolean;
  scope: boolean;
  methodology: boolean;
  chainOfCustody: boolean;
  timeline: boolean;
  conclusions: boolean;
  appendices: boolean;
}

export interface ReportPresetConfig {
  id: ReportPreset;
  name: string;
  icon: string;
  description: string;
  defaultClassification: Classification;
  defaultSections: ReportPresetSections;
  defaultMethodology: string;
  defaultScope: string;
}

// =============================================================================
// REPORT TYPE OPTIONS
// =============================================================================

export const REPORT_TYPES: { value: ReportType; label: string; icon: string; description: string }[] = [
  { value: "forensic_examination", label: "Forensic Examination Report", icon: "🔬", description: "Full forensic examination with evidence, findings, and conclusions" },
  { value: "chain_of_custody", label: "Chain of Custody Report", icon: "🔗", description: "Per-item COC Form 7 with transfer records and evidence tracking" },
  { value: "investigative_activity", label: "Investigative Activity Report (IAR)", icon: "📋", description: "Start-to-finish summary of investigation activities, personnel, and key events" },
  { value: "user_activity", label: "User Activity Report", icon: "👤", description: "Analysis of target user activity across devices and accounts" },
  { value: "timeline", label: "Timeline Report", icon: "📅", description: "Chronological reconstruction of events with key event highlights" },
];

// =============================================================================
// REPORT TYPE DEFAULTS - Context-aware defaults per report type
// =============================================================================

/**
 * Default metadata values per report type. Used when the report type changes
 * so the title, classification, and field relevance adjust automatically.
 */
export interface ReportTypeDefaults {
  /** Default report title */
  title: string;
  /** Default classification for this report type */
  classification: Classification;
  /** Whether the investigation_type field is relevant */
  showInvestigationType: boolean;
  /** Whether dates section is relevant */
  showDates: boolean;
  /** Whether case description is relevant */
  showDescription: boolean;
  /** Placeholder hint for the description field */
  descriptionPlaceholder: string;
}

export const REPORT_TYPE_DEFAULTS: Record<ReportType, ReportTypeDefaults> = {
  forensic_examination: {
    title: "Digital Forensic Examination Report",
    classification: "LawEnforcementSensitive",
    showInvestigationType: true,
    showDates: true,
    showDescription: true,
    descriptionPlaceholder: "Brief description of the case and examination request...",
  },
  chain_of_custody: {
    title: "Chain of Custody Report",
    classification: "LawEnforcementSensitive",
    showInvestigationType: false,
    showDates: true,
    showDescription: false,
    descriptionPlaceholder: "",
  },
  investigative_activity: {
    title: "Investigative Activity Report",
    classification: "LawEnforcementSensitive",
    showInvestigationType: true,
    showDates: true,
    showDescription: true,
    descriptionPlaceholder: "Summary of the investigation and activities documented...",
  },
  user_activity: {
    title: "User Activity Analysis Report",
    classification: "Confidential",
    showInvestigationType: false,
    showDates: true,
    showDescription: true,
    descriptionPlaceholder: "Description of target user and activity analysis scope...",
  },
  timeline: {
    title: "Timeline Analysis Report",
    classification: "Internal",
    showInvestigationType: false,
    showDates: true,
    showDescription: true,
    descriptionPlaceholder: "Description of timeline scope and key events to reconstruct...",
  },
};

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

// =============================================================================
// DEVICE TYPE OPTIONS (Evidence Collection)
// =============================================================================

export const DEVICE_TYPES = [
  { value: "desktop_computer", label: "Desktop Computer" },
  { value: "laptop", label: "Laptop" },
  { value: "server", label: "Server" },
  { value: "mobile_phone", label: "Mobile Phone" },
  { value: "tablet", label: "Tablet" },
  { value: "external_hdd", label: "External Hard Drive" },
  { value: "external_ssd", label: "External SSD" },
  { value: "usb_flash_drive", label: "USB Flash Drive" },
  { value: "memory_card", label: "Memory Card (SD/microSD)" },
  { value: "nvr_dvr", label: "NVR / DVR" },
  { value: "network_device", label: "Network Device (Router/Switch)" },
  { value: "gaming_console", label: "Gaming Console" },
  { value: "drone", label: "Drone / UAV" },
  { value: "camera", label: "Camera" },
  { value: "gps_device", label: "GPS Device" },
  { value: "wearable", label: "Wearable (Watch/Fitness)" },
  { value: "iot_device", label: "IoT Device" },
  { value: "virtual_machine", label: "Virtual Machine" },
  { value: "cloud_account", label: "Cloud Account / Storage" },
  { value: "optical_disc", label: "Optical Disc (CD/DVD/Blu-ray)" },
  { value: "tape_media", label: "Tape Media" },
  { value: "other", label: "Other" },
] as const;

// =============================================================================
// STORAGE INTERFACE TYPE OPTIONS (Evidence Collection)
// =============================================================================

export const STORAGE_INTERFACE_TYPES = [
  { value: "sata", label: "SATA" },
  { value: "ide_pata", label: "IDE / PATA" },
  { value: "nvme_m2", label: "NVMe M.2" },
  { value: "sata_m2", label: "SATA M.2" },
  { value: "usb_2", label: "USB 2.0" },
  { value: "usb_3", label: "USB 3.0 / 3.1 / 3.2" },
  { value: "usb_c", label: "USB-C / Thunderbolt" },
  { value: "firewire", label: "FireWire (IEEE 1394)" },
  { value: "scsi_sas", label: "SCSI / SAS" },
  { value: "raid", label: "RAID Array" },
  { value: "emmc", label: "eMMC" },
  { value: "sd_card", label: "SD / microSD" },
  { value: "esata", label: "eSATA" },
  { value: "pcie", label: "PCIe" },
  { value: "network_nas", label: "Network / NAS" },
  { value: "n_a", label: "N/A" },
  { value: "other", label: "Other" },
] as const;

// =============================================================================
// FORENSIC IMAGE FORMAT OPTIONS (Evidence Collection)
// =============================================================================

export const IMAGE_FORMAT_OPTIONS = [
  { value: "e01", label: "E01 (EnCase)" },
  { value: "ex01", label: "Ex01 (EnCase v7+)" },
  { value: "l01", label: "L01 (EnCase Logical)" },
  { value: "ad1", label: "AD1 (FTK/AccessData)" },
  { value: "dd_raw", label: "DD / Raw (.dd, .raw, .img, .bin)" },
  { value: "001", label: "001 (Split Raw)" },
  { value: "aff", label: "AFF / AFF4" },
  { value: "vhd", label: "VHD / VHDX" },
  { value: "vmdk", label: "VMDK (VMware)" },
  { value: "dmg", label: "DMG (Apple)" },
  { value: "tar", label: "TAR" },
  { value: "7z", label: "7z" },
  { value: "zip", label: "ZIP" },
  { value: "ufdr", label: "UFDR (Cellebrite)" },
  { value: "gfz", label: "GFZ (AXIOM)" },
  { value: "sparse", label: "Sparse Image" },
  { value: "mem_dump", label: "Memory Dump (.mem, .dmp)" },
  { value: "none", label: "No Image Created" },
  { value: "other", label: "Other" },
] as const;

// =============================================================================
// ACQUISITION METHOD OPTIONS (Evidence Collection)
// =============================================================================

export const ACQUISITION_METHODS = [
  { value: "physical", label: "Physical (Full Disk)" },
  { value: "logical_partition", label: "Logical Partition" },
  { value: "logical_file_folder", label: "Logical File/Folder" },
  { value: "file_system", label: "File System" },
  { value: "native_file", label: "Native File Copy" },
  { value: "chip_off", label: "Chip-Off" },
  { value: "jtag", label: "JTAG" },
  { value: "isp", label: "ISP (In-System Programming)" },
  { value: "cloud_extraction", label: "Cloud Extraction" },
  { value: "memory_capture", label: "Memory Capture (RAM)" },
  { value: "network_capture", label: "Network Capture" },
  { value: "targeted_collection", label: "Targeted Collection" },
  { value: "consent_download", label: "Consent Download" },
  { value: "screen_capture", label: "Screen Capture / Manual" },
  { value: "other", label: "Other" },
] as const;

// =============================================================================
// COC EVIDENCE CONDITIONS
// =============================================================================

export const EVIDENCE_CONDITIONS = [
  { value: "sealed", label: "Sealed" },
  { value: "unsealed", label: "Unsealed" },
  { value: "factory_sealed", label: "Factory Sealed" },
  { value: "tamper_evident", label: "Tamper-Evident Bag" },
  { value: "good", label: "Good Condition" },
  { value: "damaged", label: "Damaged" },
  { value: "powered_on", label: "Powered On" },
  { value: "powered_off", label: "Powered Off" },
  { value: "other", label: "Other" },
] as const;

// =============================================================================
// COC DISPOSITION OPTIONS
// =============================================================================

export const COC_DISPOSITIONS = [
  { value: "in_custody", label: "In Custody" },
  { value: "released", label: "Released to Owner" },
  { value: "returned", label: "Returned to Submitter" },
  { value: "destroyed", label: "Destroyed" },
] as const;

// =============================================================================
// COC TRANSFER METHODS
// =============================================================================

export const COC_TRANSFER_METHODS = [
  { value: "in_person", label: "In Person" },
  { value: "courier", label: "Courier" },
  { value: "mail", label: "Mail/Shipping" },
  { value: "locker", label: "Evidence Locker" },
  { value: "other", label: "Other" },
] as const;

// =============================================================================
// IAR EVENT CATEGORIES
// =============================================================================

export const IAR_EVENT_CATEGORIES: { value: IAREventCategory; label: string; icon: string }[] = [
  { value: "search_warrant", label: "Search Warrant Execution", icon: "⚖️" },
  { value: "evidence_acquisition", label: "Evidence Acquisition", icon: "💾" },
  { value: "evidence_transfer", label: "Evidence Transfer", icon: "🔄" },
  { value: "processing", label: "Processing / Imaging", icon: "⚙️" },
  { value: "analysis", label: "Analysis / Examination", icon: "🔬" },
  { value: "keyword_search", label: "Keyword Search", icon: "🔍" },
  { value: "privileged_review", label: "Privileged/Attorney Review", icon: "🔒" },
  { value: "attorney_review", label: "Attorney Work Product Review", icon: "📑" },
  { value: "report_generation", label: "Report Generation", icon: "📝" },
  { value: "court_testimony", label: "Court Testimony", icon: "🏛️" },
  { value: "consultation", label: "Consultation", icon: "💬" },
  { value: "administrative", label: "Administrative", icon: "📁" },
  { value: "other", label: "Other", icon: "📌" },
];

// =============================================================================
// COC TRANSFER PURPOSES
// =============================================================================

export const COC_TRANSFER_PURPOSES = [
  { value: "examination", label: "Forensic Examination" },
  { value: "analysis", label: "Analysis" },
  { value: "storage", label: "Secure Storage" },
  { value: "court", label: "Court Presentation" },
  { value: "return", label: "Return to Owner" },
  { value: "review", label: "Attorney/Privileged Review" },
  { value: "other", label: "Other" },
] as const;

// =============================================================================
// USER ACTIVITY CATEGORIES
// =============================================================================

export const USER_ACTIVITY_CATEGORIES = [
  { value: "login", label: "Login/Logoff" },
  { value: "file_access", label: "File Access" },
  { value: "file_modification", label: "File Modification" },
  { value: "file_deletion", label: "File Deletion" },
  { value: "web_browsing", label: "Web Browsing" },
  { value: "email", label: "Email Activity" },
  { value: "messaging", label: "Messaging/Chat" },
  { value: "application", label: "Application Usage" },
  { value: "usb_device", label: "USB Device Connection" },
  { value: "network", label: "Network Activity" },
  { value: "cloud_storage", label: "Cloud Storage" },
  { value: "social_media", label: "Social Media" },
  { value: "printing", label: "Print Activity" },
  { value: "search", label: "Search Queries" },
  { value: "other", label: "Other" },
] as const;

// =============================================================================
// REPORT PRESETS
// =============================================================================

export const REPORT_PRESETS: ReportPresetConfig[] = [
  {
    id: "law_enforcement",
    name: "Law Enforcement",
    icon: "🛡️",
    description: "Court-ready formatting with chain of custody",
    defaultClassification: "LawEnforcementSensitive",
    defaultSections: {
      executiveSummary: true,
      scope: true,
      methodology: true,
      chainOfCustody: true,
      timeline: true,
      conclusions: true,
      appendices: true,
    },
    defaultMethodology: `This examination was conducted in accordance with accepted forensic procedures and practices. The following methodology was employed:

1. **Evidence Intake**: All evidence was received, documented, and secured following chain of custody protocols.
2. **Forensic Imaging**: Bit-for-bit forensic images were created using validated forensic tools. Hash values were calculated to verify image integrity.
3. **Analysis**: The forensic images were analyzed using industry-standard forensic software. All artifacts were documented with their source locations and timestamps.
4. **Verification**: All findings were verified through multiple methods where possible. Hash values were re-calculated to ensure data integrity throughout the examination.
5. **Documentation**: All actions taken during the examination were documented in examination notes and are reflected in this report.`,
    defaultScope: `This examination was requested to analyze digital evidence related to the investigation. The scope includes:

• Analysis of forensic images of digital storage media
• Recovery and examination of files and artifacts
• Timeline reconstruction of relevant events
• Hash verification of evidence integrity
• Documentation of findings for potential court presentation`,
  },
  {
    id: "corporate",
    name: "Corporate / Internal",
    icon: "🏢",
    description: "Business-focused for internal investigations",
    defaultClassification: "Confidential",
    defaultSections: {
      executiveSummary: true,
      scope: true,
      methodology: true,
      chainOfCustody: false,
      timeline: true,
      conclusions: true,
      appendices: false,
    },
    defaultMethodology: `This investigation followed corporate digital forensics and incident response procedures:

1. **Evidence Collection**: Digital evidence was collected following corporate data handling policies and legal hold requirements.
2. **Analysis**: Evidence was analyzed using approved forensic tools and techniques to identify relevant artifacts.
3. **Documentation**: All findings were documented with supporting evidence references.
4. **Review**: Findings were reviewed for accuracy and completeness before inclusion in this report.`,
    defaultScope: `This examination was conducted as part of an internal investigation. The scope includes analysis of relevant digital evidence to determine facts and findings related to the matter under investigation.`,
  },
  {
    id: "academic",
    name: "Academic / Research",
    icon: "🎓",
    description: "Research-oriented with reproducibility focus",
    defaultClassification: "Internal",
    defaultSections: {
      executiveSummary: true,
      scope: true,
      methodology: true,
      chainOfCustody: false,
      timeline: false,
      conclusions: true,
      appendices: true,
    },
    defaultMethodology: `This analysis employed the following research methodology:

1. **Data Acquisition**: Evidence was acquired using forensically sound methods to preserve data integrity.
2. **Tool Validation**: All analysis tools were validated for accuracy and reliability.
3. **Analysis Protocol**: A systematic analysis protocol was followed to ensure reproducibility.
4. **Peer Review**: Findings were subject to peer review prior to final documentation.`,
    defaultScope: `This examination was conducted for research and educational purposes, analyzing digital evidence using forensic techniques and documenting the methodology for reproducibility.`,
  },
  {
    id: "minimal",
    name: "Minimal / Quick",
    icon: "⚡",
    description: "Essential information only",
    defaultClassification: "Internal",
    defaultSections: {
      executiveSummary: true,
      scope: false,
      methodology: false,
      chainOfCustody: false,
      timeline: false,
      conclusions: true,
      appendices: false,
    },
    defaultMethodology: "",
    defaultScope: "",
  },
  {
    id: "custom",
    name: "Custom",
    icon: "✏️",
    description: "Start from scratch",
    defaultClassification: "Internal",
    defaultSections: {
      executiveSummary: false,
      scope: false,
      methodology: false,
      chainOfCustody: false,
      timeline: false,
      conclusions: false,
      appendices: false,
    },
    defaultMethodology: "",
    defaultScope: "",
  },
];

/** Get preset config by id */
export function getPresetById(id: ReportPreset): ReportPresetConfig | undefined {
  return REPORT_PRESETS.find(p => p.id === id);
}

/** Get default preset */
export function getDefaultPreset(): ReportPresetConfig {
  return REPORT_PRESETS[0]; // law_enforcement
}