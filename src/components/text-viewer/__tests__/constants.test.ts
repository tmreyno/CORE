// =============================================================================
// text-viewer constants — language map and numeric constant tests
// =============================================================================

import { describe, it, expect, vi } from "vitest";

// Mock the preferences import
vi.mock("../../preferences", () => ({
  getPreference: vi.fn(() => 100), // 100 MB default
}));

import {
  INITIAL_LOAD_SIZE,
  LOAD_MORE_SIZE,
  SCROLL_THRESHOLD,
  getMaxLoadedChars,
  LANGUAGE_MAP,
} from "../constants";

describe("numeric constants", () => {
  it("INITIAL_LOAD_SIZE is 100KB", () => {
    expect(INITIAL_LOAD_SIZE).toBe(100000);
  });

  it("LOAD_MORE_SIZE is 50KB", () => {
    expect(LOAD_MORE_SIZE).toBe(50000);
  });

  it("SCROLL_THRESHOLD is 300px", () => {
    expect(SCROLL_THRESHOLD).toBe(300);
  });
});

describe("getMaxLoadedChars", () => {
  it("converts MB preference to chars (1 char ≈ 1 byte for ASCII)", () => {
    // Mock returns 100 MB
    expect(getMaxLoadedChars()).toBe(100 * 1024 * 1024);
  });
});

describe("LANGUAGE_MAP", () => {
  it("has entries for common web languages", () => {
    expect(LANGUAGE_MAP["js"]).toBe("javascript");
    expect(LANGUAGE_MAP["ts"]).toBe("typescript");
    expect(LANGUAGE_MAP["jsx"]).toBe("javascript");
    expect(LANGUAGE_MAP["tsx"]).toBe("typescript");
    expect(LANGUAGE_MAP["html"]).toBe("html");
    expect(LANGUAGE_MAP["css"]).toBe("css");
  });

  it("has entries for systems languages", () => {
    expect(LANGUAGE_MAP["rs"]).toBe("rust");
    expect(LANGUAGE_MAP["c"]).toBe("c");
    expect(LANGUAGE_MAP["cpp"]).toBe("cpp");
    expect(LANGUAGE_MAP["go"]).toBe("go");
    expect(LANGUAGE_MAP["java"]).toBe("java");
  });

  it("has entries for scripting languages", () => {
    expect(LANGUAGE_MAP["py"]).toBe("python");
    expect(LANGUAGE_MAP["rb"]).toBe("ruby");
    expect(LANGUAGE_MAP["php"]).toBe("php");
  });

  it("maps yaml/yml to the same language", () => {
    expect(LANGUAGE_MAP["yaml"]).toBe("yaml");
    expect(LANGUAGE_MAP["yml"]).toBe("yaml");
  });

  it("maps shell variants to bash", () => {
    expect(LANGUAGE_MAP["sh"]).toBe("bash");
    expect(LANGUAGE_MAP["bash"]).toBe("bash");
    expect(LANGUAGE_MAP["zsh"]).toBe("bash");
  });

  it("maps Windows script extensions", () => {
    expect(LANGUAGE_MAP["ps1"]).toBe("powershell");
    expect(LANGUAGE_MAP["bat"]).toBe("batch");
    expect(LANGUAGE_MAP["cmd"]).toBe("batch");
  });

  it("maps C/C++ header files correctly", () => {
    expect(LANGUAGE_MAP["h"]).toBe("c");
    expect(LANGUAGE_MAP["hpp"]).toBe("cpp");
  });

  it("maps CSS preprocessors", () => {
    expect(LANGUAGE_MAP["scss"]).toBe("scss");
    expect(LANGUAGE_MAP["sass"]).toBe("sass");
    expect(LANGUAGE_MAP["less"]).toBe("less");
  });

  it("maps data/config formats", () => {
    expect(LANGUAGE_MAP["json"]).toBe("json");
    expect(LANGUAGE_MAP["xml"]).toBe("xml");
    expect(LANGUAGE_MAP["toml"]).toBe("toml");
    expect(LANGUAGE_MAP["sql"]).toBe("sql");
    expect(LANGUAGE_MAP["md"]).toBe("markdown");
  });

  it("maps htm to html", () => {
    expect(LANGUAGE_MAP["htm"]).toBe("html");
  });

  it("returns undefined for unknown extensions", () => {
    expect(LANGUAGE_MAP["zzz"]).toBeUndefined();
    expect(LANGUAGE_MAP["bin"]).toBeUndefined();
  });

  it("has at least 30 entries", () => {
    expect(Object.keys(LANGUAGE_MAP).length).toBeGreaterThanOrEqual(30);
  });
});
