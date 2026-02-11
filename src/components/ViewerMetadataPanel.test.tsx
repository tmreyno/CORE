// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { ViewerMetadataPanel } from "./ViewerMetadataPanel";
import type { ViewerMetadata } from "../types/viewerMetadata";

// Helper to render and return the container
function renderComponent(component: () => any) {
  const container = document.createElement("div");
  document.body.appendChild(container);
  const dispose = render(component, container);
  return { container, dispose };
}

// Base metadata factory
function makeMetadata(overrides: Partial<ViewerMetadata> = {}): ViewerMetadata {
  return {
    fileInfo: {
      name: "test.bin",
      path: "/evidence/test.bin",
      size: 1024,
      extension: "bin",
    },
    viewerType: "Hex",
    sections: [],
    ...overrides,
  };
}

describe("ViewerMetadataPanel", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
  });

  describe("tab rendering", () => {
    it("renders File Info tab always", () => {
      const metadata = makeMetadata();
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      expect(container.textContent).toContain("File Info");
      dispose();
    });

    it("does not render viewer tab when no sections", () => {
      const metadata = makeMetadata({ sections: [] });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      // Only File Info tab should exist, not a viewer-specific tab
      const buttons = container.querySelectorAll("button");
      const tabLabels = Array.from(buttons).map(b => b.textContent?.trim());
      expect(tabLabels).toContain("File Info");
      // Should not have a second tab
      expect(tabLabels.filter(l => l && l !== "File Info")).toHaveLength(0);
      dispose();
    });

    it("renders EXIF tab when exif section present", () => {
      const metadata = makeMetadata({
        viewerType: "Image",
        sections: [{
          kind: "exif",
          make: "Canon",
          model: "EOS R5",
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      expect(container.textContent).toContain("EXIF");
      dispose();
    });

    it("renders Registry tab when registry section present", () => {
      const metadata = makeMetadata({
        viewerType: "Registry",
        sections: [{
          kind: "registry",
          hiveName: "SYSTEM",
          hiveType: "System",
          rootKeyName: "CMI-CreateHive",
          totalKeys: 500,
          totalValues: 1200,
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      expect(container.textContent).toContain("Registry");
      dispose();
    });

    it("renders Database tab when database section present", () => {
      const metadata = makeMetadata({
        viewerType: "Database",
        sections: [{
          kind: "database",
          path: "/tmp/test.db",
          pageSize: 4096,
          pageCount: 100,
          sizeBytes: 409600,
          tableCount: 5,
          tables: [
            { name: "users", rowCount: 42, columnCount: 3, isSystem: false },
          ],
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      expect(container.textContent).toContain("Database");
      dispose();
    });

    it("renders Binary tab when binary section present", () => {
      const metadata = makeMetadata({
        viewerType: "Binary",
        sections: [{
          kind: "binary",
          format: "PE32+",
          architecture: "x86_64",
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      expect(container.textContent).toContain("Binary");
      dispose();
    });

    it("renders Email tab when email section present", () => {
      const metadata = makeMetadata({
        viewerType: "Email",
        sections: [{
          kind: "email",
          subject: "Test Subject",
          from: "test@example.com",
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      expect(container.textContent).toContain("Email");
      dispose();
    });

    it("renders Plist tab when plist section present", () => {
      const metadata = makeMetadata({
        viewerType: "Plist",
        sections: [{
          kind: "plist",
          format: "Binary",
          entryCount: 15,
          rootType: "Dictionary",
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      expect(container.textContent).toContain("Plist");
      dispose();
    });

    it("renders Document tab when document section present", () => {
      const metadata = makeMetadata({
        viewerType: "PDF",
        sections: [{
          kind: "document",
          format: "PDF",
          title: "Test Document",
          author: "Test Author",
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      expect(container.textContent).toContain("Document");
      dispose();
    });

    it("renders Spreadsheet tab when spreadsheet section present", () => {
      const metadata = makeMetadata({
        viewerType: "Spreadsheet",
        sections: [{
          kind: "spreadsheet",
          format: "XLSX",
          sheetCount: 3,
          sheets: [
            { name: "Sheet1", rowCount: 100, columnCount: 5 },
            { name: "Sheet2", rowCount: 50, columnCount: 3 },
            { name: "Sheet3", rowCount: 10, columnCount: 2 },
          ],
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      expect(container.textContent).toContain("Spreadsheet");
      dispose();
    });
  });

  describe("file info tab content", () => {
    it("shows file name and path", () => {
      const metadata = makeMetadata({
        fileInfo: {
          name: "evidence.jpg",
          path: "/photos/evidence.jpg",
          size: 2048576,
          extension: "jpg",
        },
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      // Click on File Info tab first
      const fileTabBtn = Array.from(container.querySelectorAll("button"))
        .find(b => b.textContent?.trim() === "File Info");
      fileTabBtn?.click();

      expect(container.textContent).toContain("evidence.jpg");
      expect(container.textContent).toContain("/photos/evidence.jpg");
      dispose();
    });

    it("shows container info when present", () => {
      const metadata = makeMetadata({
        fileInfo: {
          name: "test.dat",
          path: "/files/test.dat",
          size: 512,
          containerPath: "/evidence/case.ad1",
          containerType: "AD1",
        },
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      const fileTabBtn = Array.from(container.querySelectorAll("button"))
        .find(b => b.textContent?.trim() === "File Info");
      fileTabBtn?.click();

      expect(container.textContent).toContain("Container");
      expect(container.textContent).toContain("AD1");
      dispose();
    });

    it("shows viewer type", () => {
      const metadata = makeMetadata({ viewerType: "Registry" });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      const fileTabBtn = Array.from(container.querySelectorAll("button"))
        .find(b => b.textContent?.trim() === "File Info");
      fileTabBtn?.click();

      expect(container.textContent).toContain("Registry");
      dispose();
    });

    it("shows source type badges for disk files", () => {
      const metadata = makeMetadata({
        fileInfo: {
          name: "test.dat",
          path: "/files/test.dat",
          size: 100,
          isDiskFile: true,
        },
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      const fileTabBtn = Array.from(container.querySelectorAll("button"))
        .find(b => b.textContent?.trim() === "File Info");
      fileTabBtn?.click();

      expect(container.textContent).toContain("Disk File");
      dispose();
    });
  });

  describe("viewer-specific section content", () => {
    it("shows EXIF camera info", () => {
      const metadata = makeMetadata({
        viewerType: "Image",
        sections: [{
          kind: "exif",
          make: "Canon",
          model: "EOS R5",
          iso: 400,
          exposureTime: "1/250",
          fNumber: "2.8",
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      // EXIF tab should be active by default when sections exist
      expect(container.textContent).toContain("Canon");
      expect(container.textContent).toContain("EOS R5");
      expect(container.textContent).toContain("400");
      dispose();
    });

    it("shows EXIF GPS with map link", () => {
      const metadata = makeMetadata({
        viewerType: "Image",
        sections: [{
          kind: "exif",
          gps: {
            latitude: 37.7749,
            longitude: -122.4194,
            latitudeRef: "N",
            longitudeRef: "W",
          },
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      expect(container.textContent).toContain("37.774900");
      expect(container.textContent).toContain("Google Maps");
      dispose();
    });

    it("shows EXIF forensic identifiers", () => {
      const metadata = makeMetadata({
        viewerType: "Image",
        sections: [{
          kind: "exif",
          serialNumber: "ABC12345",
          imageUniqueId: "unique-id-xyz",
          ownerName: "John Doe",
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      expect(container.textContent).toContain("ABC12345");
      expect(container.textContent).toContain("unique-id-xyz");
      expect(container.textContent).toContain("John Doe");
      dispose();
    });

    it("shows registry hive info", () => {
      const metadata = makeMetadata({
        viewerType: "Registry",
        sections: [{
          kind: "registry",
          hiveName: "SOFTWARE",
          hiveType: "Software",
          rootKeyName: "CMI-CreateHive",
          totalKeys: 12000,
          totalValues: 45000,
          selectedKeyPath: "Microsoft\\Windows\\CurrentVersion",
          selectedKeyInfo: {
            subkeyCount: 15,
            valueCount: 3,
            lastModified: "2024-01-15 10:30:00",
          },
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      expect(container.textContent).toContain("SOFTWARE");
      expect(container.textContent).toContain("12000");
      expect(container.textContent).toContain("45000");
      expect(container.textContent).toContain("Microsoft\\Windows\\CurrentVersion");
      dispose();
    });

    it("shows database table info", () => {
      const metadata = makeMetadata({
        viewerType: "Database",
        sections: [{
          kind: "database",
          path: "/tmp/test.db",
          pageSize: 4096,
          pageCount: 50,
          sizeBytes: 204800,
          tableCount: 3,
          tables: [
            { name: "messages", rowCount: 500, columnCount: 8, isSystem: false },
            { name: "contacts", rowCount: 100, columnCount: 5, isSystem: false },
            { name: "sqlite_master", rowCount: 3, columnCount: 5, isSystem: true },
          ],
          selectedTable: "messages",
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      // Selected table info is shown in "Selected Table" group (defaultOpen)
      expect(container.textContent).toContain("messages");
      // Row/column counts for selected table shown as separate fields
      expect(container.textContent).toContain("500");
      expect(container.textContent).toContain("8");
      // "Tables (2)" group title is visible even when collapsed
      expect(container.textContent).toContain("Tables (2)");
      dispose();
    });

    it("shows binary executable info", () => {
      const metadata = makeMetadata({
        viewerType: "Binary",
        sections: [{
          kind: "binary",
          format: "PE32+",
          architecture: "x86_64",
          entryPoint: "0x00401000",
          sectionCount: 5,
          importCount: 42,
          exportCount: 10,
          isStripped: false,
          isDynamic: true,
          subsystem: "Windows GUI",
          compiledDate: "2024-03-15 14:30:00",
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      expect(container.textContent).toContain("PE32+");
      expect(container.textContent).toContain("x86_64");
      expect(container.textContent).toContain("0x00401000");
      expect(container.textContent).toContain("Windows GUI");
      dispose();
    });

    it("shows email info with headers", () => {
      const metadata = makeMetadata({
        viewerType: "Email",
        sections: [{
          kind: "email",
          subject: "Important Evidence",
          from: "suspect@example.com",
          to: ["detective@police.gov", "analyst@lab.gov"],
          date: "2024-01-15 09:30:00",
          messageId: "<abc123@example.com>",
          attachmentCount: 2,
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      expect(container.textContent).toContain("Important Evidence");
      expect(container.textContent).toContain("suspect@example.com");
      expect(container.textContent).toContain("detective@police.gov");
      dispose();
    });

    it("shows plist info with notable keys", () => {
      const metadata = makeMetadata({
        viewerType: "Plist",
        sections: [{
          kind: "plist",
          format: "Binary",
          entryCount: 25,
          rootType: "Dictionary",
          notableKeys: [
            { key: "CFBundleIdentifier", value: "com.apple.Maps" },
            { key: "CFBundleName", value: "Maps" },
          ],
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      expect(container.textContent).toContain("Binary");
      expect(container.textContent).toContain("25");
      expect(container.textContent).toContain("com.apple.Maps");
      dispose();
    });

    it("shows spreadsheet sheet list", () => {
      const metadata = makeMetadata({
        viewerType: "Spreadsheet",
        sections: [{
          kind: "spreadsheet",
          format: "XLSX",
          sheetCount: 2,
          sheets: [
            { name: "Data", rowCount: 500, columnCount: 10 },
            { name: "Summary", rowCount: 20, columnCount: 4 },
          ],
          selectedSheet: "Data",
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      expect(container.textContent).toContain("Data");
      expect(container.textContent).toContain("Summary");
      expect(container.textContent).toContain("500×10");
      dispose();
    });
  });

  describe("collapsible groups", () => {
    it("groups are collapsible", () => {
      const metadata = makeMetadata({
        viewerType: "Image",
        sections: [{
          kind: "exif",
          make: "Canon",
          model: "EOS R5",
          width: 8192,
          height: 5464,
        }],
      });
      const { container, dispose } = renderComponent(() =>
        <ViewerMetadataPanel metadata={metadata} />
      );
      // Should have collapsible groups with chevron icons
      const groupButtons = container.querySelectorAll("button");
      expect(groupButtons.length).toBeGreaterThan(0);
      dispose();
    });
  });
});
