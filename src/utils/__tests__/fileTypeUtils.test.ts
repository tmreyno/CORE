// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  isImage,
  isVideo,
  isAudio,
  isDocument,
  isSpreadsheet,
  isCode,
  isDatabase,
  isArchive,
  isPdf,
  isEmail,
  isPlist,
  isBinaryExecutable,
  detectFileType,
} from "../fileTypeUtils";

describe("fileTypeUtils", () => {
  // ===========================================================================
  // isImage
  // ===========================================================================
  describe("isImage", () => {
    it("detects common image formats", () => {
      expect(isImage("photo.jpg")).toBe(true);
      expect(isImage("photo.jpeg")).toBe(true);
      expect(isImage("photo.png")).toBe(true);
      expect(isImage("photo.gif")).toBe(true);
      expect(isImage("photo.bmp")).toBe(true);
      expect(isImage("photo.webp")).toBe(true);
      expect(isImage("photo.svg")).toBe(true);
    });

    it("detects professional/RAW formats", () => {
      expect(isImage("photo.tiff")).toBe(true);
      expect(isImage("photo.heic")).toBe(true);
      expect(isImage("photo.cr2")).toBe(true);
      expect(isImage("photo.nef")).toBe(true);
      expect(isImage("photo.dng")).toBe(true);
    });

    it("is case-insensitive", () => {
      expect(isImage("PHOTO.JPG")).toBe(true);
      expect(isImage("Photo.PNG")).toBe(true);
    });

    it("returns false for non-images", () => {
      expect(isImage("document.pdf")).toBe(false);
      expect(isImage("video.mp4")).toBe(false);
      expect(isImage("README")).toBe(false);
    });

    it("handles paths with directories", () => {
      expect(isImage("/path/to/photo.jpg")).toBe(true);
    });
  });

  // ===========================================================================
  // isVideo
  // ===========================================================================
  describe("isVideo", () => {
    it("detects common video formats", () => {
      expect(isVideo("clip.mp4")).toBe(true);
      expect(isVideo("clip.avi")).toBe(true);
      expect(isVideo("clip.mov")).toBe(true);
      expect(isVideo("clip.mkv")).toBe(true);
      expect(isVideo("clip.webm")).toBe(true);
    });

    it("returns false for non-video", () => {
      expect(isVideo("photo.jpg")).toBe(false);
      expect(isVideo("song.mp3")).toBe(false);
    });
  });

  // ===========================================================================
  // isAudio
  // ===========================================================================
  describe("isAudio", () => {
    it("detects common audio formats", () => {
      expect(isAudio("song.mp3")).toBe(true);
      expect(isAudio("song.wav")).toBe(true);
      expect(isAudio("song.flac")).toBe(true);
      expect(isAudio("song.aac")).toBe(true);
      expect(isAudio("song.ogg")).toBe(true);
      expect(isAudio("song.m4a")).toBe(true);
    });

    it("returns false for non-audio", () => {
      expect(isAudio("video.mp4")).toBe(false);
      expect(isAudio("photo.jpg")).toBe(false);
    });
  });

  // ===========================================================================
  // isDocument
  // ===========================================================================
  describe("isDocument", () => {
    it("detects PDF", () => {
      expect(isDocument("file.pdf")).toBe(true);
    });

    it("detects Office formats", () => {
      expect(isDocument("file.doc")).toBe(true);
      expect(isDocument("file.docx")).toBe(true);
      expect(isDocument("file.xls")).toBe(true);
      expect(isDocument("file.xlsx")).toBe(true);
      expect(isDocument("file.ppt")).toBe(true);
      expect(isDocument("file.pptx")).toBe(true);
    });

    it("detects text formats", () => {
      expect(isDocument("file.txt")).toBe(true);
      expect(isDocument("file.md")).toBe(true);
      expect(isDocument("file.rtf")).toBe(true);
      expect(isDocument("file.csv")).toBe(true);
    });

    it("returns false for non-documents", () => {
      expect(isDocument("photo.jpg")).toBe(false);
      expect(isDocument("code.rs")).toBe(false);
    });
  });

  // ===========================================================================
  // isSpreadsheet
  // ===========================================================================
  describe("isSpreadsheet", () => {
    it("detects spreadsheet formats", () => {
      expect(isSpreadsheet("data.xlsx")).toBe(true);
      expect(isSpreadsheet("data.xls")).toBe(true);
      expect(isSpreadsheet("data.csv")).toBe(true);
      expect(isSpreadsheet("data.ods")).toBe(true);
    });

    it("returns false for non-spreadsheets", () => {
      expect(isSpreadsheet("doc.pdf")).toBe(false);
      expect(isSpreadsheet("photo.png")).toBe(false);
    });
  });

  // ===========================================================================
  // isCode
  // ===========================================================================
  describe("isCode", () => {
    it("detects source code files", () => {
      expect(isCode("app.ts")).toBe(true);
      expect(isCode("app.tsx")).toBe(true);
      expect(isCode("main.rs")).toBe(true);
      expect(isCode("script.py")).toBe(true);
      expect(isCode("main.go")).toBe(true);
      expect(isCode("Program.cs")).toBe(true);
    });

    it("detects config/data files as code", () => {
      expect(isCode("config.json")).toBe(true);
      expect(isCode("config.yaml")).toBe(true);
      expect(isCode("config.toml")).toBe(true);
      expect(isCode("style.css")).toBe(true);
    });

    it("detects shell scripts", () => {
      expect(isCode("script.sh")).toBe(true);
      expect(isCode("script.bash")).toBe(true);
      expect(isCode("script.ps1")).toBe(true);
    });

    it("returns false for non-code", () => {
      expect(isCode("photo.jpg")).toBe(false);
      expect(isCode("README")).toBe(false);
    });
  });

  // ===========================================================================
  // isDatabase
  // ===========================================================================
  describe("isDatabase", () => {
    it("detects database files", () => {
      expect(isDatabase("data.db")).toBe(true);
      expect(isDatabase("data.sqlite")).toBe(true);
      expect(isDatabase("data.sqlite3")).toBe(true);
      expect(isDatabase("data.mdb")).toBe(true);
    });

    it("returns false for non-databases", () => {
      expect(isDatabase("doc.pdf")).toBe(false);
    });
  });

  // ===========================================================================
  // isArchive
  // ===========================================================================
  describe("isArchive", () => {
    it("detects archive formats", () => {
      expect(isArchive("data.zip")).toBe(true);
      expect(isArchive("data.7z")).toBe(true);
      expect(isArchive("data.rar")).toBe(true);
      expect(isArchive("data.tar")).toBe(true);
      expect(isArchive("data.gz")).toBe(true);
    });

    it("returns false for non-archives", () => {
      expect(isArchive("doc.pdf")).toBe(false);
      expect(isArchive("photo.png")).toBe(false);
    });
  });

  // ===========================================================================
  // isPdf
  // ===========================================================================
  describe("isPdf", () => {
    it("detects PDF files", () => {
      expect(isPdf("document.pdf")).toBe(true);
      expect(isPdf("DOCUMENT.PDF")).toBe(true);
    });

    it("returns false for non-PDF", () => {
      expect(isPdf("document.doc")).toBe(false);
    });
  });

  // ===========================================================================
  // isEmail
  // ===========================================================================
  describe("isEmail", () => {
    it("detects email formats", () => {
      expect(isEmail("message.eml")).toBe(true);
      expect(isEmail("mailbox.mbox")).toBe(true);
      expect(isEmail("outlook.msg")).toBe(true);
    });

    it("is case-insensitive", () => {
      expect(isEmail("MESSAGE.EML")).toBe(true);
      expect(isEmail("Mailbox.MBOX")).toBe(true);
    });

    it("handles paths with directories", () => {
      expect(isEmail("/evidence/email/inbox.eml")).toBe(true);
    });

    it("returns false for non-email files", () => {
      expect(isEmail("document.pdf")).toBe(false);
      expect(isEmail("photo.jpg")).toBe(false);
      expect(isEmail("data.csv")).toBe(false);
    });
  });

  // ===========================================================================
  // isPlist
  // ===========================================================================
  describe("isPlist", () => {
    it("detects plist formats", () => {
      expect(isPlist("Info.plist")).toBe(true);
      expect(isPlist("profile.mobileprovision")).toBe(true);
    });

    it("is case-insensitive", () => {
      expect(isPlist("Info.PLIST")).toBe(true);
    });

    it("handles paths with directories", () => {
      expect(isPlist("/Library/Preferences/com.apple.finder.plist")).toBe(true);
    });

    it("returns false for non-plist files", () => {
      expect(isPlist("config.json")).toBe(false);
      expect(isPlist("settings.xml")).toBe(false);
    });
  });

  // ===========================================================================
  // isBinaryExecutable
  // ===========================================================================
  describe("isBinaryExecutable", () => {
    it("detects Windows executable formats", () => {
      expect(isBinaryExecutable("program.exe")).toBe(true);
      expect(isBinaryExecutable("library.dll")).toBe(true);
      expect(isBinaryExecutable("driver.sys")).toBe(true);
      expect(isBinaryExecutable("driver.drv")).toBe(true);
      expect(isBinaryExecutable("screensaver.scr")).toBe(true);
    });

    it("detects Linux/Unix executable formats", () => {
      expect(isBinaryExecutable("libfoo.so")).toBe(true);
      expect(isBinaryExecutable("program.elf")).toBe(true);
      expect(isBinaryExecutable("program.bin")).toBe(true);
    });

    it("detects macOS executable formats", () => {
      expect(isBinaryExecutable("libfoo.dylib")).toBe(true);
    });

    it("is case-insensitive", () => {
      expect(isBinaryExecutable("PROGRAM.EXE")).toBe(true);
      expect(isBinaryExecutable("Library.DLL")).toBe(true);
    });

    it("returns false for non-executables", () => {
      expect(isBinaryExecutable("document.pdf")).toBe(false);
      expect(isBinaryExecutable("script.py")).toBe(false);
      expect(isBinaryExecutable("photo.jpg")).toBe(false);
    });
  });

  // ===========================================================================
  // detectFileType
  // ===========================================================================
  describe("detectFileType", () => {
    it("detects forensic containers", () => {
      expect(detectFileType("evidence.ad1")).toBe("container");
      expect(detectFileType("evidence.e01")).toBe("container");
      expect(detectFileType("evidence.l01")).toBe("container");
      expect(detectFileType("evidence.ufd")).toBe("container");
      expect(detectFileType("disk.dd")).toBe("container");
      expect(detectFileType("disk.raw")).toBe("container");
    });

    it("detects media types", () => {
      expect(detectFileType("photo.jpg")).toBe("image");
      expect(detectFileType("clip.mp4")).toBe("video");
      expect(detectFileType("song.mp3")).toBe("audio");
    });

    it("detects documents and spreadsheets", () => {
      expect(detectFileType("doc.pdf")).toBe("document");
      expect(detectFileType("doc.docx")).toBe("document");
      // Spreadsheets are checked before documents
      expect(detectFileType("data.xlsx")).toBe("spreadsheet");
      expect(detectFileType("data.csv")).toBe("spreadsheet");
    });

    it("detects email files", () => {
      expect(detectFileType("message.eml")).toBe("email");
      expect(detectFileType("mailbox.mbox")).toBe("email");
    });

    it("detects plist files", () => {
      expect(detectFileType("Info.plist")).toBe("plist");
      expect(detectFileType("profile.mobileprovision")).toBe("plist");
    });

    it("detects binary executables", () => {
      expect(detectFileType("program.exe")).toBe("binary");
      expect(detectFileType("library.dll")).toBe("binary");
      expect(detectFileType("libfoo.so")).toBe("binary");
      expect(detectFileType("libfoo.dylib")).toBe("binary");
    });

    it("detects code", () => {
      expect(detectFileType("app.ts")).toBe("code");
      expect(detectFileType("main.rs")).toBe("code");
    });

    it("detects databases", () => {
      expect(detectFileType("data.sqlite")).toBe("database");
    });

    it("detects archives", () => {
      expect(detectFileType("backup.zip")).toBe("archive");
      expect(detectFileType("backup.7z")).toBe("archive");
    });

    it("returns unknown for unrecognized extensions", () => {
      expect(detectFileType("file.xyz")).toBe("unknown");
      expect(detectFileType("README")).toBe("unknown");
    });
  });
});
