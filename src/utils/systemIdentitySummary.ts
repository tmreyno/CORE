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
  { groupId: "device", key: "system.productSku", label: "Product SKU" },
  { groupId: "device", key: "system.family", label: "Family" },
  { groupId: "device", key: "system.cpuType", label: "CPU" },
  { groupId: "device", key: "system.cpuModels", label: "CPU Models" },
  { groupId: "device", key: "system.cpuVendors", label: "CPU Vendors" },
  { groupId: "device", key: "system.cpuCoreCounts", label: "CPU Cores" },
  { groupId: "device", key: "system.cpuLogicalProcessorCount", label: "Logical CPUs" },
  { groupId: "device", key: "system.cpuArchitectures", label: "CPU Architectures" },
  { groupId: "device", key: "system.cpuHardware", label: "CPU Hardware" },
  { groupId: "device", key: "system.memoryTotalBytes", label: "Memory Bytes" },
  { groupId: "device", key: "system.boardVendor", label: "Board Vendor" },
  { groupId: "device", key: "system.boardName", label: "Board Name" },
  { groupId: "device", key: "system.boardSerial", label: "Board Serial" },
  { groupId: "device", key: "system.boardSerialNumber", label: "Board Serial Number" },
  { groupId: "device", key: "system.boardVersion", label: "Board Version" },
  { groupId: "device", key: "system.boardAssetTag", label: "Board Asset Tag" },
  { groupId: "device", key: "system.baseboardManufacturer", label: "Baseboard Manufacturer" },
  { groupId: "device", key: "system.baseboardProduct", label: "Baseboard Product" },
  { groupId: "device", key: "system.baseboardVersion", label: "Baseboard Version" },
  { groupId: "device", key: "system.baseboardSerial", label: "Baseboard Serial" },
  { groupId: "device", key: "system.biosVendor", label: "BIOS Vendor" },
  { groupId: "device", key: "system.biosVersion", label: "BIOS Version" },
  { groupId: "device", key: "system.biosReleaseDate", label: "BIOS Date" },
  { groupId: "device", key: "system.biosDate", label: "BIOS Date" },
  { groupId: "device", key: "system.biosRelease", label: "BIOS Release" },
  { groupId: "device", key: "system.systemBiosVersion", label: "System BIOS" },
  { groupId: "device", key: "system.videoBiosVersion", label: "Video BIOS" },
  { groupId: "device", key: "system.bootRomVersion", label: "Boot ROM" },
  { groupId: "device", key: "system.smcVersion", label: "SMC Version" },
  { groupId: "device", key: "system.hardwareId", label: "Hardware ID" },
  { groupId: "device", key: "system.hardwareIds", label: "Hardware IDs" },
  { groupId: "device", key: "system.chassis", label: "Chassis" },
  { groupId: "device", key: "system.chassisType", label: "Chassis Type" },
  { groupId: "device", key: "system.chassisVendor", label: "Chassis Vendor" },
  { groupId: "device", key: "system.chassisSerial", label: "Chassis Serial" },
  { groupId: "device", key: "system.chassisVersion", label: "Chassis Version" },
  { groupId: "device", key: "system.chassisAssetTag", label: "Chassis Asset Tag" },

  { groupId: "os", key: "system.hostname", label: "Hostname" },
  { groupId: "os", key: "system.computerName", label: "Computer Name" },
  { groupId: "os", key: "system.activeComputerName", label: "Active Computer Name" },
  { groupId: "os", key: "system.localHostname", label: "Local Hostname" },
  { groupId: "os", key: "system.prettyHostname", label: "Pretty Hostname" },
  { groupId: "os", key: "system.networkHostname", label: "Network Hostname" },
  { groupId: "os", key: "system.domain", label: "Domain" },
  { groupId: "os", key: "system.osFamily", label: "OS Family" },
  { groupId: "os", key: "system.osName", label: "OS Name" },
  { groupId: "os", key: "system.osPrettyName", label: "OS Pretty Name" },
  { groupId: "os", key: "system.osId", label: "OS ID" },
  { groupId: "os", key: "system.osVersion", label: "OS Version" },
  { groupId: "os", key: "system.osVersionDetail", label: "OS Version Detail" },
  { groupId: "os", key: "system.osDisplayVersion", label: "Display Version" },
  { groupId: "os", key: "system.osBuild", label: "OS Build" },
  { groupId: "os", key: "system.osBuildNumber", label: "OS Build Number" },
  { groupId: "os", key: "system.osUpdateBuildRevision", label: "OS Update Build Revision" },
  { groupId: "os", key: "system.osBuildLab", label: "OS Build Lab" },
  { groupId: "os", key: "system.osBuildLabExtended", label: "OS Build Lab Extended" },
  { groupId: "os", key: "system.osEdition", label: "OS Edition" },
  { groupId: "os", key: "system.osCompositionEdition", label: "OS Composition Edition" },
  { groupId: "os", key: "system.osInstallationType", label: "OS Installation Type" },
  { groupId: "os", key: "system.osInstallDateEpoch", label: "OS Install Date Epoch" },
  { groupId: "os", key: "system.osPath", label: "OS Path" },
  { groupId: "os", key: "system.systemRoot", label: "System Root" },
  { groupId: "os", key: "system.productId", label: "Product ID" },
  { groupId: "os", key: "system.productVersion", label: "Product Version" },
  { groupId: "os", key: "system.registeredOwner", label: "Registered Owner" },
  { groupId: "os", key: "system.registeredOrganization", label: "Registered Organization" },
  { groupId: "os", key: "system.machineId", label: "Machine ID" },
  { groupId: "os", key: "system.machineGuid", label: "Machine GUID" },
  { groupId: "os", key: "system.timeZone", label: "Time Zone" },
  { groupId: "os", key: "system.timeZoneStandardName", label: "Time Zone Standard Name" },
  { groupId: "os", key: "system.locale", label: "Locale" },
  { groupId: "os", key: "system.language", label: "Language" },
  { groupId: "os", key: "system.languages", label: "Languages" },
  { groupId: "os", key: "system.deployment", label: "Deployment" },
  { groupId: "os", key: "system.location", label: "Location" },

  { groupId: "users", key: "system.localUsers", label: "Local Users" },
  { groupId: "users", key: "system.localUserCount", label: "Local User Count" },
  { groupId: "users", key: "system.regularUsers", label: "Regular Users" },
  { groupId: "users", key: "system.loginUsers", label: "Login Users" },
  { groupId: "users", key: "system.adminUsers", label: "Admin Users" },
  { groupId: "users", key: "system.localGroups", label: "Local Groups" },
  { groupId: "users", key: "system.localGroupCount", label: "Local Group Count" },
  { groupId: "users", key: "system.adminGroups", label: "Admin Groups" },
  { groupId: "users", key: "system.groupMembers", label: "Group Members" },
  { groupId: "users", key: "system.accountConfigType", label: "Account Source" },
  { groupId: "users", key: "system.administratorAccountPresent", label: "Administrator Present" },
  { groupId: "users", key: "system.guestAccountPresent", label: "Guest Present" },
  { groupId: "users", key: "system.profileCount", label: "Profile Count" },
  { groupId: "users", key: "system.profileNames", label: "Profile Names" },
  { groupId: "users", key: "system.profileSids", label: "Profile SIDs" },
  { groupId: "users", key: "system.profilePaths", label: "Profile Paths" },
  { groupId: "users", key: "system.profiles", label: "Profiles" },
  { groupId: "users", key: "system.userCount", label: "User Count" },
  { groupId: "users", key: "system.groupCount", label: "Group Count" },
  { groupId: "users", key: "system.homeDirectories", label: "Home Directories" },
  { groupId: "users", key: "system.loginShells", label: "Login Shells" },

  { groupId: "storage", key: "system.driveLetters", label: "Drive Letters" },
  { groupId: "storage", key: "system.volumeNames", label: "Volume Names" },
  { groupId: "storage", key: "system.volumeUuids", label: "Volume UUIDs" },
  { groupId: "storage", key: "system.volumeGuids", label: "Volume GUIDs" },
  { groupId: "storage", key: "system.diskIdentifiers", label: "Disk Identifiers" },
  { groupId: "storage", key: "system.volumeFilesystems", label: "Filesystems" },
  { groupId: "storage", key: "system.volumeMounts", label: "Mounts" },
  { groupId: "storage", key: "system.rootDevice", label: "Root Device" },
  { groupId: "storage", key: "system.mounts", label: "Mount Points" },
  { groupId: "storage", key: "system.volumes", label: "Volumes" },
  { groupId: "storage", key: "system.volumeCount", label: "Volume Count" },
  { groupId: "storage", key: "system.mountedDeviceCount", label: "Mounted Device Count" },
  { groupId: "storage", key: "system.mountedDevices", label: "Mounted Devices" },
  { groupId: "storage", key: "system.totalVolumeBytes", label: "Total Volume Bytes" },

  { groupId: "network", key: "system.interfaces", label: "Interfaces" },
  { groupId: "network", key: "system.networkInterfaces", label: "Network Interfaces" },
  { groupId: "network", key: "system.networkInterfaceCount", label: "Network Interface Count" },
  { groupId: "network", key: "system.primaryMacAddress", label: "Primary MAC Address" },
  { groupId: "network", key: "system.macAddresses", label: "MAC Addresses" },
  { groupId: "network", key: "system.ipv4Addresses", label: "IPv4 Addresses" },
  { groupId: "network", key: "system.gateways", label: "Gateways" },
  { groupId: "network", key: "system.dnsServers", label: "DNS Servers" },
  { groupId: "network", key: "system.searchDomains", label: "Search Domains" },
  { groupId: "network", key: "system.dnsSearchDomains", label: "DNS Search Domains" },
  { groupId: "network", key: "system.networkDomains", label: "Network Domains" },
  { groupId: "network", key: "system.dhcpServers", label: "DHCP Servers" },
  { groupId: "network", key: "system.networkMethods", label: "Network Methods" },
  { groupId: "network", key: "system.networkProfileCount", label: "Network Profile Count" },
  { groupId: "network", key: "system.networkProfileNames", label: "Network Profile Names" },
  { groupId: "network", key: "system.networkProfileCategories", label: "Network Profile Categories" },
  { groupId: "network", key: "system.networkProfiles", label: "Network Profiles" },
  { groupId: "network", key: "system.wifiSsids", label: "Wi-Fi SSIDs" },
  { groupId: "network", key: "system.wifiKnownNetworkCount", label: "Known Wi-Fi Count" },
  { groupId: "network", key: "system.wifiSecurityTypes", label: "Wi-Fi Security" },
  { groupId: "network", key: "system.wifiAuthTypes", label: "Wi-Fi Auth" },
  { groupId: "network", key: "system.wifiEncryptionTypes", label: "Wi-Fi Encryption" },
  { groupId: "network", key: "system.wifiAutoJoinSsids", label: "Wi-Fi Auto Join" },
  { groupId: "network", key: "system.wifiLastConnected", label: "Wi-Fi Last Connected" },
  { groupId: "network", key: "system.firewallState", label: "Firewall State" },
  { groupId: "network", key: "system.firewallConfigType", label: "Firewall Config Type" },
  { groupId: "network", key: "system.firewallGlobalState", label: "Firewall Global State" },
  { groupId: "network", key: "system.firewallStealthEnabled", label: "Firewall Stealth" },
  { groupId: "network", key: "system.firewallRuleCount", label: "Firewall Rule Count" },
  { groupId: "network", key: "system.firewallTables", label: "Firewall Tables" },
  { groupId: "network", key: "system.firewallPolicies", label: "Firewall Policies" },
  { groupId: "network", key: "system.driverServiceCount", label: "Driver Service Count" },
  { groupId: "network", key: "system.driverServices", label: "Driver Services" },
  { groupId: "network", key: "system.driverImagePaths", label: "Driver Image Paths" },
  { groupId: "network", key: "system.driverGroups", label: "Driver Groups" },
  { groupId: "network", key: "system.driverStartTypes", label: "Driver Start Types" },
  { groupId: "network", key: "system.driverServiceDetails", label: "Driver Service Details" },
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
