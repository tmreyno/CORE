// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { SpreadsheetViewer } from "./SpreadsheetViewer";
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

// Mock spreadsheet info with multiple sheets
const mockSpreadsheetInfo = {
  path: "/evidence/data.xlsx",
  format: "xlsx",
  sheets: [
    { name: "Sheet1", row_count: 3, col_count: 3 },
    { name: "Summary", row_count: 10, col_count: 5 },
  ],
  total_sheets: 2,
};

// Mock single-sheet info
const mockSingleSheetInfo = {
  path: "/evidence/simple.csv",
  format: "csv",
  sheets: [{ name: "Sheet1", row_count: 2, col_count: 2 }],
  total_sheets: 1,
};

// Mock cell data (3 rows x 3 cols)
const mockRows = [
  [
    { type: "String" as const, value: "Name" },
    { type: "String" as const, value: "Age" },
    { type: "String" as const, value: "Score" },
  ],
  [
    { type: "String" as const, value: "Alice" },
    { type: "Int" as const, value: 30 },
    { type: "Float" as const, value: 95.5 },
  ],
  [
    { type: "String" as const, value: "Bob" },
    { type: "Int" as const, value: 25 },
    { type: "Bool" as const, value: true },
  ],
];

// Empty rows
const mockEmptyRows: any[] = [];

describe("SpreadsheetViewer", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
  });

  describe("loading state", () => {
    it("shows loading spinner initially", () => {
      mockInvoke.mockReturnValue(new Promise(() => {}));

      const { container } = renderComponent(() => (
        <SpreadsheetViewer path="/evidence/data.xlsx" />
      ));

      expect(container.textContent).toContain("Loading spreadsheet...");
    });

    it("calls spreadsheet_info with the correct path", async () => {
      mockInvoke.mockResolvedValueOnce(mockSpreadsheetInfo);
      mockInvoke.mockResolvedValueOnce(mockRows);

      renderComponent(() => <SpreadsheetViewer path="/evidence/data.xlsx" />);
      await tick();

      expect(mockInvoke).toHaveBeenCalledWith("spreadsheet_info", {
        path: "/evidence/data.xlsx",
      });
    });

    it("loads first sheet automatically after info", async () => {
      mockInvoke.mockResolvedValueOnce(mockSpreadsheetInfo);
      mockInvoke.mockResolvedValueOnce(mockRows);

      renderComponent(() => <SpreadsheetViewer path="/evidence/data.xlsx" />);
      await tick();

      expect(mockInvoke).toHaveBeenCalledWith("spreadsheet_read_sheet", {
        path: "/evidence/data.xlsx",
        sheetName: "Sheet1",
        startRow: 0,
        maxRows: 500,
      });
    });
  });

  describe("successful render", () => {
    it("displays the format label in toolbar", async () => {
      mockInvoke.mockResolvedValueOnce(mockSpreadsheetInfo);
      mockInvoke.mockResolvedValueOnce(mockRows);

      const { container } = renderComponent(() => (
        <SpreadsheetViewer path="/evidence/data.xlsx" />
      ));
      await tick();

      expect(container.textContent).toContain("XLSX");
    });

    it("renders cell data in a table", async () => {
      mockInvoke.mockResolvedValueOnce(mockSpreadsheetInfo);
      mockInvoke.mockResolvedValueOnce(mockRows);

      const { container } = renderComponent(() => (
        <SpreadsheetViewer path="/evidence/data.xlsx" />
      ));
      await tick();

      expect(container.textContent).toContain("Name");
      expect(container.textContent).toContain("Alice");
      expect(container.textContent).toContain("Bob");
    });

    it("renders numeric values", async () => {
      mockInvoke.mockResolvedValueOnce(mockSpreadsheetInfo);
      mockInvoke.mockResolvedValueOnce(mockRows);

      const { container } = renderComponent(() => (
        <SpreadsheetViewer path="/evidence/data.xlsx" />
      ));
      await tick();

      expect(container.textContent).toContain("30");
      expect(container.textContent).toContain("25");
    });

    it("renders boolean values as TRUE/FALSE", async () => {
      mockInvoke.mockResolvedValueOnce(mockSpreadsheetInfo);
      mockInvoke.mockResolvedValueOnce(mockRows);

      const { container } = renderComponent(() => (
        <SpreadsheetViewer path="/evidence/data.xlsx" />
      ));
      await tick();

      expect(container.textContent).toContain("TRUE");
    });

    it("displays row count", async () => {
      mockInvoke.mockResolvedValueOnce(mockSpreadsheetInfo);
      mockInvoke.mockResolvedValueOnce(mockRows);

      const { container } = renderComponent(() => (
        <SpreadsheetViewer path="/evidence/data.xlsx" />
      ));
      await tick();

      expect(container.textContent).toContain("3 rows");
    });

    it("renders column headers as letters (A, B, C)", async () => {
      mockInvoke.mockResolvedValueOnce(mockSpreadsheetInfo);
      mockInvoke.mockResolvedValueOnce(mockRows);

      const { container } = renderComponent(() => (
        <SpreadsheetViewer path="/evidence/data.xlsx" />
      ));
      await tick();

      const headers = container.querySelectorAll("th");
      // First header is "#" (row numbers), then A, B, C
      const headerTexts = Array.from(headers).map((h) => h.textContent?.trim());
      expect(headerTexts).toContain("A");
      expect(headerTexts).toContain("B");
      expect(headerTexts).toContain("C");
    });

    it("renders row numbers starting from 1", async () => {
      mockInvoke.mockResolvedValueOnce(mockSpreadsheetInfo);
      mockInvoke.mockResolvedValueOnce(mockRows);

      const { container } = renderComponent(() => (
        <SpreadsheetViewer path="/evidence/data.xlsx" />
      ));
      await tick();

      // Row number cells have specific classes
      const rowNumCells = container.querySelectorAll("td.font-mono");
      const rowNums = Array.from(rowNumCells).map((c) => c.textContent?.trim());
      expect(rowNums).toContain("1");
      expect(rowNums).toContain("2");
      expect(rowNums).toContain("3");
    });
  });

  describe("multi-sheet navigation", () => {
    it("shows sheet navigation for multi-sheet files", async () => {
      mockInvoke.mockResolvedValueOnce(mockSpreadsheetInfo);
      mockInvoke.mockResolvedValueOnce(mockRows);

      const { container } = renderComponent(() => (
        <SpreadsheetViewer path="/evidence/data.xlsx" />
      ));
      await tick();

      // Should have a select element with sheet names
      const select = container.querySelector("select");
      expect(select).not.toBeNull();
      const options = select!.querySelectorAll("option");
      expect(options.length).toBe(2);
      expect(options[0].textContent).toBe("Sheet1");
      expect(options[1].textContent).toBe("Summary");
    });

    it("hides sheet navigation for single-sheet files", async () => {
      mockInvoke.mockResolvedValueOnce(mockSingleSheetInfo);
      mockInvoke.mockResolvedValueOnce(mockRows);

      const { container } = renderComponent(() => (
        <SpreadsheetViewer path="/evidence/simple.csv" />
      ));
      await tick();

      const select = container.querySelector("select");
      expect(select).toBeNull();
    });

    it("displays CSV format label for CSV files", async () => {
      mockInvoke.mockResolvedValueOnce(mockSingleSheetInfo);
      mockInvoke.mockResolvedValueOnce(mockRows);

      const { container } = renderComponent(() => (
        <SpreadsheetViewer path="/evidence/simple.csv" />
      ));
      await tick();

      expect(container.textContent).toContain("CSV");
    });
  });

  describe("error state", () => {
    it("shows error message when loading fails", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("Unsupported format"));

      const { container } = renderComponent(() => (
        <SpreadsheetViewer path="/evidence/bad.xyz" />
      ));
      await tick();

      expect(container.textContent).toContain("Failed to load spreadsheet");
      expect(container.textContent).toContain("Unsupported format");
    });

    it("shows retry button on error", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("Read error"));

      const { container } = renderComponent(() => (
        <SpreadsheetViewer path="/evidence/bad.xlsx" />
      ));
      await tick();

      expect(container.textContent).toContain("Retry");
    });

    it("retries loading when retry button clicked", async () => {
      mockInvoke.mockRejectedValueOnce(new Error("Temporary error"));

      const { container } = renderComponent(() => (
        <SpreadsheetViewer path="/evidence/data.xlsx" />
      ));
      await tick();

      expect(container.textContent).toContain("Failed to load spreadsheet");

      // Now mock success
      mockInvoke.mockResolvedValueOnce(mockSpreadsheetInfo);
      mockInvoke.mockResolvedValueOnce(mockRows);

      const retryBtn = Array.from(container.querySelectorAll("button")).find(
        (b) => b.textContent?.includes("Retry")
      ) as HTMLButtonElement;
      expect(retryBtn).not.toBeNull();
      retryBtn.click();
      await tick();

      expect(container.textContent).toContain("Name");
      expect(container.textContent).toContain("Alice");
    });
  });

  describe("empty data", () => {
    it("shows empty message when sheet has no data", async () => {
      mockInvoke.mockResolvedValueOnce(mockSingleSheetInfo);
      mockInvoke.mockResolvedValueOnce(mockEmptyRows);

      const { container } = renderComponent(() => (
        <SpreadsheetViewer path="/evidence/empty.csv" />
      ));
      await tick();

      expect(container.textContent).toContain("No data in this sheet");
    });
  });

  describe("optional class", () => {
    it("applies custom class when provided", async () => {
      mockInvoke.mockResolvedValueOnce(mockSpreadsheetInfo);
      mockInvoke.mockResolvedValueOnce(mockRows);

      const { container } = renderComponent(() => (
        <SpreadsheetViewer path="/evidence/data.xlsx" class="my-custom" />
      ));
      await tick();

      const viewer = container.querySelector(".spreadsheet-viewer");
      expect(viewer?.classList.contains("my-custom")).toBe(true);
    });
  });
});
