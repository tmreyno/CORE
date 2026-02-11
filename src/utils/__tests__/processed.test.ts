// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  ellipsePath,
  getDbTypeName,
  getDbTypeIcon,
  getCategoryIcon,
} from "../processed";
import type { ProcessedDbType } from "../../types/processed";

// =============================================================================
// ellipsePath
// =============================================================================

describe("ellipsePath", () => {
  it("returns empty string for empty input", () => {
    expect(ellipsePath("")).toBe("");
  });

  it("returns full path when under maxLen", () => {
    expect(ellipsePath("/short/path.txt", 40)).toBe("/short/path.txt");
  });

  it("returns full path when exactly at maxLen", () => {
    const path = "a".repeat(40);
    expect(ellipsePath(path, 40)).toBe(path);
  });

  it("truncates long path preserving filename", () => {
    const path = "/very/long/deeply/nested/directory/structure/file.txt";
    const result = ellipsePath(path, 20);
    expect(result).toContain("file.txt");
    expect(result.startsWith("...")).toBe(true);
    expect(result.length).toBeLessThanOrEqual(20);
  });

  it("truncates filename itself when filename exceeds maxLen", () => {
    const longFilename = "a".repeat(50) + ".txt";
    const path = "/dir/" + longFilename;
    const result = ellipsePath(path, 30);
    expect(result.startsWith("...")).toBe(true);
    expect(result.length).toBeLessThanOrEqual(30);
  });

  it("uses default maxLen of 40", () => {
    const shortPath = "/short.txt";
    expect(ellipsePath(shortPath)).toBe(shortPath);

    const longPath = "/very/long/deeply/nested/directory/structure/with/lots/of/parts/file.txt";
    const result = ellipsePath(longPath);
    expect(result.length).toBeLessThanOrEqual(40);
  });

  it("handles path with no slashes", () => {
    const long = "a".repeat(50);
    const result = ellipsePath(long, 20);
    expect(result.startsWith("...")).toBe(true);
  });

  it("handles Windows-style paths (backslash separators)", () => {
    // split('/') won't separate backslash, so full path treated as filename
    const path = "C:\\Users\\data\\file.txt";
    expect(ellipsePath(path, 40)).toBe(path);
  });
});

// =============================================================================
// getDbTypeName
// =============================================================================

describe("getDbTypeName", () => {
  it("returns correct name for MagnetAxiom", () => {
    expect(getDbTypeName("MagnetAxiom")).toBe("Magnet AXIOM");
  });

  it("returns correct name for CellebritePA", () => {
    expect(getDbTypeName("CellebritePA")).toBe("Cellebrite PA");
  });

  it("returns correct name for XWays", () => {
    expect(getDbTypeName("XWays")).toBe("X-Ways Forensics");
  });

  it("returns correct name for Autopsy", () => {
    expect(getDbTypeName("Autopsy")).toBe("Autopsy");
  });

  it("returns correct name for EnCase", () => {
    expect(getDbTypeName("EnCase")).toBe("EnCase");
  });

  it("returns correct name for FTK", () => {
    expect(getDbTypeName("FTK")).toBe("FTK");
  });

  it("returns correct name for GenericSqlite", () => {
    expect(getDbTypeName("GenericSqlite")).toBe("SQLite Database");
  });

  it("returns correct name for Unknown", () => {
    expect(getDbTypeName("Unknown")).toBe("Unknown Format");
  });

  it("returns raw type for unrecognized values", () => {
    expect(getDbTypeName("SomethingNew" as ProcessedDbType)).toBe("SomethingNew");
  });
});

// =============================================================================
// getDbTypeIcon
// =============================================================================

describe("getDbTypeIcon", () => {
  const expectedIcons: Record<ProcessedDbType, string> = {
    MagnetAxiom: "🧲",
    CellebritePA: "📱",
    XWays: "🔬",
    Autopsy: "🔍",
    EnCase: "📦",
    FTK: "🛠️",
    GenericSqlite: "🗄️",
    Unknown: "❓",
  };

  for (const [type, icon] of Object.entries(expectedIcons)) {
    it(`returns ${icon} for ${type}`, () => {
      expect(getDbTypeIcon(type as ProcessedDbType)).toBe(icon);
    });
  }

  it("returns default icon for unrecognized type", () => {
    expect(getDbTypeIcon("SomethingNew" as ProcessedDbType)).toBe("📁");
  });
});

// =============================================================================
// getCategoryIcon
// =============================================================================

describe("getCategoryIcon", () => {
  it("returns 🌐 for WebHistory", () => {
    expect(getCategoryIcon("WebHistory")).toBe("🌐");
  });

  it("returns 🌐 for Web", () => {
    expect(getCategoryIcon("Web")).toBe("🌐");
  });

  it("returns 📧 for Email", () => {
    expect(getCategoryIcon("Email")).toBe("📧");
  });

  it("returns 📧 for Email & Calendar", () => {
    expect(getCategoryIcon("Email & Calendar")).toBe("📧");
  });

  it("returns 💬 for Communication", () => {
    expect(getCategoryIcon("Communication")).toBe("💬");
  });

  it("returns 💬 for Chat", () => {
    expect(getCategoryIcon("Chat")).toBe("💬");
  });

  it("returns 🖼️ for Media", () => {
    expect(getCategoryIcon("Media")).toBe("🖼️");
  });

  it("returns 📄 for Documents", () => {
    expect(getCategoryIcon("Documents")).toBe("📄");
  });

  it("returns 📂 for FileSystem", () => {
    expect(getCategoryIcon("FileSystem")).toBe("📂");
  });

  it("returns 📂 for File System", () => {
    expect(getCategoryIcon("File System")).toBe("📂");
  });

  it("returns ⚙️ for System", () => {
    expect(getCategoryIcon("System")).toBe("⚙️");
  });

  it("returns 📱 for Mobile", () => {
    expect(getCategoryIcon("Mobile")).toBe("📱");
  });

  it("returns 📍 for Location", () => {
    expect(getCategoryIcon("Location")).toBe("📍");
  });

  it("returns ☁️ for Cloud", () => {
    expect(getCategoryIcon("Cloud")).toBe("☁️");
  });

  it("returns 🔒 for Encryption", () => {
    expect(getCategoryIcon("Encryption")).toBe("🔒");
  });

  it("returns 📋 as default for unknown categories", () => {
    expect(getCategoryIcon("SomeNewCategory")).toBe("📋");
  });

  it("returns 📋 for empty string", () => {
    expect(getCategoryIcon("")).toBe("📋");
  });
});
