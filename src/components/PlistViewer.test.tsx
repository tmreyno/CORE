// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { createSignal } from "solid-js";
import { render } from "solid-js/web";
import { PlistViewer } from "./PlistViewer";
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

// Mock plist data
const mockPlistData = {
  path: "/tmp/Info.plist",
  format: "Binary",
  root_type: "Dictionary",
  entry_count: 5,
  entries: [
    { key_path: "CFBundleName", value_type: "String", value_preview: "MyApp" },
    { key_path: "CFBundleVersion", value_type: "String", value_preview: "1.0.0" },
    { key_path: "CFBundleIdentifier", value_type: "String", value_preview: "com.example.myapp" },
    { key_path: "LSMinimumSystemVersion", value_type: "String", value_preview: "10.15" },
    { key_path: "NSAppTransportSecurity.NSAllowsArbitraryLoads", value_type: "Boolean", value_preview: "true" },
  ],
};

describe("PlistViewer", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
  });

  describe("Rendering", () => {
    it("renders plist entries in a table", async () => {
      mockInvoke.mockResolvedValueOnce(mockPlistData);

      const { container } = renderComponent(() => (
        <PlistViewer path="/tmp/Info.plist" />
      ));
      await tick();

      expect(container.textContent).toContain("CFBundleName");
      expect(container.textContent).toContain("MyApp");
      expect(container.textContent).toContain("CFBundleVersion");
      expect(container.textContent).toContain("1.0.0");
    });

    it("calls plist_read with the file path", async () => {
      mockInvoke.mockResolvedValueOnce(mockPlistData);

      renderComponent(() => <PlistViewer path="/tmp/Info.plist" />);
      await tick();

      expect(mockInvoke).toHaveBeenCalledWith("plist_read", { path: "/tmp/Info.plist" });
    });

    it("calls plist_read_source when an evidence source is provided", async () => {
      mockInvoke.mockResolvedValueOnce({
        ...mockPlistData,
        path: "/evidence/image.ad1:Library/Preferences/app.plist",
      });
      const source = {
        containerPath: "/evidence/image.ad1",
        entryPath: "Library/Preferences/app.plist",
        containerType: "ad1",
        size: 2048,
      };

      renderComponent(() => <PlistViewer path="/tmp/Info.plist" source={source} />);
      await tick();

      expect(mockInvoke).toHaveBeenCalledWith("plist_read_source", { source });
    });

    it("shows format badge (Binary/XML)", async () => {
      mockInvoke.mockResolvedValueOnce(mockPlistData);

      const { container } = renderComponent(() => (
        <PlistViewer path="/tmp/Info.plist" />
      ));
      await tick();

      expect(container.textContent).toContain("Binary");
    });

    it("displays entry count in status bar", async () => {
      mockInvoke.mockResolvedValueOnce(mockPlistData);

      const { container } = renderComponent(() => (
        <PlistViewer path="/tmp/Info.plist" />
      ));
      await tick();

      expect(container.textContent).toContain("5");
    });

    it("shows value type badges", async () => {
      mockInvoke.mockResolvedValueOnce(mockPlistData);

      const { container } = renderComponent(() => (
        <PlistViewer path="/tmp/Info.plist" />
      ));
      await tick();

      expect(container.textContent).toContain("String");
      expect(container.textContent).toContain("Boolean");
    });
  });

  describe("Loading and error states", () => {
    it("shows loading state initially", () => {
      mockInvoke.mockReturnValue(new Promise(() => {}));

      const { container } = renderComponent(() => (
        <PlistViewer path="/tmp/Info.plist" />
      ));

      expect(container.textContent).toContain("Parsing");
    });

    it("shows error when parsing fails", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("Invalid plist format"));

      const { container } = renderComponent(() => (
        <PlistViewer path="/tmp/bad.plist" />
      ));
      await tick();

      expect(container.textContent).toContain("Invalid plist format");
    });

    it("ignores stale plist loads after the selected path changes", async () => {
      let resolveSlow: (value: typeof mockPlistData) => void = () => {};
      const slowResult = new Promise<typeof mockPlistData>((resolve) => {
        resolveSlow = resolve;
      });
      const currentData = {
        ...mockPlistData,
        path: "/tmp/current.plist",
        entries: [
          {
            key_path: "CurrentSelection",
            value_type: "String",
            value_preview: "current-value",
          },
        ],
      };
      const staleData = {
        ...mockPlistData,
        path: "/tmp/slow.plist",
        entries: [
          {
            key_path: "StaleSelection",
            value_type: "String",
            value_preview: "stale-value",
          },
        ],
      };

      mockInvoke.mockImplementation((command, args) => {
        if (command === "plist_read" && args?.path === "/tmp/slow.plist") {
          return slowResult;
        }
        if (command === "plist_read" && args?.path === "/tmp/current.plist") {
          return Promise.resolve(currentData);
        }
        return Promise.reject(new Error(`Unexpected invoke: ${command}`));
      });

      const [path, setPath] = createSignal("/tmp/slow.plist");
      const { container } = renderComponent(() => <PlistViewer path={path()} />);
      await tick();

      setPath("/tmp/current.plist");
      await tick();

      expect(container.textContent).toContain("CurrentSelection");
      expect(container.textContent).toContain("current-value");

      resolveSlow(staleData);
      await tick();

      expect(container.textContent).toContain("CurrentSelection");
      expect(container.textContent).not.toContain("StaleSelection");
      expect(container.textContent).not.toContain("stale-value");
    });
  });

  describe("Edge cases", () => {
    it("handles plist with no entries", async () => {
      mockInvoke.mockResolvedValueOnce({
        ...mockPlistData,
        entry_count: 0,
        entries: [],
      });

      const { container } = renderComponent(() => (
        <PlistViewer path="/tmp/empty.plist" />
      ));
      await tick();

      // Should render without crashing
      expect(container.innerHTML).toBeTruthy();
    });

    it("handles nested key paths", async () => {
      mockInvoke.mockResolvedValueOnce(mockPlistData);

      const { container } = renderComponent(() => (
        <PlistViewer path="/tmp/Info.plist" />
      ));
      await tick();

      // The nested key path should be displayed
      expect(container.textContent).toContain("NSAppTransportSecurity");
    });
  });
});
