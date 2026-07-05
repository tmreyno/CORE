import { describe, expect, it } from "vitest";
import type { SelectedEntry } from "../EvidenceTree/types";
import {
  buildSystemIdentitySourceInput,
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

describe("system identity source helpers", () => {
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
});
