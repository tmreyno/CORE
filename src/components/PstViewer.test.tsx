// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach, vi } from "vitest";
import { createSignal } from "solid-js";
import { render } from "solid-js/web";
import { PstViewer } from "./PstViewer";
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

// Mock PST folder/message data (matches Rust types via src/types/pst.ts)
const mockPstInfo = {
  path: "/tmp/outlook.pst",
  displayName: "Outlook Data File",
  totalFolders: 5,
  folders: [
    {
      name: "Inbox",
      nodeId: 1001,
      contentCount: 20,
      unreadCount: 5,
      hasSubFolders: false,
      children: [],
    },
    {
      name: "Sent Items",
      nodeId: 1002,
      contentCount: 15,
      unreadCount: 0,
      hasSubFolders: false,
      children: [],
    },
    {
      name: "Deleted Items",
      nodeId: 1003,
      contentCount: 7,
      unreadCount: 0,
      hasSubFolders: true,
      children: [
        {
          name: "Old Trash",
          nodeId: 1004,
          contentCount: 3,
          unreadCount: 0,
          hasSubFolders: false,
          children: [],
        },
      ],
    },
  ],
};

describe("PstViewer", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
  });

  describe("Folder tree rendering", () => {
    it("calls pst_get_folders on mount", async () => {
      mockInvoke.mockResolvedValueOnce(mockPstInfo);

      renderComponent(() => <PstViewer path="/tmp/outlook.pst" />);
      await tick();

      expect(mockInvoke).toHaveBeenCalledWith("pst_get_folders", {
        path: "/tmp/outlook.pst",
      });
    });

    it("calls pst_get_folders_source when an evidence source is provided", async () => {
      const source = {
        containerPath: "/evidence/case.ad1",
        entryPath: "mail/outlook.pst",
        containerType: "ad1",
        size: 1024,
      };
      mockInvoke.mockResolvedValueOnce(mockPstInfo);

      renderComponent(() => <PstViewer path="/tmp/outlook.pst" source={source} />);
      await tick();

      expect(mockInvoke).toHaveBeenCalledWith("pst_get_folders_source", {
        source,
      });
    });

    it("renders folder names", async () => {
      mockInvoke.mockResolvedValueOnce(mockPstInfo);

      const { container } = renderComponent(() => (
        <PstViewer path="/tmp/outlook.pst" />
      ));
      await tick();

      expect(container.textContent).toContain("Inbox");
      expect(container.textContent).toContain("Sent Items");
    });

    it("displays message counts per folder", async () => {
      mockInvoke.mockResolvedValueOnce(mockPstInfo);

      const { container } = renderComponent(() => (
        <PstViewer path="/tmp/outlook.pst" />
      ));
      await tick();

      expect(container.textContent).toContain("20");
    });
  });

  describe("Loading and error states", () => {
    it("shows loading state initially", () => {
      mockInvoke.mockReturnValue(new Promise(() => {}));

      const { container } = renderComponent(() => (
        <PstViewer path="/tmp/outlook.pst" />
      ));

      expect(container.textContent).toContain("Loading");
    });

    it("shows error when PST parsing fails", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("Invalid PST format"));

      const { container } = renderComponent(() => (
        <PstViewer path="/tmp/bad.pst" />
      ));
      await tick();

      expect(container.textContent).toContain("Invalid PST format");
    });

    it("ignores stale PST folder loads after the selected archive changes", async () => {
      let resolveSlow: (value: typeof mockPstInfo) => void = () => {};
      const slowFolders = new Promise<typeof mockPstInfo>((resolve) => {
        resolveSlow = resolve;
      });
      const currentInfo = {
        ...mockPstInfo,
        path: "/tmp/current.pst",
        displayName: "Current Archive",
        folders: [
          {
            name: "Current Inbox",
            nodeId: 2001,
            contentCount: 1,
            unreadCount: 0,
            hasSubFolders: false,
            children: [],
          },
        ],
        totalFolders: 1,
      };
      const staleInfo = {
        ...mockPstInfo,
        path: "/tmp/slow.pst",
        displayName: "Stale Archive",
        folders: [
          {
            name: "Stale Inbox",
            nodeId: 3001,
            contentCount: 1,
            unreadCount: 0,
            hasSubFolders: false,
            children: [],
          },
        ],
        totalFolders: 1,
      };

      mockInvoke.mockImplementation((command, args) => {
        if (command === "pst_get_folders" && args?.path === "/tmp/slow.pst") {
          return slowFolders;
        }
        if (command === "pst_get_folders" && args?.path === "/tmp/current.pst") {
          return Promise.resolve(currentInfo);
        }
        return Promise.reject(new Error(`Unexpected invoke: ${command}`));
      });

      const [path, setPath] = createSignal("/tmp/slow.pst");
      const { container } = renderComponent(() => <PstViewer path={path()} />);
      await tick();

      setPath("/tmp/current.pst");
      await tick();

      expect(container.textContent).toContain("Current Archive");
      expect(container.textContent).toContain("Current Inbox");

      resolveSlow(staleInfo);
      await tick();

      expect(container.textContent).toContain("Current Archive");
      expect(container.textContent).toContain("Current Inbox");
      expect(container.textContent).not.toContain("Stale Archive");
      expect(container.textContent).not.toContain("Stale Inbox");
    });
  });

  describe("Edge cases", () => {
    it("handles PST with no folders", async () => {
      mockInvoke.mockResolvedValueOnce({
        ...mockPstInfo,
        folders: [],
        totalFolders: 0,
        totalMessages: 0,
      });

      const { container } = renderComponent(() => (
        <PstViewer path="/tmp/empty.pst" />
      ));
      await tick();

      // Should render without crashing
      expect(container.innerHTML).toBeTruthy();
    });

    it("emits metadata callback when provided", async () => {
      mockInvoke.mockResolvedValueOnce(mockPstInfo);
      const onMetadata = vi.fn();

      renderComponent(() => (
        <PstViewer path="/tmp/outlook.pst" onMetadata={onMetadata} />
      ));
      await tick();

      expect(onMetadata).toHaveBeenCalled();
    });
  });
});
