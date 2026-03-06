// =============================================================================
// canPreview — file preview eligibility tests
// =============================================================================

import { describe, it, expect } from "vitest";
import { canPreview } from "../canPreview";

describe("canPreview", () => {
  // PDF
  it("returns true for PDF files", () => {
    expect(canPreview("report.pdf")).toBe(true);
    expect(canPreview("DOCUMENT.PDF")).toBe(true);
  });

  // Images
  it("returns true for image files", () => {
    expect(canPreview("photo.jpg")).toBe(true);
    expect(canPreview("image.png")).toBe(true);
    expect(canPreview("pic.gif")).toBe(true);
    expect(canPreview("raw.bmp")).toBe(true);
    expect(canPreview("shot.webp")).toBe(true);
  });

  // Spreadsheets
  it("returns true for spreadsheet files", () => {
    expect(canPreview("data.xlsx")).toBe(true);
    expect(canPreview("data.xls")).toBe(true);
    expect(canPreview("data.csv")).toBe(true);
    expect(canPreview("data.ods")).toBe(true);
  });

  // Office documents
  it("returns true for office documents", () => {
    expect(canPreview("doc.docx")).toBe(true);
    expect(canPreview("doc.doc")).toBe(true);
    expect(canPreview("slides.pptx")).toBe(true);
    expect(canPreview("slides.ppt")).toBe(true);
    expect(canPreview("doc.odt")).toBe(true);
    expect(canPreview("doc.rtf")).toBe(true);
  });

  // Text documents
  it("returns true for text documents", () => {
    expect(canPreview("readme.txt")).toBe(true);
    expect(canPreview("notes.md")).toBe(true);
    expect(canPreview("log.log")).toBe(true);
  });

  // Code files
  it("returns true for code files", () => {
    expect(canPreview("app.js")).toBe(true);
    expect(canPreview("main.ts")).toBe(true);
    expect(canPreview("script.py")).toBe(true);
    expect(canPreview("main.rs")).toBe(true);
    expect(canPreview("app.go")).toBe(true);
  });

  // Config files
  it("returns true for config files", () => {
    expect(canPreview("config.json")).toBe(true);
    expect(canPreview("settings.yaml")).toBe(true);
    expect(canPreview("data.xml")).toBe(true);
    expect(canPreview("config.toml")).toBe(true);
    expect(canPreview("app.ini")).toBe(true);
    // .env is a dotfile — getExtension returns "" for leading-dot-only names
    expect(canPreview(".env")).toBe(false);
  });

  // Email files
  it("returns true for email files", () => {
    expect(canPreview("message.eml")).toBe(true);
    expect(canPreview("mail.msg")).toBe(true);
    expect(canPreview("archive.mbox")).toBe(true);
  });

  // PST files
  it("returns true for PST files", () => {
    expect(canPreview("outlook.pst")).toBe(true);
    expect(canPreview("archive.ost")).toBe(true);
  });

  // Plist files
  it("returns true for plist files", () => {
    expect(canPreview("info.plist")).toBe(true);
  });

  // Binary executables
  it("returns true for binary executables", () => {
    expect(canPreview("program.exe")).toBe(true);
    expect(canPreview("library.dll")).toBe(true);
    expect(canPreview("binary.elf")).toBe(true);
  });

  // Database files
  it("returns true for database files", () => {
    expect(canPreview("data.db")).toBe(true);
    expect(canPreview("data.sqlite")).toBe(true);
    expect(canPreview("data.sqlite3")).toBe(true);
  });

  // Registry hives
  it("returns true for registry hive files", () => {
    expect(canPreview("NTUSER.DAT")).toBe(true);
    expect(canPreview("SYSTEM")).toBe(true);
    expect(canPreview("SOFTWARE")).toBe(true);
    expect(canPreview("SAM")).toBe(true);
  });

  // Non-previewable files
  it("returns false for unknown/non-previewable files", () => {
    // .bin IS previewable (binary executable viewer)
    expect(canPreview("data.bin")).toBe(true);
    expect(canPreview("archive.zip")).toBe(false);
    expect(canPreview("image.e01")).toBe(false);
    expect(canPreview("file.ad1")).toBe(false);
    expect(canPreview("dump.mem")).toBe(false);
  });

  it("returns false for files without extensions", () => {
    // Note: some registry hives have no extension but match by name
    // Testing a random name that doesn't match
    expect(canPreview("randomfile")).toBe(false);
  });
});
