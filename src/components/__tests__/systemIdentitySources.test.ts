import { beforeEach, describe, expect, it } from "vitest";
import { mockInvoke } from "../../__tests__/setup";
import type { TreeEntry, VfsEntry } from "../../types";
import type { SelectedEntry } from "../EvidenceTree/types";
import {
  buildTreeBinaryArtifactSourceInput,
  buildTreeSystemIdentitySourceInput,
  collectBinaryArtifactEntries,
  buildSystemIdentitySourceInput,
  collectSystemIdentityEntries,
  isLikelyBinaryArtifactEntry,
  isLikelySystemIdentityEntry,
} from "../systemIdentitySources";

function entry(overrides: Partial<SelectedEntry>): SelectedEntry {
  return {
    name: "file",
    containerPath: "/case/disk.E01",
    entryPath: "/file",
    size: 128,
    isDir: false,
    isVfsEntry: true,
    ...overrides,
  };
}

function treeEntry(overrides: Partial<TreeEntry>): TreeEntry {
  return {
    name: "file",
    path: "/file",
    is_dir: false,
    size: 128,
    item_type: 0,
    ...overrides,
  };
}

function vfsEntry(overrides: Partial<VfsEntry>): VfsEntry {
  return {
    name: "file",
    path: "/file",
    isDir: false,
    size: 128,
    fileType: "file",
    ...overrides,
  };
}

describe("system identity source helpers", () => {
  beforeEach(() => {
    mockInvoke.mockReset();
  });

  it("classifies Windows registry hives", () => {
    expect(
      isLikelySystemIdentityEntry(
        entry({
          name: "SYSTEM",
          entryPath: String.raw`\Windows\System32\config\SYSTEM`,
        }),
      ),
    ).toBe(true);
    expect(
      isLikelySystemIdentityEntry(
        entry({
          name: "SAM",
          entryPath: "/Windows/System32/config/SAM",
        }),
      ),
    ).toBe(true);
  });

  it("classifies Linux DMI, account, and network identity sources", () => {
    expect(
      isLikelySystemIdentityEntry(
        entry({
          name: "product_serial",
          entryPath: "/sys/class/dmi/id/product_serial",
        }),
      ),
    ).toBe(true);
    expect(
      isLikelySystemIdentityEntry(
        entry({
          name: "passwd",
          entryPath: "/etc/passwd",
        }),
      ),
    ).toBe(true);
    expect(
      isLikelySystemIdentityEntry(
        entry({
          name: "corp.nmconnection",
          entryPath: "/etc/NetworkManager/system-connections/corp.nmconnection",
        }),
      ),
    ).toBe(true);
    expect(
      isLikelySystemIdentityEntry(
        entry({
          name: "corp.nmconnection",
          entryPath: "/var/lib/NetworkManager/system-connections/corp.nmconnection",
        }),
      ),
    ).toBe(true);
    expect(
      isLikelySystemIdentityEntry(
        entry({
          name: "20-wired.network",
          entryPath: "/etc/systemd/network/20-wired.network",
        }),
      ),
    ).toBe(true);
  });

  it("classifies macOS system plist identity sources", () => {
    expect(
      isLikelySystemIdentityEntry(
        entry({
          name: "SystemVersion.plist",
          entryPath: "/System/Library/CoreServices/SystemVersion.plist",
        }),
      ),
    ).toBe(true);
    expect(
      isLikelySystemIdentityEntry(
        entry({
          name: "alice.plist",
          entryPath: "/private/var/db/dslocal/nodes/Default/users/alice.plist",
        }),
      ),
    ).toBe(true);
  });

  it("does not classify unrelated user documents", () => {
    expect(
      isLikelySystemIdentityEntry(
        entry({
          name: "notes.txt",
          entryPath: "/Users/test/Documents/notes.txt",
        }),
      ),
    ).toBe(false);
  });

  it("classifies Windows drivers and Linux kernel modules for binary artifact collection", () => {
    expect(
      isLikelyBinaryArtifactEntry(
        entry({
          name: "ndis.sys",
          entryPath: "/Windows/System32/drivers/ndis.sys",
        }),
      ),
    ).toBe(true);
    expect(
      isLikelyBinaryArtifactEntry(
        entry({
          name: "contoso.ko",
          entryPath: "/lib/modules/6.8.0/kernel/drivers/net/contoso.ko",
        }),
      ),
    ).toBe(true);
    expect(
      isLikelyBinaryArtifactEntry(
        entry({
          name: "pagefile.sys",
          entryPath: "/pagefile.sys",
        }),
      ),
    ).toBe(false);
    expect(
      isLikelyBinaryArtifactEntry(
        entry({
          name: "notes.sys",
          entryPath: "/Users/test/Documents/notes.sys",
        }),
      ),
    ).toBe(false);
  });

  it("builds source input only for identity entries", () => {
    const systemSource = buildSystemIdentitySourceInput(
      entry({
        name: "SOFTWARE",
        entryPath: "/Windows/System32/config/SOFTWARE",
        size: 4096,
      }),
    );

    expect(systemSource).toEqual({
      containerPath: "/case/disk.E01",
      entryPath: "/Windows/System32/config/SOFTWARE",
      containerType: "e01",
      size: 4096,
      dataAddr: undefined,
      itemAddr: undefined,
    });
    expect(
      buildSystemIdentitySourceInput(
        entry({
          name: "photo.jpg",
          entryPath: "/Users/test/Pictures/photo.jpg",
        }),
      ),
    ).toBeNull();
  });

  it("builds AD1 tree source input with item and data addresses", () => {
    expect(
      buildTreeSystemIdentitySourceInput(
        "/case/source.AD1",
        treeEntry({
          name: "SYSTEM",
          path: "/Windows/System32/config/SYSTEM",
          size: 8192,
          data_addr: 123,
          item_addr: 456,
        }),
        "ad1",
      ),
    ).toEqual({
      containerPath: "/case/source.AD1",
      entryPath: "/Windows/System32/config/SYSTEM",
      containerType: "ad1",
      size: 8192,
      dataAddr: 123,
      itemAddr: 456,
    });
  });

  it("builds VFS tree source input for mounted image entries", () => {
    expect(
      buildTreeSystemIdentitySourceInput(
        "/case/disk.E01",
        vfsEntry({
          name: "SystemVersion.plist",
          path: "/Partition1_APFS/System/Library/CoreServices/SystemVersion.plist",
          size: 2048,
        }),
        "e01",
      ),
    ).toEqual({
      containerPath: "/case/disk.E01",
      entryPath: "/Partition1_APFS/System/Library/CoreServices/SystemVersion.plist",
      containerType: "e01",
      size: 2048,
      dataAddr: undefined,
      itemAddr: undefined,
    });
  });

  it("does not build tree source input for directories or unrelated files", () => {
    expect(
      buildTreeSystemIdentitySourceInput(
        "/case/source.AD1",
        treeEntry({
          name: "SYSTEM",
          path: "/Windows/System32/config/SYSTEM",
          is_dir: true,
        }),
        "ad1",
      ),
    ).toBeNull();
    expect(
      buildTreeSystemIdentitySourceInput(
        "/case/source.AD1",
        treeEntry({
          name: "vacation.jpg",
          path: "/Users/test/Pictures/vacation.jpg",
        }),
        "ad1",
      ),
    ).toBeNull();
  });

  it("builds binary artifact tree source input for driver files", () => {
    expect(
      buildTreeBinaryArtifactSourceInput(
        "/case/source.AD1",
        treeEntry({
          name: "contosoflt.sys",
          path: "/Windows/System32/drivers/contosoflt.sys",
          size: 32768,
          data_addr: 321,
          item_addr: 654,
        }),
        "ad1",
      ),
    ).toEqual({
      containerPath: "/case/source.AD1",
      entryPath: "/Windows/System32/drivers/contosoflt.sys",
      containerType: "ad1",
      size: 32768,
      dataAddr: 321,
      itemAddr: 654,
    });

    expect(
      buildTreeBinaryArtifactSourceInput(
        "/case/source.AD1",
        treeEntry({
          name: "pagefile.sys",
          path: "/pagefile.sys",
        }),
        "ad1",
      ),
    ).toBeNull();
  });

  it("collects matching identity entries when a project database is open", async () => {
    mockInvoke.mockImplementation(async (command: string) => {
      if (command === "project_db_is_open") return true;
      if (command === "project_db_collect_system_identity_sources") {
        return { scanned: 1, matched: 1, inserted: 1, skipped: 0, errors: [] };
      }
      throw new Error(`Unexpected command: ${command}`);
    });

    await collectSystemIdentityEntries(
      "/case/disk.E01",
      [
        vfsEntry({
          name: "product_serial",
          path: "/Partition1_EXT4/sys/class/dmi/id/product_serial",
          size: 32,
        }),
      ],
      "e01",
      "test-extractor",
    );

    expect(mockInvoke).toHaveBeenCalledWith("project_db_collect_system_identity_sources", {
      request: {
        sources: [
          {
            containerPath: "/case/disk.E01",
            entryPath: "/Partition1_EXT4/sys/class/dmi/id/product_serial",
            containerType: "e01",
            size: 32,
            dataAddr: undefined,
            itemAddr: undefined,
          },
        ],
        extractor: "test-extractor",
      },
    });
  });

  it("does not collect when no matching entries exist", async () => {
    await collectSystemIdentityEntries(
      "/case/disk.E01",
      [vfsEntry({ name: "notes.txt", path: "/Users/test/Documents/notes.txt" })],
      "e01",
    );

    expect(mockInvoke).not.toHaveBeenCalled();
  });

  it("does not collect when the project database is closed", async () => {
    mockInvoke.mockImplementation(async (command: string) => {
      if (command === "project_db_is_open") return false;
      throw new Error(`Unexpected command: ${command}`);
    });

    await collectSystemIdentityEntries(
      "/case/disk.E01",
      [
        vfsEntry({
          name: "product_name",
          path: "/Partition1_EXT4/sys/class/dmi/id/product_name",
        }),
      ],
      "e01",
    );

    expect(mockInvoke).toHaveBeenCalledWith("project_db_is_open");
    expect(mockInvoke).not.toHaveBeenCalledWith(
      "project_db_collect_system_identity_sources",
      expect.anything(),
    );
  });

  it("collects matching binary artifact entries when a project database is open", async () => {
    mockInvoke.mockImplementation(async (command: string) => {
      if (command === "project_db_is_open") return true;
      if (command === "project_db_collect_binary_artifact_sources") {
        return { scanned: 1, matched: 1, inserted: 1, skipped: 0, errors: [] };
      }
      throw new Error(`Unexpected command: ${command}`);
    });

    await collectBinaryArtifactEntries(
      "/case/disk.E01",
      [
        vfsEntry({
          name: "contoso.ko",
          path: "/Partition1_EXT4/lib/modules/6.8.0/kernel/drivers/net/contoso.ko",
          size: 65536,
        }),
      ],
      "e01",
      "test-binary-extractor",
    );

    expect(mockInvoke).toHaveBeenCalledWith("project_db_collect_binary_artifact_sources", {
      request: {
        sources: [
          {
            containerPath: "/case/disk.E01",
            entryPath: "/Partition1_EXT4/lib/modules/6.8.0/kernel/drivers/net/contoso.ko",
            containerType: "e01",
            size: 65536,
            dataAddr: undefined,
            itemAddr: undefined,
          },
        ],
        extractor: "test-binary-extractor",
      },
    });
  });
});
