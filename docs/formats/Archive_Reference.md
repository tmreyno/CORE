# Archive Reference

This document summarizes archive signatures and parsing notes used by CORE-FFX.

## ZIP

Signatures:

- Local header: `50 4B 03 04`
- Central directory: `50 4B 01 02`
- EOCD: `50 4B 05 06`
- ZIP64 EOCD locator: `50 4B 06 07`

Notes:

- Metadata authority is the central directory
- ZIP64 is used when size/offset fields are 0xFFFFFFFF
- Extraction is implemented only for ZIP

## 7-Zip

Signature:

- `37 7A BC AF 27 1C`

Notes:

- Next Header may be compressed or encrypted
- CORE-FFX reads structural metadata only

## Implementation

See `src-tauri/src/archive/` for detection and metadata parsing.
