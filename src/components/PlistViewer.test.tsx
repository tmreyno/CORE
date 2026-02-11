// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { PlistViewer } from "./PlistViewer";
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
