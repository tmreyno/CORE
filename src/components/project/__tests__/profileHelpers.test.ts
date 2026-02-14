// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  getProfileTypeLabel,
  getProfileTypeColor,
  PROFILE_TYPE_OPTIONS,
} from "../profileHelpers";
import type { ProfileType } from "../../../hooks/useWorkspaceProfiles";

// =============================================================================
// getProfileTypeLabel
// =============================================================================
describe("getProfileTypeLabel", () => {
  const expectedLabels: Record<ProfileType, string> = {
    Investigation: "Investigation",
    Analysis: "Analysis",
    Review: "Review",
    Mobile: "Mobile",
    Computer: "Computer",
    Network: "Network",
    IncidentResponse: "Incident Response",
    Custom: "Custom",
  };

  it.each(Object.entries(expectedLabels))(
    "returns '%s' for profile type '%s'",
    (type, label) => {
      expect(getProfileTypeLabel(type as ProfileType)).toBe(label);
    }
  );

  it("returns the raw type string for unknown types", () => {
    // Cast to ProfileType to test the fallback branch
    expect(getProfileTypeLabel("UnknownType" as ProfileType)).toBe("UnknownType");
  });
});

// =============================================================================
// getProfileTypeColor
// =============================================================================
describe("getProfileTypeColor", () => {
  const expectedColors: Record<ProfileType, string> = {
    Investigation: "text-type-ad1",
    Analysis: "text-type-e01",
    Review: "text-warning",
    Mobile: "text-type-ufed",
    Computer: "text-info",
    Network: "text-accent",
    IncidentResponse: "text-error",
    Custom: "text-txt-secondary",
  };

  it.each(Object.entries(expectedColors))(
    "returns '%s' for profile type '%s'",
    (type, color) => {
      expect(getProfileTypeColor(type as ProfileType)).toBe(color);
    }
  );

  it("returns default color for unknown types", () => {
    expect(getProfileTypeColor("UnknownType" as ProfileType)).toBe("text-txt-secondary");
  });
});

// =============================================================================
// PROFILE_TYPE_OPTIONS
// =============================================================================
describe("PROFILE_TYPE_OPTIONS", () => {
  it("contains all 8 profile types", () => {
    expect(PROFILE_TYPE_OPTIONS).toHaveLength(8);
  });

  it("has all expected profile type values", () => {
    const values = PROFILE_TYPE_OPTIONS.map((o) => o.value);
    expect(values).toContain("Investigation");
    expect(values).toContain("Analysis");
    expect(values).toContain("Review");
    expect(values).toContain("Mobile");
    expect(values).toContain("Computer");
    expect(values).toContain("Network");
    expect(values).toContain("IncidentResponse");
    expect(values).toContain("Custom");
  });

  it("each option has a non-empty label", () => {
    for (const option of PROFILE_TYPE_OPTIONS) {
      expect(option.label).toBeTruthy();
      expect(option.label.length).toBeGreaterThan(0);
    }
  });

  it("uses human-readable labels for multi-word types", () => {
    const irOption = PROFILE_TYPE_OPTIONS.find((o) => o.value === "IncidentResponse");
    expect(irOption?.label).toBe("Incident Response");

    const mobileOption = PROFILE_TYPE_OPTIONS.find((o) => o.value === "Mobile");
    expect(mobileOption?.label).toBe("Mobile Forensics");

    const computerOption = PROFILE_TYPE_OPTIONS.find((o) => o.value === "Computer");
    expect(computerOption?.label).toBe("Computer Forensics");

    const networkOption = PROFILE_TYPE_OPTIONS.find((o) => o.value === "Network");
    expect(networkOption?.label).toBe("Network Forensics");
  });

  it("has unique values", () => {
    const values = PROFILE_TYPE_OPTIONS.map((o) => o.value);
    expect(new Set(values).size).toBe(values.length);
  });
});
