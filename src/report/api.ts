// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

/**
 * Report API - Tauri command wrappers for report generation
 *
 * This module provides TypeScript interfaces for the Rust report commands
 */

import { invoke } from "@tauri-apps/api/core";
import type { ContainerInfo, DiscoveredFile } from "../types";
import type {
  DbNormalizedArtifact,
  DbSourceAnalysisRecord,
  ProjectDbAnnotationRecord,
} from "../api/commands";
import { formatBytes } from "../utils";
import { isTauri } from "../utils/platform";

export interface ProjectDbReportAppendix {
  appendix_id: string;
  title: string;
  content_type: "Markdown";
  content: string;
}

// =============================================================================
// Input Types (sent to backend)
// =============================================================================

export interface StoredHashInput {
  algorithm: string;
  hash: string;
  verified?: boolean;
}

export interface ContainerInfoInput {
  container_type: string;
  path: string;
  filename: string;
  size: number;
  // EWF fields
  case_number?: string;
  evidence_number?: string;
  examiner_name?: string;
  description?: string;
  notes?: string;
  acquiry_date?: string;
  model?: string;
  serial_number?: string;
  total_size?: number;
  // Hash info
  stored_hashes?: StoredHashInput[];
  computed_hash?: StoredHashInput;
}

// =============================================================================
// Output Types (received from backend)
// =============================================================================

export interface HashRecord {
  item: string;
  algorithm: string;
  value: string;
  computed_at?: string;
  verified?: boolean;
  source_id?: string;
  source_ref?: unknown;
}

export interface ImageInfo {
  format: string;
  file_names: string[];
  total_size: number;
  segments?: number;
  compression?: string;
  acquisition_tool?: string;
  acquisition_date?: string;
}

export interface EvidenceItem {
  evidence_id: string;
  description: string;
  evidence_type: string;
  make?: string;
  model?: string;
  serial_number?: string;
  capacity?: string;
  condition?: string;
  received_date?: string;
  submitted_by?: string;
  acquisition_hashes: HashRecord[];
  image_info?: ImageInfo;
  notes?: string;
}

export interface ProjectDbReportEvidence {
  evidenceItems: EvidenceItem[];
  hashRecords: HashRecord[];
  hashAlgorithmSummaries: ReportHashAlgorithmSummary[];
  verificationResultSummaries: ReportVerificationResultSummary[];
  artifacts: DbNormalizedArtifact[];
  artifactSummaries: ReportArtifactSummary[];
  artifactCategories: ReportArtifactCategorySummary[];
  artifactEvidenceSummaries: ReportArtifactEvidenceSummary[];
  artifactExtractorSummaries: ReportArtifactExtractorSummary[];
  sourceAnalyses: DbSourceAnalysisRecord[];
  sourceAnalysisSummaries: ReportSourceAnalysisSummary[];
  sourceAnalysisCategorySummaries: ReportSourceAnalysisCategorySummary[];
  annotations: ProjectDbAnnotationRecord[];
}

export interface ReportHashAlgorithmSummary {
  algorithm: string;
  algorithmLabel: string;
  count: number;
  evidenceFileCount: number;
  sourceCount: number;
  latestComputedAt?: string | null;
}

export interface ReportVerificationResultSummary {
  result: string;
  count: number;
  hashCount: number;
  latestVerifiedAt?: string | null;
}

export interface ReportArtifactSummary {
  id: string;
  evidenceFileId?: string | null;
  sourceId: string;
  sourceRef?: unknown | null;
  name: string;
  category: string;
  typeDescription: string;
  mimeType?: string | null;
  size: number;
  sizeDisplay: string;
  confidence: string;
  isText: boolean;
  preview?: string | null;
  metadata: Record<string, string>;
  extractor: string;
  extractedAt: string;
}

export interface ReportArtifactCategorySummary {
  category: string;
  count: number;
}

export interface ReportArtifactEvidenceSummary {
  evidenceFileId?: string | null;
  count: number;
  totalSize: number;
  totalSizeDisplay: string;
  textCount: number;
  categoryCount: number;
  latestExtractedAt?: string | null;
}

export interface ReportArtifactExtractorSummary {
  extractor: string;
  count: number;
  totalSize: number;
  totalSizeDisplay: string;
  textCount: number;
  categoryCount: number;
  evidenceFileCount: number;
  latestExtractedAt?: string | null;
}

export interface ReportSourceAnalysisSummary {
  id: string;
  evidenceFileId?: string | null;
  sourceId: string;
  sourceRef?: unknown | null;
  totalSize: number;
  totalSizeDisplay: string;
  offset: number;
  bytesAnalyzed: number;
  bytesAnalyzedDisplay: string;
  magicHex: string;
  signatureCount: number;
  primarySignature?: string | null;
  primaryMimeType?: string | null;
  primaryCategory: string;
  entropy: number;
  printableRatio: number;
  isLikelyText: boolean;
  indicators: ReportSourceIndicator[];
  indicatorCount: number;
  preview?: string | null;
  analyzedAt: string;
  analyzer: string;
}

export interface ReportSourceIndicator {
  indicatorType: string;
  value: string;
  offset: number;
  length: number;
  confidence: string;
}

export interface ReportSourceAnalysisCategorySummary {
  category: string;
  count: number;
  evidenceFileCount: number;
  avgEntropy: number;
  textLikeCount: number;
  latestAnalyzedAt?: string | null;
}

// =============================================================================
// Conversion Helpers
// =============================================================================

/**
 * Convert frontend container info to backend input format
 */
export function containerToInput(
  file: DiscoveredFile,
  info: ContainerInfo | undefined,
  hashInfo:
    { algorithm: string; hash: string; verified?: boolean | null } | undefined,
): ContainerInfoInput {
  // Extract data from various container types
  const ewfInfo = info?.e01 || info?.l01;
  const ad1Info = info?.ad1;
  const ufedInfo = info?.ufed;

  // Get stored hashes from container
  const storedHashes: StoredHashInput[] = [];

  if (ewfInfo?.stored_hashes) {
    for (const h of ewfInfo.stored_hashes) {
      storedHashes.push({
        algorithm: h.algorithm,
        hash: h.hash,
        verified: h.verified ?? undefined,
      });
    }
  }

  if (ad1Info?.companion_log) {
    const log = ad1Info.companion_log;
    if (log.md5_hash) {
      storedHashes.push({ algorithm: "MD5", hash: log.md5_hash });
    }
    if (log.sha1_hash) {
      storedHashes.push({ algorithm: "SHA1", hash: log.sha1_hash });
    }
  }

  return {
    container_type: file.container_type,
    path: file.path,
    filename: file.filename,
    size: file.size,
    case_number:
      ewfInfo?.case_number ??
      ad1Info?.companion_log?.case_number ??
      ufedInfo?.case_info?.case_identifier ??
      undefined,
    evidence_number:
      ewfInfo?.evidence_number ??
      ad1Info?.companion_log?.evidence_number ??
      ufedInfo?.evidence_number ??
      undefined,
    examiner_name:
      ewfInfo?.examiner_name ??
      ad1Info?.companion_log?.examiner ??
      ufedInfo?.case_info?.examiner_name ??
      undefined,
    description: ewfInfo?.description ?? undefined,
    notes: ewfInfo?.notes ?? ad1Info?.companion_log?.notes ?? undefined,
    acquiry_date:
      ewfInfo?.acquiry_date ??
      ad1Info?.companion_log?.acquisition_date ??
      ufedInfo?.extraction_info?.start_time ??
      undefined,
    model: ewfInfo?.model ?? ufedInfo?.device_info?.model ?? undefined,
    serial_number:
      ewfInfo?.serial_number ??
      ufedInfo?.device_info?.serial_number ??
      undefined,
    total_size:
      ewfInfo?.total_size ?? ad1Info?.total_size ?? ufedInfo?.size ?? undefined,
    stored_hashes: storedHashes.length > 0 ? storedHashes : undefined,
    computed_hash: hashInfo
      ? {
          algorithm: hashInfo.algorithm,
          hash: hashInfo.hash,
          verified: hashInfo.verified ?? undefined,
        }
      : undefined,
  };
}

/**
 * Convert multiple containers to input format
 */
export function containersToInputs(
  files: DiscoveredFile[],
  fileInfoMap: Map<string, ContainerInfo>,
  fileHashMap: Map<
    string,
    { algorithm: string; hash: string; verified?: boolean | null }
  >,
): ContainerInfoInput[] {
  return files.map((file) =>
    containerToInput(
      file,
      fileInfoMap.get(file.path),
      fileHashMap.get(file.path),
    ),
  );
}

// =============================================================================
// API Commands
// =============================================================================

/**
 * Extract evidence items from container info
 *
 * This sends container data to the backend for conversion to properly
 * formatted evidence items suitable for reports.
 */
export async function extractEvidenceFromContainers(
  containers: ContainerInfoInput[],
): Promise<EvidenceItem[]> {
  if (!isTauri) {
    void containers;
    return [];
  }

  return invoke<EvidenceItem[]>("extract_evidence_from_containers", {
    containers,
  });
}

/**
 * Extract report-ready evidence, hashes, and normalized artifacts from .ffxdb.
 */
export async function extractReportEvidenceFromProjectDb(): Promise<ProjectDbReportEvidence> {
  if (!isTauri) {
    return emptyProjectDbReportEvidence();
  }

  return invoke<ProjectDbReportEvidence>(
    "extract_report_evidence_from_project_db",
  );
}

function emptyProjectDbReportEvidence(): ProjectDbReportEvidence {
  return {
    evidenceItems: [],
    hashRecords: [],
    hashAlgorithmSummaries: [],
    verificationResultSummaries: [],
    artifacts: [],
    artifactSummaries: [],
    artifactCategories: [],
    artifactEvidenceSummaries: [],
    artifactExtractorSummaries: [],
    sourceAnalyses: [],
    sourceAnalysisSummaries: [],
    sourceAnalysisCategorySummaries: [],
    annotations: [],
  };
}

/**
 * Convert project DB engine facts into report appendices accepted by the
 * ForensicReport renderers.
 */
export function buildProjectDbEvidenceAppendices(
  evidence: ProjectDbReportEvidence,
  startIndex = 0,
): ProjectDbReportAppendix[] {
  const appendices: ProjectDbReportAppendix[] = [];

  const hashContent = buildHashAppendix(evidence);
  if (hashContent) {
    appendices.push(
      makeAppendix(
        startIndex + appendices.length,
        "Project Hash and Verification Summary",
        hashContent,
      ),
    );
  }

  const artifactContent = buildArtifactAppendix(evidence);
  if (artifactContent) {
    appendices.push(
      makeAppendix(
        startIndex + appendices.length,
        "Normalized Artifact Summary",
        artifactContent,
      ),
    );
  }

  const sourceAnalysisContent = buildSourceAnalysisAppendix(evidence);
  if (sourceAnalysisContent) {
    appendices.push(
      makeAppendix(
        startIndex + appendices.length,
        "Source Analysis Summary",
        sourceAnalysisContent,
      ),
    );
  }

  const annotationContent = buildAnnotationAppendix(evidence);
  if (annotationContent) {
    appendices.push(
      makeAppendix(
        startIndex + appendices.length,
        "Hex Review and Annotation Findings",
        annotationContent,
      ),
    );
  }

  return appendices;
}

/**
 * Create a single evidence item from container info
 */
export async function createEvidenceFromContainer(
  container: ContainerInfoInput,
  evidenceId: string,
): Promise<EvidenceItem> {
  if (!isTauri) {
    void container;
    void evidenceId;
    throw new Error("Report evidence extraction is available in the desktop app.");
  }

  return invoke<EvidenceItem>("create_evidence_from_container", {
    container,
    evidenceId,
  });
}

function makeAppendix(
  index: number,
  title: string,
  content: string,
): ProjectDbReportAppendix {
  return {
    appendix_id: String.fromCharCode("A".charCodeAt(0) + index),
    title,
    content_type: "Markdown",
    content,
  };
}

function buildHashAppendix(evidence: ProjectDbReportEvidence): string {
  const lines: string[] = [];

  if (evidence.hashAlgorithmSummaries.length > 0) {
    lines.push("### Hash Algorithms");
    lines.push("");
    lines.push(
      "| Algorithm | Hashes | Evidence Files | Sources | Latest Computed |",
    );
    lines.push("| --- | ---: | ---: | ---: | --- |");
    for (const summary of evidence.hashAlgorithmSummaries) {
      lines.push(
        `| ${summary.algorithmLabel || summary.algorithm} | ${summary.count} | ${summary.evidenceFileCount} | ${summary.sourceCount} | ${summary.latestComputedAt || "-"} |`,
      );
    }
    lines.push("");
  }

  if (evidence.verificationResultSummaries.length > 0) {
    lines.push("### Verification Results");
    lines.push("");
    lines.push("| Result | Checks | Hashes | Latest Verified |");
    lines.push("| --- | ---: | ---: | --- |");
    for (const summary of evidence.verificationResultSummaries) {
      lines.push(
        `| ${summary.result} | ${summary.count} | ${summary.hashCount} | ${summary.latestVerifiedAt || "-"} |`,
      );
    }
    lines.push("");
  }

  if (evidence.hashRecords.length > 0) {
    lines.push("### Hash Records");
    lines.push("");
    lines.push("| Item | Source | Algorithm | Hash Value | Computed At | Verified |");
    lines.push("| --- | --- | --- | --- | --- | --- |");
    for (const record of evidence.hashRecords.slice(0, 250)) {
      lines.push(
        `| ${record.item || "-"} | ${record.source_id || "-"} | ${record.algorithm} | \`${record.value}\` | ${record.computed_at || "-"} | ${record.verified ?? "-"} |`,
      );
    }
    if (evidence.hashRecords.length > 250) {
      lines.push(
        `| ... | ... | ... | ... | ... | ${evidence.hashRecords.length - 250} more records omitted |`,
      );
    }
    lines.push("");
  }

  return lines.join("\n").trim();
}

function buildArtifactAppendix(evidence: ProjectDbReportEvidence): string {
  const lines: string[] = [];

  if (evidence.artifactCategories.length > 0) {
    lines.push("### Artifact Categories");
    lines.push("");
    lines.push("| Category | Count |");
    lines.push("| --- | ---: |");
    for (const summary of evidence.artifactCategories) {
      lines.push(`| ${summary.category} | ${summary.count} |`);
    }
    lines.push("");
  }

  if (evidence.artifactExtractorSummaries.length > 0) {
    lines.push("### Extractor Coverage");
    lines.push("");
    lines.push(
      "| Extractor | Artifacts | Total Size | Text | Categories | Evidence Files | Latest Extracted |",
    );
    lines.push("| --- | ---: | ---: | ---: | ---: | ---: | --- |");
    for (const summary of evidence.artifactExtractorSummaries) {
      lines.push(
        `| ${summary.extractor} | ${summary.count} | ${summary.totalSizeDisplay} | ${summary.textCount} | ${summary.categoryCount} | ${summary.evidenceFileCount} | ${summary.latestExtractedAt || "-"} |`,
      );
    }
    lines.push("");
  }

  if (evidence.artifactSummaries.length > 0) {
    lines.push("### Extracted Artifacts");
    lines.push("");
    lines.push(
      "| Name | Category | Type | Size | Key Metadata | Source | Extractor |",
    );
    lines.push("| --- | --- | --- | ---: | --- | --- | --- |");
    for (const artifact of evidence.artifactSummaries.slice(0, 250)) {
      lines.push(
        `| ${artifact.name} | ${artifact.category} | ${artifact.typeDescription} | ${artifact.sizeDisplay} | ${artifactMetadataSummary(artifact.metadata)} | ${artifact.sourceId} | ${artifact.extractor} |`,
      );
    }
    if (evidence.artifactSummaries.length > 250) {
      lines.push(
        `| ... | ... | ... | ... | ... | ${evidence.artifactSummaries.length - 250} additional artifact(s) omitted from appendix preview | ... |`,
      );
    }
    lines.push("");
  }

  return lines.join("\n").trim();
}

function artifactMetadataSummary(metadata: Record<string, string>): string {
  const pairs = [
    ["image.dimensions", "dimensions"],
    ["image.format", "format"],
    ["exif.make", "make"],
    ["exif.model", "model"],
    ["exif.dateTimeOriginal", "captured"],
    ["exif.dateTime", "modified"],
    ["exif.lensModel", "lens"],
    ["exif.bodySerialNumber", "serial"],
    ["indicators.emailCount", "emails"],
    ["indicators.emails", "email values"],
    ["indicators.ipv4Count", "IPv4s"],
    ["indicators.ipv4", "IPv4 values"],
    ["indicators.urlCount", "URLs"],
    ["indicators.urls", "URL values"],
    ["indicators.windowsPathCount", "Windows paths"],
    ["indicators.windowsPaths", "Windows path values"],
    ["gps.latitude", "lat"],
    ["gps.longitude", "lon"],
    ["pdf.version", "pdf"],
    ["sqlite.pageSize", "page size"],
    ["sqlite.pageCount", "pages"],
    ["sqlite.textEncoding", "encoding"],
    ["sqlite.tableCount", "tables"],
    ["sqlite.viewCount", "views"],
    ["sqlite.totalRows", "rows"],
    ["sqlite.tableNames", "table names"],
    ["sqlite.largestTable", "largest table"],
    ["email.from", "from"],
    ["email.to", "to"],
    ["email.subject", "subject"],
    ["email.date", "date"],
    ["email.messageId", "message id"],
    ["email.attachmentCount", "attachments"],
    ["email.inlineAttachmentCount", "inline"],
    ["email.attachmentNames", "attachment names"],
    ["email.contentTypes", "content types"],
    ["registry.version", "registry"],
    ["registry.dirty", "dirty"],
    ["registry.lastWriteTime", "last write"],
    ["registry.hiveBinsDataSize", "hive bins"],
    ["registry.path", "path"],
    ["binary.analysisStatus", "binary"],
    ["binary.indexAnalysisStatus", "binary"],
    ["binary.format", "format"],
    ["binary.architecture", "arch"],
    ["binary.entryPoint", "entry"],
    ["binary.importLibraries", "imports"],
    ["binary.exports", "exports"],
    ["binary.sections", "sections"],
    ["binary.executableSections", "executable sections"],
    ["binary.writableSections", "writable sections"],
    ["binary.maxSectionEntropy", "max entropy"],
    ["binary.highEntropySections", "high entropy"],
    ["pe.isDriver", "driver"],
    ["pe.driverType", "driver type"],
    ["pe.driverIndicators", "driver indicators"],
    ["pe.driverServiceNames", "services"],
    ["pe.driverDeviceNames", "devices"],
    ["pe.driverDosDeviceNames", "DOS devices"],
    ["pe.driverRegistryPaths", "registry paths"],
    ["pe.driverPdbPaths", "PDB"],
    ["pe.driverUrls", "URLs"],
    ["pe.driverGuids", "GUIDs"],
    ["pe.version.CompanyName", "company"],
    ["pe.version.FileDescription", "description"],
    ["pe.version.FileVersion", "file version"],
    ["pe.version.OriginalFilename", "original name"],
    ["pe.version.ProductName", "product"],
    ["pe.version.ProductVersion", "product version"],
    ["macho.cpuType", "CPU"],
    ["macho.fileType", "Mach-O type"],
    ["linux.moduleDetected", "module"],
    ["linux.moduleNames", "module names"],
    ["linux.moduleVermagic", "vermagic"],
    ["linux.moduleVersions", "module versions"],
    ["linux.moduleLicenses", "licenses"],
    ["linux.moduleAuthors", "authors"],
    ["linux.moduleDescriptions", "descriptions"],
    ["linux.moduleAliases", "aliases"],
    ["linux.moduleDependencies", "dependencies"],
    ["linux.moduleFirmware", "firmware"],
    ["linux.moduleSigners", "signers"],
    ["linux.moduleSignatures", "signatures"],
    ["system.osFamily", "OS family"],
    ["system.hostname", "hostname"],
    ["system.manufacturer", "manufacturer"],
    ["system.model", "model"],
    ["system.modelIdentifier", "model id"],
    ["system.serialNumber", "serial"],
    ["system.uuid", "uuid"],
    ["system.hardwareUuid", "hardware uuid"],
    ["system.bootRomVersion", "Boot ROM"],
    ["system.smcVersion", "SMC"],
    ["system.cpuType", "CPU"],
    ["system.processorSpeed", "CPU speed"],
    ["system.cpuLogicalProcessorCount", "logical CPUs"],
    ["system.cpuModels", "CPU models"],
    ["system.cpuVendors", "CPU vendors"],
    ["system.cpuCoreCounts", "CPU cores"],
    ["system.cpuArchitectures", "CPU architectures"],
    ["system.cpuHardware", "CPU hardware"],
    ["system.cpuFeatures", "CPU features"],
    ["system.memoryTotalKiB", "memory KiB"],
    ["system.memoryTotalBytes", "memory bytes"],
    ["system.machineId", "machine id"],
    ["system.osName", "OS"],
    ["system.osVersion", "OS version"],
    ["system.osDisplayVersion", "OS display"],
    ["system.osBuild", "OS build"],
    ["system.osBuildNumber", "OS build number"],
    ["system.osUpdateBuildRevision", "UBR"],
    ["system.osEdition", "edition"],
    ["system.osInstallationType", "install type"],
    ["system.osInstallDateEpoch", "install date"],
    ["system.productId", "product id"],
    ["system.registeredOwner", "owner"],
    ["system.registeredOrganization", "organization"],
    ["system.oemManufacturer", "OEM"],
    ["system.oemModel", "OEM model"],
    ["system.oemSupportUrl", "OEM support"],
    ["system.osPrettyName", "OS"],
    ["system.osId", "OS id"],
    ["system.prettyHostname", "pretty hostname"],
    ["system.localHostname", "local hostname"],
    ["system.locale", "locale"],
    ["system.language", "language"],
    ["system.localeTime", "time locale"],
    ["system.localeNumeric", "numeric locale"],
    ["system.timeZone", "time zone"],
    ["system.timeZoneFormat", "time zone format"],
    ["system.timeZoneFileVersion", "time zone file"],
    ["system.timeZoneRule", "time zone rule"],
    ["system.rootDevice", "root device"],
    ["system.mountCount", "mounts"],
    ["system.mounts", "mount table"],
    ["system.iconName", "icon"],
    ["system.chassis", "chassis"],
    ["system.deployment", "deployment"],
    ["system.location", "location"],
    ["system.sku", "SKU"],
    ["system.family", "family"],
    ["system.hardwareId", "hardware id"],
    ["system.boardVendor", "board vendor"],
    ["system.boardName", "board"],
    ["system.boardSerial", "board serial"],
    ["system.baseboardManufacturer", "baseboard vendor"],
    ["system.baseboardProduct", "baseboard"],
    ["system.baseboardVersion", "baseboard version"],
    ["system.biosVendor", "BIOS vendor"],
    ["system.biosVersion", "BIOS"],
    ["system.biosReleaseDate", "BIOS date"],
    ["system.systemBiosVersion", "system BIOS"],
    ["system.videoBiosVersion", "video BIOS"],
    ["system.chassisSerial", "chassis serial"],
    ["system.activeComputerName", "active computer"],
    ["system.computerName", "computer"],
    ["system.profileCount", "profiles"],
    ["system.profileNames", "profile names"],
    ["system.networkProfileCount", "networks"],
    ["system.networkProfiles", "network names"],
    ["system.networkConfigType", "network config"],
    ["system.networkInterfaceCount", "interfaces"],
    ["system.networkInterfaces", "interfaces"],
    ["system.ipv4Addresses", "IP addresses"],
    ["system.gateways", "gateways"],
    ["system.dnsServers", "DNS"],
    ["system.dnsSearchDomains", "DNS search"],
    ["system.hostAliases", "host aliases"],
    ["system.networkMethods", "network methods"],
    ["system.connectionIds", "connections"],
    ["system.connectionUuids", "connection UUIDs"],
    ["system.primaryMacAddress", "primary MAC"],
    ["system.macAddresses", "MAC addresses"],
    ["system.wifiKnownNetworkCount", "known Wi-Fi"],
    ["system.wifiSsids", "Wi-Fi SSIDs"],
    ["system.wifiAuthTypes", "Wi-Fi auth"],
    ["system.wifiSecurityTypes", "Wi-Fi security"],
    ["system.wifiEncryptionTypes", "Wi-Fi encryption"],
    ["system.wifiAutoJoinSsids", "Wi-Fi auto-join"],
    ["system.wifiLastConnected", "Wi-Fi last connected"],
    ["system.networkConnectionModes", "connection modes"],
    ["system.accountConfigType", "account config"],
    ["system.localUserCount", "local users"],
    ["system.regularUserCount", "regular users"],
    ["system.loginUserCount", "login users"],
    ["system.localGroupCount", "local groups"],
    ["system.shadowEntryCount", "shadow entries"],
    ["system.passwordHashUserCount", "password hashes"],
    ["system.passwordLockedUserCount", "locked passwords"],
    ["system.passwordDisabledUserCount", "disabled passwords"],
    ["system.passwordEmptyUserCount", "empty passwords"],
    ["system.rootAccountPresent", "root account"],
    ["system.userUidRange", "UID range"],
    ["system.localUsers", "users"],
    ["system.regularUsers", "regular user names"],
    ["system.loginUsers", "login accounts"],
    ["system.passwordHashUsers", "hash users"],
    ["system.passwordLockedUsers", "locked users"],
    ["system.passwordDisabledUsers", "disabled users"],
    ["system.passwordEmptyUsers", "empty-password users"],
    ["system.passwordHashAlgorithms", "hash algorithms"],
    ["system.homeDirectories", "home directories"],
    ["system.loginShells", "login shells"],
    ["system.localGroups", "groups"],
    ["system.adminGroups", "admin groups"],
    ["system.groupMembers", "group members"],
    ["system.driveLetters", "drives"],
    ["system.volumeGuids", "volumes"],
    ["system.driverServiceCount", "driver services"],
    ["system.driverServices", "driver names"],
    ["system.driverServiceNames", "driver names"],
    ["system.driverImagePaths", "driver paths"],
    ["system.driverServiceDetails", "driver details"],
    ["system.firewallConfigType", "firewall"],
    ["system.firewallRuleCount", "firewall rules"],
    ["system.firewallTables", "firewall tables"],
    ["system.firewallChains", "firewall chains"],
    ["system.firewallPolicies", "firewall policies"],
    ["system.firewallLogEntryCount", "firewall log entries"],
    ["system.firewallAllowedCount", "firewall allowed"],
    ["system.firewallDroppedCount", "firewall dropped"],
    ["system.firewallProtocols", "firewall protocols"],
    ["system.firewallGlobalState", "firewall state"],
    ["system.firewallStealthEnabled", "stealth mode"],
    ["system.firewallAllowSignedEnabled", "signed apps"],
    ["system.firewallLoggingEnabled", "firewall logging"],
    ["system.firewallApplicationRuleCount", "app firewall rules"],
    ["system.setupLogType", "setup log"],
    ["system.setupLogLineCount", "setup lines"],
    ["system.setupDeviceInstallCount", "device installs"],
    ["system.setupComputerNames", "setup computers"],
    ["system.setupHostOsVersions", "host OS"],
    ["system.setupBuildVersions", "setup build"],
    ["system.setupManufacturers", "setup manufacturers"],
    ["system.setupModels", "setup models"],
    ["system.setupBiosVersions", "setup BIOS"],
    ["system.setupArchitectures", "setup arch"],
    ["system.setupDeviceHardwareIds", "hardware ids"],
    ["system.setupDeviceDescriptions", "devices"],
    ["system.setupDriverProviders", "driver providers"],
    ["system.setupDriverVersions", "driver versions"],
    ["system.setupInfNames", "INF names"],
    ["system.installHistoryCount", "installs"],
    ["system.latestInstallName", "latest install"],
    ["system.latestInstallVersion", "install version"],
    ["system.latestInstallDate", "install date"],
    ["system.latestInstallPackages", "install packages"],
    ["activity.commandHistoryType", "history"],
    ["activity.commandCount", "commands"],
    ["activity.networkCommandCount", "network commands"],
    ["activity.privilegedCommandCount", "privileged commands"],
    ["activity.fileTransferCommandCount", "file transfers"],
    ["activity.commandNames", "command names"],
    ["os.release.name", "OS"],
    ["os.release.version", "version"],
    ["os.release.buildId", "build"],
    ["plist.format", "plist"],
    ["plist.rootType", "root"],
    ["plist.topLevelKeys", "keys"],
    ["plist.CFBundleIdentifier", "bundle id"],
    ["plist.CFBundleName", "bundle"],
    ["plist.Label", "label"],
    ["plist.Program", "program"],
    ["plist.ProgramArguments", "arguments"],
  ]
    .map(([key, label]) => {
      const value = metadata[key];
      return value ? `${label}: ${value}` : null;
    })
    .filter((value): value is string => value !== null);

  return pairs.length > 0 ? pairs.slice(0, 6).join("; ") : "-";
}

function buildSourceAnalysisAppendix(
  evidence: ProjectDbReportEvidence,
): string {
  const lines: string[] = [];

  if (evidence.sourceAnalysisCategorySummaries.length > 0) {
    lines.push("### Source Analysis Categories");
    lines.push("");
    lines.push(
      "| Category | Analyses | Evidence Files | Average Entropy | Text-like | Latest Analyzed |",
    );
    lines.push("| --- | ---: | ---: | ---: | ---: | --- |");
    for (const summary of evidence.sourceAnalysisCategorySummaries) {
      lines.push(
        `| ${summary.category} | ${summary.count} | ${summary.evidenceFileCount} | ${summary.avgEntropy.toFixed(3)} | ${summary.textLikeCount} | ${summary.latestAnalyzedAt || "-"} |`,
      );
    }
    lines.push("");
  }

  if (evidence.sourceAnalysisSummaries.length > 0) {
    lines.push("### Source Analysis Records");
    lines.push("");
    lines.push(
      "| Source | Category | Signature | Bytes Analyzed | Entropy | Printable Ratio | Text-like | Analyzer |",
    );
    lines.push("| --- | --- | --- | ---: | ---: | ---: | --- | --- |");
    for (const summary of evidence.sourceAnalysisSummaries.slice(0, 250)) {
      lines.push(
        `| ${summary.sourceId} | ${summary.primaryCategory} | ${summary.primarySignature || "-"} | ${summary.bytesAnalyzedDisplay} | ${summary.entropy.toFixed(3)} | ${(summary.printableRatio * 100).toFixed(1)}% | ${summary.isLikelyText ? "yes" : "no"} | ${summary.analyzer} |`,
      );
    }
    if (evidence.sourceAnalysisSummaries.length > 250) {
      lines.push(
        `| ... | ... | ... | ... | ... | ... | ... | ${evidence.sourceAnalysisSummaries.length - 250} additional analysis record(s) omitted from appendix preview |`,
      );
    }
    lines.push("");
  }

  const indicatorRows = evidence.sourceAnalysisSummaries
    .flatMap((summary) =>
      (summary.indicators ?? []).slice(0, 20).map((indicator) => ({
        sourceId: summary.sourceId,
        ...indicator,
      })),
    )
    .slice(0, 250);
  if (indicatorRows.length > 0) {
    lines.push("### Extracted Source Indicators");
    lines.push("");
    lines.push("| Source | Type | Value | Offset | Confidence |");
    lines.push("| --- | --- | --- | ---: | --- |");
    for (const indicator of indicatorRows) {
      lines.push(
        `| ${indicator.sourceId} | ${indicator.indicatorType} | ${indicator.value} | 0x${indicator.offset.toString(16).toUpperCase()} | ${indicator.confidence} |`,
      );
    }
    lines.push("");
  }

  return lines.join("\n").trim();
}

function buildAnnotationAppendix(evidence: ProjectDbReportEvidence): string {
  const lines: string[] = [];
  const annotations = evidence.annotations ?? [];

  if (annotations.length === 0) return "";

  lines.push("### Offset and Review Annotations");
  lines.push("");
  lines.push(
    "| Source | Type | Range | Label | Finding | Created By | Created At |",
  );
  lines.push("| --- | --- | --- | --- | --- | --- | --- |");
  for (const annotation of annotations.slice(0, 250)) {
    lines.push(
      `| ${annotation.filePath} | ${annotation.annotationType} | ${annotationRange(annotation)} | ${annotation.label} | ${annotation.content || "-"} | ${annotation.createdBy} | ${annotation.createdAt} |`,
    );
  }
  if (annotations.length > 250) {
    lines.push(
      `| ... | ... | ... | ... | ${annotations.length - 250} additional annotation(s) omitted from appendix preview | ... | ... |`,
    );
  }
  lines.push("");

  return lines.join("\n").trim();
}

function annotationRange(annotation: ProjectDbAnnotationRecord): string {
  if (typeof annotation.offsetStart === "number") {
    const start = `0x${annotation.offsetStart.toString(16).toUpperCase()}`;
    const end =
      typeof annotation.offsetEnd === "number"
        ? `0x${annotation.offsetEnd.toString(16).toUpperCase()}`
        : "";
    return end ? `${start}-${end}` : start;
  }

  if (typeof annotation.lineStart === "number") {
    const end =
      typeof annotation.lineEnd === "number" ? `-${annotation.lineEnd}` : "";
    return `line ${annotation.lineStart}${end}`;
  }

  return "-";
}

/**
 * Generate evidence items from current app state
 *
 * Convenience function that converts file data and calls the backend.
 */
export async function generateEvidenceFromFiles(
  files: DiscoveredFile[],
  fileInfoMap: Map<string, ContainerInfo>,
  fileHashMap: Map<
    string,
    { algorithm: string; hash: string; verified?: boolean | null }
  >,
): Promise<EvidenceItem[]> {
  const containers = containersToInputs(files, fileInfoMap, fileHashMap);
  return extractEvidenceFromContainers(containers);
}

/**
 * Get a report template for a specific investigation type
 */
export async function getReportTemplate(
  investigationType: string,
): Promise<unknown> {
  if (!isTauri) {
    void investigationType;
    return null;
  }

  return invoke("get_report_template", { investigationType });
}

/**
 * Check if AI assistant is available
 */
export async function isAiAvailable(): Promise<boolean> {
  if (!isTauri) {
    return false;
  }

  return invoke<boolean>("is_ai_available");
}

// =============================================================================
// AI Assistant Types
// =============================================================================

export interface AiProviderInfo {
  id: string;
  name: string;
  description: string;
  requires_api_key: boolean;
  default_model: string;
  available_models: string[];
}

export type NarrativeType =
  | "executive_summary"
  | "finding"
  | "timeline"
  | "evidence"
  | "methodology"
  | "conclusion";

// =============================================================================
// AI Assistant Commands
// =============================================================================

/**
 * Get available AI providers
 */
export async function getAiProviders(): Promise<AiProviderInfo[]> {
  if (!isTauri) {
    return [];
  }

  return invoke<AiProviderInfo[]>("get_ai_providers");
}

/**
 * Check if Ollama is running locally
 */
export async function checkOllamaConnection(): Promise<boolean> {
  if (!isTauri) {
    return false;
  }

  return invoke<boolean>("check_ollama_connection");
}

/**
 * Generate AI narrative for a report section
 */
export async function generateAiNarrative(
  context: string,
  narrativeType: NarrativeType,
  provider: string,
  model: string,
  apiKey?: string,
): Promise<string> {
  if (!isTauri) {
    void context;
    void narrativeType;
    void provider;
    void model;
    void apiKey;
    throw new Error("AI report narrative generation is available in the desktop app.");
  }

  return invoke<string>("generate_ai_narrative", {
    context,
    narrativeType,
    provider,
    model,
    apiKey: apiKey ?? null,
  });
}

/**
 * Build context string from evidence items for AI
 */
export function buildEvidenceContext(evidenceItems: EvidenceItem[]): string {
  const lines: string[] = [];

  lines.push("=== EVIDENCE ITEMS ===\n");

  for (const item of evidenceItems) {
    lines.push(`Evidence ID: ${item.evidence_id}`);
    lines.push(`Description: ${item.description}`);
    lines.push(`Type: ${item.evidence_type}`);

    if (item.model) lines.push(`Model: ${item.model}`);
    if (item.serial_number) lines.push(`Serial Number: ${item.serial_number}`);
    if (item.capacity) lines.push(`Capacity: ${item.capacity}`);

    if (item.image_info) {
      lines.push(`Image Format: ${item.image_info.format}`);
      lines.push(`Total Size: ${formatBytes(item.image_info.total_size)}`);
      if (item.image_info.acquisition_tool) {
        lines.push(`Acquisition Tool: ${item.image_info.acquisition_tool}`);
      }
    }

    if (item.acquisition_hashes.length > 0) {
      lines.push("Hash Values:");
      for (const hash of item.acquisition_hashes) {
        const verified = hash.verified ? " (verified)" : "";
        lines.push(`  ${hash.algorithm}: ${hash.value}${verified}`);
      }
    }

    if (item.notes) lines.push(`Notes: ${item.notes}`);

    lines.push(""); // Blank line between items
  }

  return lines.join("\n");
}

/**
 * Export report to JSON
 */
export async function exportReportJson(report: unknown): Promise<string> {
  if (!isTauri) {
    return JSON.stringify(report, null, 2);
  }

  return invoke<string>("export_report_json", { report });
}

/**
 * Import report from JSON
 */
export async function importReportJson(json: string): Promise<unknown> {
  if (!isTauri) {
    return JSON.parse(json);
  }

  return invoke("import_report_json", { json });
}
