# EWF (Expert Witness Format) Complete Hex Reference

> Comprehensive documentation for E01/Ex01/L01/Lx01 forensic image formats

## Overview

EWF (Expert Witness Format) is a forensic disk imaging format created by Guidance Software (now OpenText). It supports compression, hashing, and rich metadata for forensic investigations.

### CORE-FFX Parser Notes

CORE-FFX implements EVF/LVF detection and metadata parsing for E01/Ex01 and L01/Lx01. SMART (.s01) formats may be detected during scans but are not parsed by the current EWF module.

### Format Variants

| Extension | Format | Description |
|-----------|--------|-------------|
| `.E01-.EFF` | EWF-E01 | Original EnCase format (physical images) |
| `.Ex01` | EWF2 | EnCase v7+ enhanced format |
| `.L01-.LFF` | EWF-L01 | Logical Evidence File (logical acquisition) |
| `.Lx01` | EWF2-L | EnCase v7+ logical format |
| `.s01-.sFF` | SMART | ASR Data SMART format |

---

## File Structure

### 1. Magic Signature

```
EWF v1 (E01/L01):
┌────────┬────────────────────────────────────────────────────────┐
│ Offset │ Bytes                                                  │
├────────┼────────────────────────────────────────────────────────┤
│ 0x00   │ 45 56 46 09 0D 0A FF 00  (E01: "EVF\x09\x0D\x0A\xFF\0")│
│        │ 4C 56 46 09 0D 0A FF 00  (L01: "LVF\x09\x0D\x0A\xFF\0")│
└────────┴────────────────────────────────────────────────────────┘

EWF v2 (Ex01/Lx01):
┌────────┬────────────────────────────────────────────────────────┐
│ Offset │ Bytes                                                  │
├────────┼────────────────────────────────────────────────────────┤
│ 0x00   │ 45 56 46 32 0D 0A 81 00  ("EVF2\x0D\x0A\x81\0")        │
│        │ 4C 56 46 32 0D 0A 81 00  ("LVF2\x0D\x0A\x81\0")        │
└────────┴────────────────────────────────────────────────────────┘
```

### 2. Segment Number

```
┌────────┬──────┬──────────────────────────────────────────────────┐
│ Offset │ Size │ Description                                      │
├────────┼──────┼──────────────────────────────────────────────────┤
│ 0x08   │ 1    │ Fields start marker (0x01)                       │
│ 0x09   │ 2    │ Segment number (uint16 LE)                       │
│        │      │ E01=1, E02=2, ... EFF=255                        │
│ 0x0B   │ 2    │ Reserved (0x00 0x00)                             │
└────────┴──────┴──────────────────────────────────────────────────┘
```

---

## Section Structure

EWF files are organized into sections. Each section has a 76-byte header:

### Section Header (76 bytes)

```
┌────────┬──────┬───────────────────────────────────────────────────┐
│ Offset │ Size │ Field                                             │
├────────┼──────┼───────────────────────────────────────────────────┤
│ 0x00   │ 16   │ Type (ASCII, null-padded)                         │
│        │      │ "header", "header2", "volume", "disk", "sectors", │
│        │      │ "table", "table2", "hash", "digest", "error2",    │
│        │      │ "session", "data", "next", "done"                 │
├────────┼──────┼───────────────────────────────────────────────────┤
│ 0x10   │ 8    │ Next section offset (uint64 LE, absolute)         │
│        │      │ 0 = no next section / last in file                │
├────────┼──────┼───────────────────────────────────────────────────┤
│ 0x18   │ 8    │ Section size (uint64 LE, includes 76-byte header) │
├────────┼──────┼───────────────────────────────────────────────────┤
│ 0x20   │ 40   │ Padding (zeros)                                   │
├────────┼──────┼───────────────────────────────────────────────────┤
│ 0x48   │ 4    │ Checksum (Adler-32 of bytes 0x00-0x47)            │
└────────┴──────┴───────────────────────────────────────────────────┘
```

### Section Types Reference

| Type | Purpose | Compression | Notes |
|------|---------|-------------|-------|
| `header` | Case metadata (ASCII) | zlib | Deprecated, use header2 |
| `header2` | Case metadata (UTF-16) | zlib | Preferred for case info |
| `volume` | Media information | No | Chunk count, sector info |
| `disk` | Disk geometry | No | EWF-S format only |
| `sectors` | Image data chunks | zlib | Bulk of file size |
| `table` | Chunk offset table | No | Maps chunks to offsets |
| `table2` | Backup offset table | No | Redundant copy |
| `hash` | MD5 hash | No | Legacy, use digest |
| `digest` | MD5/SHA1/SHA256 | No | Integrity verification |
| `error2` | Acquisition errors | No | Bad sector list |
| `session` | Optical disc sessions | No | CD/DVD specific |
| `next` | Points to next segment | No | Multi-file spanning |
| `done` | End of segment | No | Final marker |
| `data` | Generic data | zlib | EWF-L format |

---

## Detailed Section Specifications

### Header/Header2 Section (Case Metadata)

After decompression, contains newline-separated key=value pairs:

```
Format (tab-separated categories):
1<TAB>main
2<TAB>srce
3<TAB>sub

Category "main" fields:
┌─────┬────────────────────┬─────────────────────────────────────────┐
│ Key │ Name               │ Example                                 │
├─────┼────────────────────┼─────────────────────────────────────────┤
│ a   │ Description        │ "Hard drive from suspect laptop"        │
│ c   │ Case Number        │ "2024-CYBER-001"                        │
│ n   │ Evidence Number    │ "E001"                                  │
│ e   │ Examiner Name      │ "John Smith"                            │
│ t   │ Notes              │ "Acquired using write blocker"          │
│ av  │ Acquisition SW Ver │ "8.06.00"                               │
│ ov  │ Acquisition OS     │ "Windows 10 Pro"                        │
│ m   │ Acquired Date      │ "2024-01-15T10:30:00"                   │
│ u   │ System Date        │ "2024-01-15T10:30:00"                   │
│ p   │ Password Hash      │ (MD5 if encrypted)                      │
│ pid │ Process ID         │ "12345"                                 │
│ dc  │ Unknown            │ Various                                 │
│ ext │ Extents            │ Logical acquisition info                │
└─────┴────────────────────┴─────────────────────────────────────────┘

Category "srce" fields (source device info):
┌─────┬────────────────────┬─────────────────────────────────────────┐
│ Key │ Name               │ Example                                 │
├─────┼────────────────────┼─────────────────────────────────────────┤
│ p   │ (reserved)         │                                         │
│ n   │ Device Type        │ "Disk"                                  │
│ l   │ EWF-L identifier   │ Logical file path                       │
│ tb  │ Total bytes        │ "500107862016"                          │
│ lo  │ Logical offset     │ "0"                                     │
│ po  │ Physical offset    │ "0"                                     │
│ ah  │ Acquire hash       │ Hash during acquisition                 │
│ gu  │ GUID               │ "{GUID}"                                │
│ id  │ Device ID          │ "\\.\PhysicalDrive0"                    │
│ md  │ Model              │ "WDC WD5000AAKX"                        │
│ sn  │ Serial Number      │ "WD-WMAYUS123456"                       │
└─────┴────────────────────┴─────────────────────────────────────────┘

Category "sub" fields (sub-source info):
┌─────┬────────────────────┬─────────────────────────────────────────┐
│ Key │ Name               │ Example                                 │
├─────┼────────────────────┼─────────────────────────────────────────┤
│ p   │ (reserved)         │                                         │
│ n   │ Sub-source name    │ Partition name                          │
│ gu  │ GUID               │ Partition GUID                          │
│ id  │ Identifier         │ Partition ID                            │
└─────┴────────────────────┴─────────────────────────────────────────┘
```

### Volume Section

```
Volume Section Data (after 76-byte section header):
┌────────┬──────┬───────────────────────────────────────────────────┐
│ Offset │ Size │ Field                                             │
├────────┼──────┼───────────────────────────────────────────────────┤
│ 0x00   │ 4    │ Reserved (zeros)                                  │
│ 0x04   │ 4    │ Chunk count (uint32 LE)                           │
│ 0x08   │ 4    │ Sectors per chunk (uint32 LE, typically 64)       │
│ 0x0C   │ 4    │ Bytes per sector (uint32 LE, typically 512)       │
│ 0x10   │ 8    │ Sector count (uint64 LE)                          │
│ 0x18   │ 4    │ CHS Cylinders (legacy)                            │
│ 0x1C   │ 4    │ CHS Heads (legacy)                                │
│ 0x20   │ 4    │ CHS Sectors (legacy)                              │
│ 0x24   │ 4    │ Media type (0=removable, 1=fixed, 3=optical)      │
│ 0x28   │ 4    │ Unknown                                           │
│ 0x2C   │ 4    │ PAL entries (palm pilot)                          │
│ 0x30   │ 4    │ Unknown                                           │
│ 0x34   │ 4    │ SMART logs count                                  │
│ 0x38   │ 1    │ Compression level (0=none, 1-9=zlib levels)       │
│ 0x39   │ 3    │ Reserved                                          │
│ 0x3C   │ 16   │ GUID (optional)                                   │
│ 0x4C   │ 4    │ Checksum (Adler-32)                               │
└────────┴──────┴───────────────────────────────────────────────────┘
```

### Table Section (Chunk Offset Map)

```
Table Header:
┌────────┬──────┬───────────────────────────────────────────────────┐
│ Offset │ Size │ Field                                             │
├────────┼──────┼───────────────────────────────────────────────────┤
│ 0x00   │ 4    │ Chunk count in this table (uint32 LE)             │
│ 0x04   │ 16   │ Padding (zeros)                                   │
│ 0x14   │ 4    │ Checksum of header (Adler-32)                     │
└────────┴──────┴───────────────────────────────────────────────────┘

Table Entries (4 bytes each × chunk count):
┌────────┬──────┬───────────────────────────────────────────────────┐
│ Bits   │ Size │ Field                                             │
├────────┼──────┼───────────────────────────────────────────────────┤
│ 31     │ 1    │ Compression flag (1=compressed, 0=raw)            │
│ 0-30   │ 31   │ Offset from sectors section start                 │
└────────┴──────┴───────────────────────────────────────────────────┘

Table Footer:
┌────────┬──────┬───────────────────────────────────────────────────┐
│ Offset │ Size │ Field                                             │
├────────┼──────┼───────────────────────────────────────────────────┤
│ N×4    │ 4    │ Checksum of all entries (Adler-32)                │
└────────┴──────┴───────────────────────────────────────────────────┘
```

### Sectors Section (Image Data)

```
┌─────────────────────────────────────────────────────────────────────┐
│ Sectors Section Layout                                              │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  [Chunk 0] [Chunk 1] [Chunk 2] ... [Chunk N]                        │
│                                                                     │
│  Each chunk = sectors_per_chunk × bytes_per_sector                  │
│  Default: 64 × 512 = 32,768 bytes (32 KB) uncompressed              │
│                                                                     │
│  Compressed chunks use zlib deflate                                 │
│  Size determined by offset difference in table                      │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### Hash Section

```
┌────────┬──────┬───────────────────────────────────────────────────┐
│ Offset │ Size │ Field                                             │
├────────┼──────┼───────────────────────────────────────────────────┤
│ 0x00   │ 16   │ MD5 hash of uncompressed image data               │
│ 0x10   │ 4    │ Checksum (Adler-32)                               │
└────────┴──────┴───────────────────────────────────────────────────┘
```

### Digest Section

```
┌────────┬──────┬───────────────────────────────────────────────────┐
│ Offset │ Size │ Field                                             │
├────────┼──────┼───────────────────────────────────────────────────┤
│ 0x00   │ 16   │ MD5 hash                                          │
│ 0x10   │ 20   │ SHA1 hash                                         │
│ 0x24   │ 4    │ Checksum (Adler-32)                               │
└────────┴──────┴───────────────────────────────────────────────────┘

Extended (with SHA256):
┌────────┬──────┬───────────────────────────────────────────────────┐
│ Offset │ Size │ Field                                             │
├────────┼──────┼───────────────────────────────────────────────────┤
│ 0x00   │ 16   │ MD5 hash                                          │
│ 0x10   │ 20   │ SHA1 hash                                         │
│ 0x24   │ 32   │ SHA256 hash                                       │
│ 0x44   │ 4    │ Checksum (Adler-32)                               │
└────────┴──────┴───────────────────────────────────────────────────┘
```

### Error2 Section (Acquisition Errors)

```
┌────────┬──────┬───────────────────────────────────────────────────┐
│ Offset │ Size │ Field                                             │
├────────┼──────┼───────────────────────────────────────────────────┤
│ 0x00   │ 4    │ Number of errors (uint32 LE)                      │
│ 0x04   │ 4    │ Reserved                                          │
│ 0x08   │ 4    │ Checksum of header                                │
└────────┴──────┴───────────────────────────────────────────────────┘

Error Entry (8 bytes each):
┌────────┬──────┬───────────────────────────────────────────────────┐
│ Offset │ Size │ Field                                             │
├────────┼──────┼───────────────────────────────────────────────────┤
│ 0x00   │ 4    │ First error sector (uint32 LE)                    │
│ 0x04   │ 4    │ Number of sectors with errors (uint32 LE)         │
└────────┴──────┴───────────────────────────────────────────────────┘

Footer:
┌────────┬──────┬───────────────────────────────────────────────────┐
│ Offset │ Size │ Field                                             │
├────────┼──────┼───────────────────────────────────────────────────┤
│ N×8    │ 4    │ Checksum of all error entries                     │
└────────┴──────┴───────────────────────────────────────────────────┘
```

### Session Section (Optical Disc)

```
┌────────┬──────┬───────────────────────────────────────────────────┐
│ Offset │ Size │ Field                                             │
├────────┼──────┼───────────────────────────────────────────────────┤
│ 0x00   │ 4    │ Number of sessions                                │
│ 0x04   │ 4    │ Reserved                                          │
│ 0x08   │ 4    │ Checksum of header                                │
└────────┴──────┴───────────────────────────────────────────────────┘

Session Entry (32 bytes each):
┌────────┬──────┬───────────────────────────────────────────────────┐
│ Offset │ Size │ Field                                             │
├────────┼──────┼───────────────────────────────────────────────────┤
│ 0x00   │ 4    │ First sector                                      │
│ 0x04   │ 4    │ Sector count                                      │
│ 0x08   │ 24   │ Reserved                                          │
└────────┴──────┴───────────────────────────────────────────────────┘
```

---

## Multi-Segment Files

Large acquisitions span multiple segment files:

```
┌─────────────────────────────────────────────────────────────────────┐
│ Example.E01 (Segment 1)                                             │
├─────────────────────────────────────────────────────────────────────┤
│ ├─ Magic: "EVF\x09\x0D\x0A\xFF\x00"                                 │
│ ├─ Segment: 0x0001                                                  │
│ ├─ Sections: header, header2, volume, table, sectors (partial)     │
│ └─ "next" section → points to E02                                   │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│ Example.E02 (Segment 2)                                             │
├─────────────────────────────────────────────────────────────────────┤
│ ├─ Magic: "EVF\x09\x0D\x0A\xFF\x00"                                 │
│ ├─ Segment: 0x0002                                                  │
│ ├─ Sections: table, sectors (continued)                             │
│ └─ "next" section → points to E03                                   │
└─────────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────────┐
│ Example.E03 (Final Segment)                                         │
├─────────────────────────────────────────────────────────────────────┤
│ ├─ Magic: "EVF\x09\x0D\x0A\xFF\x00"                                 │
│ ├─ Segment: 0x0003                                                  │
│ ├─ Sections: table, sectors (final), hash, digest                   │
│ └─ "done" section                                                   │
└─────────────────────────────────────────────────────────────────────┘

Segment numbering:
E01-E99 = 1-99
EAA-EZZ = 100-775 (base-26 encoded)
```

---

## Hex Layout Visualization

```
E01 File Hex Dump (annotated):
════════════════════════════════════════════════════════════════════════

0x00000000: ┌─────────────────────────────────────────────────────────┐
            │ 45 56 46 09 0D 0A FF 00 │ EVF.....    │ 🔴 SIGNATURE    │
0x00000008: │ 01 00 01 00 00          │ .....       │ 🟠 SEGMENT      │
            └─────────────────────────────────────────────────────────┘

0x0000000D: ┌─────────────────────────────────────────────────────────┐
            │ SECTION HEADER (76 bytes)                               │
            │─────────────────────────────────────────────────────────│
            │ 68 65 61 64 65 72 00 00 │ header..    │ 🟡 TYPE         │
            │ 00 00 00 00 00 00 00 00 │ ........    │                 │
0x0000001D: │ XX XX XX XX XX XX XX XX │ ........    │ 🔵 NEXT OFFSET  │
0x00000025: │ XX XX XX XX XX XX XX XX │ ........    │ 🔵 SIZE         │
0x0000002D: │ 00 00 00 00 00 00 00 00 │ ........    │ ⚪ PADDING      │
            │ 00 00 00 00 00 00 00 00 │ ........    │                 │
            │ 00 00 00 00 00 00 00 00 │ ........    │                 │
            │ 00 00 00 00 00 00 00 00 │ ........    │                 │
            │ 00 00 00 00 00 00 00 00 │ ........    │                 │
0x00000055: │ XX XX XX XX             │ ....        │ 🟣 CHECKSUM     │
            └─────────────────────────────────────────────────────────┘

0x00000059: ┌─────────────────────────────────────────────────────────┐
            │ COMPRESSED HEADER DATA (zlib)                           │
            │ 78 9C ... (zlib deflate stream)                         │
            │ Contains case metadata after decompression              │
            └─────────────────────────────────────────────────────────┘

0x000001XX: ┌─────────────────────────────────────────────────────────┐
            │ SECTION: "volume" (76 + 80 bytes)                       │
            │─────────────────────────────────────────────────────────│
            │ 76-byte section header                                  │
            │ XX XX XX XX             │ Chunk count                   │
            │ XX XX XX XX             │ Sectors/chunk (64)            │
            │ XX XX XX XX             │ Bytes/sector (512)            │
            │ XX XX XX XX XX XX XX XX │ Total sectors                 │
            │ ...                     │                               │
            └─────────────────────────────────────────────────────────┘

0x000002XX: ┌─────────────────────────────────────────────────────────┐
            │ SECTION: "table"                                        │
            │─────────────────────────────────────────────────────────│
            │ Chunk offset entries (4 bytes each)                     │
            │ Bit 31 = compressed flag                                │
            │ Bits 0-30 = offset into sectors section                 │
            └─────────────────────────────────────────────────────────┘

0x000XXXXX: ┌─────────────────────────────────────────────────────────┐
            │ SECTION: "sectors" (bulk of file)                       │
            │─────────────────────────────────────────────────────────│
            │ [Chunk 0: 32KB compressed]                              │
            │ [Chunk 1: 32KB compressed]                              │
            │ [Chunk 2: 32KB compressed]                              │
            │ ...                                                     │
            │ [Chunk N: remaining data]                               │
            └─────────────────────────────────────────────────────────┘

0xXXXXXXXX: ┌─────────────────────────────────────────────────────────┐
            │ SECTION: "digest"                                       │
            │─────────────────────────────────────────────────────────│
            │ XX XX XX XX XX XX XX XX │ MD5 (16 bytes)                │
            │ XX XX XX XX XX XX XX XX │                               │
            │ XX XX XX XX XX XX XX XX │ SHA1 (20 bytes)               │
            │ XX XX XX XX XX XX XX XX │                               │
            │ XX XX XX XX             │                               │
            │ XX XX XX XX             │ Checksum                      │
            └─────────────────────────────────────────────────────────┘

0xXXXXXXXX: ┌─────────────────────────────────────────────────────────┐
            │ SECTION: "done"                                         │
            │─────────────────────────────────────────────────────────│
            │ 64 6F 6E 65 00 00 ...   │ done........│ Final section   │
            │ (76-byte header only)                                   │
            └─────────────────────────────────────────────────────────┘
```

---

## Color Coding Reference (for HexViewer)

| Region | Color | CSS Class | Hex Color |
|--------|-------|-----------|-----------|
| Magic Signature | 🔴 Red | `region-signature` | `#EF4444` |
| Segment Number | 🟠 Orange | `region-segment` | `#F97316` |
| Section Type | 🟡 Yellow | `region-section-type` | `#EAB308` |
| Offsets/Sizes | 🔵 Blue | `region-offset` | `#3B82F6` |
| Checksums | 🟣 Purple | `region-checksum` | `#8B5CF6` |
| Padding/Reserved | ⚪ Gray | `region-reserved` | `#6B7280` |
| Compressed Data | 🟢 Green | `region-data` | `#22C55E` |
| Hash Values | 🟤 Brown | `region-hash` | `#A16207` |
| Metadata | 🩵 Cyan | `region-metadata` | `#06B6D4` |

---

## Checksum Calculation

EWF uses Adler-32 checksums throughout:

```rust
fn adler32(data: &[u8]) -> u32 {
    const MOD_ADLER: u32 = 65521;
    let mut a: u32 = 1;
    let mut b: u32 = 0;
    
    for byte in data {
        a = (a + *byte as u32) % MOD_ADLER;
        b = (b + a) % MOD_ADLER;
    }
    
    (b << 16) | a
}
```

---

## Common Values

| Field | Typical Value | Notes |
|-------|---------------|-------|
| Sectors per chunk | 64 | Standard chunk size |
| Bytes per sector | 512 | HDD standard |
| Bytes per sector | 2048 | CD/DVD standard |
| Chunk size | 32,768 | 64 × 512 bytes |
| Compression | 6 | Default zlib level |
| Segment size | 2 GB | Common split size |

---

## References

- [libewf Documentation](https://github.com/libyal/libewf)
- [EnCase Evidence File Format Specification](https://www.guidancesoftware.com)
- [Forensics Wiki - EWF](https://forensicswiki.xyz/wiki/index.php?title=Encase_image_file_format)
- [libyal/libewf specifications](https://github.com/libyal/libewf/blob/main/documentation/)
