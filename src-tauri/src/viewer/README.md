# Viewer Module

File viewing subsystem for hex/text viewing and document content rendering.

## Structure

```text
viewer/
├── mod.rs           # Module root, hex/text viewing functions
├── types.rs         # FileChunk, FileTypeInfo, ParsedMetadata
├── parsers/         # Format header parsers
│   ├── mod.rs       # Parser exports
│   ├── ad1.rs       # AD1 header parsing
│   ├── ewf.rs       # EWF header parsing
│   ├── archives.rs  # ZIP/7z/RAR/TAR headers
│   └── disk_images.rs # VMDK/VHD/QCOW2 headers
└── document/        # Content viewers
    ├── mod.rs       # Document service
    ├── universal.rs # UniversalFormat detection
    ├── commands.rs  # Tauri viewer commands
    ├── types.rs     # Document types
    ├── error.rs     # Error handling
    ├── pdf.rs       # PDF viewing
    ├── docx.rs      # Word documents
    ├── html.rs      # HTML viewing
    ├── markdown.rs  # Markdown viewing
    ├── exif.rs      # EXIF metadata extraction
    ├── email.rs     # EML/MBOX parsing
    ├── binary.rs    # PE/ELF/Mach-O analysis
    ├── plist_viewer.rs # Apple plist parsing
    └── spreadsheet.rs  # Excel/CSV/ODS
```

## Hex/Text Viewing

The root module provides chunked file reading for large files:

```rust
use crate::viewer::{read_file_chunk, FileChunk};

let chunk = read_file_chunk("/path/to/file", offset, Some(4096))?;
// Returns FileChunk with bytes, offset, total_size, is_end
```

## Header Parsing

The `parsers/` submodule extracts metadata from file headers:

```rust
use crate::viewer::parsers::parse_ewf_header;

let metadata = parse_ewf_header(&bytes)?;
// Returns ParsedMetadata with fields and highlighted regions
```

Supported header parsers:
- `parse_ad1_header` - AD1 logical images
- `parse_ewf_header` - E01/Ex01/L01 files
- `parse_zip_header`, `parse_7z_header`, `parse_rar_header` - Archives
- `parse_vmdk_header`, `parse_vhdx_header`, `parse_qcow2_header` - Disk images
- `parse_gzip_header`, `parse_raw_header` - Other formats

## Document/Content Viewing

The `document/` submodule provides universal file viewing:

### Format Detection

```rust
use crate::viewer::document::universal::{UniversalFormat, detect_format};

let format = detect_format("/path/to/file.pdf")?;
// Returns UniversalFormat::Pdf
```

### Viewer Type Hints

```rust
use crate::viewer::document::universal::get_viewer_type;

let viewer = get_viewer_type(UniversalFormat::Pdf);
// Returns ViewerType::Pdf
```

### Content Reading

```rust
use crate::viewer::document::universal::{read_as_text, read_as_data_url};

// Read text content
let text = read_as_text("/path/to/file.txt", Some(1000))?;

// Read as data URL for images
let data_url = read_as_data_url("/path/to/image.png")?;
```

## Supported Formats

| Category | Formats | ViewerType |
|----------|---------|------------|
| Documents | PDF, DOCX, HTML, MD | Pdf, Text |
| Images | PNG, JPEG, GIF, WebP, HEIC | Image |
| Email | EML, MBOX | Email |
| Binaries | PE, ELF, Mach-O | Binary |
| Data | Plist, JSON, XML, CSV | Text, Plist |
| Spreadsheets | XLSX, XLS, ODS | Text |
| Archives | ZIP, 7z, RAR | Archive |
| Database | SQLite, DB | Database |

## Tauri Commands

Document viewer commands are exposed in `document/commands.rs`:

- `universal_get_info` - Get file info and viewer hint
- `universal_detect_format` - Detect file format
- `universal_read_text` - Read text content
- `universal_read_data_url` - Read as data URL
- `universal_get_image_dimensions` - Get image dimensions
- `universal_create_thumbnail` - Create image thumbnail
- `document_read` - Read document content
- `document_render_html` - Render document as HTML
- `document_get_metadata` - Get document metadata

## Adding a New Viewer

1. Create `viewer/document/myformat.rs`
2. Add `pub mod myformat;` to `viewer/document/mod.rs`
3. Add format variant to `UniversalFormat` in `universal.rs`
4. Add viewer type mapping in `get_viewer_type()`
5. Add Tauri commands if needed in `commands.rs`
6. Register commands in `lib.rs`

---

*Last updated: January 18, 2026*
