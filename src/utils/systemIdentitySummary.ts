// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { DbNormalizedArtifact, EvidenceSourceRef } from "../api/commands";

export interface SystemIdentityField {
  key: string;
  label: string;
  value: string;
  sourceCount: number;
}

export interface SystemIdentityGroup {
  id: string;
  title: string;
  fields: SystemIdentityField[];
}

export interface SystemIdentitySummary {
  recordCount: number;
  sourceCount: number;
  groups: SystemIdentityGroup[];
}

interface FieldDefinition {
  key: string;
  label: string;
  groupId: string;
}

interface GroupDefinition {
  id: string;
  title: string;
}

const GROUPS: GroupDefinition[] = [
  { id: "device", title: "Device and BIOS" },
  { id: "os", title: "Computer and OS" },
  { id: "users", title: "Users and Groups" },
  { id: "storage", title: "Storage and Volumes" },
  { id: "network", title: "Network" },
  { id: "sources", title: "Source Files" },
];

const FIELD_DEFINITIONS: FieldDefinition[] = [
  { groupId: "device", key: "system.manufacturer", label: "Manufacturer" },
  { groupId: "device", key: "system.oemManufacturer", label: "OEM Manufacturer" },
  { groupId: "device", key: "system.model", label: "Model" },
  { groupId: "device", key: "system.oemModel", label: "OEM Model" },
  { groupId: "device", key: "system.modelIdentifier", label: "Model Identifier" },
  { groupId: "device", key: "system.serialNumber", label: "Serial Number" },
  { groupId: "device", key: "system.hardwareUuid", label: "Hardware UUID" },
  { groupId: "device", key: "system.uuid", label: "System UUID" },
  { groupId: "device", key: "system.sku", label: "SKU" },
  { groupId: "device", key: "system.family", label: "Family" },
  { groupId: "device", key: "system.cpuType", label: "CPU" },
  { groupId: "device", key: "system.boardVendor", label: "Board Vendor" },
  { groupId: "device", key: "system.boardName", label: "Board Name" },
  { groupId: "device", key: "system.boardSerial", label: "Board Serial" },
  { groupId: "device", key: "system.baseboardManufacturer", label: "Baseboard Manufacturer" },
  { groupId: "device", key: "system.baseboardProduct", label: "Baseboard Product" },
  { groupId: "device", key: "system.baseboardSerial", label: "Baseboard Serial" },
  { groupId: "device", key: "system.biosVendor", label: "BIOS Vendor" },
  { groupId: "device", key: "system.biosVersion", label: "BIOS Version" },
  { groupId: "device", key: "system.biosReleaseDate", label: "BIOS Date" },
  { groupId: "device", key: "system.systemBiosVersion", label: "System BIOS" },
  { groupId: "device", key: "system.videoBiosVersion", label: "Video BIOS" },
  { groupId: "device", key: "system.bootRomVersion", label: "Boot ROM" },
  { groupId: "device", key: "system.smcVersion", label: "SMC Version" },

  { groupId: "os", key: "system.hostname", label: "Hostname" },
  { groupId: "os", key: "system.computerName", label: "Computer Name" },
  { groupId: "os", key: "system.activeComputerName", label: "Active Computer Name" },
  { groupId: "os", key: "system.localHostname", label: "Local Hostname" },
  { groupId: "os", key: "system.prettyHostname", label: "Pretty Hostname" },
  { groupId: "os", key: "system.osFamily", label: "OS Family" },
  { groupId: "os", key: "system.osName", label: "OS Name" },
  { groupId: "os", key: "system.osPrettyName", label: "OS Pretty Name" },
  { groupId: "os", key: "system.osVersion", label: "OS Version" },
  { groupId: "os", key: "system.osDisplayVersion", label: "Display Version" },
  { groupId: "os", key: "system.osBuild", label: "OS Build" },
  { groupId: "os", key: "system.osEdition", label: "OS Edition" },
  { groupId: "os", key: "system.productId", label: "Product ID" },
  { groupId: "os", key: "system.registeredOwner", label: "Registered Owner" },
  { groupId: "os", key: "system.machineId", label: "Machine ID" },

  { groupId: "users", key: "system.localUsers", label: "Local Users" },
  { groupId: "users", key: "system.regularUsers", label: "Regular Users" },
  { groupId: "users", key: "system.loginUsers", label: "Login Users" },
  { groupId: "users", key: "system.adminUsers", label: "Admin Users" },
  { groupId: "users", key: "system.localGroups", label: "Local Groups" },
  { groupId: "users", key: "system.adminGroups", label: "Admin Groups" },
  { groupId: "users", key: "system.userCount", label: "User Count" },
  { groupId: "users", key: "system.groupCount", label: "Group Count" },
  { groupId: "users", key: "system.homeDirectories", label: "Home Directories" },
  { groupId: "users", key: "system.loginShells", label: "Login Shells" },

  { groupId: "storage", key: "system.driveLetters", label: "Drive Letters" },
  { groupId: "storage", key: "system.volumeNames", label: "Volume Names" },
  { groupId: "storage", key: "system.volumeUuids", label: "Volume UUIDs" },
  { groupId: "storage", key: "system.diskIdentifiers", label: "Disk Identifiers" },
  { groupId: "storage", key: "system.volumeFilesystems", label: "Filesystems" },
  { groupId: "storage", key: "system.volumeMounts", label: "Mounts" },
  { groupId: "storage", key: "system.rootDevice", label: "Root Device" },
  { groupId: "storage", key: "system.mounts", label: "Mount Points" },
  { groupId: "storage", key: "system.volumes", label: "Volumes" },
  { groupId: "storage", key: "system.totalVolumeBytes", label: "Total Volume Bytes" },

  { groupId: "network", key: "system.interfaces", label: "Interfaces" },
  { groupId: "network", key: "system.networkInterfaces", label: "Network Interfaces" },
  { groupId: "network", key: "system.macAddresses", label: "MAC Addresses" },
  { groupId: "network", key: "system.ipv4Addresses", label: "IPv4 Addresses" },
  { groupId: "network", key: "system.gateways", label: "Gateways" },
  { groupId: "network", key: "system.dnsServers", label: "DNS Servers" },
  { groupId: "network", key: "system.searchDomains", label: "Search Domains" },
  { groupId: "network", key: "system.wifiSsids", label: "Wi-Fi SSIDs" },
  { groupId: "network", key: "system.firewallState", label: "Firewall State" },
];

const FIELD_BY_KEY = new Map(FIELD_DEFINITIONS.map((field) => [field.key, field]));

export function parseArtifactMetadata(record: DbNormalizedArtifact): Record<string, string> {
  if (!record.metadataJson) return {};
  try {
    const parsed = JSON.parse(record.metadataJson) as unknown;
    if (!parsed || typeof parsed !== "object" || Array.isArray(parsed)) return {};
    return Object.fromEntries(
      Object.entries(parsed as Record<string, unknown>)
        .filter(([, value]) => value !== null && value !== undefined)
        .map(([key, value]) => [key, String(value)]),
    );
  } catch {
    return {};
  }
}

export function artifactMatchesEvidence(
  record: DbNormalizedArtifact,
  evidencePath: string,
): boolean {
  if (!evidencePath) return false;
  if (record.evidenceFileId === evidencePath) return true;
  if (record.sourceId.includes(evidencePath)) return true;

  const sourceRef = parseSourceRef(record.sourceRefJson);
  if (!sourceRef) {
    return record.sourceRefJson.includes(evidencePath);
  }

  if (sourceRef.kind === "localFile") return sourceRef.path === evidencePath;
  if (sourceRef.kind === "nestedContainerEntry") {
    return sourceRef.containerPath === evidencePath || sourceRef.nestedContainerPath === evidencePath;
  }
  return sourceRef.containerPath === evidencePath;
}

export function buildSystemIdentitySummary(
  records: DbNormalizedArtifact[],
): SystemIdentitySummary {
  const identityRecords = records.filter((record) => record.category === "systeminfo");
  const groupedValues = new Map<string, Map<string, Set<string>>>();
  const sourceIds = new Set<string>();

  for (const record of identityRecords) {
    sourceIds.add(record.sourceId);
    const metadata = parseArtifactMetadata(record);
    for (const [key, value] of Object.entries(metadata)) {
      const field = FIELD_BY_KEY.get(key);
      if (!field || !value.trim()) continue;
      const valuesBySource = groupedValues.get(key) ?? new Map<string, Set<string>>();
      const values = valuesBySource.get(value) ?? new Set<string>();
      values.add(record.sourceId);
      valuesBySource.set(value, values);
      groupedValues.set(key, valuesBySource);
    }
  }

  const groups: SystemIdentityGroup[] = [];
  for (const group of GROUPS) {
    const fields: SystemIdentityField[] = FIELD_DEFINITIONS
      .filter((field) => field.groupId === group.id)
      .map((field) => {
        const valuesBySource = groupedValues.get(field.key);
        if (!valuesBySource) return null;
        const sourceCount = new Set([...valuesBySource.values()].flatMap((sources) => [...sources])).size;
        return {
          key: field.key,
          label: field.label,
          value: [...valuesBySource.keys()].join("; "),
          sourceCount,
        };
      })
      .filter((field): field is SystemIdentityField => field !== null);

    if (group.id === "sources") {
      const sourceFields = buildSourceFields(identityRecords);
      fields.push(...sourceFields);
    }

    if (fields.length > 0) {
      groups.push({ ...group, fields });
    }
  }

  return {
    recordCount: identityRecords.length,
    sourceCount: sourceIds.size,
    groups,
  };
}

export function formatSystemIdentitySummaryForClipboard(summary: SystemIdentitySummary): string {
  const lines = [
    "System Identity Summary",
    `Records: ${summary.recordCount}`,
    `Sources: ${summary.sourceCount}`,
  ];

  for (const group of summary.groups) {
    lines.push("", group.title);
    for (const field of group.fields) {
      lines.push(`${field.label}: ${field.value}`);
    }
  }

  return lines.join("\n");
}

export function buildSystemIdentityReportMarkdown(records: DbNormalizedArtifact[]): string {
  const summary = buildSystemIdentitySummary(records);
  if (summary.recordCount === 0 || summary.groups.length === 0) return "";

  const lines: string[] = [
    `System identity extraction found ${summary.recordCount} artifact(s) from ${summary.sourceCount} source file(s).`,
    "",
  ];

  for (const group of summary.groups) {
    lines.push(`### ${group.title}`, "");
    lines.push("| Field | Value | Sources |");
    lines.push("| --- | --- | ---: |");
    for (const field of group.fields) {
      lines.push(`| ${field.label} | ${escapeMarkdownTable(field.value)} | ${field.sourceCount} |`);
    }
    lines.push("");
  }

  return lines.join("\n").trim();
}

function buildSourceFields(records: DbNormalizedArtifact[]): SystemIdentityField[] {
  const names = new Map<string, Set<string>>();
  for (const record of records) {
    const label = record.name || sourceTail(record.sourceId);
    const sources = names.get(label) ?? new Set<string>();
    sources.add(record.sourceId);
    names.set(label, sources);
  }
  if (names.size === 0) return [];
  return [
    {
      key: "sources.files",
      label: "Files Parsed",
      value: [...names.keys()].sort((a, b) => a.localeCompare(b)).join("; "),
      sourceCount: new Set([...names.values()].flatMap((sources) => [...sources])).size,
    },
  ];
}

function parseSourceRef(sourceRefJson: string): EvidenceSourceRef | null {
  try {
    const parsed = JSON.parse(sourceRefJson) as EvidenceSourceRef;
    if (!parsed || typeof parsed !== "object" || !("kind" in parsed)) return null;
    return parsed;
  } catch {
    return null;
  }
}

function sourceTail(sourceId: string): string {
  const normalized = sourceId.replace(/\\/g, "/");
  return normalized.split("/").filter(Boolean).pop() ?? sourceId;
}

function escapeMarkdownTable(value: string): string {
  return value.replace(/\|/g, "\\|").replace(/\n/g, "<br>");
}
