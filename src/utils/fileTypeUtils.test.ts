// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  IMAGE_EXTENSIONS,
  VIDEO_EXTENSIONS,
  AUDIO_EXTENSIONS,
  DOCUMENT_EXTENSIONS,
  DATABASE_EXTENSIONS,
  REGISTRY_HIVE_NAMES,
  ARCHIVE_EXTENSIONS,
  BINARY_EXECUTABLE_EXTENSIONS,
  CONFIG_EXTENSIONS,
  isImage,
  isVideo,
  isAudio,
  isDocument,
  isSpreadsheet,
  isTextDocument,
  isOffice,
  isCode,
  isDatabase,
  isRegistryHive,
  isArchive,
  isEmail,
  isPlist,
  isBinaryExecutable,
  isConfig,
  isPdf,
  detectFileType,
} from "./fileTypeUtils";

// =============================================================================
// Extension Arrays - Sanity Checks
// =============================================================================

describe("Extension Arrays", () => {
  it("IMAGE_EXTENSIONS contains common image formats", () => {
    expect(IMAGE_EXTENSIONS).toContain("jpg");
    expect(IMAGE_EXTENSIONS).toContain("jpeg");
    expect(IMAGE_EXTENSIONS).toContain("png");
    expect(IMAGE_EXTENSIONS).toContain("gif");
    expect(IMAGE_EXTENSIONS).toContain("bmp");
    expect(IMAGE_EXTENSIONS).toContain("webp");
    expect(IMAGE_EXTENSIONS).toContain("svg");
  });

  it("IMAGE_EXTENSIONS contains RAW camera formats", () => {
    expect(IMAGE_EXTENSIONS).toContain("cr2");
    expect(IMAGE_EXTENSIONS).toContain("nef");
    expect(IMAGE_EXTENSIONS).toContain("dng");
    expect(IMAGE_EXTENSIONS).toContain("arw");
    expect(IMAGE_EXTENSIONS).toContain("orf");
    expect(IMAGE_EXTENSIONS).toContain("rw2");
  });

  it("IMAGE_EXTENSIONS contains modern formats", () => {
    expect(IMAGE_EXTENSIONS).toContain("avif");
    expect(IMAGE_EXTENSIONS).toContain("heic");
    expect(IMAGE_EXTENSIONS).toContain("heif");
  });

  it("VIDEO_EXTENSIONS contains common video formats", () => {
    expect(VIDEO_EXTENSIONS).toContain("mp4");
    expect(VIDEO_EXTENSIONS).toContain("avi");
    expect(VIDEO_EXTENSIONS).toContain("mov");
    expect(VIDEO_EXTENSIONS).toContain("mkv");
  });

  it("AUDIO_EXTENSIONS contains common audio formats", () => {
    expect(AUDIO_EXTENSIONS).toContain("mp3");
    expect(AUDIO_EXTENSIONS).toContain("wav");
    expect(AUDIO_EXTENSIONS).toContain("flac");
    expect(AUDIO_EXTENSIONS).toContain("aac");
  });

  it("DOCUMENT_EXTENSIONS contains office and text formats", () => {
    expect(DOCUMENT_EXTENSIONS).toContain("pdf");
    expect(DOCUMENT_EXTENSIONS).toContain("docx");
    expect(DOCUMENT_EXTENSIONS).toContain("xlsx");
    expect(DOCUMENT_EXTENSIONS).toContain("txt");
    expect(DOCUMENT_EXTENSIONS).toContain("md");
  });

  it("DATABASE_EXTENSIONS contains database formats", () => {
    expect(DATABASE_EXTENSIONS).toContain("db");
    expect(DATABASE_EXTENSIONS).toContain("db3");
    expect(DATABASE_EXTENSIONS).toContain("sqlite");
    expect(DATABASE_EXTENSIONS).toContain("sqlite3");
    expect(DATABASE_EXTENSIONS).toContain("sqlitedb");
  });

  it("REGISTRY_HIVE_NAMES contains Windows registry hive names", () => {
    expect(REGISTRY_HIVE_NAMES).toContain("ntuser.dat");
    expect(REGISTRY_HIVE_NAMES).toContain("sam");
    expect(REGISTRY_HIVE_NAMES).toContain("system");
    expect(REGISTRY_HIVE_NAMES).toContain("software");
    expect(REGISTRY_HIVE_NAMES).toContain("security");
  });

  it("ARCHIVE_EXTENSIONS contains archive formats", () => {
    expect(ARCHIVE_EXTENSIONS).toContain("zip");
    expect(ARCHIVE_EXTENSIONS).toContain("7z");
    expect(ARCHIVE_EXTENSIONS).toContain("rar");
    expect(ARCHIVE_EXTENSIONS).toContain("tar");
    expect(ARCHIVE_EXTENSIONS).toContain("gz");
  });

  it("BINARY_EXECUTABLE_EXTENSIONS contains executable formats", () => {
    expect(BINARY_EXECUTABLE_EXTENSIONS).toContain("exe");
    expect(BINARY_EXECUTABLE_EXTENSIONS).toContain("dll");
    expect(BINARY_EXECUTABLE_EXTENSIONS).toContain("so");
    expect(BINARY_EXECUTABLE_EXTENSIONS).toContain("dylib");
  });

  it("CONFIG_EXTENSIONS contains config/settings formats", () => {
    expect(CONFIG_EXTENSIONS).toContain("log");
    expect(CONFIG_EXTENSIONS).toContain("ini");
    expect(CONFIG_EXTENSIONS).toContain("cfg");
    expect(CONFIG_EXTENSIONS).toContain("conf");
    expect(CONFIG_EXTENSIONS).toContain("properties");
    expect(CONFIG_EXTENSIONS).toContain("env");
    expect(CONFIG_EXTENSIONS).toContain("gitignore");
    expect(CONFIG_EXTENSIONS).toContain("editorconfig");
    expect(CONFIG_EXTENSIONS).toContain("dockerignore");
    expect(CONFIG_EXTENSIONS).toContain("npmrc");
  });
});

// =============================================================================
// Type Guards
// =============================================================================

describe("isImage", () => {
  it("returns true for common image extensions", () => {
    expect(isImage("photo.jpg")).toBe(true);
    expect(isImage("photo.jpeg")).toBe(true);
    expect(isImage("icon.png")).toBe(true);
    expect(isImage("anim.gif")).toBe(true);
    expect(isImage("logo.svg")).toBe(true);
    expect(isImage("banner.webp")).toBe(true);
  });

  it("is case-insensitive", () => {
    expect(isImage("photo.JPG")).toBe(true);
    expect(isImage("photo.Jpeg")).toBe(true);
    expect(isImage("photo.PNG")).toBe(true);
  });

  it("handles full paths", () => {
    expect(isImage("/path/to/photo.jpg")).toBe(true);
    expect(isImage("C:\\Users\\photo.png")).toBe(true);
  });

  it("returns true for RAW formats", () => {
    expect(isImage("shot.cr2")).toBe(true);
    expect(isImage("shot.nef")).toBe(true);
    expect(isImage("shot.arw")).toBe(true);
    expect(isImage("shot.dng")).toBe(true);
    expect(isImage("shot.orf")).toBe(true);
    expect(isImage("shot.rw2")).toBe(true);
  });

  it("returns true for modern image formats", () => {
    expect(isImage("photo.avif")).toBe(true);
    expect(isImage("photo.heic")).toBe(true);
    expect(isImage("photo.heif")).toBe(true);
  });

  it("returns false for non-image files", () => {
    expect(isImage("document.pdf")).toBe(false);
    expect(isImage("video.mp4")).toBe(false);
    expect(isImage("README")).toBe(false);
  });
});

describe("isVideo", () => {
  it("returns true for video extensions", () => {
    expect(isVideo("clip.mp4")).toBe(true);
    expect(isVideo("movie.avi")).toBe(true);
    expect(isVideo("recording.mov")).toBe(true);
    expect(isVideo("film.mkv")).toBe(true);
  });

  it("is case-insensitive", () => {
    expect(isVideo("clip.MP4")).toBe(true);
    expect(isVideo("clip.Avi")).toBe(true);
  });

  it("returns false for non-video files", () => {
    expect(isVideo("photo.jpg")).toBe(false);
    expect(isVideo("song.mp3")).toBe(false);
  });
});

describe("isAudio", () => {
  it("returns true for audio extensions", () => {
    expect(isAudio("song.mp3")).toBe(true);
    expect(isAudio("track.wav")).toBe(true);
    expect(isAudio("album.flac")).toBe(true);
    expect(isAudio("podcast.aac")).toBe(true);
    expect(isAudio("voice.m4a")).toBe(true);
  });

  it("returns false for non-audio files", () => {
    expect(isAudio("video.mp4")).toBe(false);
    expect(isAudio("photo.jpg")).toBe(false);
  });
});

describe("isDocument", () => {
  it("returns true for office documents", () => {
    expect(isDocument("report.pdf")).toBe(true);
    expect(isDocument("letter.docx")).toBe(true);
    expect(isDocument("letter.doc")).toBe(true);
    expect(isDocument("data.xlsx")).toBe(true);
    expect(isDocument("slides.pptx")).toBe(true);
  });

  it("returns true for text formats", () => {
    expect(isDocument("readme.txt")).toBe(true);
    expect(isDocument("notes.md")).toBe(true);
    expect(isDocument("format.rtf")).toBe(true);
  });

  it("returns true for data formats", () => {
    expect(isDocument("data.csv")).toBe(true);
    expect(isDocument("data.tsv")).toBe(true);
  });

  it("returns false for non-document files", () => {
    expect(isDocument("photo.jpg")).toBe(false);
    expect(isDocument("program.exe")).toBe(false);
  });
});

describe("isSpreadsheet", () => {
  it("returns true for spreadsheet extensions", () => {
    expect(isSpreadsheet("data.xlsx")).toBe(true);
    expect(isSpreadsheet("data.xls")).toBe(true);
    expect(isSpreadsheet("data.ods")).toBe(true);
    expect(isSpreadsheet("data.csv")).toBe(true);
    expect(isSpreadsheet("data.numbers")).toBe(true);
  });

  it("returns false for non-spreadsheet files", () => {
    expect(isSpreadsheet("doc.docx")).toBe(false);
    expect(isSpreadsheet("photo.jpg")).toBe(false);
  });
});

describe("isTextDocument", () => {
  it("returns true for text document extensions", () => {
    expect(isTextDocument("page.html")).toBe(true);
    expect(isTextDocument("page.htm")).toBe(true);
    expect(isTextDocument("notes.md")).toBe(true);
    expect(isTextDocument("readme.txt")).toBe(true);
  });

  it("returns false for office documents (handled by isOffice)", () => {
    expect(isTextDocument("doc.docx")).toBe(false);
    expect(isTextDocument("letter.rtf")).toBe(false);
    expect(isTextDocument("slides.pptx")).toBe(false);
    expect(isTextDocument("slides.ppt")).toBe(false);
    expect(isTextDocument("slides.odp")).toBe(false);
    expect(isTextDocument("document.odt")).toBe(false);
  });

  it("returns false for non-text documents", () => {
    expect(isTextDocument("data.xlsx")).toBe(false);
    expect(isTextDocument("photo.jpg")).toBe(false);
  });
});

describe("isOffice", () => {
  it("returns true for OOXML documents", () => {
    expect(isOffice("report.docx")).toBe(true);
    expect(isOffice("slides.pptx")).toBe(true);
  });

  it("returns true for legacy Office formats", () => {
    expect(isOffice("report.doc")).toBe(true);
    expect(isOffice("slides.ppt")).toBe(true);
  });

  it("returns true for OpenDocument formats", () => {
    expect(isOffice("document.odt")).toBe(true);
    expect(isOffice("slides.odp")).toBe(true);
  });

  it("returns true for RTF", () => {
    expect(isOffice("letter.rtf")).toBe(true);
  });

  it("returns false for non-office files", () => {
    expect(isOffice("data.xlsx")).toBe(false);
    expect(isOffice("photo.jpg")).toBe(false);
    expect(isOffice("page.html")).toBe(false);
    expect(isOffice("notes.md")).toBe(false);
  });
});

describe("isCode", () => {
  it("returns true for JavaScript/TypeScript", () => {
    expect(isCode("app.js")).toBe(true);
    expect(isCode("app.ts")).toBe(true);
    expect(isCode("component.tsx")).toBe(true);
    expect(isCode("component.jsx")).toBe(true);
  });

  it("returns true for Python", () => {
    expect(isCode("script.py")).toBe(true);
  });

  it("returns true for Rust", () => {
    expect(isCode("main.rs")).toBe(true);
  });

  it("returns true for data formats", () => {
    expect(isCode("config.json")).toBe(true);
    expect(isCode("config.yaml")).toBe(true);
    expect(isCode("config.toml")).toBe(true);
    expect(isCode("config.xml")).toBe(true);
  });

  it("returns true for web files", () => {
    expect(isCode("page.html")).toBe(true);
    expect(isCode("style.css")).toBe(true);
    expect(isCode("style.scss")).toBe(true);
  });

  it("returns true for shell scripts", () => {
    expect(isCode("run.sh")).toBe(true);
    expect(isCode("run.bash")).toBe(true);
    expect(isCode("script.ps1")).toBe(true);
  });

  it("returns false for non-code files", () => {
    expect(isCode("photo.jpg")).toBe(false);
    expect(isCode("song.mp3")).toBe(false);
  });
});

describe("isDatabase", () => {
  it("returns true for database extensions", () => {
    expect(isDatabase("data.db")).toBe(true);
    expect(isDatabase("data.db3")).toBe(true);
    expect(isDatabase("data.sqlite")).toBe(true);
    expect(isDatabase("data.sqlite3")).toBe(true);
    expect(isDatabase("data.sqlitedb")).toBe(true);
  });

  it("returns false for non-database files", () => {
    expect(isDatabase("data.csv")).toBe(false);
    expect(isDatabase("photo.jpg")).toBe(false);
    // Non-SQLite database formats are no longer in DATABASE_EXTENSIONS
    expect(isDatabase("legacy.mdb")).toBe(false);
    expect(isDatabase("legacy.accdb")).toBe(false);
  });
});

describe("isRegistryHive", () => {
  it("returns true for known registry hive names", () => {
    expect(isRegistryHive("NTUSER.DAT")).toBe(true);
    expect(isRegistryHive("ntuser.dat")).toBe(true);
    expect(isRegistryHive("SAM")).toBe(true);
    expect(isRegistryHive("SYSTEM")).toBe(true);
    expect(isRegistryHive("SOFTWARE")).toBe(true);
    expect(isRegistryHive("SECURITY")).toBe(true);
    expect(isRegistryHive("DEFAULT")).toBe(true);
    expect(isRegistryHive("Amcache.hve")).toBe(true);
  });

  it("handles full paths", () => {
    expect(isRegistryHive("/Windows/System32/config/SAM")).toBe(true);
    expect(isRegistryHive("C:\\Users\\user\\NTUSER.DAT")).toBe(true);
  });

  it("returns false for non-registry files", () => {
    expect(isRegistryHive("document.txt")).toBe(false);
    expect(isRegistryHive("NTUSER.DAT.LOG1")).toBe(false);
  });
});

describe("isArchive", () => {
  it("returns true for archive extensions", () => {
    expect(isArchive("backup.zip")).toBe(true);
    expect(isArchive("backup.7z")).toBe(true);
    expect(isArchive("backup.rar")).toBe(true);
    expect(isArchive("backup.tar")).toBe(true);
    expect(isArchive("backup.gz")).toBe(true);
    expect(isArchive("backup.xz")).toBe(true);
    expect(isArchive("backup.tgz")).toBe(true);
  });

  it("returns false for non-archive files", () => {
    expect(isArchive("photo.jpg")).toBe(false);
    expect(isArchive("document.pdf")).toBe(false);
  });
});

describe("isEmail", () => {
  it("returns true for email extensions", () => {
    expect(isEmail("message.eml")).toBe(true);
    expect(isEmail("mailbox.mbox")).toBe(true);
    expect(isEmail("outlook.msg")).toBe(true);
  });

  it("returns false for non-email files", () => {
    expect(isEmail("document.txt")).toBe(false);
    expect(isEmail("photo.jpg")).toBe(false);
  });
});

describe("isPlist", () => {
  it("returns true for plist extensions", () => {
    expect(isPlist("info.plist")).toBe(true);
    expect(isPlist("app.mobileprovision")).toBe(true);
  });

  it("returns false for non-plist files", () => {
    expect(isPlist("config.json")).toBe(false);
    expect(isPlist("data.xml")).toBe(false);
  });
});

describe("isBinaryExecutable", () => {
  it("returns true for executable extensions", () => {
    expect(isBinaryExecutable("app.exe")).toBe(true);
    expect(isBinaryExecutable("lib.dll")).toBe(true);
    expect(isBinaryExecutable("lib.so")).toBe(true);
    expect(isBinaryExecutable("lib.dylib")).toBe(true);
    expect(isBinaryExecutable("driver.sys")).toBe(true);
  });

  it("returns false for non-executable files", () => {
    expect(isBinaryExecutable("script.py")).toBe(false);
    expect(isBinaryExecutable("doc.pdf")).toBe(false);
  });
});

describe("isConfig", () => {
  it("returns true for config/settings file extensions", () => {
    expect(isConfig("app.log")).toBe(true);
    expect(isConfig("settings.ini")).toBe(true);
    expect(isConfig("server.cfg")).toBe(true);
    expect(isConfig("nginx.conf")).toBe(true);
    expect(isConfig("app.properties")).toBe(true);
    expect(isConfig("dev.env")).toBe(true);
    expect(isConfig("file.gitignore")).toBe(true);
    expect(isConfig("file.editorconfig")).toBe(true);
    expect(isConfig("file.eslintrc")).toBe(true);
    expect(isConfig("file.prettierrc")).toBe(true);
    expect(isConfig("file.dockerignore")).toBe(true);
    expect(isConfig("file.npmrc")).toBe(true);
    expect(isConfig("file.yarnrc")).toBe(true);
    expect(isConfig("file.hgignore")).toBe(true);
  });

  it("returns false for non-config files", () => {
    expect(isConfig("script.py")).toBe(false);
    expect(isConfig("photo.jpg")).toBe(false);
    expect(isConfig("data.json")).toBe(false);
  });

  it("is case-insensitive", () => {
    expect(isConfig("APP.LOG")).toBe(true);
    expect(isConfig("Settings.INI")).toBe(true);
  });
});

describe("isPdf", () => {
  it("returns true for PDF files", () => {
    expect(isPdf("document.pdf")).toBe(true);
    expect(isPdf("REPORT.PDF")).toBe(true);
  });

  it("returns false for non-PDF files", () => {
    expect(isPdf("document.docx")).toBe(false);
    expect(isPdf("photo.jpg")).toBe(false);
  });
});

// =============================================================================
// Edge Cases
// =============================================================================

describe("Type guard edge cases", () => {
  it("handles empty strings", () => {
    expect(isImage("")).toBe(false);
    expect(isVideo("")).toBe(false);
    expect(isAudio("")).toBe(false);
    expect(isDocument("")).toBe(false);
  });

  it("handles files without extensions", () => {
    expect(isImage("README")).toBe(false);
    expect(isDocument("Makefile")).toBe(false);
    expect(isCode("Dockerfile")).toBe(false);
  });

  it("handles dot-only filenames", () => {
    expect(isImage(".")).toBe(false);
    expect(isImage("file.")).toBe(false);
  });

  it("handles hidden files", () => {
    expect(isCode(".gitignore")).toBe(false); // "gitignore" is not in CODE_EXTENSIONS
    // ".jpg" has lastDot at 0, so getExtension returns "" → not an image
    expect(isImage(".jpg")).toBe(false);
  });
});

// =============================================================================
// detectFileType
// =============================================================================

describe("detectFileType", () => {
  it("detects forensic container types", () => {
    expect(detectFileType("evidence.ad1")).toBe("container");
    expect(detectFileType("disk.e01")).toBe("container");
    expect(detectFileType("disk.ex01")).toBe("container");
    expect(detectFileType("logical.l01")).toBe("container");
    expect(detectFileType("logical.lx01")).toBe("container");
    expect(detectFileType("phone.ufd")).toBe("container");
    expect(detectFileType("phone.ufdr")).toBe("container");
    expect(detectFileType("raw.dd")).toBe("container");
    expect(detectFileType("raw.raw")).toBe("container");
    expect(detectFileType("raw.img")).toBe("container");
  });

  it("detects image files", () => {
    expect(detectFileType("photo.jpg")).toBe("image");
    expect(detectFileType("icon.png")).toBe("image");
  });

  it("detects video files", () => {
    expect(detectFileType("clip.mp4")).toBe("video");
    expect(detectFileType("movie.avi")).toBe("video");
  });

  it("detects audio files", () => {
    expect(detectFileType("song.mp3")).toBe("audio");
    expect(detectFileType("track.wav")).toBe("audio");
  });

  it("detects email files", () => {
    expect(detectFileType("message.eml")).toBe("email");
    expect(detectFileType("inbox.mbox")).toBe("email");
  });

  it("detects PST/OST as email", () => {
    expect(detectFileType("outlook.pst")).toBe("email");
    expect(detectFileType("archive.ost")).toBe("email");
  });

  it("detects plist files", () => {
    expect(detectFileType("info.plist")).toBe("plist");
  });

  it("detects binary executables", () => {
    expect(detectFileType("app.exe")).toBe("binary");
    expect(detectFileType("lib.dll")).toBe("binary");
  });

  it("detects registry hives as binary", () => {
    expect(detectFileType("NTUSER.DAT")).toBe("binary");
    expect(detectFileType("SOFTWARE")).toBe("binary");
    expect(detectFileType("SYSTEM")).toBe("binary");
  });

  it("detects spreadsheets before generic documents", () => {
    // xlsx/xls/csv are in both SPREADSHEET and DOCUMENT arrays
    // spreadsheet should win
    expect(detectFileType("data.xlsx")).toBe("spreadsheet");
    expect(detectFileType("data.xls")).toBe("spreadsheet");
    expect(detectFileType("data.csv")).toBe("spreadsheet");
  });

  it("detects documents", () => {
    expect(detectFileType("report.pdf")).toBe("document");
    expect(detectFileType("letter.docx")).toBe("document");
    expect(detectFileType("readme.txt")).toBe("document");
  });

  it("detects code files", () => {
    expect(detectFileType("app.js")).toBe("code");
    expect(detectFileType("main.rs")).toBe("code");
    expect(detectFileType("script.py")).toBe("code");
    expect(detectFileType("app.scala")).toBe("code");
  });

  it("detects config files as code", () => {
    expect(detectFileType("settings.ini")).toBe("code");
    expect(detectFileType("config.cfg")).toBe("code");
    expect(detectFileType("app.env")).toBe("code");
  });

  it("detects database files", () => {
    expect(detectFileType("data.db")).toBe("database");
    expect(detectFileType("data.sqlite")).toBe("database");
  });

  it("detects archive files", () => {
    expect(detectFileType("backup.zip")).toBe("archive");
    expect(detectFileType("backup.7z")).toBe("archive");
  });

  it("returns unknown for unrecognized extensions", () => {
    expect(detectFileType("file.xyz")).toBe("unknown");
    expect(detectFileType("README")).toBe("unknown");
    expect(detectFileType("")).toBe("unknown");
  });

  it("is case-insensitive for containers", () => {
    expect(detectFileType("evidence.AD1")).toBe("container");
    expect(detectFileType("disk.E01")).toBe("container");
  });

  it("handles full paths", () => {
    expect(detectFileType("/evidence/disk.e01")).toBe("container");
    expect(detectFileType("C:\\Evidence\\photo.jpg")).toBe("image");
  });
});
