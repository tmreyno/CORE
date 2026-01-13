# AD1 File Structure - Complete Technical Specification

**Based on libad1 C Implementation Analysis**  
*Reference: https://github.com/Seabreg/AD1-tools*

---

## Table of Contents

1. [Overview](#overview)
2. [File Layout](#file-layout)
3. [Segment Header](#segment-header)
4. [Logical Header](#logical-header)
5. [Item Structure](#item-structure)
6. [Metadata Structure](#metadata-structure)
7. [Zlib Compression](#zlib-compression)
8. [Address Calculation](#address-calculation)
9. [Data Types Reference](#data-types-reference)
10. [Complete Hex Map](#complete-hex-map)

---

## Overview

### File Format
- **Extension**: `.ad1`, `.ad2`, `.ad3`, etc. (multi-segment)
- **Endianness**: Little-endian for all numeric values
- **Addressing**: 64-bit logical offset space across all segments
- **Compression**: Zlib for file data

### Key Constants

```c
#define AD1_LOGICAL_MARGIN   512        // Offset to logical header in each segment
#define AD1_FOLDER_SIGNATURE 0x05       // Item type for folders
```

---

## File Layout

### Single Segment Structure

```
┌─────────────────────────────────────────┐  Offset: 0x0000
│      Segment Header (512 bytes)         │
├─────────────────────────────────────────┤  Offset: 0x0200 (512)
│      Logical Header (~300 bytes)        │  ← AD1_LOGICAL_MARGIN
├─────────────────────────────────────────┤
│                                         │
│         Logical Data Space              │
│  (Items, Metadata, Compressed Files)    │
│                                         │
│                                         │
└─────────────────────────────────────────┘
```

### Multi-Segment Structure

```
File: image.ad1 (Segment 1)
┌─────────────────────────────────────────┐
│      Segment Header                     │  segment_index = 1
│      segment_number = 3                 │  Total segments in set
├─────────────────────────────────────────┤
│      Logical Header                     │  Only in first segment
│      first_item_addr → Tree Root        │
├─────────────────────────────────────────┤
│      Logical Data                       │
│      (continues across segments)        │
└─────────────────────────────────────────┘

File: image.ad2 (Segment 2)
┌─────────────────────────────────────────┐
│      Segment Header                     │  segment_index = 2
├─────────────────────────────────────────┤
│      Logical Data (continued)           │
│      Fragment space = (fragments_size   │
│                        * 65536) - 512   │
└─────────────────────────────────────────┘

File: image.ad3 (Segment 3)
┌─────────────────────────────────────────┐
│      Segment Header                     │  segment_index = 3
├─────────────────────────────────────────┤
│      Logical Data (continued)           │
└─────────────────────────────────────────┘
```

---

## Segment Header

**Location**: Offset `0x0000` in each segment file  
**Size**: 512 bytes (padded)  
**Signature**: `"ADSEGMENTEDFILE\0"` (16 bytes)

### C Structure

```c
typedef struct ad1_segment_header {
    unsigned char signature[16];      // 0x00: "ADSEGMENTEDFILE\0"
    unsigned int segment_index;       // 0x18: Current segment number (1-based)
    unsigned int segment_number;      // 0x1C: Total number of segments
    unsigned int fragments_size;      // 0x22: Fragment size multiplier
    unsigned int header_size;         // 0x28: Size of this header
} ad1_segment_header;
```

### Hex Layout

| Offset | Size | Type   | Field Name       | Description                           |
|--------|------|--------|------------------|---------------------------------------|
| 0x0000 | 16   | char[] | signature        | "ADSEGMENTEDFILE\0"                   |
| 0x0010 | 8    | -      | (padding)        | Reserved/padding                      |
| 0x0018 | 4    | u32    | segment_index    | Current segment (1, 2, 3...)          |
| 0x001C | 4    | u32    | segment_number   | Total segments in set                 |
| 0x0020 | 2    | -      | (padding)        | Reserved                              |
| 0x0022 | 4    | u32    | fragments_size   | Fragment size multiplier              |
| 0x0026 | 2    | -      | (padding)        | Reserved                              |
| 0x0028 | 4    | u32    | header_size      | Size of segment header                |
| 0x002C | 468  | -      | (padding)        | Pad to 512 bytes                      |

### Fragment Size Calculation

```c
// Size of data space per segment (excluding logical margin)
fragment_space = (fragments_size * 65536) - AD1_LOGICAL_MARGIN
               = (fragments_size * 65536) - 512
```

**Example**: If `fragments_size = 2`:
```
fragment_space = (2 * 65536) - 512
               = 131072 - 512
               = 130560 bytes per segment
```

---

## Logical Header

**Location**: Offset `0x0200` (512) in **first segment only**  
**Size**: Variable (~300-600 bytes depending on data source name)  
**Signature**: `"ADLOGICALIMAGE\0\0"` (16 bytes)

### C Structure

```c
typedef struct ad1_logical_header {
    unsigned char signature[16];           // 0x200: "ADLOGICALIMAGE\0\0"
    unsigned int image_version;            // 0x210: Version number
    unsigned int some_data;                // 0x214: Unknown/reserved
    unsigned int zlib_chunk_size;          // 0x218: Default zlib chunk size
    unsigned long logical_metadata_addr;   // 0x21C: Metadata chain address
    unsigned long first_item_addr;         // 0x224: Root item address ★
    unsigned int data_source_name_length;  // 0x22C: Length of source name
    unsigned char ad_signature[4];         // 0x230: "AD\0\0" signature
    unsigned long data_source_name_addr;   // 0x234: Address of source name
    unsigned long attrguid_footer_addr;    // 0x23C: Attribute GUID footer
    unsigned long locsguid_footer_addr;    // 0x24C: Location GUID footer
    unsigned char* data_source_name;       // 0x25C: Variable length string
} ad1_logical_header;
```

### Hex Layout

| Offset | Size | Type     | Field Name                | Description                      |
|--------|------|----------|---------------------------|----------------------------------|
| 0x0200 | 16   | char[]   | signature                 | "ADLOGICALIMAGE\0\0"             |
| 0x0210 | 4    | u32      | image_version             | Image version (typically 1)      |
| 0x0214 | 4    | u32      | some_data                 | Unknown/reserved                 |
| 0x0218 | 4    | u32      | zlib_chunk_size           | Zlib chunk size (65536 typical)  |
| 0x021C | 8    | u64      | logical_metadata_addr     | Logical metadata chain start     |
| 0x0224 | 8    | u64      | first_item_addr           | **ROOT ITEM ADDRESS** ★          |
| 0x022C | 4    | u32      | data_source_name_length   | Length of data source name       |
| 0x0230 | 4    | char[4]  | ad_signature              | "AD\0\0"                         |
| 0x0234 | 8    | u64      | data_source_name_addr     | Address of data source name      |
| 0x023C | 8    | u64      | attrguid_footer_addr      | Attribute GUID footer address    |
| 0x0244 | 8    | u64      | (reserved)                | Reserved space                   |
| 0x024C | 8    | u64      | locsguid_footer_addr      | Location GUID footer address     |
| 0x0254 | 8    | -        | (reserved)                | Reserved space                   |
| 0x025C | var  | UTF-16LE | data_source_name          | Variable length name string      |

### Important Note
- **`first_item_addr`** at offset `0x224` is the **entry point** to the file tree
- This address points to the first root item in the logical data space
- All addresses are **logical offsets** from the start of logical data (after segment headers)

---

## Item Structure

**Purpose**: Represents a file or folder in the AD1 container  
**Size**: Variable (minimum ~64 bytes + name length)  
**Linked List**: Items form a tree using `next_item_addr` and `first_child_addr`

### C Structure

```c
typedef struct ad1_item_header {
    unsigned long next_item_addr;         // 0x00: Next sibling item
    unsigned long first_child_addr;       // 0x08: First child (if folder)
    unsigned long first_metadata_addr;    // 0x10: Metadata chain start
    unsigned long zlib_metadata_addr;     // 0x18: Compressed data metadata
    unsigned long decompressed_size;      // 0x20: Uncompressed file size
    unsigned int item_type;               // 0x28: Type (0x05 = folder, etc.)
    unsigned int item_name_length;        // 0x2C: Name length in bytes
    unsigned char* item_name;             // 0x30: UTF-16LE name (variable)
    unsigned long parent_folder;          // After name: Parent item address
} ad1_item_header;
```

### Hex Layout

| Offset       | Size | Type     | Field Name            | Description                           |
|--------------|------|----------|-----------------------|---------------------------------------|
| 0x00         | 8    | u64      | next_item_addr        | Address of next sibling (0 if last)   |
| 0x08         | 8    | u64      | first_child_addr      | Address of first child (0 if file)    |
| 0x10         | 8    | u64      | first_metadata_addr   | Start of metadata chain               |
| 0x18         | 8    | u64      | zlib_metadata_addr    | Zlib chunk table address              |
| 0x20         | 8    | u64      | decompressed_size     | Uncompressed file size                |
| 0x28         | 4    | u32      | item_type             | Type code (see below)                 |
| 0x2C         | 4    | u32      | item_name_length      | Name length in bytes                  |
| 0x30         | var  | UTF-16LE | item_name             | Item name (UTF-16LE)                  |
| 0x30+namelen | 8    | u64      | parent_folder         | Parent folder item address            |

### Item Types

```c
enum ad_item_type_value {
    REGULAR_FILE        = 0x31,  // Standard file
    PLACEHOLDER         = 0x32,  // Placeholder entry
    REGULAR_FOLDER      = 0x33,  // Standard folder
    FILESYSTEM_METADATA = 0x34,  // FS metadata
    FOLDER              = 0x05,  // Folder (AD1_FOLDER_SIGNATURE)
    FILESLACK           = 0x36,  // File slack space
    SYMLINK             = 0x39   // Symbolic link
};
```

### Tree Navigation Example

```
Root Item (addr: 0x1000)
├─ next_item_addr = 0x2000      → Next sibling
├─ first_child_addr = 0x1500    → First child (if folder)
└─ parent_folder = 0            → No parent (root)

Child Item (addr: 0x1500)
├─ next_item_addr = 0x1800      → Next sibling at same level
├─ first_child_addr = 0         → No children (file)
└─ parent_folder = 0x1000       → Points back to parent
```

### libad1 Reading Code

```c
// From libad1_reader.c - arbitrary_read_item()
ad1_item_header* arbitrary_read_item(ad1_session* session, unsigned long offset) {
    ad1_item_header* item_header = calloc(1, sizeof(ad1_item_header));

    // Read fixed-size fields
    item_header->next_item_addr       = arbitrary_read_long_little_endian(session, offset + 0x00);
    item_header->first_child_addr     = arbitrary_read_long_little_endian(session, offset + 0x08);
    item_header->first_metadata_addr  = arbitrary_read_long_little_endian(session, offset + 0x10);
    item_header->zlib_metadata_addr   = arbitrary_read_long_little_endian(session, offset + 0x18);
    item_header->decompressed_size    = arbitrary_read_long_little_endian(session, offset + 0x20);
    item_header->item_type            = arbitrary_read_int_little_endian(session, offset + 0x28);
    item_header->item_name_length     = arbitrary_read_int_little_endian(session, offset + 0x2C);

    // Read variable-length name
    item_header->item_name = calloc(item_header->item_name_length + 1, sizeof(char));
    arbitrary_read_ad1_string(session, item_header->item_name, 
                             item_header->item_name_length, offset + 0x30);

    // Replace slashes with underscores
    for (int i = 0; i < item_header->item_name_length; i++) {
        if (item_header->item_name[i] == 47) {  // ASCII '/'
            item_header->item_name[i] = 95;     // ASCII '_'
        }
    }

    // Read parent folder address after name
    item_header->parent_folder = arbitrary_read_long_little_endian(session,
                                    offset + 0x30 + item_header->item_name_length);

    return item_header;
}
```

---

## Metadata Structure

**Purpose**: Stores file attributes (hashes, timestamps, permissions, etc.)  
**Size**: Variable (minimum 24 bytes + data length)  
**Linked List**: Forms a chain via `next_metadata_addr`

### C Structure

```c
typedef struct ad1_metadata {
    unsigned long next_metadata_addr;  // 0x00: Next metadata in chain
    unsigned int category;             // 0x08: Metadata category
    unsigned int key;                  // 0x0C: Metadata key/type
    unsigned int data_length;          // 0x10: Length of data field
    unsigned char* data;               // 0x14: Variable length data
} ad1_metadata;
```

### Hex Layout

| Offset | Size | Type   | Field Name         | Description                        |
|--------|------|--------|--------------------|-------------------------------------|
| 0x00   | 8    | u64    | next_metadata_addr | Next metadata entry (0 if last)     |
| 0x08   | 4    | u32    | category           | Category code (see below)           |
| 0x0C   | 4    | u32    | key                | Key within category                 |
| 0x10   | 4    | u32    | data_length        | Data payload length                 |
| 0x14   | var  | bytes  | data               | Raw data bytes                      |

### Metadata Categories

```c
enum category {
    HASH_INFO       = 0x01,  // Hash values (MD5, SHA1)
    ITEM_TYPE       = 0x02,  // Item type information
    ITEM_SIZE       = 0x03,  // Size information
    WINDOWS_FLAGS   = 0x04,  // Windows file attributes
    TIMESTAMP       = 0x05   // Timestamps (access, modified, change)
};
```

### Hash Info Keys (Category 0x01)

```c
enum ad_hash_key {
    MD5_HASH          = 0x5001,   // MD5 hash (16 bytes)
    SHA1_HASH         = 0x5002,   // SHA1 hash (20 bytes)
    DATA_SOURCE_NAME  = 0x10002   // Data source name
};
```

**MD5 Example** (category=0x01, key=0x5001):
```
Data Length: 16 (0x10)
Data: [raw 16 bytes of MD5 hash]
```

**SHA1 Example** (category=0x01, key=0x5002):
```
Data Length: 20 (0x14)
Data: [raw 20 bytes of SHA1 hash]
```

### Timestamp Keys (Category 0x05)

```c
enum ad_timestamp_key {
    ACCESS   = 0x07,  // Last access time
    MODIFIED = 0x08,  // Last modified time
    CHANGE   = 0x09   // Change time (inode change)
};
```

**Timestamp Format**: Windows FILETIME (8 bytes, little-endian)
- 100-nanosecond intervals since January 1, 1601 UTC
- Conversion to Unix time:
  ```c
  const FILETIME_EPOCH_DIFF = 116444736000000000;
  unix_time = (filetime - FILETIME_EPOCH_DIFF) / 10000000;
  ```

### Windows Flags Keys (Category 0x04)

```c
enum ad_windows_flag_key {
    ENCRYPTED        = 0x0D,    // File is encrypted
    COMPRESSED       = 0x0E,    // File is compressed
    HIDDEN           = 0x1002,  // Hidden attribute
    READ_ONLY        = 0x1004,  // Read-only attribute
    READY_TO_ARCHIVE = 0x1005   // Archive bit set
};
```

### Metadata Chain Example

```
Item at 0x1000:
  first_metadata_addr = 0x5000

Metadata Chain:
  0x5000: category=0x01, key=0x5001, len=16, data=[MD5 hash]
          next_metadata_addr=0x5030
  
  0x5030: category=0x01, key=0x5002, len=20, data=[SHA1 hash]
          next_metadata_addr=0x5058
  
  0x5058: category=0x05, key=0x08, len=8, data=[Modified timestamp]
          next_metadata_addr=0x0000  (end of chain)
```

### libad1 Reading Code

```c
// From libad1_reader.c - arbitrary_read_metadata()
ad1_metadata* arbitrary_read_metadata(ad1_session* session, unsigned long offset) {
    ad1_metadata* metadata = calloc(1, sizeof(ad1_metadata));

    metadata->next_metadata_addr = arbitrary_read_long_little_endian(session, offset + 0x00);
    metadata->category           = arbitrary_read_int_little_endian(session, offset + 0x08);
    metadata->key                = arbitrary_read_int_little_endian(session, offset + 0x0C);
    metadata->data_length        = arbitrary_read_int_little_endian(session, offset + 0x10);

    metadata->data = calloc(metadata->data_length + 1, sizeof(char));
    arbitrary_read_ad1_string(session, metadata->data, metadata->data_length, offset + 0x14);

    return metadata;
}

// From libad1_tree.c - build metadata chain recursively
void build_next_metadata(ad1_session* session, ad1_metadata* parent_metadata) {
    if (parent_metadata->next_metadata_addr == 0) {
        return;  // End of chain
    }

    ad1_metadata* new_metadata = arbitrary_read_metadata(session, 
                                    parent_metadata->next_metadata_addr);
    parent_metadata->next_metadata = new_metadata;
    build_next_metadata(session, new_metadata);  // Recurse
}
```

---

## Zlib Compression

**Purpose**: File data is compressed using zlib in chunks  
**Chunk Size**: Typically 65536 bytes (configurable)  
**Metadata**: Stored at `item->zlib_metadata_addr`

### Zlib Metadata Structure

```
┌────────────────────────────────────────┐  Offset: zlib_metadata_addr
│  chunk_count (u64, 8 bytes)            │  Number of compressed chunks
├────────────────────────────────────────┤  +0x08
│  chunk_0_addr (u64)                    │  Address of first chunk
├────────────────────────────────────────┤  +0x10
│  chunk_1_addr (u64)                    │  Address of second chunk
├────────────────────────────────────────┤  +0x18
│  ...                                   │
├────────────────────────────────────────┤
│  chunk_N_addr (u64)                    │  Address of last chunk
├────────────────────────────────────────┤  +(chunk_count+1)*8
│  end_addr (u64)                        │  End of last chunk
└────────────────────────────────────────┘
```

### Example

File with 3 chunks:
```
zlib_metadata_addr = 0x10000

0x10000: chunk_count    = 3
0x10008: chunk_0_addr   = 0x11000  (start of chunk 0)
0x10010: chunk_1_addr   = 0x12000  (start of chunk 1)
0x10018: chunk_2_addr   = 0x13000  (start of chunk 2)
0x10020: end_addr       = 0x13500  (end of chunk 2)

Chunk sizes:
  Chunk 0: 0x12000 - 0x11000 = 4096 bytes (compressed)
  Chunk 1: 0x13000 - 0x12000 = 4096 bytes (compressed)
  Chunk 2: 0x13500 - 0x13000 = 1280 bytes (compressed)
```

### Decompression Algorithm

```c
// From libad1_file_reader.c - read_file_data()
unsigned char* read_file_data(ad1_session* session, ad1_item_header* ad1_item) {
    if (ad1_item->decompressed_size == 0) {
        return calloc(1, sizeof(unsigned char));
    }

    // Allocate output buffer for decompressed data
    unsigned char* file_data = calloc(ad1_item->decompressed_size, sizeof(unsigned char));

    // Read chunk count
    unsigned long chunk_numbers = arbitrary_read_long_little_endian(session, 
                                    ad1_item->zlib_metadata_addr);

    // Read all chunk addresses (including end marker)
    unsigned long addresses[chunk_numbers + 1];
    for (int i = 0; i < chunk_numbers + 1; i++) {
        addresses[i] = arbitrary_read_long_little_endian(session,
                         ad1_item->zlib_metadata_addr + ((i + 1) * 0x08));
    }

    // Decompress each chunk
    unsigned long data_index = 0;
    for (int i = 0; i < chunk_numbers; i++) {
        unsigned long chunk_addr = addresses[i];
        unsigned int chunk_size = addresses[i + 1] - addresses[i];
        
        data_index += read_zlib_chunk(session, file_data + data_index,
                                     chunk_addr, chunk_size,
                                     ad1_item->decompressed_size);
    }

    return file_data;
}
```

### Zlib Inflation

```c
long read_zlib_chunk(ad1_session* session, unsigned char* output_data_ptr,
                     unsigned long offset, unsigned int zlib_chunk_size,
                     unsigned long decompressed_size) {
    // Read compressed data
    unsigned char* compressed_data = calloc(zlib_chunk_size, sizeof(char));
    arbitrary_read(session, compressed_data, zlib_chunk_size, offset);

    // Decompress using zlib
    unsigned long chunk_decompressed_size = zlib_inflate(compressed_data,
                                                         zlib_chunk_size,
                                                         output_data_ptr,
                                                         decompressed_size);

    free(compressed_data);
    return chunk_decompressed_size;
}
```

---

## Address Calculation

### Logical Address Space

All addresses in AD1 (item addresses, metadata addresses, zlib addresses) are **logical offsets** into a virtual address space that spans all segments.

### Multi-Segment Calculation

```c
// From libad1_reader.c - arbitrary_read()
void arbitrary_read(ad1_session* session, unsigned char* buf,
                    unsigned long length, unsigned long offset) {
    unsigned long toRead = length;
    unsigned int char_cursor = 0;

    // Calculate which segment file contains this offset
    unsigned int file_cursor = (unsigned int)(offset / 
        ((session->segment_header->fragments_size * 65536) - AD1_LOGICAL_MARGIN));

    // Calculate offset within that segment's data space
    unsigned long data_cursor = offset - 
        (((session->segment_header->fragments_size * 65536) - AD1_LOGICAL_MARGIN) * file_cursor);

    while (toRead > 0) {
        // How much can we read from current segment?
        unsigned long trunc_size_read = toRead;
        
        if (toRead + data_cursor > session->ad1_files[file_cursor]->size) {
            trunc_size_read = session->ad1_files[file_cursor]->size - data_cursor;
        }

        // Read from segment file (add logical margin for physical offset)
        fseek(session->ad1_files[file_cursor]->adfile, 
              data_cursor + AD1_LOGICAL_MARGIN, SEEK_SET);
        fread(&buf[char_cursor], 1, trunc_size_read,
              session->ad1_files[file_cursor]->adfile);

        // Move to next segment if needed
        char_cursor += trunc_size_read;
        toRead -= trunc_size_read;
        data_cursor = 0;
        file_cursor += 1;
    }
}
```

### Formula Breakdown

```
Given:
  fragments_size = value from segment header
  AD1_LOGICAL_MARGIN = 512

Calculate:
  fragment_space = (fragments_size * 65536) - 512
  
For logical offset 'L':
  segment_index = L / fragment_space
  segment_offset = L % fragment_space
  physical_offset = segment_offset + 512

Example with fragments_size=2:
  fragment_space = (2 * 65536) - 512 = 130560
  
  Logical offset 150000:
    segment_index = 150000 / 130560 = 1 (segment .ad2)
    segment_offset = 150000 % 130560 = 19440
    physical_offset = 19440 + 512 = 19952
```

### Visual Example

```
fragments_size = 2
fragment_space = 130560 bytes per segment

Logical Address Space:
  0x00000 - 0x1FDFF   → Segment 1 (.ad1), physical 0x200 - 0x1FFFF
  0x1FE00 - 0x3FBFF   → Segment 2 (.ad2), physical 0x200 - 0x1FFFF
  0x3FC00 - 0x5F9FF   → Segment 3 (.ad3), physical 0x200 - 0x1FFFF
  ...
```

---

## Data Types Reference

### Primitive Types

| Type           | Size    | Description          | Example              |
|----------------|---------|----------------------|----------------------|
| `unsigned char`| 1 byte  | Byte/octet           | `0x5A`               |
| `short`        | 2 bytes | Signed 16-bit int    | `-1234`              |
| `unsigned int` | 4 bytes | Unsigned 32-bit int  | `0x12345678`         |
| `unsigned long`| 8 bytes | Unsigned 64-bit int  | `0x123456789ABCDEF0` |

### String Encoding

- **UTF-16LE**: Item names stored as 2 bytes per character, little-endian
- **Null-terminated**: Strings may or may not have null terminators
- **Slash replacement**: Forward slashes (0x2F / 47) replaced with underscores (0x5F / 95)

### Endianness

All multi-byte integers are **little-endian**:

```
Example: 0x12345678 in memory:
  [0x78] [0x56] [0x34] [0x12]
   ↑     ↑     ↑     ↑
  LSB                MSB
```

---

## Complete Hex Map

### Quick Reference Table

| Structure        | Base Offset       | Key Fields                                          |
|------------------|-------------------|-----------------------------------------------------|
| Segment Header   | 0x0000            | signature, segment_index, segment_number            |
| Logical Header   | 0x0200 (512)      | first_item_addr @0x224 ★                            |
| Item Header      | Variable          | next_item @0x00, first_child @0x08, metadata @0x10  |
| Metadata         | Variable          | next_metadata @0x00, category @0x08, key @0x0C      |
| Zlib Metadata    | Variable          | chunk_count @0x00, chunk_addrs @0x08+               |

### Segment Header Detailed Map

```
0x0000: [16 bytes] "ADSEGMENTEDFILE\0"
0x0010: [08 bytes] padding
0x0018: [04 bytes] segment_index (u32 LE)
0x001C: [04 bytes] segment_number (u32 LE)
0x0020: [02 bytes] padding
0x0022: [04 bytes] fragments_size (u32 LE)
0x0026: [02 bytes] padding
0x0028: [04 bytes] header_size (u32 LE)
0x002C: [468 bytes] padding to 512 bytes
```

### Logical Header Detailed Map

```
0x0200: [16 bytes] "ADLOGICALIMAGE\0\0"
0x0210: [04 bytes] image_version (u32 LE)
0x0214: [04 bytes] some_data (u32 LE)
0x0218: [04 bytes] zlib_chunk_size (u32 LE)
0x021C: [08 bytes] logical_metadata_addr (u64 LE)
0x0224: [08 bytes] first_item_addr (u64 LE) ★★★ ENTRY POINT
0x022C: [04 bytes] data_source_name_length (u32 LE)
0x0230: [04 bytes] "AD\0\0"
0x0234: [08 bytes] data_source_name_addr (u64 LE)
0x023C: [08 bytes] attrguid_footer_addr (u64 LE)
0x0244: [08 bytes] reserved
0x024C: [08 bytes] locsguid_footer_addr (u64 LE)
0x0254: [08 bytes] reserved
0x025C: [variable] data_source_name (UTF-16LE)
```

### Item Header Detailed Map

```
+0x00: [08 bytes] next_item_addr (u64 LE)
+0x08: [08 bytes] first_child_addr (u64 LE)
+0x10: [08 bytes] first_metadata_addr (u64 LE)
+0x18: [08 bytes] zlib_metadata_addr (u64 LE)
+0x20: [08 bytes] decompressed_size (u64 LE)
+0x28: [04 bytes] item_type (u32 LE)
+0x2C: [04 bytes] item_name_length (u32 LE)
+0x30: [variable] item_name (UTF-16LE, length from 0x2C)
+0x30+namelen: [08 bytes] parent_folder (u64 LE)
```

### Metadata Header Detailed Map

```
+0x00: [08 bytes] next_metadata_addr (u64 LE)
+0x08: [04 bytes] category (u32 LE)
+0x0C: [04 bytes] key (u32 LE)
+0x10: [04 bytes] data_length (u32 LE)
+0x14: [variable] data (raw bytes, length from 0x10)
```

### Zlib Metadata Detailed Map

```
+0x00: [08 bytes] chunk_count (u64 LE)
+0x08: [08 bytes] chunk_0_start_addr (u64 LE)
+0x10: [08 bytes] chunk_1_start_addr (u64 LE)
...
+0x08+(N*8): [08 bytes] chunk_N_start_addr (u64 LE)
+0x08+((N+1)*8): [08 bytes] end_addr (u64 LE)
```

---

## Usage Examples

### Example 1: Reading Root Items

```rust
// 1. Open first segment
let mut file = File::open("image.ad1")?;

// 2. Read segment header at 0x0000
let segment_header = read_segment_header(&mut file)?;

// 3. Read logical header at 0x0200
let logical_header = read_logical_header(&mut file)?;

// 4. Get root item address
let first_item_addr = logical_header.first_item_addr;  // e.g., 0x10000

// 5. Read first root item
let root_item = read_item_at_logical_address(first_item_addr)?;

// 6. Traverse siblings
let mut current_addr = root_item.next_item_addr;
while current_addr != 0 {
    let sibling = read_item_at_logical_address(current_addr)?;
    println!("Found: {}", sibling.name);
    current_addr = sibling.next_item_addr;
}
```

### Example 2: Reading File Data

```rust
// Given an item with compressed data
if item.zlib_metadata_addr != 0 && item.decompressed_size > 0 {
    // Read zlib metadata
    let chunk_count = read_u64_at(item.zlib_metadata_addr)?;
    
    // Read chunk addresses
    let mut chunk_addrs = Vec::new();
    for i in 0..=chunk_count {
        let addr = read_u64_at(item.zlib_metadata_addr + ((i + 1) * 8))?;
        chunk_addrs.push(addr);
    }
    
    // Decompress each chunk
    let mut decompressed = Vec::new();
    for i in 0..chunk_count {
        let chunk_start = chunk_addrs[i as usize];
        let chunk_size = chunk_addrs[(i + 1) as usize] - chunk_start;
        
        let compressed = read_at_logical_address(chunk_start, chunk_size)?;
        let decompressed_chunk = zlib_decompress(&compressed)?;
        decompressed.extend(decompressed_chunk);
    }
}
```

### Example 3: Extracting Metadata

```rust
// Read metadata chain for an item
if item.first_metadata_addr != 0 {
    let mut current_addr = item.first_metadata_addr;
    
    while current_addr != 0 {
        // Read metadata entry
        let next_addr = read_u64_at(current_addr + 0x00)?;
        let category = read_u32_at(current_addr + 0x08)?;
        let key = read_u32_at(current_addr + 0x0C)?;
        let data_length = read_u32_at(current_addr + 0x10)?;
        let data = read_bytes_at(current_addr + 0x14, data_length)?;
        
        // Process metadata
        match (category, key) {
            (0x01, 0x5001) => {  // MD5 hash
                let md5 = hex::encode(data);
                println!("MD5: {}", md5);
            },
            (0x05, 0x08) => {  // Modified time
                let filetime = u64::from_le_bytes(data[0..8].try_into()?);
                let unix_time = (filetime - 116444736000000000) / 10000000;
                println!("Modified: {}", unix_time);
            },
            _ => {}
        }
        
        current_addr = next_addr;
    }
}
```

---

## Performance Considerations

### Caching Strategy (from libad1)

```c
// libad1 uses LRU-style cache for decompressed files
#define CACHE_SIZE 100

typedef struct {
    ad1_item_header* cached_item;
    unsigned char* data;
    int counter;  // Reference count for LRU
} ad1_cache_entry;

ad1_cache_entry ad1_file_cache[CACHE_SIZE];
```

### Thread Safety

- libad1 uses pthread mutexes for file I/O
- Protect concurrent access to segment files
- Cache operations are thread-safe

---

## References

- **libad1 Source**: https://github.com/Seabreg/AD1-tools
- **Author**: Maxim Suhanov
- **License**: Check repository for license details

---

## Document Version

- **Version**: 1.0
- **Date**: January 11, 2026
- **Based on**: libad1 C implementation analysis
- **Analysis Source**: `/Users/terryreynolds/GitHub/CORE/libad1/AD1-tools/AD1Tools/libad1/`

---

**END OF DOCUMENT**
