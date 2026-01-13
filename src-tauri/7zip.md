# 7-Zip (7z) Format Notes

This document summarizes the 7z structures used by CORE-FFX for detection and metadata extraction.

## Signature Header

Magic bytes (offset 0x00):

```
37 7A BC AF 27 1C
```

Signature header fields (32 bytes total):

| Offset | Size | Field |
|-------:|-----:|-------|
| 0x00 | 6 | Signature |
| 0x06 | 2 | Version (major, minor) |
| 0x08 | 4 | Start Header CRC |
| 0x0C | 8 | Next Header Offset |
| 0x14 | 8 | Next Header Size |
| 0x1C | 4 | Next Header CRC |

## Next Header

The Next Header contains archive metadata. It may be stored, compressed, and/or encrypted. CORE-FFX only reads structural metadata and does not attempt to decrypt encrypted headers.

## Implementation Notes

- Detection and metadata parsing are implemented in `src-tauri/src/archive/sevenz.rs`
- Extraction for 7z is not implemented (ZIP-only extraction)
