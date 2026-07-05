// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { createSignal } from "solid-js";
import { render } from "solid-js/web";
import { RegistryViewer } from "./RegistryViewer";
import { mockInvoke } from "../__tests__/setup";

vi.mock("../utils/platform", () => ({
  isTauri: true,
}));

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

  it("invokes source commands when an evidence source is provided", async () => {
    const source = {
      containerPath: "/evidence/case.ad1",
      entryPath: "Users/Alice/NTUSER.DAT",
      containerType: "ad1",
      size: 65536,
    };
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "registry_get_info_source") return mockHiveInfo;
      if (cmd === "registry_get_subkeys_source") return mockSubkeysResponse;
      if (cmd === "registry_get_key_info_source") return mockKeyInfo;
      return null;
    });

    renderComponent(() => <RegistryViewer path="/tmp/NTUSER.DAT" source={source} />);
    await tick();

    expect(mockInvoke).toHaveBeenCalledWith("registry_get_info_source", { source });
    expect(mockInvoke).toHaveBeenCalledWith("registry_get_subkeys_source", {
      source,
      keyPath: "",
    });
    expect(mockInvoke).toHaveBeenCalledWith("registry_get_key_info_source", {
      source,
      keyPath: "CMI-CreateHive\\Software",
    });
  });

  it("ignores stale registry hive reads after the selected hive changes", async () => {
    let resolveSlowInfo: (value: typeof mockHiveInfo) => void = () => {};
    const slowInfo = new Promise<typeof mockHiveInfo>((resolve) => {
      resolveSlowInfo = resolve;
    });
    const currentHiveInfo = {
      ...mockHiveInfo,
      path: "/test/current/SOFTWARE",
      rootKeyName: "CURRENT-HIVE",
      rootKeyPath: "CURRENT-HIVE",
      totalKeys: 10,
      totalValues: 20,
    };
    const currentSubkeys = {
      parentPath: "CURRENT-HIVE",
      subkeys: [
        {
          name: "Microsoft",
          path: "CURRENT-HIVE\\Microsoft",
          timestamp: "2024-02-01 00:00:00 UTC",
          subkeyCount: 1,
          valueCount: 1,
          hasSubkeys: false,
        },
      ],
    };
    const currentKeyInfo = {
      ...mockKeyInfo,
      name: "Microsoft",
      path: "CURRENT-HIVE\\Microsoft",
      prettyPath: "CURRENT-HIVE\\Microsoft",
      values: [
        {
          name: "ProductName",
          dataType: "REG_SZ",
          data: "Current Windows",
          size: 30,
        },
      ],
    };

    mockInvoke.mockImplementation((cmd: string, args: any) => {
      if (cmd === "registry_get_info" && args?.path === "/test/slow/SOFTWARE") {
        return slowInfo;
      }
      if (cmd === "registry_get_info" && args?.path === "/test/current/SOFTWARE") {
        return Promise.resolve(currentHiveInfo);
      }
      if (
        cmd === "registry_get_subkeys" &&
        args?.hivePath === "/test/current/SOFTWARE"
      ) {
        return Promise.resolve(currentSubkeys);
      }
      if (
        cmd === "registry_get_key_info" &&
        args?.hivePath === "/test/current/SOFTWARE"
      ) {
        return Promise.resolve(currentKeyInfo);
      }
      if (
        cmd === "registry_get_subkeys" &&
        args?.hivePath === "/test/slow/SOFTWARE"
      ) {
        return Promise.resolve({
          parentPath: "STALE-HIVE",
          subkeys: [
            {
              name: "StaleKey",
              path: "STALE-HIVE\\StaleKey",
              timestamp: "2024-01-01 00:00:00 UTC",
              subkeyCount: 0,
              valueCount: 1,
              hasSubkeys: false,
            },
          ],
        });
      }
      if (
        cmd === "registry_get_key_info" &&
        args?.hivePath === "/test/slow/SOFTWARE"
      ) {
        return Promise.resolve({
          ...mockKeyInfo,
          values: [
            {
              name: "StaleValue",
              dataType: "REG_SZ",
              data: "Stale registry data",
              size: 38,
            },
          ],
        });
      }
      return Promise.reject(new Error(`Unexpected invoke: ${cmd}`));
    });

    const [path, setPath] = createSignal("/test/slow/SOFTWARE");
    const { container } = renderComponent(() => <RegistryViewer path={path()} />);
    await tick();

    setPath("/test/current/SOFTWARE");
    await tick();

    expect(container.textContent).toContain("CURRENT-HIVE");
    expect(container.textContent).toContain("Current Windows");

    resolveSlowInfo({
      ...mockHiveInfo,
      path: "/test/slow/SOFTWARE",
      rootKeyName: "STALE-HIVE",
      rootKeyPath: "STALE-HIVE",
    });
    await tick();

    expect(container.textContent).toContain("CURRENT-HIVE");
    expect(container.textContent).toContain("Current Windows");
    expect(container.textContent).not.toContain("STALE-HIVE");
    expect(container.textContent).not.toContain("Stale registry data");
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
