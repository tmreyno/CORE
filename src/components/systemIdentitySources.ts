// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import type { HashSourceInput } from "../api/commands";
import { commands } from "../api/commands";
import type { TreeEntry, VfsEntry } from "../types";
import type { SelectedEntry } from "./EvidenceTree/types";
import { buildEvidenceSourceInput } from "./evidenceSourceInput";

export function buildSystemIdentitySourceInput(entry: SelectedEntry): HashSourceInput | null {
  if (!isLikelySystemIdentityEntry(entry)) return null;
  return buildEvidenceSourceInput(null, entry);
}

export async function collectSystemIdentityEntries(
  containerPath: string,
  entries: Array<TreeEntry | VfsEntry>,
  containerType: string,
  extractor = "evidence-tree-system-identity",
): Promise<void> {
  const sources = entries
    .filter((entry) => !isDirectoryEntry(entry))
    .map((entry) => buildTreeSystemIdentitySourceInput(containerPath, entry, containerType))
    .filter((source): source is HashSourceInput => source !== null);

  if (sources.length === 0) return;
  if (!(await commands.projectDb.isOpen().catch(() => false))) return;

  await commands.artifact.collectSystemIdentitySources({
    sources,
    extractor,
  });
}

export async function collectBinaryArtifactEntries(
  containerPath: string,
  entries: Array<TreeEntry | VfsEntry>,
  containerType: string,
  extractor = "evidence-tree-binary-artifact",
): Promise<void> {
  const sources = entries
    .map((entry) => buildTreeBinaryArtifactSourceInput(containerPath, entry, containerType))
    .filter((source): source is HashSourceInput => source !== null);

  if (sources.length === 0) return;
  if (!(await commands.projectDb.isOpen().catch(() => false))) return;

  await commands.artifact.collectBinaryArtifactSources({
    sources,
    extractor,
  });
}

export function buildTreeSystemIdentitySourceInput(
  containerPath: string,
  entry: TreeEntry | VfsEntry,
  containerType: string,
): HashSourceInput | null {
  if (isDirectoryEntry(entry)) return null;

  const normalizedEntry = {
    name: entry.name,
    entryPath: entry.path,
  };
  if (!isLikelySystemIdentityEntry(normalizedEntry)) return null;

  return {
    containerPath,
    entryPath: normalizedEntry.entryPath,
    containerType,
    size: entry.size,
    dataAddr: "data_addr" in entry ? entry.data_addr : undefined,
    itemAddr: "item_addr" in entry ? entry.item_addr : undefined,
  };
}

export function buildTreeBinaryArtifactSourceInput(
  containerPath: string,
  entry: TreeEntry | VfsEntry,
  containerType: string,
): HashSourceInput | null {
  if (isDirectoryEntry(entry)) return null;
  if (!isLikelyBinaryArtifactEntry({ name: entry.name, entryPath: entry.path })) return null;

  return {
    containerPath,
    entryPath: entry.path,
    containerType,
    size: entry.size,
    dataAddr: "data_addr" in entry ? entry.data_addr : undefined,
    itemAddr: "item_addr" in entry ? entry.item_addr : undefined,
  };
}

export function isLikelySystemIdentityEntry(entry: Pick<SelectedEntry, "entryPath" | "name">): boolean {
  const path = entry.entryPath.replace(/\\/g, "/").toLowerCase();
  const name = entry.name.toLowerCase();

  if (
    path.endsWith("/windows/system32/config/system") ||
    path.endsWith("/windows/system32/config/software") ||
    path.endsWith("/windows/system32/config/sam") ||
    path.endsWith("/config/system") ||
    path.endsWith("/config/software") ||
    path.endsWith("/config/sam")
  ) {
    return true;
  }

  if (
    path.endsWith("/system/library/coreservices/systemversion.plist") ||
    path.endsWith("/library/preferences/systemconfiguration/preferences.plist") ||
    path.endsWith("/library/preferences/systemconfiguration/networkinterfaces.plist") ||
    path.endsWith("/library/preferences/systemconfiguration/com.apple.airport.preferences.plist") ||
    path.endsWith("/library/preferences/com.apple.wifi.known-networks.plist") ||
    path.endsWith("/library/preferences/com.apple.alf.plist") ||
    path.endsWith("/library/preferences/.globalpreferences.plist") ||
    path.endsWith("/library/receipts/installhistory.plist") ||
    path.endsWith("/var/db/diskmanagement.plist") ||
    path.endsWith("/ioplatformexpertdevice.plist") ||
    path.endsWith("/ioregistry.plist") ||
    path.endsWith("/sphardwaredatatype.plist") ||
    path.endsWith("/system_profiler.spx") ||
    path.includes("/private/var/db/dslocal/nodes/default/users/") ||
    path.includes("/private/var/db/dslocal/nodes/default/groups/")
  ) {
    return true;
  }

  if (
    path.endsWith("/etc/network/interfaces") ||
    path.endsWith("/etc/resolv.conf") ||
    path.endsWith("/private/etc/resolv.conf") ||
    path.endsWith("/etc/hosts") ||
    path.endsWith("/private/etc/hosts") ||
    path.endsWith("/windows/system32/drivers/etc/hosts") ||
    path.endsWith("/etc/passwd") ||
    path.endsWith("/private/etc/passwd") ||
    path.endsWith("/etc/group") ||
    path.endsWith("/private/etc/group") ||
    path.endsWith("/etc/shadow") ||
    path.endsWith("/private/etc/shadow") ||
    path.endsWith("/etc/gshadow") ||
    path.endsWith("/private/etc/gshadow") ||
    path.includes("/etc/networkmanager/system-connections/") ||
    path.includes("/var/lib/networkmanager/system-connections/") ||
    path.includes("/etc/sysconfig/network-scripts/ifcfg-") ||
    (path.includes("/etc/systemd/network/") &&
      (path.endsWith(".network") || path.endsWith(".netdev") || path.endsWith(".link"))) ||
    path.includes("/etc/netplan/") ||
    path.includes("/sys/class/dmi/id/") ||
    path.includes("/sys/devices/virtual/dmi/id/") ||
    path.endsWith("/var/lib/dbus/machine-id") ||
    path.endsWith("/etc/machine-info") ||
    path.endsWith("/etc/default/locale")
  ) {
    return true;
  }

  if (
    path.includes("/programdata/microsoft/wlansvc/profiles/interfaces/") ||
    path.endsWith("/consolehost_history.txt") ||
    path.endsWith("/.bash_history") ||
    path.endsWith("/.zsh_history") ||
    path.endsWith("/etc/sysconfig/iptables") ||
    path.includes("/etc/iptables/") ||
    path.endsWith("/windows/system32/logfiles/firewall/pfirewall.log") ||
    path.endsWith("/windows/panther/setupact.log") ||
    path.endsWith("/windows/panther/setuperr.log") ||
    path.endsWith("/windows/inf/setupapi.dev.log") ||
    path.endsWith("/windows/inf/setupapi.app.log")
  ) {
    return true;
  }

  return SYSTEM_IDENTITY_FILE_NAMES.has(name);
}

export function isLikelyBinaryArtifactEntry(entry: Pick<SelectedEntry, "entryPath" | "name">): boolean {
  const path = entry.entryPath.replace(/\\/g, "/").toLowerCase();
  const name = entry.name.toLowerCase();

  if (name.endsWith(".sys") || name.endsWith(".drv")) {
    return (
      path.includes("/windows/system32/drivers/") ||
      path.includes("/winnt/system32/drivers/") ||
      path.includes("/system32/drivers/") ||
      path.includes("/drivers/")
    );
  }

  return (
    name.endsWith(".ko") &&
    (path.includes("/lib/modules/") ||
      path.includes("/usr/lib/modules/") ||
      path.includes("/kernel/drivers/"))
  );
}

const SYSTEM_IDENTITY_FILE_NAMES = new Set([
  "os-release",
  "lsb-release",
  "redhat-release",
  "debian_version",
  "machine-id",
  "hostname",
  "cpuinfo",
  "meminfo",
  "timezone",
  "localtime",
  "locale",
  "fstab",
  "mtab",
  "product_uuid",
  "product_serial",
  "product_name",
  "product_version",
  "product_family",
  "product_sku",
  "sys_vendor",
  "board_asset_tag",
  "board_serial",
  "board_name",
  "board_vendor",
  "board_version",
  "bios_version",
  "bios_vendor",
  "bios_date",
  "bios_release",
  "chassis_asset_tag",
  "chassis_vendor",
  "chassis_type",
  "chassis_serial",
  "chassis_version",
  "dmidecode",
  "dmidecode.txt",
  "lshw",
  "lshw.txt",
  "lshw-short.txt",
]);

function isDirectoryEntry(entry: TreeEntry | VfsEntry): boolean {
  return "is_dir" in entry ? entry.is_dir : entry.isDir;
}
