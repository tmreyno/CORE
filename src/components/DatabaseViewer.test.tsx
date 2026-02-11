// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect, beforeEach } from "vitest";
import { render } from "solid-js/web";
import { DatabaseViewer } from "./DatabaseViewer";
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

const mockDatabaseInfo = {
  path: "/test/evidence.db",
  fileSize: 2048000,
  pageSize: 4096,
  pageCount: 500,
  sqliteVersion: "3.42.0",
  tables: [
    {
      name: "contacts",
      rowCount: 150,
      columnCount: 4,
      isSystem: false,
    },
    {
      name: "messages",
      rowCount: 3200,
      columnCount: 6,
      isSystem: false,
    },
    {
      name: "sqlite_master",
      rowCount: 5,
      columnCount: 5,
      isSystem: true,
    },
    {
      name: "sqlite_sequence",
      rowCount: 2,
      columnCount: 2,
      isSystem: true,
    },
    {
      name: "call_log",
      rowCount: 89,
      columnCount: 8,
      isSystem: false,
    },
  ],
};

const mockTableSchema = {
  name: "contacts",
  columns: [
    { index: 0, name: "id", dataType: "INTEGER", isPrimaryKey: true, nullable: false, defaultValue: null },
    { index: 1, name: "name", dataType: "TEXT", isPrimaryKey: false, nullable: false, defaultValue: null },
    { index: 2, name: "phone", dataType: "TEXT", isPrimaryKey: false, nullable: true, defaultValue: null },
    { index: 3, name: "email", dataType: "TEXT", isPrimaryKey: false, nullable: true, defaultValue: "''" },
  ],
  rowCount: 150,
  createSql: "CREATE TABLE contacts (id INTEGER PRIMARY KEY, name TEXT NOT NULL, phone TEXT, email TEXT DEFAULT '')",
  indexes: [],
};

const mockTableRows = {
  tableName: "contacts",
  columns: ["id", "name", "phone", "email"],
  rows: [
    ["1", "John Doe", "+1-555-0101", "john@example.com"],
    ["2", "Jane Smith", "+1-555-0102", "jane@example.com"],
    ["3", "Bob Wilson", null, "bob@example.com"],
  ],
  totalCount: 150,
  page: 0,
  pageSize: 100,
  hasMore: true,
};

// ============================================================================
// Tests
// ============================================================================

describe("DatabaseViewer", () => {
  beforeEach(() => {
    document.body.innerHTML = "";
    mockInvoke.mockReset();
  });

  it("renders loading state initially", () => {
    mockInvoke.mockReturnValue(new Promise(() => {}));
    const { container } = renderComponent(() => <DatabaseViewer path="/test/evidence.db" />);
    expect(container.textContent).toContain("Loading database");
  });

  it("renders error state on failure", async () => {
    mockInvoke.mockRejectedValue(new Error("Not a valid SQLite database"));
    const { container } = renderComponent(() => <DatabaseViewer path="/test/evidence.db" />);
    await tick();
    expect(container.textContent).toContain("Failed to load database");
    expect(container.textContent).toContain("Not a valid SQLite database");
  });

  it("renders database info after loading", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "database_get_info") return mockDatabaseInfo;
      if (cmd === "database_get_table_schema") return mockTableSchema;
      if (cmd === "database_query_table") return mockTableRows;
      return null;
    });

    const { container } = renderComponent(() => <DatabaseViewer path="/test/evidence.db" />);
    await tick();

    expect(container.textContent).toContain("SQLite");
    expect(container.textContent).toContain("5 tables");
  });

  it("renders user tables in table list", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "database_get_info") return mockDatabaseInfo;
      if (cmd === "database_get_table_schema") return mockTableSchema;
      if (cmd === "database_query_table") return mockTableRows;
      return null;
    });

    const { container } = renderComponent(() => <DatabaseViewer path="/test/evidence.db" />);
    await tick();

    expect(container.textContent).toContain("contacts");
    expect(container.textContent).toContain("messages");
    expect(container.textContent).toContain("call_log");
  });

  it("renders system tables section", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "database_get_info") return mockDatabaseInfo;
      if (cmd === "database_get_table_schema") return mockTableSchema;
      if (cmd === "database_query_table") return mockTableRows;
      return null;
    });

    const { container } = renderComponent(() => <DatabaseViewer path="/test/evidence.db" />);
    await tick();

    expect(container.textContent).toContain("System");
  });

  it("renders schema columns for selected table", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "database_get_info") return mockDatabaseInfo;
      if (cmd === "database_get_table_schema") return mockTableSchema;
      if (cmd === "database_query_table") return mockTableRows;
      return null;
    });

    const { container } = renderComponent(() => <DatabaseViewer path="/test/evidence.db" />);
    await tick();

    expect(container.textContent).toContain("id");
    expect(container.textContent).toContain("name");
    expect(container.textContent).toContain("phone");
    expect(container.textContent).toContain("email");
  });

  it("renders table row data", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "database_get_info") return mockDatabaseInfo;
      if (cmd === "database_get_table_schema") return mockTableSchema;
      if (cmd === "database_query_table") return mockTableRows;
      return null;
    });

    const { container } = renderComponent(() => <DatabaseViewer path="/test/evidence.db" />);
    await tick();

    expect(container.textContent).toContain("John Doe");
    expect(container.textContent).toContain("Jane Smith");
    expect(container.textContent).toContain("Bob Wilson");
  });

  it("invokes correct commands on load", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "database_get_info") return mockDatabaseInfo;
      if (cmd === "database_get_table_schema") return mockTableSchema;
      if (cmd === "database_query_table") return mockTableRows;
      return null;
    });

    renderComponent(() => <DatabaseViewer path="/test/evidence.db" />);
    await tick();

    expect(mockInvoke).toHaveBeenCalledWith("database_get_info", { path: "/test/evidence.db" });
  });

  it("shows row counts for tables", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "database_get_info") return mockDatabaseInfo;
      if (cmd === "database_get_table_schema") return mockTableSchema;
      if (cmd === "database_query_table") return mockTableRows;
      return null;
    });

    const { container } = renderComponent(() => <DatabaseViewer path="/test/evidence.db" />);
    await tick();

    expect(container.textContent).toContain("150");
  });

  it("shows pagination info", async () => {
    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "database_get_info") return mockDatabaseInfo;
      if (cmd === "database_get_table_schema") return mockTableSchema;
      if (cmd === "database_query_table") return mockTableRows;
      return null;
    });

    const { container } = renderComponent(() => <DatabaseViewer path="/test/evidence.db" />);
    await tick();

    expect(container.textContent).toContain("150 rows");
  });

  it("handles database with no user tables", async () => {
    const emptyDbInfo = {
      ...mockDatabaseInfo,
      tables: [
        {
          name: "sqlite_master",
          rowCount: 1,
          columnCount: 5,
          isSystem: true,
        },
      ],
    };

    mockInvoke.mockImplementation(async (cmd: string) => {
      if (cmd === "database_get_info") return emptyDbInfo;
      return null;
    });

    const { container } = renderComponent(() => <DatabaseViewer path="/test/empty.db" />);
    await tick();

    expect(container.textContent).toContain("SQLite");
  });
});
