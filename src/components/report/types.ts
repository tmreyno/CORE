// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Report Types - Type definitions for forensic report generation
 * 
 * These types match the Rust backend structures for serialization compatibility.
 */

// =============================================================================
// ENUMS / UNION TYPES
// =============================================================================

/** Document classification levels */
export type Classification = 
  | "Public"
  | "Internal"
  | "Confidential"
  | "Restricted"
  | "LawEnforcementSensitive";

/** Finding severity levels */
export type Severity = "Critical" | "High" | "Medium" | "Low" | "Informational";

/** Evidence type categories - must match Rust EvidenceType enum */
export type EvidenceType = 
  | "HardDrive" 
  | "SSD" 
  | "UsbDrive" 
  | "ExternalDrive" 
  | "MemoryCard" 
  | "MobilePhone" 
  | "Tablet" 
  | "Computer" 
  | "Laptop" 
  | "OpticalDisc" 
  | "CloudStorage" 
  | "NetworkCapture" 
  | "ForensicImage" 
  | "Other";

/** Hash algorithm types - must match Rust HashAlgorithmType enum */
export type HashAlgorithmType = 
  | "MD5" 
  | "SHA1" 
  | "SHA256" 
  | "SHA512" 
  | "Blake2b" 
  | "Blake3" 
  | "XXH3" 
  | "XXH64";

/** Appendix content types */
export type AppendixContentType = 
  | "Markdown" 
  | "Text" 
  | "FileListing" 
  | "HashTable" 
  | "FileReference";

/** Signature role types */
export type SignatureRole = "examiner" | "supervisor" | "reviewer";

/** Output format types */
export type OutputFormatType = "Pdf" | "Docx" | "Html" | "Markdown" | "Typst";

/** Report type - determines which wizard flow and output format to use */
export type ReportType =
  | "forensic_examination"
  | "chain_of_custody"
  | "investigative_activity"
  | "user_activity"
  | "timeline"
  | "evidence_collection";

// =============================================================================
// CHAIN OF CUSTODY (COC) TYPES — EPA CID OCEFT Form 7-01
// =============================================================================

/** Individual Chain of Custody item - one per evidence item (OCEFT Form 7-01) */
export interface COCItem {
  /** Internal UI identifier for SolidJS list keying */
  id: string;
  /** Unique COC item number (e.g., "0464-24-AH-01") */
  coc_number: string;
  /** Evidence item ID reference (Item/Box Number on Form 7-01) */
  evidence_id: string;

  // ── Form 7-01 Header Section ──
  /** Case Title (Form 7-01 header) */
  case_title?: string;
  /** Office (Form 7-01 header) */
  office?: string;
  /** Case number */
  case_number: string;

  // ── Owner / Source / Contact Section ──
  /** Owner Name (Form 7-01) */
  owner_name?: string;
  /** Owner Address (Form 7-01) */
  owner_address?: string;
  /** Owner Phone Number (Form 7-01) */
  owner_phone?: string;
  /** Source of the evidence (Form 7-01) */
  source?: string;
  /** Other Contact Name (Form 7-01) */
  other_contact_name?: string;
  /** Relation to Owner (Form 7-01) */
  other_contact_relation?: string;
  /** Other Contact Phone Number (Form 7-01) */
  other_contact_phone?: string;

  // ── Collection Method (Form 7-01 checkboxes) ──
  /** Collection method (search_warrant, grand_jury_subpoena, consent_seizure, etc.) */
  collection_method?: string;
  /** Other collection method description */
  collection_method_other?: string;

  // ── Item Details ──
  /** Item description */
  description: string;
  /** Evidence type/category */
  item_type: EvidenceType;
  /** Make/manufacturer */
  make?: string;
  /** Model */
  model?: string;
  /** Serial number */
  serial_number?: string;
  /** Capacity/size description */
  capacity?: string;
  /** Condition when received (e.g., "Sealed", "Unsealed", "Damaged") */
  condition: string;

  // ── Collection / Custody Section ──
  /** Original evidence acquisition date - defaults from forensic image metadata */
  acquisition_date: string;
  /** Date/time item entered chain of custody (editable, defaults to acquisition_date) */
  entered_custody_date: string;
  /** Who collected the evidence (Collected By on Form 7-01) */
  submitted_by: string;
  /** Date collected (Form 7-01 collected date) */
  collected_date?: string;
  /** Who received the evidence */
  received_by: string;
  /** Location where evidence was received */
  received_location?: string;
  /** Storage location */
  storage_location?: string;
  /** Reason for submission / authorization reference */
  reason_submitted?: string;
  /** Remarks (Form 7-01) */
  notes?: string;

  // ── Transfer Records ──
  /** Transfer records for this item (Relinquished to / Storage Location) */
  transfers: COCTransfer[];

  // ── Intake Verification ──
  /** Hash values at intake */
  intake_hashes: HashValue[];

  // ── Final Disposition (Form 7-01) ──
  /** Whether item has been returned/released */
  disposition?: "in_custody" | "released" | "returned" | "destroyed";
  /** Final Disposition By (Print/Sign) */
  disposition_by?: string;
  /** Returned to (Sign/Date) */
  returned_to?: string;
  /** Destruction Date */
  destruction_date?: string;
  /** Disposition date */
  disposition_date?: string;
  /** Disposition notes / Other Disposition (Describe) */
  disposition_notes?: string;

  // ── Immutability (schema v5) ──
  /** Immutability status: 'draft', 'locked', 'voided' */
  status?: "draft" | "locked" | "voided";
  /** When the item was locked (ISO 8601) */
  locked_at?: string;
  /** Who locked the item (initials) */
  locked_by?: string;
}

/** COC transfer/handoff record (Form 7-01: Relinquished to / Storage Location) */
export interface COCTransfer {
  /** Internal UI identifier */
  id: string;
  /** Date/time of transfer */
  timestamp: string;
  /** Person releasing custody (Relinquished By on Form 7-01) */
  released_by: string;
  /** Person receiving custody (Relinquished To on Form 7-01) */
  received_by: string;
  /** Purpose of transfer */
  purpose: string;
  /** Location of transfer */
  location?: string;
  /** Storage location (Form 7-01: Storage Location and Date Entered) */
  storage_location?: string;
  /** Storage date when entered into storage (Form 7-01) */
  storage_date?: string;
  /** Transfer method (in-person, courier, mail) */
  method?: string;
  /** Notes */
  notes?: string;
}

// =============================================================================
// INVESTIGATIVE ACTIVITY REPORT (IAR) TYPES
// =============================================================================

/** IAR event category */
export type IAREventCategory =
  | "search_warrant"
  | "evidence_acquisition"
  | "evidence_transfer"
  | "processing"
  | "analysis"
  | "keyword_search"
  | "privileged_review"
  | "attorney_review"
  | "report_generation"
  | "court_testimony"
  | "consultation"
  | "administrative"
  | "other";

/** Single IAR activity entry */
export interface IAREntry {
  /** Unique entry ID */
  id: string;
  /** Date/time of activity */
  date: string;
  /** End date/time (for multi-day activities) */
  end_date?: string;
  /** Category of activity */
  category: IAREventCategory;
  /** Personnel who performed the activity */
  personnel: string;
  /** Personnel title/role */
  personnel_role?: string;
  /** Description of activity */
  description: string;
  /** Related evidence item IDs */
  evidence_refs: string[];
  /** Location where activity occurred */
  location?: string;
  /** Hours spent */
  hours_spent?: number;
  /** Key findings or outcomes */
  outcome?: string;
  /** Authorization reference (warrant #, order #) */
  authorization_ref?: string;
  /** Keywords used (for keyword search activities) */
  keywords?: string[];
  /** Tools/software used */
  tools_used?: string[];
  /** Notes */
  notes?: string;
}

/** IAR summary section */
export interface IARSummary {
  /** Investigation start date (e.g., SW execution date) */
  investigation_start: string;
  /** Investigation end/current date */
  investigation_end?: string;
  /** Lead investigator/examiner */
  lead_examiner: string;
  /** Case synopsis */
  synopsis: string;
  /** Authorization details (search warrant info) */
  authorization: string;
  /** List of all personnel involved */
  personnel_list: IARPersonnel[];
  /** Total hours expended */
  total_hours?: number;
}

/** Personnel entry for IAR */
export interface IARPersonnel {
  name: string;
  role: string;
  organization?: string;
  hours_contributed?: number;
}

// =============================================================================
// EVIDENCE COLLECTION REPORT TYPES
// =============================================================================

/** Evidence collection report data */
export interface EvidenceCollectionData {
  /** Date and time evidence collection started */
  collection_date: string;
  /** System date/time if different from actual collection time */
  system_date_time?: string;
  /** Collecting officer/examiner */
  collecting_officer: string;
  /** Authorization (warrant/consent) */
  authorization: string;
  /** Authorization date */
  authorization_date?: string;
  /** Authorizing authority (judge name) */
  authorizing_authority?: string;
  /** Witnesses present */
  witnesses: string[];
  /** Items collected with collection-specific details */
  collected_items: CollectedItem[];
  /** Photography/documentation notes */
  documentation_notes?: string;
  /** Environmental conditions */
  conditions?: string;
}

/** Individual collected evidence item */
export interface CollectedItem {
  /** Internal UI identifier */
  id: string;
  /** Item number (sequential) */
  item_number: string;
  /** Description of the item */
  description: string;

  // --- Per-Item Collection Info ---
  /** Collection date/time for this specific item (may differ from header) */
  item_collection_datetime?: string;
  /** Device system clock date/time at moment of collection */
  item_system_datetime?: string;
  /** Collecting officer for this item (overrides header if different) */
  item_collecting_officer?: string;
  /** Authorization for this specific item (overrides header if different) */
  item_authorization?: string;

  // --- Device Identification ---
  /** Device type (Computer, Laptop, Mobile Phone, Tablet, Server, etc.) */
  device_type: string;
  /** Custom device type if "Other" selected */
  device_type_other?: string;
  /** Storage interface type (USB, SATA, IDE, NVMe M.2, RAID, etc.) */
  storage_interface: string;
  /** Custom storage interface if "Other" selected */
  storage_interface_other?: string;
  /** Brand / Manufacturer */
  brand?: string;
  /** Make */
  make?: string;
  /** Model */
  model?: string;
  /** Color */
  color?: string;
  /** Serial number */
  serial_number?: string;
  /** IMEI number (mobile devices) */
  imei?: string;
  /** Other device identifiers (asset tag, MAC address, etc.) */
  other_identifiers?: string;

  // --- Location ---
  /** Building where evidence was located */
  building?: string;
  /** Room where evidence was located */
  room?: string;
  /** Other location details */
  location_other?: string;

  // --- Forensic Image ---
  /** Forensic image/container format (AD1, E01, L01, DD, 001, VHD, VMDK, TAR, 7Z, ZIP, etc.) */
  image_format?: string;
  /** Custom image format if "Other" selected */
  image_format_other?: string;
  /** Acquisition method (Logical File/Folder, Logical Partition, Physical, Native File, etc.) */
  acquisition_method?: string;
  /** Custom acquisition method if "Other" selected */
  acquisition_method_other?: string;

  // --- Condition & Packaging ---
  /** Condition at collection */
  condition: string;
  /** How it was packaged */
  packaging: string;

  // --- Additional Info ---
  /** Other HDD/SSD/USB/NVMe information */
  storage_notes?: string;
  /** Notes — passwords, BitLocker, encryption, phone details, device user details, etc. */
  notes?: string;
  /** Photo reference numbers */
  photo_refs?: string[];

  // --- Linkage ---
  /** FK to evidence_files — links this collected item to a discovered evidence container */
  evidence_file_id?: string;
  /** FK to coc_items — cross-references this item to its COC record */
  coc_item_id?: string;
}

// =============================================================================
// USER ACTIVITY REPORT TYPES
// =============================================================================

/** User activity report data */
export interface UserActivityData {
  /** Target user/account being investigated */
  target_user: string;
  /** User's known aliases/accounts */
  user_aliases?: string[];
  /** Time range of interest */
  time_range_start?: string;
  time_range_end?: string;
  /** Activity categories found */
  activity_entries: UserActivityEntry[];
  /** Summary of findings */
  summary?: string;
}

/** Single user activity entry */
export interface UserActivityEntry {
  /** Activity ID */
  id: string;
  /** Timestamp */
  timestamp: string;
  /** Activity category */
  category: string;
  /** Description */
  description: string;
  /** Source artifact/file */
  source_artifact: string;
  /** Source evidence item */
  evidence_ref?: string;
  /** User account associated */
  user_account?: string;
  /** Application/program involved */
  application?: string;
  /** Significance level */
  significance: Severity;
  /** Notes */
  notes?: string;
}

// =============================================================================
// TIMELINE REPORT TYPES
// =============================================================================

/** Timeline report configuration */
export interface TimelineReportData {
  /** Time range filter */
  time_range_start?: string;
  time_range_end?: string;
  /** Event categories to include */
  included_categories: string[];
  /** Events (from project activity log + manual entries) */
  events: TimelineEvent[];
  /** Key events highlighted for the report */
  key_events: string[];
  /** Narrative connecting events */
  narrative?: string;
}

// =============================================================================
// CORE INTERFACES
// =============================================================================

/** Report metadata */
export interface ReportMetadata {
  title: string;
  report_number: string;
  version: string;
  classification: Classification;
  generated_at: string;
  generated_by: string;
}

/** Case information */
export interface CaseInfo {
  case_number: string;
  case_name?: string;
  agency?: string;
  requestor?: string;
  request_date?: string;
  exam_start_date?: string;
  exam_end_date?: string;
  investigation_type?: string;
  description?: string;
}

/** Examiner information */
export interface ExaminerInfo {
  name: string;
  title?: string;
  organization?: string;
  email?: string;
  phone?: string;
  certifications: string[];
  badge_number?: string;
}

/** Hash value record */
export interface HashValue {
  item: string;
  algorithm: HashAlgorithmType;
  value: string;
  computed_at?: string;
  verified?: boolean;
}

/** Evidence item */
export interface EvidenceItem {
  evidence_id: string;
  description: string;
  evidence_type: EvidenceType;
  make?: string;
  model?: string;
  serial_number?: string;
  capacity?: string;
  acquisition_date?: string;
  acquisition_method?: string;
  acquisition_tool?: string;
  acquisition_hashes: HashValue[];
  verification_hashes: HashValue[];
  notes?: string;
}

/** Finding/artifact record */
export interface Finding {
  id: string;
  title: string;
  severity: Severity;
  category: string;
  description: string;
  artifact_paths: string[];
  timestamps: string[];
  evidence_refs: string[];
  analysis: string;
  conclusions?: string;
}

/** Timeline event */
export interface TimelineEvent {
  timestamp: string;
  event_type: string;
  description: string;
  source: string;
  evidence_ref?: string;
  artifact_path?: string;
}

/** Tool information */
export interface ToolInfo {
  name: string;
  version: string;
  vendor?: string;
  purpose?: string;
}

/** Chain of custody record */
export interface CustodyRecord {
  timestamp: string;
  action: string;
  handler: string;
  location?: string;
  notes?: string;
}

/** Hash record for report */
export interface HashRecord {
  algorithm: string;
  value: string;
  verified?: boolean;
  timestamp?: string;
  item_reference?: string;
}

/** Report appendix */
export interface Appendix {
  appendix_id: string;
  title: string;
  content_type: AppendixContentType;
  content: string;
}

/** Signature/approval record */
export interface SignatureRecord {
  role: SignatureRole;
  name: string;
  signature?: string;
  signed_date?: string;
  notes?: string;
  certified?: boolean;
}

/** Output format configuration */
export interface OutputFormat {
  format: OutputFormatType;
  name: string;
  description: string;
  extension: string;
  supported: boolean;
}

// =============================================================================
// MAIN REPORT INTERFACE
// =============================================================================

/** Complete forensic report structure */
export interface ForensicReport {
  metadata: ReportMetadata;
  case_info: CaseInfo;
  examiner: ExaminerInfo;
  /** Report type - determines output structure */
  report_type?: ReportType;
  executive_summary?: string;
  scope?: string;
  methodology?: string;
  evidence_items: EvidenceItem[];
  chain_of_custody: CustodyRecord[];
  findings: Finding[];
  timeline: TimelineEvent[];
  hash_records: HashRecord[];
  tools: ToolInfo[];
  conclusions?: string;
  appendices: Appendix[];
  signatures?: SignatureRecord[];
  notes?: string;
  /** Chain of Custody items (per-evidence Form 7 data) */
  coc_items?: COCItem[];
  /** Investigative Activity Report data */
  iar_data?: { summary: IARSummary; entries: IAREntry[] };
  /** Evidence Collection Report data */
  evidence_collection?: EvidenceCollectionData;
  /** User Activity Report data */
  user_activity?: UserActivityData;
  /** Timeline Report data */
  timeline_report?: TimelineReportData;
}
