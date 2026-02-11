// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { RegistryViewer } from "./RegistryViewer";
import { mockInvoke } from "../__tests__/setup";

// Helper to render and return the container
function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

// Wait for async updates
const tick = (ms = 50) => new Promise(resolve => setTimeout(resolve, ms));

// ============================================================================
// Test Data
// ============================================================================

const mockHiveInfo = {
  path: "/test/NTUSER.DAT",
  rootKeyName: "CMI-CreateHive",
  rootKeyPath: "CMI-CreateHive",
  rootTimestamp: "2024-01-15 10:30:00 UTC",
  totalKeys: 1234,
  totalValues: 5678,
  rootSubkeyCount: 5,
  rootValueCount: 2,
};

const mockSubkeysResponse = {
  parentPath: "CMI-CreateHive",
  subkeys: [
    {
      name: "Software",
      path: "CMI-CreateHive\\Software",
      timestamp: "2024-01-14 09:00:00 UTC",
      subkeyCount: 10,
      valueCount: 0,
      hasSubkeys: true,
    },
    {
      name: "Environment",
      path: "CMI-CreateHive\\Environment",
      timestamp: "2024-01-13 08:00:00 UTC",
      subkeyCount: 0,
      valueCount: 3,
      hasSubkeys: false,
    },
  ],
};

const mockKeyInfo = {
  name: "Software",
  path: "CMI-CreateHive\\Software",
  prettyPath: "CMI-CreateHive\\Software",
  timestamp: "2024-01-14 09:00:00 UTC",
  subkeyCount: 10,
  valueCount: 2,
  values: [
    {
      name: "(Default)",
      dataType: "REG_SZ",
      data: "Default Value",
      size: 26,
    },
    {
      name: "Version",
      dataType: "REG_DWORD",
      data: "0x0000000a (10)",
      size: 4,
    },
  ],
  subkeys: [
    {
      name: "Microsoft",
      path: "CMI-CreateHive\\Software\\Microsoft",
      timestamp: "2024-01-12 07:00:00 UTC",
      subkeyCount: 5,
      valueCount: 0,
      hasSubkeys: true,
    },
  ],
};

// ============================================================================
// Tests
// ============================================================================

describe("RegistryViewer", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
  });

  it("renders loading state initially", () => {
    mockInvoke.mockReturnValue(new Promise(() => {}));
    const { container } = renderComponent(() => <RegistryViewer path="/test/NTUSER.DAT" />);
    expect(container.textContent).toContain("Loading registry hive");
  });

  it("renders error state on failure", async () => {
    mockInvoke.mockRejectedValue(new Error("Corrupt hive file"));
    const { container } = renderComponent(() => <RegistryViewer path="/test/NTUSER.DAT" />);
    await tick();
    expect(container.textContent).toContain("Failed to load registry hive");
    expect(container.textContent).toContain("Corrupt hive file");
  });

  it("renders hive info after loading", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "registry_get_info") return mockHiveInfo;
      if (cmd === "registry_get_subkeys") return mockSubkeysResponse;
      if (cmd === "registry_get_key_info") return mockKeyInfo;
      return null;
    });

    const { container } = renderComponent(() => <RegistryViewer path="/test/NTUSER.DAT" />);
    await tick();

    expect(container.textContent).toContain("CMI-CreateHive");
    expect(container.textContent).toContain("1,234 keys");
    expect(container.textContent).toContain("5,678 values");
  });

  it("renders subkeys in tree", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "registry_get_info") return mockHiveInfo;
      if (cmd === "registry_get_subkeys") return mockSubkeysResponse;
      if (cmd === "registry_get_key_info") return mockKeyInfo;
      return null;
    });

    const { container } = renderComponent(() => <RegistryViewer path="/test/NTUSER.DAT" />);
    await tick();

    expect(container.textContent).toContain("Software");
    expect(container.textContent).toContain("Environment");
  });

  it("renders values table for selected key", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "registry_get_info") return mockHiveInfo;
      if (cmd === "registry_get_subkeys") return mockSubkeysResponse;
      if (cmd === "registry_get_key_info") return mockKeyInfo;
      return null;
    });

    const { container } = renderComponent(() => <RegistryViewer path="/test/NTUSER.DAT" />);
    await tick();

    expect(container.textContent).toContain("(Default)");
    expect(container.textContent).toContain("REG_SZ");
    expect(container.textContent).toContain("Default Value");
    expect(container.textContent).toContain("Version");
    expect(container.textContent).toContain("REG_DWORD");
  });

  it("invokes correct commands on load", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "registry_get_info") return mockHiveInfo;
      if (cmd === "registry_get_subkeys") return mockSubkeysResponse;
      if (cmd === "registry_get_key_info") return mockKeyInfo;
      return null;
    });

    renderComponent(() => <RegistryViewer path="/test/NTUSER.DAT" />);
    await tick();

    expect(mockInvoke).toHaveBeenCalledWith("registry_get_info", { path: "/test/NTUSER.DAT" });
    expect(mockInvoke).toHaveBeenCalledWith("registry_get_subkeys", {
      hivePath: "/test/NTUSER.DAT",
      keyPath: "",
    });
  });

  it("shows Registry badge in header", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "registry_get_info") return mockHiveInfo;
      if (cmd === "registry_get_subkeys") return mockSubkeysResponse;
      if (cmd === "registry_get_key_info") return mockKeyInfo;
      return null;
    });

    const { container } = renderComponent(() => <RegistryViewer path="/test/NTUSER.DAT" />);
    await tick();

    expect(container.textContent).toContain("Registry");
  });

  it("shows value table headers", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "registry_get_info") return mockHiveInfo;
      if (cmd === "registry_get_subkeys") return mockSubkeysResponse;
      if (cmd === "registry_get_key_info") return mockKeyInfo;
      return null;
    });

    const { container } = renderComponent(() => <RegistryViewer path="/test/NTUSER.DAT" />);
    await tick();

    expect(container.textContent).toContain("Name");
    expect(container.textContent).toContain("Type");
    expect(container.textContent).toContain("Data");
    expect(container.textContent).toContain("Size");
  });

  it("shows filter input", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "registry_get_info") return mockHiveInfo;
      if (cmd === "registry_get_subkeys") return mockSubkeysResponse;
      if (cmd === "registry_get_key_info") return mockKeyInfo;
      return null;
    });

    const { container } = renderComponent(() => <RegistryViewer path="/test/NTUSER.DAT" />);
    await tick();

    const filterInput = container.querySelector('input[placeholder="Filter values..."]');
    expect(filterInput).toBeTruthy();
  });

  it("renders key metadata in right panel", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "registry_get_info") return mockHiveInfo;
      if (cmd === "registry_get_subkeys") return mockSubkeysResponse;
      if (cmd === "registry_get_key_info") return mockKeyInfo;
      return null;
    });

    const { container } = renderComponent(() => <RegistryViewer path="/test/NTUSER.DAT" />);
    await tick();

    // Key info panel should show subkey/value counts
    expect(container.textContent).toContain("10");
    expect(container.textContent).toContain("2");
  });
});
