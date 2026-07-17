// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, expect, it } from "vitest";
import type { DbNormalizedArtifact } from "../../api/commands";
import {
  artifactMatchesEvidence,
  buildSystemIdentityReportMarkdown,
  buildSystemIdentitySummary,
  formatSystemIdentitySummaryForClipboard,
} from "../systemIdentitySummary";

function artifact(overrides: Partial<DbNormalizedArtifact> = {}): DbNormalizedArtifact {
  return {
    id: "artifact-1",
    evidenceFileId: "/case/disk.E01",
    sourceId: "e01:/case/disk.E01:/Windows/System32/config/SOFTWARE",
    sourceRefJson: JSON.stringify({
      kind: "vfsEntry",
      containerPath: "/case/disk.E01",
      entryPath: "/Windows/System32/config/SOFTWARE",
      containerType: "e01",
    }),
    name: "SOFTWARE",
    extension: null,
    size: 65536,
    mimeType: null,
    typeDescription: "Windows Registry Hive",
    category: "systeminfo",
    confidence: "high",
    isText: false,
    contentPreview: null,
    metadataJson: JSON.stringify({
      "system.manufacturer": "Dell Inc.",
      "system.model": "Precision 5680",
      "system.serialNumber": "ABC1234",
      "system.baseboardVersion": "A01",
      "system.hardwareIds": "ABCDEF01-2345-6789-ABCD-EF0123456789",
      "system.activeComputerName": "DESKTOP-CASE01",
      "system.osName": "Windows 11 Pro",
      "system.osBuildNumber": "22631",
      "system.machineGuid": "6f2d5a21-24e0-47cc-b9b2-7dc8c763f9c3",
      "system.localUsers": "terry; admin",
      "system.securityHivePresent": "true",
      "system.securityAccountSidCount": "3",
      "system.lsaSecretCount": "2",
      "system.lsaSecretsPresent": "true",
      "system.profileNames": "terry; service",
      "system.driveLetters": "C:; D:",
      "system.volumeGuids": "Volume{11111111-2222-3333-4444-555555555555}",
      "system.macAddresses": "00:11:22:33:44:55",
      "system.networkProfiles": "LabNet (private)",
      "system.driverServices": "volmgr; disk",
      "system.setupComputerNames": "DESKTOP-CASE01",
      "system.firewallDroppedCount": "14",
      "system.identityStatus": "parsed",
    }),
    extractedAt: "2026-07-06T10:00:00Z",
    extractor: "test-system-identity",
    ...overrides,
  };
}

describe("system identity summary", () => {
  it("matches artifacts to the selected evidence by evidence id and source ref", () => {
    expect(artifactMatchesEvidence(artifact(), "/case/disk.E01")).toBe(true);
    expect(
      artifactMatchesEvidence(
        artifact({
          evidenceFileId: null,
          sourceRefJson: JSON.stringify({
            kind: "nestedContainerEntry",
            containerPath: "/case/disk.E01",
            nestedContainerPath: "/case/inner.ad1",
            entryPath: "/etc/passwd",
          }),
        }),
        "/case/disk.E01",
      ),
    ).toBe(true);
    expect(artifactMatchesEvidence(artifact(), "/case/other.E01")).toBe(false);
  });

  it("groups extracted device, OS, user, storage, and network facts", () => {
    const summary = buildSystemIdentitySummary([artifact()]);

    expect(summary.recordCount).toBe(1);
    expect(summary.sourceCount).toBe(1);
    expect(summary.groups.map((group) => group.title)).toEqual([
      "Device and BIOS",
      "Computer and OS",
      "Users and Groups",
      "Storage and Volumes",
      "Network",
      "Additional System Details",
      "Source Files",
    ]);
    expect(formatSystemIdentitySummaryForClipboard(summary)).toContain("Serial Number: ABC1234");
    expect(formatSystemIdentitySummaryForClipboard(summary)).toContain("Hardware IDs: ABCDEF01-2345-6789-ABCD-EF0123456789");
    expect(formatSystemIdentitySummaryForClipboard(summary)).toContain("Machine GUID: 6f2d5a21-24e0-47cc-b9b2-7dc8c763f9c3");
    expect(formatSystemIdentitySummaryForClipboard(summary)).toContain("Network Profiles: LabNet (private)");
    expect(formatSystemIdentitySummaryForClipboard(summary)).toContain("Security Hive Present: true");
    expect(formatSystemIdentitySummaryForClipboard(summary)).toContain("LSA Secret Count: 2");
    expect(formatSystemIdentitySummaryForClipboard(summary)).toContain("Setup Computer Names: DESKTOP-CASE01");
    expect(formatSystemIdentitySummaryForClipboard(summary)).toContain("Firewall Dropped Count: 14");
    expect(formatSystemIdentitySummaryForClipboard(summary)).not.toContain("Identity Status: parsed");
  });

  it("builds report markdown from raw artifact records", () => {
    const markdown = buildSystemIdentityReportMarkdown([artifact()]);

    expect(markdown).toContain("System identity extraction found 1 artifact");
    expect(markdown).toContain("### Device and BIOS");
    expect(markdown).toContain("| Serial Number | ABC1234 | 1 |");
    expect(markdown).toContain("| Baseboard Version | A01 | 1 |");
    expect(markdown).toContain("| OS Build Number | 22631 | 1 |");
    expect(markdown).toContain("| Volume GUIDs | Volume{11111111-2222-3333-4444-555555555555} | 1 |");
    expect(markdown).toContain("| Driver Services | volmgr; disk | 1 |");
    expect(markdown).toContain("| Security Account SID Count | 3 | 1 |");
    expect(markdown).toContain("| LSA Secrets Present | true | 1 |");
    expect(markdown).toContain("### Additional System Details");
    expect(markdown).toContain("| Setup Computer Names | DESKTOP-CASE01 | 1 |");
    expect(markdown).toContain("### Users and Groups");
  });
});
