// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

import { describe, it, expect } from "vitest";
import {
  isVfsContainer,
  isUnsupportedVfsContainer,
  isL01Container,
  isE01Container,
  isAd1Container,
  isArchiveContainer,
  isUfedContainer,
  isMemoryDump,
  isMemoryDumpFile,
  isContainerFile,
  getContainerType,
  getContainerCategory,
  isNestedContainerFile,
  getNestedContainerType,
  VFS_CONTAINER_TYPES,
  ARCHIVE_CONTAINER_TYPES,
  UFED_CONTAINER_TYPES,
  MEMORY_DUMP_TYPES,
  CONTAINER_EXTENSIONS,
} from "../containerDetection";

// =============================================================================
// Constants
// =============================================================================

describe("containerDetection constants", () => {
  it("VFS_CONTAINER_TYPES includes EnCase and raw formats", () => {
    expect(VFS_CONTAINER_TYPES).toContain("e01");
    expect(VFS_CONTAINER_TYPES).toContain("ex01");
    expect(VFS_CONTAINER_TYPES).toContain("raw");
    expect(VFS_CONTAINER_TYPES).toContain("dd");
    expect(VFS_CONTAINER_TYPES).toContain("img");
    expect(VFS_CONTAINER_TYPES).toContain("001");
    expect(VFS_CONTAINER_TYPES).toContain("l01");
  });

  it("ARCHIVE_CONTAINER_TYPES includes standard archive formats", () => {
    expect(ARCHIVE_CONTAINER_TYPES).toContain("zip");
    expect(ARCHIVE_CONTAINER_TYPES).toContain("7z");
    expect(ARCHIVE_CONTAINER_TYPES).toContain("rar");
    expect(ARCHIVE_CONTAINER_TYPES).toContain("tar");
    expect(ARCHIVE_CONTAINER_TYPES).toContain("dmg");
  });

  it("UFED_CONTAINER_TYPES includes all UFED variants", () => {
    expect(UFED_CONTAINER_TYPES).toContain("ufed");
    expect(UFED_CONTAINER_TYPES).toContain("ufd");
    expect(UFED_CONTAINER_TYPES).toContain("ufdr");
    expect(UFED_CONTAINER_TYPES).toContain("ufdx");
  });

  it("MEMORY_DUMP_TYPES has memory dump identifiers", () => {
    expect(MEMORY_DUMP_TYPES).toContain("memory");
    expect(MEMORY_DUMP_TYPES).toContain("memdump");
  });

  it("CONTAINER_EXTENSIONS covers forensic container extensions", () => {
    expect(CONTAINER_EXTENSIONS).toContain("ad1");
    expect(CONTAINER_EXTENSIONS).toContain("e01");
    expect(CONTAINER_EXTENSIONS).toContain("ufd");
  });
});

// =============================================================================
// isVfsContainer
// =============================================================================

describe("isVfsContainer", () => {
  it("returns true for E01 formats", () => {
    expect(isVfsContainer("e01")).toBe(true);
    expect(isVfsContainer("E01")).toBe(true);
    expect(isVfsContainer("ex01")).toBe(true);
    expect(isVfsContainer("ewf")).toBe(true);
    expect(isVfsContainer("encase")).toBe(true);
  });

  it("returns true for raw image formats", () => {
    expect(isVfsContainer("raw")).toBe(true);
    expect(isVfsContainer("dd")).toBe(true);
    expect(isVfsContainer("img")).toBe(true);
    expect(isVfsContainer("001")).toBe(true);
    expect(isVfsContainer("raw image")).toBe(true);
  });

  it("returns true for L01 logical formats", () => {
    expect(isVfsContainer("l01")).toBe(true);
    expect(isVfsContainer("lx01")).toBe(true);
    expect(isVfsContainer("lvf")).toBe(true);
  });

  it("returns true for ISO format", () => {
    expect(isVfsContainer("iso")).toBe(true);
    expect(isVfsContainer("iso 9660")).toBe(true);
  });

  it("returns false for archives", () => {
    expect(isVfsContainer("zip")).toBe(false);
    expect(isVfsContainer("7z")).toBe(false);
    expect(isVfsContainer("rar")).toBe(false);
  });

  it("returns false for AD1", () => {
    expect(isVfsContainer("ad1")).toBe(false);
  });

  it("is case-insensitive", () => {
    expect(isVfsContainer("E01")).toBe(true);
    expect(isVfsContainer("RAW")).toBe(true);
    expect(isVfsContainer("ISO")).toBe(true);
  });
});

// =============================================================================
// isUnsupportedVfsContainer
// =============================================================================

describe("isUnsupportedVfsContainer", () => {
  it("returns false for currently supported formats", () => {
    expect(isUnsupportedVfsContainer("e01")).toBe(false);
    expect(isUnsupportedVfsContainer("raw")).toBe(false);
    expect(isUnsupportedVfsContainer("dmg")).toBe(false);
  });
});

// =============================================================================
// isL01Container
// =============================================================================

describe("isL01Container", () => {
  it("detects L01 logical evidence types", () => {
    expect(isL01Container("l01")).toBe(true);
    expect(isL01Container("L01")).toBe(true);
    expect(isL01Container("lx01")).toBe(true);
    expect(isL01Container("lvf")).toBe(true);
  });

  it("rejects non-L01 types", () => {
    expect(isL01Container("e01")).toBe(false);
    expect(isL01Container("ad1")).toBe(false);
    expect(isL01Container("zip")).toBe(false);
  });
});

// =============================================================================
// isE01Container
// =============================================================================

describe("isE01Container", () => {
  it("detects E01/EnCase formats", () => {
    expect(isE01Container("e01")).toBe(true);
    expect(isE01Container("E01")).toBe(true);
    expect(isE01Container("ex01")).toBe(true);
    expect(isE01Container("ewf")).toBe(true);
    expect(isE01Container("encase")).toBe(true);
  });

  it("rejects non-E01 types", () => {
    expect(isE01Container("ad1")).toBe(false);
    expect(isE01Container("l01")).toBe(false);
    expect(isE01Container("zip")).toBe(false);
  });
});

// =============================================================================
// isAd1Container
// =============================================================================

describe("isAd1Container", () => {
  it("detects AD1 containers", () => {
    expect(isAd1Container("ad1")).toBe(true);
    expect(isAd1Container("AD1")).toBe(true);
    expect(isAd1Container("Ad1")).toBe(true);
  });

  it("rejects non-AD1 types", () => {
    expect(isAd1Container("e01")).toBe(false);
    expect(isAd1Container("zip")).toBe(false);
  });
});

// =============================================================================
// isArchiveContainer
// =============================================================================

describe("isArchiveContainer", () => {
  it("detects standard archive types", () => {
    expect(isArchiveContainer("zip")).toBe(true);
    expect(isArchiveContainer("7z")).toBe(true);
    expect(isArchiveContainer("7-zip")).toBe(true);
    expect(isArchiveContainer("rar")).toBe(true);
    expect(isArchiveContainer("tar")).toBe(true);
    expect(isArchiveContainer("archive")).toBe(true);
  });

  it("detects compressed archive types", () => {
    expect(isArchiveContainer("gz")).toBe(true);
    expect(isArchiveContainer("bz2")).toBe(true);
    expect(isArchiveContainer("xz")).toBe(true);
    expect(isArchiveContainer("zst")).toBe(true);
  });

  it("detects combined tar archives", () => {
    expect(isArchiveContainer("tar.gz")).toBe(true);
    expect(isArchiveContainer("tgz")).toBe(true);
    expect(isArchiveContainer("tar.xz")).toBe(true);
  });

  it("detects DMG as archive", () => {
    expect(isArchiveContainer("dmg")).toBe(true);
  });

  it("is case-insensitive", () => {
    expect(isArchiveContainer("ZIP")).toBe(true);
    expect(isArchiveContainer("RAR")).toBe(true);
  });
});

// =============================================================================
// isUfedContainer
// =============================================================================

describe("isUfedContainer", () => {
  it("detects UFED container types", () => {
    expect(isUfedContainer("ufed")).toBe(true);
    expect(isUfedContainer("ufd")).toBe(true);
    expect(isUfedContainer("ufdr")).toBe(true);
    expect(isUfedContainer("ufdx")).toBe(true);
  });

  it("is case-insensitive", () => {
    expect(isUfedContainer("UFED")).toBe(true);
    expect(isUfedContainer("UFD")).toBe(true);
  });

  it("rejects non-UFED types", () => {
    expect(isUfedContainer("ad1")).toBe(false);
    expect(isUfedContainer("e01")).toBe(false);
  });
});

// =============================================================================
// isMemoryDump / isMemoryDumpFile
// =============================================================================

describe("isMemoryDump", () => {
  it("detects memory dump types", () => {
    expect(isMemoryDump("memory")).toBe(true);
    expect(isMemoryDump("memdump")).toBe(true);
    expect(isMemoryDump("ramdump")).toBe(true);
  });

  it("rejects non-memory types", () => {
    expect(isMemoryDump("e01")).toBe(false);
    expect(isMemoryDump("ad1")).toBe(false);
  });
});

describe("isMemoryDumpFile", () => {
  it("detects memory dump filenames", () => {
    expect(isMemoryDumpFile("capture_mem.raw")).toBe(true);
    expect(isMemoryDumpFile("system.mem")).toBe(true);
    expect(isMemoryDumpFile("snapshot.vmem")).toBe(true);
    expect(isMemoryDumpFile("crash.dmp")).toBe(true);
    expect(isMemoryDumpFile("system_memdump.bin")).toBe(true);
  });

  it("is case-insensitive", () => {
    expect(isMemoryDumpFile("System.VMEM")).toBe(true);
    expect(isMemoryDumpFile("CRASH.DMP")).toBe(true);
  });

  it("rejects non-memory filenames", () => {
    expect(isMemoryDumpFile("document.pdf")).toBe(false);
    expect(isMemoryDumpFile("image.jpg")).toBe(false);
    expect(isMemoryDumpFile("archive.zip")).toBe(false);
  });
});

// =============================================================================
// isContainerFile / getContainerType
// =============================================================================

describe("isContainerFile", () => {
  it("detects forensic container files by extension", () => {
    expect(isContainerFile("evidence.ad1")).toBe(true);
    expect(isContainerFile("disk.E01")).toBe(true);
    expect(isContainerFile("image.dd")).toBe(true);
    expect(isContainerFile("backup.zip")).toBe(true);
    expect(isContainerFile("phone.ufd")).toBe(true);
    expect(isContainerFile("disk.dmg")).toBe(true);
  });

  it("rejects non-container files", () => {
    expect(isContainerFile("document.pdf")).toBe(false);
    expect(isContainerFile("image.jpg")).toBe(false);
    expect(isContainerFile("script.py")).toBe(false);
  });

  it("is case-insensitive", () => {
    expect(isContainerFile("EVIDENCE.AD1")).toBe(true);
    expect(isContainerFile("Disk.E01")).toBe(true);
  });
});

describe("getContainerType", () => {
  it("extracts container type from filename", () => {
    expect(getContainerType("evidence.ad1")).toBe("ad1");
    expect(getContainerType("disk.e01")).toBe("e01");
    expect(getContainerType("backup.zip")).toBe("zip");
    expect(getContainerType("phone.ufd")).toBe("ufd");
  });

  it("returns 'unknown' for non-container files", () => {
    expect(getContainerType("document.pdf")).toBe("unknown");
    expect(getContainerType("image.jpg")).toBe("unknown");
  });

  it("is case-insensitive", () => {
    expect(getContainerType("EVIDENCE.AD1")).toBe("ad1");
  });
});

// =============================================================================
// getContainerCategory
// =============================================================================

describe("getContainerCategory", () => {
  it("returns 'ad1' for AD1 containers", () => {
    expect(getContainerCategory("ad1")).toBe("ad1");
  });

  it("returns 'vfs' for VFS containers", () => {
    expect(getContainerCategory("e01")).toBe("vfs");
    expect(getContainerCategory("raw")).toBe("vfs");
    expect(getContainerCategory("dd")).toBe("vfs");
  });

  it("returns 'archive' for archive containers", () => {
    expect(getContainerCategory("zip")).toBe("archive");
    expect(getContainerCategory("7z")).toBe("archive");
    expect(getContainerCategory("dmg")).toBe("archive");
  });

  it("returns 'ufed' for UFED containers", () => {
    expect(getContainerCategory("ufed")).toBe("ufed");
    expect(getContainerCategory("ufd")).toBe("ufed");
  });

  it("returns 'memory' for memory dumps", () => {
    expect(getContainerCategory("memory")).toBe("memory");
    expect(getContainerCategory("memdump")).toBe("memory");
  });

  it("returns 'unknown' for unrecognized types", () => {
    expect(getContainerCategory("xyz")).toBe("unknown");
    expect(getContainerCategory("pdf")).toBe("unknown");
  });

  it("prioritizes AD1 over other categories", () => {
    // AD1 check comes first in the function
    expect(getContainerCategory("ad1")).toBe("ad1");
  });
});

// =============================================================================
// isNestedContainerFile / getNestedContainerType
// =============================================================================

describe("isNestedContainerFile", () => {
  it("detects nested container files", () => {
    expect(isNestedContainerFile("inner.ad1")).toBe(true);
    expect(isNestedContainerFile("nested.e01")).toBe(true);
    expect(isNestedContainerFile("archive.zip")).toBe(true);
    expect(isNestedContainerFile("disk.dmg")).toBe(true);
    expect(isNestedContainerFile("phone.ufd")).toBe(true);
  });

  it("rejects non-container files", () => {
    expect(isNestedContainerFile("file.txt")).toBe(false);
    expect(isNestedContainerFile("image.png")).toBe(false);
  });
});

describe("getNestedContainerType", () => {
  it("returns the container type for nested files", () => {
    expect(getNestedContainerType("inner.ad1")).toBe("ad1");
    expect(getNestedContainerType("disk.e01")).toBe("e01");
    expect(getNestedContainerType("archive.zip")).toBe("zip");
  });

  it("returns null for non-container files", () => {
    expect(getNestedContainerType("file.txt")).toBeNull();
    expect(getNestedContainerType("image.png")).toBeNull();
  });
});
