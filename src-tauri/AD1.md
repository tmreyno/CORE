# AD1 Format Notes (AccessData Logical Image)

This document summarizes the AD1 structures used by the CORE-FFX parser.

## Signatures

- Segment signature: `ADSEGMENTEDFILE`
- Logical header signature fields are read from the logical header block

## Segment Header (Public Fields)

See `src-tauri/src/ad1/types.rs` for full definitions:

- `signature`
- `segment_index`
- `segment_number`
- `fragments_size`
- `header_size`

## Logical Header (Public Fields)

- `image_version`
- `zlib_chunk_size`
- `logical_metadata_addr`
- `first_item_addr`
- `data_source_name`

## Item Layout (Parser Offsets)

From `src-tauri/src/ad1/parser.rs`:

- 0x00: next_item_addr
- 0x08: first_child_addr
- 0x10: first_metadata_addr
- 0x18: zlib_metadata_addr
- 0x20: decompressed_size
- 0x28: item_type
- 0x2C: name_length
- 0x30: name_bytes

## Metadata Layout (Parser Offsets)

- 0x00: next_metadata_addr
- 0x08: category
- 0x0C: key
- 0x10: data_length
- 0x14: data

## Parser Capabilities

- Lazy tree enumeration via addresses
- Zlib decompression with LRU cache
- Hash verification using stored hashes when present
- File extraction to filesystem with timestamps

## References

- `src-tauri/src/ad1/parser.rs`
- `src-tauri/src/ad1/types.rs`
