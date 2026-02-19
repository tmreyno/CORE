# Crate API Quick-Reference for AI Coding Agents

> **Purpose:** Prevent wrong-API-assumption errors by documenting the _actual_ signatures of every third-party crate used in CORE-FFX. Before writing code that calls any of these crates, **check this file first**.
>
> **How to verify yourself:** Run `cargo doc --no-deps -p <crate>` or grep the source in `~/.cargo/registry/src/`.

---

## Table of Contents

1. [msg_parser 0.1.1 — Outlook .MSG](#msg_parser-011)
2. [mail-parser 0.9.4 — RFC 5322 EML/MBOX](#mail-parser-094)
3. [goblin 0.8.2 — PE/ELF/Mach-O Binary Analysis](#goblin-082)
4. [calamine 0.26.1 — Excel/ODS Spreadsheets](#calamine-0261)
5. [cfb 0.14 — Compound File Binary (OLE2)](#cfb-014)
6. [notatin 1.0.1 — Windows Registry Hives](#notatin-101)
6. [plist 1.8 — Apple Property Lists](#plist-18)
7. [kamadak-exif 0.5.5 — EXIF Metadata](#kamadak-exif-055)
8. [lopdf 0.34 — PDF Parsing](#lopdf-034)
9. [zip 2.4 — ZIP Archives](#zip-24)
10. [libarchive2 0.2.1 — Unified Archive Backend](#libarchive2-021)
11. [unrar 0.5.8 — RAR Archives](#unrar-058)
12. [quick_xml 0.37.5 — XML Parsing (SAX-style)](#quick_xml-0375)
13. [rusqlite 0.32.1 — SQLite Database](#rusqlite-0321)
14. [ntfs 0.4.0 — NTFS Filesystem Parsing](#ntfs-040)
15. [apple-dmg 0.5.0 — Apple DMG Disk Images](#apple-dmg-050)
16. [sevenz_rust 0.6.1 — 7-Zip Archives](#sevenz_rust-061)
17. [docx_rs 0.4.18 — DOCX Read/Write](#docx_rs-0418)
18. [seven_zip (sevenzip-ffi) — 7-Zip FFI Bindings](#seven_zip-sevenzip-ffi)
19. [genpdf 0.2.0 — PDF Generation](#genpdf-020)
20. [tera 1.20 — Template Engine](#tera-120)
21. [fatfs 0.3.6 — FAT12/16/32 Filesystem](#fatfs-036)
22. [tar 0.4.44 — TAR Archives](#tar-0444)
23. [image 0.25 — Image Processing](#image-025)
24. [pdf-extract 0.7.12 — PDF Text Extraction](#pdf-extract-0712)
25. [csv 1.3 — CSV/TSV Parsing](#csv-13)
26. [langchain-rust 4.6 + async-openai 0.29 — AI Integration](#langchain-rust-46--async-openai-029)
27. [Internal Types — Gotchas & Corrections](#internal-types)

---

## msg_parser 0.1.1

Crate for parsing Outlook `.msg` files.

### Core Type: `Outlook`

```rust
// Source: msg_parser/src/outlook.rs lines 118-127
pub struct Outlook {
    pub headers:        TransportHeaders,
    pub sender:         Person,
    pub to:             Vec<Person>,
    pub cc:             Vec<Person>,
    pub bcc:            Name,          // ⚠️ Name is a String alias, NOT Vec<Person>!
    pub subject:        String,
    pub body:           String,
    pub rtf_compressed: String,
    pub attachments:    Vec<Attachment>,
}
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `outlook.body_html` | **Does NOT exist.** Only `body` (plain text) and `rtf_compressed`. |
| `outlook.bcc` type | `Name` (= `String`), not `Vec<Person>` like `to`/`cc`. |
| `att.is_content_disposition_inline()` | **Does NOT exist.** No such method on `Attachment`. |

### Supporting Types

```rust
pub struct TransportHeaders {
    pub content_type: String,
    pub date:         String,
    pub message_id:   String,
    pub reply_to:     String,
}

pub struct Person {
    pub name:  Name,   // Name = String
    pub email: Email,  // Email = String
}

pub struct Attachment {
    pub display_name: String,
    pub payload:      String,     // base64-encoded content
    pub extension:    String,
    pub mime_tag:     String,
    pub file_name:    String,
}
```

### Usage Pattern

```rust
use msg_parser::Outlook;

let outlook = Outlook::from_path("file.msg").map_err(|e| e.to_string())?;
let subject = &outlook.subject;
let body_text = &outlook.body;
let sender_name = &outlook.sender.name;
let sender_email = &outlook.sender.email;
for att in &outlook.attachments {
    let name = &att.file_name;
    let mime = &att.mime_tag;
}
```

---

## mail-parser 0.9.4

Crate for parsing RFC 5322 email messages (EML, MBOX).

### Parsing Entry Point

```rust
use mail_parser::MessageParser;

let parser = MessageParser::default();
let message: Option<Message<'_>> = parser.parse(raw_bytes);
// OR: let message: Option<Message<'_>> = Message::parse(raw_bytes);  // convenience
```

### Core Type: `Message<'x>` — Key Methods

```rust
impl<'x> Message<'x> {
    // ── Header Accessors ──
    pub fn subject(&self)    -> Option<&str>
    pub fn message_id(&self) -> Option<&str>
    pub fn date(&self)       -> Option<&DateTime>

    // ── Address Accessors — return Option<&Address<'x>> ──
    pub fn from(&self)    -> Option<&Address<'x>>
    pub fn to(&self)      -> Option<&Address<'x>>
    pub fn cc(&self)      -> Option<&Address<'x>>
    pub fn bcc(&self)     -> Option<&Address<'x>>
    pub fn sender(&self)  -> Option<&Address<'x>>
    pub fn reply_to(&self) -> Option<&Address<'x>>

    // ── Body Accessors — take a position index (usually 0) ──
    pub fn body_text(&'x self, pos: usize) -> Option<Cow<'x, str>>
    pub fn body_html(&'x self, pos: usize) -> Option<Cow<'x, str>>

    // ── Headers Collection ──
    pub fn headers(&self) -> &[Header]           // ⚠️ Returns a SLICE, not Vec
    pub fn headers_raw(&self) -> impl Iterator<Item = (&str, &str)>
    pub fn header(&self, name: impl Into<HeaderName>) -> Option<&HeaderValue>

    // ── Attachments ──
    pub fn attachments(&'x self) -> AttachmentIterator<'x>   // Iterator, not Vec
    pub fn attachment(&self, pos: usize) -> Option<&MessagePart<'x>>
    pub fn attachment_count(&self) -> usize
}
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `msg.headers()` returns `Vec` | **No.** Returns `&[Header]` (a slice). Use `.iter()` before `.map()`. |
| `msg.body_text()` takes no args | **No.** Requires `pos: usize` argument (usually `0`). |
| `msg.body_html()` takes no args | Same — requires `pos: usize`. |
| `part.is_content_disposition_inline()` | **Does NOT exist.** Use `MimeHeaders::content_disposition()`. |
| `msg.attachments()` returns `Vec` | **No.** Returns `AttachmentIterator`. |

### Address Type

```rust
pub enum Address<'x> {
    List(Vec<Addr<'x>>),       // Flat list of addresses
    Group(Vec<Group<'x>>),     // Named groups of addresses
}

pub struct Addr<'x> {
    pub name:    Option<Cow<'x, str>>,
    pub address: Option<Cow<'x, str>>,
}

pub struct Group<'x> {
    pub name:      Option<Cow<'x, str>>,
    pub addresses: Vec<Addr<'x>>,
}
```

### Extracting Addresses Pattern

```rust
fn extract_addresses(addr: &Address<'_>) -> Vec<(String, String)> {
    match addr {
        Address::List(list) => list.iter().map(|a| {
            (
                a.name.as_deref().unwrap_or("").to_string(),
                a.address.as_deref().unwrap_or("").to_string(),
            )
        }).collect(),
        Address::Group(groups) => groups.iter().flat_map(|g| {
            g.addresses.iter().map(|a| {
                (
                    a.name.as_deref().unwrap_or("").to_string(),
                    a.address.as_deref().unwrap_or("").to_string(),
                )
            })
        }).collect(),
    }
}
```

### MimeHeaders Trait (on MessagePart)

```rust
pub trait MimeHeaders<'x> {
    fn content_disposition(&self) -> Option<&ContentType<'x>>
    fn content_type(&self)        -> Option<&ContentType<'x>>
    fn attachment_name(&self)     -> Option<&str>
}

pub struct ContentType<'x> {
    pub c_type:    Cow<'x, str>,      // e.g. "text/plain"
    pub c_subtype: Option<Cow<'x, str>>,
    pub attributes: Option<Vec<(Cow<'x, str>, Cow<'x, str>)>>,
}
```

### Header Types

```rust
pub struct Header<'x> {
    pub name:         HeaderName<'x>,
    pub value:        HeaderValue<'x>,
    pub offset_field: usize,
    pub offset_start: usize,
    pub offset_end:   usize,
}

pub enum HeaderValue<'x> {
    Address(Address<'x>),
    Text(Cow<'x, str>),
    TextList(Vec<Cow<'x, str>>),
    DateTime(DateTime),
    ContentType(ContentType<'x>),
    Received(Box<Received<'x>>),
    Empty,
}
```

---

## goblin 0.8.2

Crate for parsing PE, ELF, and Mach-O binary formats.

### Top-Level Parsing

```rust
use goblin::Object;

match Object::parse(&bytes)? {
    Object::Elf(elf)     => { /* elf::Elf<'a> */ },
    Object::PE(pe)       => { /* pe::PE<'a> */ },
    Object::Mach(mach)   => { /* mach::Mach<'a> — see note below */ },
    Object::COFF(coff)   => { /* pe::Coff<'a> */ },
    Object::Archive(ar)  => { /* archive::Archive<'a> */ },
    Object::Unknown(mag) => { /* u64 magic */ },
}
```

### ⚠️ Mach is an Enum, Not a Struct

```rust
pub enum Mach<'a> {
    Fat(MultiArch<'a>),    // Universal binary container
    Binary(MachO<'a>),     // Single-arch Mach-O
}
// You must match on Mach::Binary(macho) to get a MachO.
```

### PE<'a> — Key Fields

```rust
pub struct PE<'a> {
    pub header:      header::Header,
    pub sections:    Vec<section_table::SectionTable>,
    pub size:        usize,
    pub name:        Option<&'a str>,
    pub is_lib:      bool,
    pub is_64:       bool,
    pub entry:       usize,
    pub image_base:  usize,
    pub exports:     Vec<export::Export<'a>>,    // Already resolved
    pub imports:     Vec<import::Import<'a>>,    // Already resolved
    pub libraries:   Vec<&'a str>,               // DLL names
    pub debug_data:  Option<debug::DebugData<'a>>,
    pub export_data: Option<export::ExportData<'a>>,
    pub import_data: Option<import::ImportData<'a>>,
}
```

### MachO<'a> — Key Fields & Methods

```rust
pub struct MachO<'a> {
    pub header:        header::Header,
    pub load_commands: Vec<load_command::LoadCommand>,
    pub segments:      segment::Segments<'a>,
    pub symbols:       Option<symbols::Symbols<'a>>,
    pub libs:          Vec<&'a str>,        // ⚠️ Field is `libs`, not `libraries`
    pub rpaths:        Vec<&'a str>,
    pub entry:         u64,
    pub name:          Option<&'a str>,
    pub little_endian: bool,
    pub is_64:         bool,
}

impl<'a> MachO<'a> {
    pub fn symbols(&self) -> symbols::SymbolIterator<'a>         // ⚠️ Returns ITERATOR, not Result
    pub fn imports(&self) -> error::Result<Vec<imports::Import>> // Returns Result
    pub fn exports(&self) -> error::Result<Vec<exports::Export>> // Returns Result
}
```

### Elf<'a> — Key Fields

```rust
pub struct Elf<'a> {
    pub header:          Header,
    pub program_headers: ProgramHeaders,
    pub section_headers: SectionHeaders,
    pub shdr_strtab:     Strtab<'a>,
    pub dynstrtab:       Strtab<'a>,
    pub dynsyms:         Symtab<'a>,
    pub syms:            Symtab<'a>,
    pub strtab:          Strtab<'a>,
    pub dynamic:         Option<Dynamic>,
    pub libraries:       Vec<&'a str>,
    pub soname:          Option<&'a str>,
    pub interpreter:     Option<&'a str>,
    pub is_64:           bool,
    pub is_lib:          bool,
    pub entry:           u64,
    pub little_endian:   bool,
}
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `macho.symbols()` returns `Result` | **No.** Returns `SymbolIterator` directly (no `?`). |
| `macho.libraries` | **Field is `libs`**, not `libraries`. (Elf uses `libraries`.) |
| `Object::MachO(...)` | **No.** Variant is `Object::Mach(mach::Mach)`. Must then match `Mach::Binary(macho)`. |
| Importing PE symbols by group | `pe.imports` is a flat `Vec<Import>`, each with `.dll` and `.name`. Group by `.dll` yourself. |

---

## calamine 0.26.1

Crate for reading Excel (.xlsx, .xls, .xlsb) and ODS spreadsheet files.

### Opening a Workbook

```rust
use calamine::{open_workbook, open_workbook_auto, Reader, Xlsx, Xls, Ods, Range, Data};

// Type-specific:
let mut workbook: Xlsx<_> = open_workbook("file.xlsx")?;

// Auto-detect format:
let mut workbook = open_workbook_auto("file.xlsx")?;
```

### Reader Trait — Key Methods

```rust
pub trait Reader<RS>: Sized {
    fn worksheet_range(&mut self, name: &str)  -> Result<Range<Data>, Self::Error>;
    fn worksheets(&mut self)                   -> Vec<(String, Range<Data>)>;
    fn sheet_names(&self)                      -> Vec<String>;
    fn worksheet_range_at(&mut self, n: usize) -> Option<Result<Range<Data>, Self::Error>>;
    fn metadata(&self)                         -> &Metadata;
}
```

### Range<T> — Key Methods

```rust
pub struct Range<T> {
    start: (u32, u32),   // (row, col) — private
    end:   (u32, u32),   // (row, col) — private
    inner: Vec<T>,       // private
}

impl<T: CellType> Range<T> {
    pub fn get_size(&self) -> (usize, usize)       // Returns (height, width) ⚠️ NOT (rows, cols)
    pub fn height(&self)   -> usize
    pub fn width(&self)    -> usize
    pub fn is_empty(&self) -> bool
    pub fn start(&self)    -> Option<(u32, u32)>   // (row, col) or None if empty
    pub fn end(&self)      -> Option<(u32, u32)>
    pub fn rows(&self)     -> Rows<'_, T>          // Iterator over rows
}
```

### Data Enum (Cell Values)

```rust
pub enum Data {
    Int(i64),
    Float(f64),
    String(String),
    Bool(bool),
    DateTime(ExcelDateTime),
    DateTimeIso(String),
    DurationIso(String),
    Error(CellErrorType),
    Empty,                      // Default variant
}
```

### Sheets Enum (for auto-detect)

```rust
pub enum Sheets<RS> {
    Xls(Xls<RS>),
    Xlsx(Xlsx<RS>),
    Xlsb(Xlsb<RS>),
    Ods(Ods<RS>),
}
// Implements Reader trait
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `range.get_size()` returns `(rows, cols)` | Returns `(height, width)` — same thing, but height = rows, width = cols. |
| `range.rows` is a field | **No.** `rows()` is a method returning an iterator. |
| `workbook.sheets()` | **Does not exist.** Use `sheet_names()` or `worksheets()`. |

---

## cfb 0.14

Crate for reading Compound File Binary Format (OLE2/COM Structured Storage) — used by legacy `.doc`, `.ppt`, `.xls`, and `.msg` files.

### Opening a Compound File

```rust
use cfb::CompoundFile;

// From a file path
let mut comp = CompoundFile::open("legacy.doc")?;

// From a reader (e.g., bytes in memory)
let cursor = std::io::Cursor::new(bytes);
let mut comp = CompoundFile::open(cursor)?;
```

### Listing & Reading Streams

```rust
// List all entries (directory walk)
for entry in comp.walk() {
    println!("{}: dir={}, len={}", entry.path().display(), entry.is_dir(), entry.len());
}

// Check if a stream/storage exists
let exists = comp.exists("/WordDocument");

// Read a stream into bytes
let mut stream = comp.open_stream("/WordDocument")?;
let mut buf = Vec::new();
stream.read_to_end(&mut buf)?;
```

### ⚠️ Key Gotchas

| Assumption | Reality |
|------------|---------|
| `CompoundFile::open()` returns `Result<CompoundFile<F>>` | Correct — `F` is the inner reader type (e.g., `File` or `Cursor<Vec<u8>>`). |
| Streams are at root level | Paths use `/` separator, e.g., `/WordDocument`, `/PowerPoint Document`. |
| `entry.len()` is the decompressed size | Yes — OLE2 doesn't compress, it's the raw stream size. |
| `comp.read_stream()` exists | **No.** Use `comp.open_stream(path)` → `Read` trait → `read_to_end()`. |

### Usage in CORE-FFX

Used in `viewer/document/office.rs` for legacy `.doc` and `.ppt` text extraction. Opens the OLE2 container, reads the `WordDocument` or `PowerPoint Document` stream, and scans for printable text (UTF-16LE + ASCII).

---

## notatin 1.0.1

Crate for parsing Windows registry hive files.

### Opening a Hive

```rust
use notatin::{parser_builder::ParserBuilder, parser::Parser, cell_key_node::CellKeyNode};

let mut parser = ParserBuilder::from_path("NTUSER.DAT")
    .recover_deleted(true)           // Optional
    .with_transaction_log("NTUSER.DAT.LOG1")  // Optional
    .build()?;

// Or from bytes:
let mut parser = ParserBuilder::from_file(cursor)
    .build()?;
```

### Iterating Keys

```rust
// Get root key
let root: Option<CellKeyNode> = parser.get_root_key()?;

// Iterate all keys
let mut iter = ParserIterator::new(&parser);
for key in iter.iter() {
    println!("{}", key.path);
    for (value, _) in key.value_iter() {
        let (cell_value, _logs) = value.get_content();
        println!("  {} = {}", value.get_pretty_name(), cell_value);
    }
}
```

### CellKeyNode (Registry Key)

```rust
pub struct CellKeyNode {
    pub file_offset_absolute: usize,
    pub key_name:             String,
    pub path:                 String,
    pub cell_state:           CellState,
    pub sequence_num:         Option<u32>,
    pub logs:                 Logs,
    // ... internal fields
}

impl CellKeyNode {
    pub fn read_sub_keys(&mut self, parser: &mut Parser) -> Vec<Self>
    pub fn get_sub_key_by_path(&mut self, parser: &mut Parser, sub_path: &str) -> Option<Self>
    pub fn value_iter(&self) -> CellKeyNodeValueIterator<'_>
    pub fn next_sub_key(&mut self, parser: &mut Parser) -> Option<CellKeyNode>
}
```

### CellKeyValue (Registry Value)

```rust
pub struct CellKeyValue {
    pub file_offset_absolute: usize,
    pub data_type:            CellKeyValueDataTypes,
    pub flags:                CellKeyValueFlags,
    pub cell_state:           CellState,
    pub logs:                 Logs,
    // detail is private enum (Full/Light)
}

impl CellKeyValue {
    pub fn get_content(&self) -> (CellValue, Option<Logs>)
    pub fn get_pretty_name(&self) -> String
}
```

### CellValue Enum

```rust
pub enum CellValue {
    None,
    Binary(Vec<u8>),
    String(String),
    MultiString(Vec<String>),
    U32(u32),
    I32(i32),
    U64(u64),
    I64(i64),
    Error,
}
// Implements Display trait
```

### CellKeyValueDataTypes (Common Variants)

```rust
pub enum CellKeyValueDataTypes {
    REG_NONE        = 0x0000,
    REG_SZ          = 0x0001,
    REG_EXPAND_SZ   = 0x0002,
    REG_BIN         = 0x0003,
    REG_DWORD       = 0x0004,
    REG_MULTI_SZ    = 0x0007,
    REG_QWORD       = 0x000B,
    REG_FILETIME    = 0x0010,
    // ... plus many REG_COMPOSITE_* variants for UWP settings.dat
    REG_UNKNOWN     = 999,
}
```

---

## plist 1.8

Crate for parsing Apple property list files (binary, XML, ASCII formats).

### Core Type: `Value`

```rust
use plist::Value;

pub enum Value {
    Array(Vec<Value>),
    Dictionary(Dictionary),      // NOT HashMap — use Dictionary type
    Boolean(bool),
    Data(Vec<u8>),
    Date(Date),
    Real(f64),
    Integer(Integer),            // NOT i64/u64 directly
    String(String),
    Uid(Uid),
}
```

### Opening a Plist

```rust
// Auto-detect format (binary, XML, ASCII):
let value = Value::from_file("Info.plist")?;
let value = Value::from_reader(reader)?;         // reader: Read + Seek

// Format-specific:
let value = Value::from_reader_xml(reader)?;     // reader: Read (no Seek needed)
let value = Value::from_reader_ascii(reader)?;   // reader: Read
```

### Accessor Methods (all return Option)

```rust
value.as_dictionary()         -> Option<&Dictionary>
value.as_array()              -> Option<&Vec<Value>>
value.as_string()             -> Option<&str>
value.as_boolean()            -> Option<bool>
value.as_signed_integer()     -> Option<i64>
value.as_unsigned_integer()   -> Option<u64>
value.as_real()               -> Option<f64>
value.as_data()               -> Option<&[u8]>
value.as_date()               -> Option<Date>
value.as_uid()                -> Option<&Uid>

// Consuming variants:
value.into_string()           -> Option<String>
value.into_array()            -> Option<Vec<Value>>
value.into_dictionary()       -> Option<Dictionary>
value.into_data()             -> Option<Vec<u8>>
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `value.as_integer()` | **Does NOT exist.** Use `as_signed_integer()` or `as_unsigned_integer()`. |
| `value.as_dict()` | **Does NOT exist.** Use `as_dictionary()`. |
| `Value::Dict(...)` | **Variant is `Dictionary`**, not `Dict`. |
| `Value::Int(42)` | **Variant is `Integer(Integer)`**, not `Int(i64)`. |

---

## kamadak-exif 0.5.5

Crate for EXIF metadata extraction from JPEG/TIFF/WebP images. Import as `exif`.

### Parsing EXIF Data

```rust
use exif::{In, Reader, Tag};

let reader = Reader::new();

// From a file container (JPEG/TIFF/WebP auto-detected):
let file = std::fs::File::open("photo.jpg")?;
let mut bufreader = std::io::BufReader::new(file);
let exif = reader.read_from_container(&mut bufreader)?;

// From raw EXIF bytes:
let exif = reader.read_raw(raw_exif_bytes)?;
```

### Core Types

```rust
pub struct Exif { /* private fields */ }

impl Exif {
    pub fn fields(&self) -> impl ExactSizeIterator<Item = &Field>
    pub fn get_field(&self, tag: Tag, ifd_num: In) -> Option<&Field>
    pub fn little_endian(&self) -> bool
    pub fn buf(&self) -> &[u8]
}

pub struct Field {
    pub tag:     Tag,
    pub ifd_num: In,
    pub value:   Value,
}

impl Field {
    pub fn display_value(&self) -> DisplayValue    // For human-readable display
}

// IFD selector:
pub struct In(pub u16);
// Constants:
In::PRIMARY    // IFD 0 (main image)
In::THUMBNAIL  // IFD 1 (thumbnail)

// Tag constants (use as Tag::Make, Tag::Model, etc.):
Tag::Make, Tag::Model, Tag::DateTime, Tag::ExposureTime,
Tag::FNumber, Tag::ISOSpeedRatings, Tag::FocalLength,
Tag::GPSLatitude, Tag::GPSLongitude, Tag::ImageWidth, Tag::ImageLength,
// ... hundreds more
```

### Usage Pattern

```rust
if let Some(field) = exif.get_field(Tag::Make, In::PRIMARY) {
    println!("Camera: {}", field.display_value());
}
for field in exif.fields() {
    println!("{}: {}", field.tag, field.display_value());
}
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `exif.get_field(Tag::Make)` | **Requires two args**: `get_field(tag, ifd_num)`. Always pass `In::PRIMARY`. |
| `field.value.to_string()` | Use `field.display_value()` for human-readable formatting. |
| `Reader::read(data)` | **Does NOT exist.** Use `read_raw(Vec<u8>)` or `read_from_container(&mut reader)`. |

---

## lopdf 0.34

Crate for low-level PDF parsing and structure analysis.

### Core Type: `Document`

```rust
use lopdf::Document;

// Loading:
let doc = Document::load("file.pdf")?;
let doc = Document::load_mem(&bytes)?;
let doc = Document::load_from(reader)?;         // reader: Read
```

### Key Fields & Methods

```rust
pub struct Document {
    pub version:     String,
    pub trailer:     Dictionary,
    pub objects:     BTreeMap<ObjectId, Object>,
    pub max_id:      u32,
    // ...
}

impl Document {
    pub fn get_pages(&self) -> BTreeMap<u32, ObjectId>     // page_num → ObjectId
    pub fn get_page_content(&self, page_id: ObjectId) -> Result<Vec<u8>>
    pub fn get_page_contents(&self, page_id: ObjectId) -> Vec<ObjectId>
    pub fn extract_text(&self, page_numbers: &[u32]) -> Result<String>
}
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `doc.pages()` | **Does NOT exist.** Use `get_pages()`. |
| `doc.page_count()` | **Does NOT exist.** Use `get_pages().len()`. |
| `doc.extract_text(1)` | **Takes `&[u32]`**, not a single `u32`. Use `&[1]`. |
| `Document::open(path)` | **Does NOT exist.** Use `Document::load(path)`. |

---

## zip 2.4

Crate for reading ZIP archives.

### ZipArchive — Key Methods

```rust
use zip::ZipArchive;

let file = std::fs::File::open("archive.zip")?;
let mut archive = ZipArchive::new(file)?;

impl<R: Read + Seek> ZipArchive<R> {
    pub fn len(&self) -> usize                                        // Number of entries
    pub fn is_empty(&self) -> bool
    pub fn file_names(&self) -> impl Iterator<Item = &str>
    pub fn by_index(&mut self, index: usize) -> ZipResult<ZipFile<'_>>
    pub fn by_name(&mut self, name: &str) -> ZipResult<ZipFile<'_>>
    pub fn by_index_raw(&mut self, index: usize) -> ZipResult<ZipFile<'_>>
}
```

### ZipFile — Entry Access (implements Read)

```rust
pub struct ZipFile<'a> { /* ... */ }

impl ZipFile<'_> {
    pub fn name(&self)            -> &str
    pub fn size(&self)            -> u64       // Uncompressed size
    pub fn compressed_size(&self) -> u64
    pub fn is_dir(&self)          -> bool
    pub fn is_file(&self)         -> bool
    // ZipFile implements Read — use read_to_end(), etc.
}
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `archive.entries()` | **Does NOT exist.** Iterate with `for i in 0..archive.len()` + `by_index(i)`. |
| `entry.filename()` | **Method is `name()`**, not `filename()`. |
| `ZipArchive::open(path)` | **Does NOT exist.** Use `ZipArchive::new(File::open(path)?)`. |

---

## libarchive2 0.2.1

Safe Rust bindings for libarchive v3.8.1 — unified archive reading backend.

### ReadArchive — Opening & Iterating

```rust
use libarchive2::{ReadArchive, FileType};

// Open from path:
let mut archive = ReadArchive::open("archive.7z")?;

// Open with passphrase:
let mut archive = ReadArchive::open_with_passphrase("file.rar", "password")?;

// Open from memory:
let mut archive = ReadArchive::open_memory(&data)?;

// Open multi-volume:
let mut archive = ReadArchive::open_filenames(&["vol.7z.001", "vol.7z.002"])?;

// Iterate entries:
while let Some(entry) = archive.next_entry()? {
    let path = entry.pathname();        // Option<String>
    let ftype = entry.file_type();      // FileType enum
    let size = entry.size();            // i64
    
    // Read data:
    let data = archive.read_data_to_vec()?;
}
```

### Entry Methods

```rust
impl Entry<'_> {
    pub fn pathname(&self)     -> Option<String>
    pub fn file_type(&self)    -> FileType
    pub fn size(&self)         -> i64           // ⚠️ i64, not u64
    pub fn mode(&self)         -> u32
    pub fn mtime(&self)        -> Option<SystemTime>
    pub fn is_encrypted(&self) -> bool
    pub fn uid(&self)          -> Option<u64>
    pub fn gid(&self)          -> Option<u64>
    pub fn symlink(&self)      -> Option<String>
    pub fn hardlink(&self)     -> Option<String>
}
```

### FileType Enum

```rust
pub enum FileType {
    RegularFile,
    Directory,
    SymbolicLink,
    BlockDevice,
    CharacterDevice,
    Fifo,
    Socket,
    Unknown,
}
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `entry.name()` | **Does NOT exist.** Use `pathname()` → returns full path as `Option<String>`. |
| `entry.is_dir()` | **Does NOT exist.** Use `entry.file_type() == FileType::Directory`. |
| `entry.size()` returns `u64` | **No.** Returns `i64`. |
| `archive.entries()` | **Does NOT exist.** Use `while let Some(entry) = archive.next_entry()?` loop. |

---

## unrar 0.5.8

Crate for reading RAR archives via the unrar library.

### Opening & Listing

```rust
use unrar::Archive;

// List entries:
let archive = Archive::new("file.rar")
    .open_for_listing()?;

// With password:
let archive = Archive::with_password("file.rar", "pass")
    .open_for_listing()?;

// Process (extract) entries:
let archive = Archive::new("file.rar")
    .open_for_processing()?;
```

### Cursor-Based Iteration (state machine)

```rust
// Listing:
let mut archive = Archive::new("file.rar").open_for_listing()?;
while let Ok(Some(archive)) = archive.read_header() {
    let entry = archive.entry();       // &FileHeader
    println!("{}: {}", entry.filename.display(), entry.unpacked_size);
    archive = archive.skip()?;         // Move to next entry
}

// Processing (reading data):
let mut archive = Archive::new("file.rar").open_for_processing()?;
while let Ok(Some(archive)) = archive.read_header() {
    let entry = archive.entry();
    let (data, next) = archive.read()?;   // Vec<u8> + next state
    archive = next;
}
```

### FileHeader

```rust
pub struct FileHeader {
    pub filename:      PathBuf,
    pub unpacked_size: u64,
    pub file_crc:      u32,
    pub file_time:     u32,
    pub method:        u32,
    pub file_attr:     u32,
    // flags is private
}

impl FileHeader {
    pub fn is_directory(&self)  -> bool    // ⚠️ Note: `is_directory`, NOT `is_dir`
    pub fn is_file(&self)       -> bool
    pub fn is_encrypted(&self)  -> bool
    pub fn is_split(&self)      -> bool
    pub fn is_split_after(&self) -> bool
    pub fn is_split_before(&self) -> bool
}
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `Archive::open("file.rar")` | **Does NOT exist.** Use `Archive::new(path).open_for_listing()` or `open_for_processing()`. |
| `archive.list()` | **Does NOT exist.** Use cursor-based `read_header()` → `skip()` loop. |
| `entry.is_dir()` | **Method is `is_directory()`** on `FileHeader` (opposite of `ArchiveEntryInfo.is_dir`!). |
| `entry.name` / `entry.name()` | **Field is `filename: PathBuf`**. Use `entry.filename.display()` or `.to_string_lossy()`. |
| `archive.read()` consumes `self` | Each step returns a **new state**. Must reassign: `archive = archive.skip()?`. |

---

## quick_xml 0.37.5

SAX-style XML parser, used heavily for UFED XML, DOCX internals, Axiom parsing, and generic XML files.

**Used in:** `ufed/mod.rs`, `viewer/document/docx.rs`, `viewer/document/mod.rs`, `commands/axiom.rs` (11+ files)

### Core Types

```rust
use quick_xml::Reader;
use quick_xml::events::{Event, BytesStart, BytesEnd, BytesText};

// --- Reader construction ---
// From byte slice (returns borrowed events — no allocation):
let mut reader = Reader::from_str(xml_string);   // &str input
let mut reader = Reader::from_reader(buf_reader); // impl BufRead input

// --- Event loop (buffered reader) ---
let mut buf = Vec::new();
loop {
    match reader.read_event_into(&mut buf) {        // ⚠️ NOT read_event() for buffered
        Ok(Event::Start(e)) => { /* opening tag */ },
        Ok(Event::End(e))   => { /* closing tag */ },
        Ok(Event::Empty(e)) => { /* self-closing <tag /> */ },
        Ok(Event::Text(e))  => { /* text content */ },
        Ok(Event::Eof)      => break,
        Err(e) => { /* handle error */ },
        _ => {},                                     // CData, Comment, Decl, PI, DocType
    }
    buf.clear();
}

// --- Event loop (slice reader — from_str) ---
loop {
    match reader.read_event() {                      // ⚠️ NOT read_event_into() for slices
        Ok(Event::Start(e)) => { /* ... */ },
        Ok(Event::Eof) => break,
        _ => {},
    }
}
```

### Event Enum Variants

```rust
pub enum Event<'a> {
    Start(BytesStart<'a>),   // <tag attr="val">
    End(BytesEnd<'a>),       // </tag>
    Empty(BytesStart<'a>),   // <tag /> (self-closing, shares BytesStart type!)
    Text(BytesText<'a>),     // Text content between tags
    CData(BytesCData<'a>),   // <![CDATA[...]]>
    Comment(BytesText<'a>),  // <!-- ... -->
    Decl(BytesDecl<'a>),     // <?xml ... ?>
    PI(BytesPI<'a>),         // <?target ... ?>
    DocType(BytesText<'a>),  // <!DOCTYPE ... >
    Eof,                     // End of input
}
```

### Key Methods on Event Types

```rust
// BytesStart (for Start and Empty events):
e.name()          -> QName<'a>       // Full qualified name
e.local_name()    -> LocalName<'a>   // Local name (no namespace prefix)
e.attributes()    -> Attributes<'a>  // Iterator over attributes

// Comparing tag names:
e.name().as_ref() == b"tagname"      // Compare as byte slice

// BytesText:
e.unescape()      -> Result<Cow<'a, str>>  // ⚠️ Returns Result, must unwrap/handle

// Attributes iteration:
for attr in e.attributes() {
    let attr = attr?;                // ⚠️ Each attribute is Result
    let key = attr.key.as_ref();     // &[u8]
    let val = attr.unescape_value()? // Cow<str>
}
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| Using `read_event()` with buffered reader | **Use `read_event_into(&mut buf)` for `Reader::from_reader()`**; `read_event()` is only for `Reader::from_str()`. |
| `Event::Empty` uses `BytesEnd` | **`Empty` uses `BytesStart`**, same as `Start`. Both have `name()`, `attributes()`. |
| `e.unescape()` returns `String` | **Returns `Result<Cow<'a, str>>`** — must handle the `Result`. |
| Attribute values are `String` | **Attributes are `Result<Attribute>`** — must unwrap each one, values are `Cow<[u8]>`. |
| `reader.read_to_end(tag)` takes `&str` | **Takes `QName`** — use `e.name().to_end()` to get the matching end name. |

---

## rusqlite 0.32.1

SQLite database bindings, used for hash database, Axiom case files, database viewer, and per-project .ffxdb persistence.

**Used in:** `commands/database.rs`, `commands/axiom.rs`, `viewer/document/database_viewer.rs`, `containers/traits.rs`, `project_db.rs`, `commands/project_db.rs`

### Connection

```rust
use rusqlite::{Connection, params, OpenFlags, Result};

// Open read-write (creates if missing):
let conn = Connection::open("path/to/db.sqlite")?;       // -> Result<Connection>

// Open read-only (forensic best practice):
let conn = Connection::open_with_flags(
    "path/to/db.sqlite",
    OpenFlags::SQLITE_OPEN_READ_ONLY,
)?;

// In-memory database:
let conn = Connection::open_in_memory()?;
```

### Querying

```rust
// Execute (INSERT/UPDATE/DELETE) — returns rows affected:
let count = conn.execute(
    "INSERT INTO hashes (path, hash) VALUES (?1, ?2)",
    params!["file.txt", "abc123"],                        // ⚠️ Use params![] macro
)?;  // -> Result<usize>

// Prepare + query_map (SELECT multiple rows):
let mut stmt = conn.prepare("SELECT id, name FROM files WHERE type = ?1")?;
let rows = stmt.query_map(params![file_type], |row| {
    Ok(FileRecord {
        id: row.get(0)?,           // ⚠️ get() is generic: row.get::<_, i64>(0)?
        name: row.get(1)?,         //    Type inferred from struct field
    })
})?;  // -> Result<MappedRows<'_, F>>

// Collect results (each row is Result):
let files: Vec<FileRecord> = rows.collect::<Result<Vec<_>, _>>()?;

// Single row query:
let name: String = conn.query_row(
    "SELECT name FROM files WHERE id = ?1",
    params![file_id],
    |row| row.get(0),             // -> Result<T>
)?;

// Optional single row (no error if 0 rows):
let name: Option<String> = conn.query_row(
    "SELECT name FROM files WHERE id = ?1",
    params![file_id],
    |row| row.get(0),
).optional()?;                    // ⚠️ .optional() converts no-row error to Ok(None)
```

### Row Access

```rust
// Index-based (0-indexed):
row.get::<_, String>(0)?          // Column 0 as String
row.get::<_, i64>(1)?             // Column 1 as i64
row.get::<_, Vec<u8>>(2)?        // Column 2 as blob
row.get::<_, Option<String>>(3)?  // Nullable column

// Named columns (if enabled — not commonly used in this project):
row.get::<_, String>("name")?
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `conn.execute()` returns `bool` | **Returns `Result<usize>`** (number of rows affected). |
| `params!` takes `Vec` | **`params![]` is a macro** that takes inline values: `params![a, b, c]`. For dynamic params use `rusqlite::params_from_iter()`. |
| `query_map` returns collected `Vec` | **Returns `MappedRows` iterator** — each element is `Result<T>`. Must `.collect::<Result<Vec<_>, _>>()`. |
| `row.get(0)` returns value directly | **Returns `Result<T>`** — must use `?`. Also requires type annotation or inference. |
| `query_row` on 0 results is `Ok(None)` | **0 results is an `Err`** (`QueryReturnedNoRows`). Use `.optional()?` for `Option<T>`. |
| Opening is always read-write | **Default `open()` is read-write**. Use `open_with_flags(path, SQLITE_OPEN_READ_ONLY)` for forensic read-only. |

---

## ntfs 0.4.0

Pure-Rust NTFS filesystem parser for reading NTFS volumes from raw images.

**Used in:** `containers/ntfs_driver.rs` (353 lines)

### Core Types

```rust
use ntfs::Ntfs;
use ntfs::NtfsFile;

// Initialize from a Read+Seek source (e.g., a partition slice):
let mut fs = BufReader::new(File::open("ntfs_image.raw")?);
let ntfs = Ntfs::new(&mut fs)?;                // ⚠️ Takes &mut T where T: Read + Seek
```

### Ntfs Methods

```rust
ntfs.root_directory(&mut fs)?     // -> Result<NtfsFile<'n>>  (root MFT entry)
ntfs.file(&mut fs, record_num)?   // -> Result<NtfsFile<'n>>  (MFT entry by number)
ntfs.cluster_size()               // -> u32
ntfs.sector_size()                // -> u16
ntfs.serial_number()              // -> u64
ntfs.size()                       // -> u64
ntfs.file_record_size()           // -> u32
ntfs.volume_name(&mut fs)         // -> Option<Result<NtfsVolumeName>>
ntfs.volume_info(&mut fs)?        // -> Result<NtfsVolumeInformation>
ntfs.read_upcase_table(&mut fs)?  // -> Result<()>  (needed for case-insensitive lookups)
```

### NtfsFile Methods

```rust
file.is_directory()               // -> bool
file.file_record_number()         // -> u64
file.allocated_size()             // -> u32
file.data_size()                  // -> u32
file.hard_link_count()            // -> u16
file.flags()                      // -> NtfsFileFlags
file.info()?                      // -> Result<NtfsStandardInformation>
file.attributes()                 // -> NtfsAttributes<'n, 'f>  (iterator)
file.attributes_raw()             // -> NtfsAttributesRaw<'n, 'f>

// Get data stream (named or default):
file.data(&mut fs, "stream_name") // -> Option<Result<NtfsAttributeValue>>
file.data(&mut fs, "")            // default (unnamed) stream

// Get directory index for listing children:
file.directory_index(&mut fs)?    // -> Result<NtfsIndex<'n, 'f>>
                                  // ⚠️ Only valid for directories

// Get filename:
file.name(&mut fs, None, None)?   // -> Result<NtfsFileName>
```

### NtfsFileFlags

```rust
bitflags! {
    pub struct NtfsFileFlags: u16 {
        const READ_ONLY    = 0x0001;
        const HIDDEN       = 0x0002;
        const SYSTEM       = 0x0004;
        const ARCHIVE      = 0x0020;
        const DIRECTORY    = 0x10000000; // Use file.is_directory() instead
    }
}
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `Ntfs::new(fs)` takes owned value | **Takes `&mut T`** — borrows the filesystem source. |
| `root_directory()` has no args | **Requires `&mut fs`** — all navigation needs the reader: `ntfs.root_directory(&mut fs)?`. |
| `file.data()` returns data directly | **Returns `Option<Result<...>>`** — `None` if stream doesn't exist, `Result` for I/O errors. |
| `directory_index()` works on any file | **Only valid for directories** — panics or errors on regular files. Check `is_directory()` first. |
| `file.name()` is simple | **Takes `&mut fs, Option<u32>, Option<&NtfsUpcase>`** — pass `None, None` for basic usage. |
| Lifetimes are simple | **`NtfsFile<'n>` borrows from `Ntfs`** — `Ntfs` must outlive all `NtfsFile` references. |

---

## apple-dmg 0.5.0

Apple DMG disk image reader/writer.

**Used in:** `containers/dmg_driver.rs` (429 lines)

### DmgReader

```rust
use apple_dmg::DmgReader;

// Open from file path:
let mut dmg = DmgReader::open(Path::new("disk.dmg"))?;   // -> Result<DmgReader<BufReader<File>>>

// Open from any Read+Seek:
let mut dmg = DmgReader::new(reader)?;                    // -> Result<DmgReader<R>>
```

### DmgReader Methods

```rust
// Metadata:
dmg.koly()                        // -> &KolyTrailer  (DMG header/trailer)
dmg.plist()                       // -> &Plist         (partition map XML)

// Partition access:
dmg.plist().partitions()          // -> &[Partition]   (all partitions)
dmg.partition_name(i)             // -> &str           (name of partition i)
dmg.partition_table(i)?           // -> Result<BlkxTable>  (block table)
dmg.partition_data(i)?            // -> Result<Vec<u8>>    (full decompressed partition data)

// Low-level sector reading:
dmg.sector(&chunk)?               // -> Result<impl Read>  (decompressed sector data)

// Checksum:
dmg.data_checksum()?              // -> Result<u32>  (CRC32 of data fork)
```

### Partition Struct

```rust
pub struct Partition {
    pub attributes: String,       // e.g., "0x0050"
    pub cfname: String,           // CF name
    pub data: Vec<u8>,            // Raw BlkxTable data (call .table() to parse)
    pub id: String,               // Partition ID
    pub name: String,             // Human-readable name
}

impl Partition {
    pub fn table(&self) -> Result<BlkxTable> { /* parse self.data */ }
}
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `partitions()` is on `DmgReader` | **`partitions()` is on `Plist`** — use `dmg.plist().partitions()`. |
| `partition_data(i)` is cheap | **Decompresses entire partition into `Vec<u8>`** — can be very large. |
| `Partition.data` is the content | **`Partition.data` is raw BlkxTable bytes** — call `partition.table()?` to parse, or use `dmg.partition_data(i)?` for decompressed content. |
| `DmgReader` is `Send` | **`DmgReader` borrows internal reader** — not `Send` by default. Clone data out before crossing threads. |

---

## sevenz_rust 0.6.1

7-Zip archive reader/writer.

**Used in:** `archive/sevenz.rs` (569 lines)

### Reading Archives

```rust
use sevenz_rust::{SevenZReader, Password, SevenZArchiveEntry, Archive};

// Open with no password:
let mut reader = SevenZReader::open(
    "archive.7z",
    Password::empty(),              // ⚠️ Password is required even for non-encrypted
)?;                                 // -> Result<SevenZReader<BufReader<File>>>

// Open with password:
let mut reader = SevenZReader::open(
    "archive.7z",
    "secret".into(),                // Password implements From<&str>
)?;

// From any Read+Seek:
let mut reader = SevenZReader::new(
    source,                         // impl Read + Seek
    source_len,                     // u64 — total length of source
    Password::empty(),
)?;

// Access archive metadata:
reader.archive()                    // -> &Archive
reader.archive().files              // -> &[SevenZArchiveEntry]   ⚠️ NOT a method, it's a field
```

### Iterating Entries

```rust
// for_each_entries — callback with entry + reader for content:
reader.for_each_entries(|entry, reader| {
    // entry: &SevenZArchiveEntry
    // reader: &mut dyn Read         (decompressed content stream)
    
    let name = entry.name();         // -> &str
    let size = entry.size();         // -> u64
    let is_dir = entry.is_directory(); // -> bool  ⚠️ NOT is_dir()
    
    if !is_dir {
        let mut content = Vec::new();
        reader.read_to_end(&mut content)?;
    }
    
    Ok(true)  // ⚠️ Return Ok(true) to continue, Ok(false) to stop
})?;
```

### SevenZArchiveEntry Methods

```rust
entry.name()               // -> &str
entry.size()               // -> u64             (uncompressed size)
entry.is_directory()       // -> bool            ⚠️ NOT is_dir()
entry.has_stream()         // -> bool            (has file content)
entry.creation_date()      // -> FileTime
entry.last_modified_date() // -> FileTime
entry.access_date()        // -> FileTime
entry.windows_attributes() // -> u32
entry.is_anti_item()       // -> bool
```

### Password Type

```rust
pub struct Password(Vec<u8>);

Password::empty()           // No password
Password::from("secret")    // From &str (implements From<&str>)
password.is_empty()          // -> bool
```

### Convenience Functions

```rust
use sevenz_rust::{decompress_file, decompress_file_with_password};

// Extract entire archive to directory:
decompress_file("archive.7z", "output_dir/")?;
decompress_file_with_password("archive.7z", "output_dir/", "password".into())?;
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `SevenZReader::open(path)` (one arg) | **Requires 2 args: `(path, Password)`** — always pass `Password::empty()` for non-encrypted. |
| `entry.is_dir()` | **Method is `is_directory()`** — different from `ArchiveEntryInfo.is_dir`. |
| `for_each_entries` callback returns `()` | **Must return `Result<bool>`** — `Ok(true)` to continue, `Ok(false)` to stop iteration. |
| `archive.files()` method | **`files` is a public field**, not a method: `reader.archive().files`. |
| `archive().entries()` | **No `entries()` method** — use `archive().files` for the `Vec<SevenZArchiveEntry>`. |
| `entry.compressed_size()` exists | **No `compressed_size()` on `SevenZArchiveEntry`** — only `size()` (uncompressed). |

---

## docx_rs 0.4.18

DOCX document reader/writer (builder pattern).

**Used in:** `viewer/document/docx.rs` (writing summary tables), `report/docx.rs` (report generation)

### Reading DOCX Files

```rust
use docx_rs::read_docx;

let docx = read_docx(&file_bytes)?;     // -> Result<Docx, ReaderError>
// Access document content:
docx.document                            // Document (contains children/body)
docx.styles                              // Styles
docx.images                              // Vec<(String, String, Image, Png)>  (reader only)
docx.hyperlinks                          // Vec<(String, String, String)>      (reader only)
```

### Building DOCX Documents

```rust
use docx_rs::{Docx, Paragraph, Run, Table, TableRow, TableCell};
use docx_rs::{AlignmentType, WidthType, Shading};

// Build a document:
let docx = Docx::new()
    .add_paragraph(
        Paragraph::new()
            .add_run(Run::new().add_text("Hello world"))
            .align(AlignmentType::Center)
            .style("Heading1")
    )
    .add_table(
        Table::new(vec![
            TableRow::new(vec![
                TableCell::new()
                    .add_paragraph(Paragraph::new().add_run(Run::new().add_text("Cell 1")))
                    .width(5000, WidthType::Dxa),
                TableCell::new()
                    .add_paragraph(Paragraph::new().add_run(Run::new().add_text("Cell 2"))),
            ]),
        ])
    );

// Write to bytes:
let mut buf = Vec::new();
docx.build().pack(&mut buf)?;              // ⚠️ .build() then .pack()
```

### Key Builder Types

```rust
// Paragraph:
Paragraph::new()
    .add_run(run)                           // -> Paragraph
    .align(AlignmentType::Center)           // -> Paragraph
    .style("Heading1")                      // -> Paragraph

// Run:
Run::new()
    .add_text("text")                       // -> Run
    .size(24)                               // -> Run  (half-points: 24 = 12pt)
    .bold()                                 // -> Run
    .italic()                               // -> Run
    .color("FF0000")                        // -> Run
    .fonts(RunFonts::new())                 // -> Run

// Table:
Table::new(rows)                            // Vec<TableRow>
    .add_row(row)                           // -> Table
    .set_grid(vec![5000, 5000])             // -> Table (column widths)
    .width(10000, WidthType::Dxa)           // -> Table

// TableRow:
TableRow::new(cells)                        // Vec<TableCell>

// TableCell:
TableCell::new()
    .add_paragraph(para)                    // -> TableCell
    .width(5000, WidthType::Dxa)            // -> TableCell
    .shading(Shading::new())                // -> TableCell
    .vertical_align(VAlignType::Center)     // -> TableCell
    .grid_span(2)                           // -> TableCell (column span)
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `Docx::new().pack(&mut buf)` | **Must call `.build()` first**: `docx.build().pack(&mut buf)?`. |
| `read_docx(path_str)` | **Takes `&[u8]`** (file bytes), not a path string. Read file first. |
| `Run::new().text("hello")` | **Method is `add_text()`**: `Run::new().add_text("hello")`. |
| `Paragraph::new().run(r)` | **Method is `add_run()`**: `Paragraph::new().add_run(r)`. |
| `Table::new()` no args | **Requires `Vec<TableRow>`**: `Table::new(vec![row1, row2])`. |
| `size(12)` means 12pt | **Size is in half-points**: `size(24)` = 12pt font. |
| `Docx` has a `body` field | **No `body` field** — content is in `docx.document` (a `Document` struct). |

---

## seven_zip (sevenzip-ffi)

Custom FFI bindings to the 7-Zip SDK. This is the `sevenzip-ffi` workspace crate, imported as `seven_zip`.

**Used in:** `commands/archive/tools.rs`, `commands/archive_create.rs`

### SevenZip (Main Interface)

```rust
use seven_zip::{SevenZip, CompressionLevel, CompressOptions, StreamOptions};

// Initialize (required before any operation):
let sz = SevenZip::new()?;                // -> Result<SevenZip>

// Extract:
sz.extract("archive.7z", "output_dir")?;  // -> Result<()>

// Extract with password + progress:
sz.extract_with_password(
    "archive.7z",
    "output_dir",
    Some("password"),                      // Option<&str>
    Some(Box::new(|completed, total| {     // Option<ProgressCallback>
        println!("{}/{}", completed, total);
    })),
)?;

// Extract specific files:
sz.extract_files(
    "archive.7z",
    "output_dir",
    &["file1.txt", "dir/file2.txt"],       // &[&str]
    None,                                   // password: Option<&str>
)?;

// List contents:
let entries = sz.list("archive.7z", None)?;  // -> Result<Vec<ArchiveEntry>>

// Create archive:
sz.create_archive(
    "archive.7z",
    &["file1.txt", "directory/"],           // &[impl AsRef<Path>]
    CompressionLevel::Normal,
    None,                                    // Option<&CompressOptions>
)?;

// Create encrypted archive (convenience method):
sz.create_encrypted_archive(
    "secure.7z",
    &["sensitive.doc"],
    "password",                              // &str
    CompressionLevel::Normal,
)?;

// Create with smart defaults (auto-tuned threads, incompressibility detection):
sz.create_smart_archive(
    "backup.7z",
    &["data/"],
    CompressionLevel::Normal,
)?;

// Test integrity:
sz.test_archive(
    "archive.7z",
    None,                                    // password: Option<&str>
    None,                                    // progress: Option<BytesProgressCallback>
)?;

// Repair:
sz.repair_archive(
    "corrupted.7z",
    "repaired.7z",
    None,                                    // progress: Option<BytesProgressCallback>
)?;

// Validate:
sz.validate_archive("archive.7z")?;
```

### ArchiveEntry

```rust
pub struct ArchiveEntry {
    pub name: String,           // File name (UTF-8)
    pub size: u64,              // Uncompressed size
    pub packed_size: u64,       // Compressed size
    pub modified_time: u64,     // Unix timestamp
    pub attributes: u32,        // File attributes
    pub is_directory: bool,     // ⚠️ Field, not method
}

entry.compression_ratio()       // -> f64  (percentage 0-100)
```

### CompressionLevel

```rust
pub enum CompressionLevel {
    Store,     // No compression
    Fastest,
    Fast,
    Normal,    // Balanced (default)
    Maximum,
    Ultra,     // Best ratio, slowest
}
```

### CompressOptions

```rust
pub struct CompressOptions {
    pub num_threads: usize,                  // 0 = auto-detect
    pub dict_size: u64,                      // 0 = auto
    pub solid: bool,                         // Solid archive (default: true)
    pub password: Option<String>,
    pub auto_detect_incompressible: bool,     // Skip compression for high-entropy data
}

// Builder-style:
CompressOptions::default()
    .with_threads(4)
    .with_password("secret".to_string())
    .with_auto_detect(true)

// Auto-tuned from file list:
CompressOptions::auto_tuned(&["file1.txt", "file2.bin"])?
```

### StreamOptions

```rust
pub struct StreamOptions {
    pub num_threads: usize,
    pub dict_size: u64,
    pub solid: bool,
    pub password: Option<String>,
    pub split_size: u64,           // Split archive size (bytes); 0 = no split
    pub chunk_size: u64,           // Streaming chunk size; 0 = auto
    pub temp_dir: Option<String>,
    pub delete_temp_on_error: bool,
}
```

### Callback Types

```rust
// Simple progress (bytes completed, bytes total):
pub type ProgressCallback = Box<dyn FnMut(u64, u64) + Send>;

// Detailed progress:
pub type BytesProgressCallback = Box<dyn FnMut(
    u64,   // bytes_processed (overall)
    u64,   // bytes_total (overall)
    u64,   // current_file_bytes
    u64,   // current_file_total
    &str,  // current_file_name
) + Send>;
```

### EncryptionContext

```rust
use seven_zip::EncryptionContext;

let mut ctx = EncryptionContext::new("password")?;
let ciphertext = ctx.encrypt(b"plaintext")?;     // -> Result<Vec<u8>>
let decrypted = ctx.decrypt(&ciphertext)?;        // -> Result<Vec<u8>>
ctx.iv()                                          // -> &[u8; 16]
ctx.key()                                         // -> &[u8; 32]
```

### Advanced Functions

```rust
use seven_zip::advanced;

// Split archives:
advanced::create_split_archive(/* ... */)?;
advanced::extract_split_archive(/* ... */)?;

// Raw LZMA:
advanced::compress_lzma(/* ... */)?;
advanced::decompress_lzma(/* ... */)?;
advanced::compress_lzma2(/* ... */)?;
advanced::decompress_lzma2(/* ... */)?;

// Error details:
advanced::get_version()                    // -> String
advanced::get_error_string(code)           // -> String
```

### Error Enum

```rust
pub enum Error {
    OpenFile(String),
    InvalidArchive(String),
    Memory(String),
    Extract(String),
    Compress(String),
    InvalidParameter(String),
    NotImplemented(String),
    Unknown(String),
    Io(String),
    EncryptionError(String),
    DecryptionError(String),
}

error.with_message("custom context")       // -> Error (preserves variant)
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `SevenZip::extract()` doesn't need init | **Must call `SevenZip::new()` first** to initialize the C library. |
| `ArchiveEntry.is_directory()` method | **`is_directory` is a public field** (bool), not a method. |
| `create_archive` with large dirs | **Loads files into memory**. Use `create_archive_streaming()` for dirs >1 GB. |
| `CompressOptions::auto_tuned()` infallible | **Returns `io::Result`** — can fail if file paths don't exist. |
| Progress callback is `Fn` | **Must be `FnMut + Send`** — wrapped in `Box`. |
| `test_archive(path, password)` (2 args) | **Takes 3 args**: `(path, Option<&str>, Option<BytesProgressCallback>)`. |
| `extract_files` file list is `Vec<String>` | **Takes `&[&str]`** — string slice references. |

---

## genpdf 0.2.0

PDF generation library with layout engine (no external dependencies like wkhtmltopdf).

**Used in:** `report/pdf.rs`, `viewer/document/pdf.rs`

### Document Creation

```rust
use genpdf::{Document, Alignment, Element};
use genpdf::{elements, fonts, style};

// Load fonts (required before creating Document):
let font_family = fonts::from_files(
    "/Library/Fonts",        // directory path
    "Arial",                 // font family base name
    None,                    // builtin: Option<fonts::Builtin>
)?;                          // -> Result<FontFamily<FontData>>

// Create document:
let mut doc = Document::new(font_family);  // ⚠️ Requires FontFamily, no default

// Configure:
doc.set_title("My Report");
doc.set_font_size(10);                     // Base font size (points)
doc.set_line_spacing(1.25);
doc.set_paper_size(genpdf::PaperSize::A4);
doc.set_minimal_conformance();             // Smaller PDF output

// Add page decorator (headers/footers):
let mut decorator = genpdf::SimplePageDecorator::new();
decorator.set_margins(genpdf::Margins::trbl(25, 15, 25, 15)); // top, right, bottom, left (mm)
doc.set_page_decorator(decorator);
```

### Adding Content

```rust
// Push elements (consumes ownership):
doc.push(elements::Paragraph::new("Hello world"));

// Paragraph with styling (via Element trait):
doc.push(
    elements::Paragraph::new("Bold Title")
        .aligned(Alignment::Center)
        .styled(style::Style::new().bold().with_font_size(18))
);

// Text element (inline, no paragraph break):
doc.push(
    elements::Text::new("Inline text")
        .styled(style::Style::new().italic())
);

// Line break:
doc.push(elements::Break::new(1.5));       // 1.5 lines of spacing
```

### Layout Elements

```rust
// Vertical layout (stack elements):
let mut layout = elements::LinearLayout::vertical();
layout.push(elements::Paragraph::new("Line 1"));
layout.push(elements::Paragraph::new("Line 2"));
doc.push(layout);

// Table layout:
let mut table = elements::TableLayout::new(vec![1, 3, 2, 2]); // column weights
table.set_cell_decorator(
    elements::FrameCellDecorator::new(true, true, false)  // inner, outer, continuation
);

// Add rows via builder:
table.row()
    .element(elements::Text::new("Col 1").styled(style::Style::new().bold()))
    .element(elements::Text::new("Col 2"))
    .element(elements::Text::new("Col 3"))
    .element(elements::Text::new("Col 4"))
    .push()?;                              // ⚠️ Must call .push() to commit row

doc.push(table);
```

### Element Trait Methods

All elements implement the `Element` trait which provides:

```rust
element.styled(style::Style::new().bold())  // -> StyledElement
element.padded(genpdf::Margins::all(5))     // -> PaddedElement
element.framed()                            // -> FramedElement (with border)
```

### Style

```rust
style::Style::new()
    .bold()                     // -> Style
    .italic()                   // -> Style
    .with_font_size(12)         // -> Style  (points, NOT half-points)
    .with_line_spacing(1.5)     // -> Style
    .with_font_family(family)   // -> Style
```

### Rendering

```rust
// Render to file:
doc.render_to_file("output.pdf")?;         // -> Result<()>

// Render to writer:
let mut buf = Vec::new();
doc.render(&mut buf)?;                     // -> Result<()>
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `Document::new()` no args | **Requires `FontFamily<FontData>`** — must load fonts first. |
| `Paragraph::new()` like docx_rs builder | **Immutable chain** — use `.aligned()` and `.styled()` (trait methods), not `.add_run()`. |
| `table.row().element().element()` auto-commits | **Must call `.push()?`** at end of row chain to add it to the table. |
| `style.with_font_size(24)` = 12pt | **Font size is in points directly** (unlike docx_rs which uses half-points). |
| `LinearLayout::new()` | **Constructor is private** — use `LinearLayout::vertical()`. |
| `doc.push()` returns the element | **Returns `()`** — push is not chainable. |
| `fonts::from_files()` takes `&str` name | **Expects matching files** like `Arial.ttf`, `Arial-Bold.ttf`, `Arial-Italic.ttf`, `Arial-BoldItalic.ttf` in the directory. |

---

## tera 1.20

Jinja2-like template engine, used for HTML/Markdown report rendering.

**Used in:** `report/template.rs`, `report/error.rs`

### Tera Engine

```rust
use tera::{Tera, Context};

// From glob pattern (loads all templates in directory):
let tera = Tera::new("templates/**/*.html")?;      // -> Result<Tera>

// Empty engine + add templates manually:
let mut tera = Tera::default();                     // ⚠️ Use default(), not new()
tera.add_raw_template("report.html", template_str)?;
tera.add_raw_template("report.md", md_template)?;

// Check template names:
tera.get_template_names()                           // -> impl Iterator<Item = &str>
```

### Context (Template Variables)

```rust
let mut context = Context::new();
context.insert("title", &report.title);             // ⚠️ Takes &T where T: Serialize
context.insert("items", &vec!["a", "b", "c"]);
context.insert("count", &42);

// From serializable struct:
let context = Context::from_serialize(&report)?;    // -> Result<Context>
```

### Rendering

```rust
// Render named template:
let html = tera.render("report.html", &context)?;   // -> Result<String>

// Render to writer:
tera.render_to("report.html", &context, &mut writer)?;

// One-off render (no engine needed):
let result = Tera::one_off("Hello {{ name }}", &context, true)?;
//                                                    ^ autoescape
```

### Custom Filters

```rust
// Register filter (used in templates as {{ value | filter_name }}):
tera.register_filter(
    "bytes",
    |value: &tera::Value,
     _args: &std::collections::HashMap<String, tera::Value>|
    -> tera::Result<tera::Value> {
        // value is serde_json::Value
        let n = value.as_f64().unwrap_or(0.0);
        Ok(tera::Value::String(format_bytes(n)))
    },
);
```

### ⚠️ Key Gotchas

| Trap | Reality |
|------|---------|
| `Tera::new("templates/")` loads dir | **Takes a glob pattern**, not a directory: `Tera::new("templates/**/*")?`. |
| `Tera::new()` with no args for empty engine | **`new()` requires a glob**. Use `Tera::default()` for an empty engine. |
| `context.insert("key", value)` | **Takes `&T`** where T: Serialize. Must pass reference: `&value`. |
| `tera.render()` returns `()` | **Returns `Result<String>`** — the rendered template content. |
| Filter function returns `String` | **Must return `tera::Result<tera::Value>`** — wrap in `Ok(tera::Value::String(...))`. |
| `context.insert()` key is `&str` | **Key is `impl Into<String>`** — works with `&str` or `String`. |

---

## fatfs 0.3.6

**Used in:** `common/filesystem/fat.rs` (362 lines)
**Purpose:** FAT12/16/32 filesystem parsing — read-only traversal of FAT partitions in disk images

### Core Types

```rust
// Trait bound for I/O objects (file handles, cursors, etc.)
pub trait ReadWriteSeek: Read + Write + Seek {}
// Blanket impl: any T: Read + Write + Seek automatically implements ReadWriteSeek

pub enum FatType {
    Fat12,
    Fat16,
    Fat32,
}

pub struct FsOptions { /* private */ }
pub struct FileSystem<T: ReadWriteSeek> { /* private */ }
pub struct Dir<'a, T: ReadWriteSeek + 'a> { /* private */ }
pub struct DirEntry<'a, T: ReadWriteSeek + 'a> { /* private */ }
pub struct File<'a, T: ReadWriteSeek + 'a> { /* private */ }
pub struct DirIter<'a, T: ReadWriteSeek + 'a> { /* private */ }
pub struct FileSystemStats { /* private */ }

// Time types
pub struct Date {
    pub year: u16,   // [1980, 2107]
    pub month: u16,  // [1, 12]
    pub day: u16,    // [1, 31]
}
pub struct DateTime {
    pub date: Date,
    pub time: Time,  // Time is pub(crate) — access via DateTime
}
```

### FileSystem Methods

```rust
// Construction — takes ownership of the I/O object
FileSystem::new(disk: T, options: FsOptions) -> io::Result<FileSystem<T>>

// Filesystem info
fs.root_dir() -> Dir<'_, T>           // Returns root directory handle
fs.fat_type() -> FatType              // FAT12/16/32
fs.volume_id() -> u32                 // Volume serial number
fs.volume_label() -> String           // Volume label (trimmed)
fs.cluster_size() -> u32              // Bytes per cluster
fs.stats() -> io::Result<FileSystemStats>
```

### FsOptions Methods

```rust
FsOptions::new() -> FsOptions
options.update_accessed_date(enabled: bool) -> FsOptions   // Builder pattern
options.oem_cp_converter(converter: Box<dyn OemCpConverter>) -> FsOptions
options.time_provider(provider: Box<dyn TimeProvider>) -> FsOptions
```

### FileSystemStats Methods

```rust
stats.cluster_size() -> u32
stats.total_clusters() -> u32
stats.free_clusters() -> u32
```

### Dir Methods

```rust
dir.iter() -> DirIter<'a, T>               // Iterator over DirEntry items
dir.open_dir(path: &str) -> io::Result<Dir<'a, T>>
dir.open_file(path: &str) -> io::Result<File<'a, T>>
dir.create_file(path: &str) -> io::Result<File<'a, T>>   // ⚠️ NOT for forensics (read-only!)
dir.create_dir(path: &str) -> io::Result<Dir<'a, T>>     // ⚠️ NOT for forensics
dir.remove(path: &str) -> io::Result<()>                  // ⚠️ NOT for forensics
dir.rename(src: &str, dst_dir: &Dir<T>, dst: &str) -> io::Result<()>  // ⚠️ NOT for forensics
```

### DirEntry Methods

```rust
entry.file_name() -> String              // Long filename (or short if no LFN)
entry.short_file_name() -> String        // 8.3 format name
entry.short_file_name_as_bytes() -> &[u8]
entry.attributes() -> FileAttributes     // Bitflags
entry.is_dir() -> bool                   // ✅ Correct name (NOT is_directory)
entry.is_file() -> bool
entry.len() -> u64                       // File size in bytes
entry.created() -> DateTime
entry.accessed() -> Date                 // ⚠️ Returns Date (no time), NOT DateTime
entry.modified() -> DateTime
entry.to_file() -> File<'a, T>           // Open as File handle
entry.to_dir() -> Dir<'a, T>            // Open as Dir handle
```

### File — implements Read + Write + Seek

```rust
file.truncate() -> io::Result<()>              // ⚠️ NOT for forensics
file.set_created(date_time: DateTime)          // ⚠️ NOT for forensics
file.set_accessed(date: Date)                  // ⚠️ NOT for forensics
file.set_modified(date_time: DateTime)         // ⚠️ NOT for forensics
// Read via std::io::Read trait: file.read(&mut buf), file.read_to_end(&mut vec)
// Seek via std::io::Seek trait: file.seek(SeekFrom::Start(0))
```

### FileAttributes (bitflags)

```rust
FileAttributes::READ_ONLY
FileAttributes::HIDDEN
FileAttributes::SYSTEM
FileAttributes::VOLUME_ID
FileAttributes::DIRECTORY
FileAttributes::ARCHIVE
```

### Common Gotchas

| What You Might Assume | Actual API |
|---|---|
| `FileSystem::new()` takes `&T` | **Takes ownership**: `FileSystem::new(disk, options)` where `T: ReadWriteSeek`. Use `std::io::Cursor<Vec<u8>>` for in-memory. |
| `entry.accessed()` returns `DateTime` | **Returns `Date` only** (no time component). Only `created()` and `modified()` return `DateTime`. |
| `entry.is_directory()` | **`entry.is_dir()`** — follows Rust stdlib naming convention. |
| `DirIter` yields `Result<DirEntry>` | **Yields `io::Result<DirEntry>`** — must handle errors per entry. |
| FAT has file permissions | **No Unix permissions in FAT** — only `FileAttributes` bitflags (read-only, hidden, system, etc.). |
| Can use `&mut File` for read-only | **`ReadWriteSeek` requires Write** — even for read-only use. Use `fscommon::BufStream<std::io::Cursor<Vec<u8>>>` or wrap with a dummy Write impl. |

---

## tar 0.4.44

**Used in:** `archive/tar.rs` (318 lines)
**Purpose:** TAR archive reading — list entries, extract content, with GZ/BZ2/XZ decompression wrappers

### Core Types

```rust
pub struct Archive<R: ?Sized + Read> { /* private */ }
pub struct Entries<'a, R: 'a + Read> { /* implements Iterator<Item = io::Result<Entry<'a, R>>> */ }
pub struct Entry<'a, R: 'a + Read> { /* implements Read */ }
pub struct Header { /* 512-byte TAR header block */ }
```

### Archive Methods

```rust
// Construction
Archive::new(reader: R) -> Archive<R>   // Wraps any Read impl

// Entry iteration
archive.entries() -> io::Result<Entries<R>>          // Streaming iteration (forward-only)
archive.entries_with_seek() -> io::Result<Entries<R>> // Requires R: Seek (allows rewinding)

// Bulk extraction (NOT used in forensics — use entries() to iterate instead)
archive.unpack(dst: impl AsRef<Path>) -> io::Result<()>

// Configuration (builder-style, returns &mut Self)
archive.set_ignore_zeros(ignore: bool)              // Continue past zero-filled blocks
archive.set_mask(mask: u32)                          // Permission mask for extraction
archive.set_unpack_xattrs(enabled: bool)
archive.set_preserve_permissions(preserve: bool)
archive.set_preserve_ownerships(preserve: bool)
archive.set_preserve_mtime(preserve: bool)
archive.set_overwrite(overwrite: bool)
```

### Entry Methods (implements Read for file content)

```rust
entry.path() -> io::Result<Cow<Path>>                  // Full path within archive
entry.path_bytes() -> Cow<[u8]>                         // Raw path bytes
entry.link_name() -> io::Result<Option<Cow<Path>>>      // Symlink target (if symlink)
entry.link_name_bytes() -> Option<Cow<[u8]>>
entry.header() -> &Header                               // Access header metadata
entry.size() -> u64                                     // Uncompressed content size
entry.raw_header_position() -> u64                      // Byte offset of header in stream
entry.raw_file_position() -> u64                        // Byte offset of content in stream
entry.pax_extensions() -> io::Result<Option<PaxExtensions>>  // Extended attributes
entry.unpack(dst: impl AsRef<Path>) -> io::Result<Unpacked>  // Extract single entry
entry.unpack_in(dst: impl AsRef<Path>) -> io::Result<bool>   // Extract within directory
entry.set_mask(mask: u32)
entry.set_unpack_xattrs(enabled: bool)
entry.set_preserve_permissions(preserve: bool)
entry.set_preserve_mtime(preserve: bool)
```

### Header Methods

```rust
header.path() -> io::Result<Cow<Path>>
header.path_bytes() -> Cow<[u8]>
header.size() -> io::Result<u64>                // Content size
header.entry_size() -> io::Result<u64>          // Header + content size
header.mode() -> io::Result<u32>                // Unix permissions
header.uid() -> io::Result<u64>
header.gid() -> io::Result<u64>
header.mtime() -> io::Result<u64>               // Modification time (Unix timestamp)
header.username() -> Result<Option<&str>, Utf8Error>
header.groupname() -> Result<Option<&str>, Utf8Error>
header.link_name() -> io::Result<Option<Cow<Path>>>
header.entry_type() -> EntryType                // Regular, Directory, Symlink, etc.
header.as_gnu() -> Option<&GnuHeader>
header.as_ustar() -> Option<&UstarHeader>
```

### EntryType (NOT an enum — struct wrapping u8)

```rust
// Use associated constants:
EntryType::Regular       // '0' — normal file
EntryType::Link          // '1' — hard link
EntryType::Symlink       // '2' — symbolic link
EntryType::Char          // '3' — character device
EntryType::Block         // '4' — block device
EntryType::Directory     // '5' — directory
EntryType::Fifo          // '6' — FIFO/pipe
EntryType::Continuous    // '7' — contiguous file
EntryType::GNULongName   // 'L' — long filename
EntryType::GNULongLink   // 'K' — long link name
EntryType::GNUSparse     // 'S' — sparse file
EntryType::XGlobalHeader // 'g' — global extended header
EntryType::XHeader       // 'x' — extended header

entry_type.is_file() -> bool
entry_type.is_hard_link() -> bool
entry_type.is_symlink() -> bool
entry_type.is_dir() -> bool
```

### Decompression Pattern (used in `archive/tar.rs`)

```rust
use flate2::read::GzDecoder;    // .tar.gz / .tgz
use bzip2::read::BzDecoder;     // .tar.bz2
use xz2::read::XzDecoder;       // .tar.xz
use zstd::Decoder as ZstdDecoder; // .tar.zst

// Pattern: wrap reader in decompressor, then Archive
let file = File::open(path)?;
let gz = GzDecoder::new(file);
let mut archive = Archive::new(gz);
for entry in archive.entries()? {
    let entry = entry?;
    // ...
}
```

### Common Gotchas

| What You Might Assume | Actual API |
|---|---|
| `entries()` can be called multiple times | **Consumes entries forward-only.** After iterating, you must create a new `Archive`. Use `entries_with_seek()` if R: Seek. |
| `EntryType` is an enum | **It's a newtype struct wrapping `u8`.** Use associated constants: `EntryType::Regular`, `EntryType::Directory`. Cannot `match` exhaustively. |
| `entry.path()` returns `PathBuf` | **Returns `io::Result<Cow<Path>>`** — must unwrap Result and may borrow. |
| `header.entry_type()` returns `Result` | **Returns `EntryType` directly** — no Result wrapper. |
| `Entry` auto-reads content | **Entry implements `Read`** — you must explicitly `read_to_end()` or `read()` to get content bytes. |
| `header.mtime()` returns `DateTime` | **Returns `io::Result<u64>`** — Unix timestamp. Convert with `chrono::DateTime::from_timestamp()`. |

---

## image 0.25

**Used in:** `viewer/document/universal.rs` (thumbnails, format detection, image metadata)
**Purpose:** Image reading, format detection, thumbnail generation

### Key Types Used in CORE-FFX

```rust
use image::ImageReader;         // Primary entry point for loading images
use image::ImageFormat;         // Format enum (Png, Jpeg, Gif, etc.)
use image::DynamicImage;        // Decoded image (format-agnostic)
```

### ImageReader (renamed from `io::Reader` in 0.25)

```rust
// Open and decode
ImageReader::open(path: impl AsRef<Path>) -> Result<ImageReader<BufReader<File>>, io::Error>
reader.decode() -> Result<DynamicImage, ImageError>
reader.with_guessed_format() -> Result<Self, io::Error>  // Guess format from magic bytes
reader.set_format(format: ImageFormat) -> &mut Self
reader.format() -> Option<ImageFormat>

// Full pattern:
let img = ImageReader::open(path)?
    .with_guessed_format()?
    .decode()?;
```

### DynamicImage

```rust
// Dimensions
img.width() -> u32
img.height() -> u32
img.dimensions() -> (u32, u32)

// Thumbnails — ⚠️ used in CORE-FFX
img.thumbnail(nwidth: u32, nheight: u32) -> DynamicImage  // Preserves aspect ratio
img.resize(nwidth: u32, nheight: u32, filter: FilterType) -> DynamicImage
img.resize_exact(nwidth: u32, nheight: u32, filter: FilterType) -> DynamicImage

// Encoding to bytes
img.write_to(w: &mut impl Write + Seek, format: ImageFormat) -> Result<(), ImageError>

// Color info
img.color() -> ColorType
```

### ImageFormat Enum (selected variants — feature-gated)

```rust
ImageFormat::Png
ImageFormat::Jpeg
ImageFormat::Gif
ImageFormat::WebP
ImageFormat::Bmp
ImageFormat::Ico
ImageFormat::Tiff
// Note: CORE-FFX enables: png, jpeg, gif, webp, bmp, ico, tiff features
```

### Convenience Functions

```rust
image::open(path: impl AsRef<Path>) -> Result<DynamicImage, ImageError>
// Shorthand for ImageReader::open(path)?.decode()
```

### Common Gotchas

| What You Might Assume | Actual API |
|---|---|
| `ImageReader::open()` returns `DynamicImage` | **Returns `ImageReader`** — must call `.decode()` to get `DynamicImage`. |
| `image::open()` detects format from bytes | **Detects from extension only.** For magic-byte detection, use `ImageReader::open(p)?.with_guessed_format()?.decode()`. |
| `write_to()` takes `&mut impl Write` | **Requires `Write + Seek`** — use `Cursor<Vec<u8>>` for in-memory, not raw `Vec`. |
| `thumbnail()` returns `Result` | **Returns `DynamicImage` directly** — no error. Always succeeds. |
| All formats enabled by default | **CORE-FFX uses `default-features = false`** with explicit feature list. `avif`, `hdr`, `pnm`, `farbfeld` etc. are NOT available. |

---

## pdf-extract 0.7.12

**Used in:** `viewer/document/pdf.rs`, `containers/case_documents.rs`
**Purpose:** Extract plain text from PDF files (for search/preview)

### Public Functions

```rust
// From file path
pub fn extract_text<P: AsRef<Path>>(path: P) -> Result<String, OutputError>
pub fn extract_text_encrypted<P: AsRef<Path>, PW: AsRef<[u8]>>(
    path: P, password: PW
) -> Result<String, OutputError>
pub fn extract_text_by_pages<P: AsRef<Path>>(path: P) -> Result<Vec<String>, OutputError>
pub fn extract_text_by_pages_encrypted<P: AsRef<Path>, PW: AsRef<[u8]>>(
    path: P, password: PW
) -> Result<Vec<String>, OutputError>

// From memory buffer
pub fn extract_text_from_mem(buffer: &[u8]) -> Result<String, OutputError>
pub fn extract_text_from_mem_encrypted<PW: AsRef<[u8]>>(
    buffer: &[u8], password: PW
) -> Result<String, OutputError>
pub fn extract_text_from_mem_by_pages(buffer: &[u8]) -> Result<Vec<String>, OutputError>
pub fn extract_text_from_mem_by_pages_encrypted<PW: AsRef<[u8]>>(
    buffer: &[u8], password: PW
) -> Result<Vec<String>, OutputError>

// Low-level: output doc through OutputDev trait
pub fn output_doc(doc: &Document, output: &mut dyn OutputDev) -> Result<(), OutputError>
pub fn output_doc_page(doc: &Document, output: &mut dyn OutputDev, page_num: u32) -> Result<(), OutputError>
pub fn print_metadata(doc: &Document)

// Output implementations
pub struct PlainTextOutput<W: Write> { /* private */ }
PlainTextOutput::new(writer: W) -> PlainTextOutput<W>

pub struct HTMLOutput<'a> { /* private */ }
HTMLOutput::new(file: &mut dyn Write) -> HTMLOutput
```

### Common Gotchas

| What You Might Assume | Actual API |
|---|---|
| `extract_text()` never panics | **Can panic on malformed PDFs** — wrap in `catch_unwind` or use in a thread. Some corrupted PDFs cause index-out-of-bounds. |
| Returns empty string for image-only PDFs | **Returns `Ok("")`** (empty string) — no OCR capability. |
| `extract_text_from_mem()` takes `&str` | **Takes `&[u8]`** — raw PDF bytes, not text. |
| The `Document` type is from pdf-extract | **`Document` is from `lopdf`** — use `lopdf::Document::load(path)` to create one for `output_doc()`. |
| `extract_text_by_pages()` returns page numbers | **Returns `Vec<String>`** — index is zero-based page number. |

---

## csv 1.3

**Used in:** `viewer/document/spreadsheet.rs`
**Purpose:** CSV/TSV parsing for forensic file viewing

### Core Types

```rust
pub struct ReaderBuilder { /* private */ }
pub struct Reader<R: Read> { /* private */ }
pub struct StringRecord { /* Vec<String> wrapper */ }
pub struct ByteRecord { /* Vec<u8> wrapper */ }
```

### ReaderBuilder (builder pattern)

```rust
ReaderBuilder::new() -> ReaderBuilder

// Configuration
builder.has_headers(yes: bool) -> &mut ReaderBuilder       // Default: true
builder.delimiter(delimiter: u8) -> &mut ReaderBuilder      // Default: b','
builder.quote(quote: u8) -> &mut ReaderBuilder              // Default: b'"'
builder.escape(escape: Option<u8>) -> &mut ReaderBuilder
builder.comment(comment: Option<u8>) -> &mut ReaderBuilder
builder.flexible(yes: bool) -> &mut ReaderBuilder           // Allow varying field counts
builder.trim(trim: Trim) -> &mut ReaderBuilder              // Trim::All, Trim::Headers, Trim::Fields
builder.double_quote(yes: bool) -> &mut ReaderBuilder       // "" escaping

// Build
builder.from_path(path: impl AsRef<Path>) -> Result<Reader<File>, Error>
builder.from_reader(rdr: R) -> Reader<R>   // Wraps any Read impl
```

### Reader Methods

```rust
reader.headers() -> Result<&StringRecord, Error>            // First record (if has_headers)
reader.records() -> StringRecordsIter<R>                    // Iterator<Item = Result<StringRecord>>
reader.byte_records() -> ByteRecordsIter<R>                 // Iterator<Item = Result<ByteRecord>>
reader.has_headers() -> bool
reader.into_inner() -> R                                    // Unwrap the reader
```

### StringRecord Methods

```rust
record.get(i: usize) -> Option<&str>     // Field by index
record.len() -> usize                     // Number of fields
record.is_empty() -> bool
record.iter() -> Iter<'_>                 // Iterator over &str fields
```

### Common Gotchas

| What You Might Assume | Actual API |
|---|---|
| `from_path()` is on `Reader` | **On `ReaderBuilder`**: `ReaderBuilder::new().from_path(p)?`. No `Reader::from_path()`. |
| `delimiter()` takes `char` | **Takes `u8`**: `builder.delimiter(b'\t')` for TSV. |
| `records()` includes header row | **Skips headers** when `has_headers(true)` (default). Headers accessed separately via `reader.headers()`. |
| `reader.headers()` returns `Vec<String>` | **Returns `Result<&StringRecord>`** — must unwrap and index with `.get(i)`. |
| `record[i]` panics gracefully | **Panics if out of bounds.** Use `record.get(i)` which returns `Option<&str>`. |

---

## langchain-rust 4.6 + async-openai 0.29

**Used in:** `report/ai.rs` (503 lines, behind `ai-assistant` feature flag)
**Purpose:** AI-powered report narrative generation via Ollama (local) or OpenAI (cloud)

> ⚠️ These are **optional dependencies** — gated behind `features = ["ai-assistant"]` (enabled by default in dev).

### langchain-rust — Ollama Client

```rust
use langchain_rust::llm::ollama::client::{Ollama, OllamaClient};
use langchain_rust::language_models::llm::LLM;
use std::sync::Arc;

// Create client
let ollama_client = OllamaClient::try_new(url: &str)
    -> Result<OllamaClient, Box<dyn std::error::Error>>

// Create LLM wrapper
let ollama = Ollama::new(
    client: Arc<OllamaClient>,
    model: &str,              // e.g., "llama3.2"
    options: Option<...>,     // Usually None
) -> Ollama

// Generate text (from LLM trait)
ollama.invoke(prompt: &str) -> Result<String, Box<dyn std::error::Error>>
// ⚠️ This is async — returns a Future
```

### async-openai — OpenAI Client

```rust
use async_openai::{
    Client,
    types::{
        CreateChatCompletionRequestArgs,
        ChatCompletionRequestUserMessageArgs,
    },
    config::OpenAIConfig,
};

// Create client
Client::new() -> Client<OpenAIConfig>                    // Uses OPENAI_API_KEY env var
Client::with_config(config: OpenAIConfig) -> Client<OpenAIConfig>

// Configure API key
OpenAIConfig::new() -> OpenAIConfig
config.with_api_key(key: impl Into<String>) -> OpenAIConfig

// Build chat request
CreateChatCompletionRequestArgs::default()
    .model(model: impl Into<String>)          // e.g., "gpt-4"
    .messages(messages: impl IntoIterator<Item = ChatCompletionRequestMessage>)
    .build() -> Result<CreateChatCompletionRequest, OpenAIError>

// Build user message
ChatCompletionRequestUserMessageArgs::default()
    .content(content: impl Into<String>)
    .build() -> Result<ChatCompletionRequestUserMessage, OpenAIError>
    // Then convert to generic message with .into()

// Send request
client.chat().create(request) -> Result<CreateChatCompletionResponse, OpenAIError>

// Extract response text
response.choices[0].message.content  // Option<String>
// Pattern: response.choices.first().and_then(|c| c.message.content.clone())
```

#### Azure OpenAI via AzureConfig

```rust
use async_openai::{Client, config::AzureConfig};

// Build AzureConfig
AzureConfig::new() -> AzureConfig                              // Defaults; reads OPENAI_API_KEY env var
config.with_api_base(url: impl Into<String>) -> AzureConfig    // e.g., "https://my-resource.openai.azure.com"
config.with_deployment_id(id: impl Into<String>) -> AzureConfig // e.g., "gpt-4"
config.with_api_version(ver: impl Into<String>) -> AzureConfig  // e.g., "2024-02-01"
config.with_api_key(key: impl Into<String>) -> AzureConfig      // Explicit key (overrides env var)

// Create client with Azure config
let client = Client::with_config(config);  // Client<AzureConfig>

// Then use same chat API as OpenAI:
client.chat().create(request).await
```

### Common Gotchas

| What You Might Assume | Actual API |
|---|---|
| `OllamaClient::new(url)` | **`OllamaClient::try_new(url)`** — returns `Result`, not direct construction. |
| `Ollama::new(client, model)` | **Takes `Arc<OllamaClient>`** — must wrap: `Ollama::new(Arc::new(client), model, None)`. |
| `ollama.invoke()` is sync | **Async** — must `.await`. Returns `Result<String, Box<dyn Error>>`. |
| `LLM` trait is in `langchain_rust::llm` | **`langchain_rust::language_models::llm::LLM`** — note the `language_models` path. |
| OpenAI `Client::new()` requires API key arg | **Reads from `OPENAI_API_KEY` env var.** Use `Client::with_config()` for explicit key. |
| `ChatCompletionRequestUserMessageArgs` builds directly into a message vec | **Must call `.build()?.into()`** to convert to `ChatCompletionRequestMessage`. |
| Response has `.text` field | **Access via `response.choices.first().and_then(\|c\| c.message.content.clone())`** — deeply nested `Option<String>`. |
| These crates are always available | **Feature-gated: `#[cfg(feature = "ai-assistant")]`** — code must be conditional. |

---

## Internal Types

Common CORE-FFX internal types that have caused field/variant name errors.

### ArchiveEntryInfo (`archive/libarchive_backend.rs`)

```rust
pub struct ArchiveEntryInfo {
    pub path:         String,
    pub name:         String,
    pub is_dir:       bool,      // ⚠️ NOT `is_directory`
    pub size:         u64,
    pub parent:       String,
    pub index:        usize,
    pub is_encrypted: bool,
    pub mtime:        Option<i64>,
    // ⚠️ NO `compressed_size` field
}
```

### ArchiveFormat (`archive/types.rs`)

```rust
pub enum ArchiveFormat {
    SevenZip, Zip, Zip64,
    Rar4,                      // ⚠️ NOT `Rar` — separate Rar4 and Rar5
    Rar5,
    Gzip, Tar, TarGz, Xz, Bzip2, Lz4, Zstd,
    Aff, Aff4, Smart,
    Iso,                       // ⚠️ NOT `Iso9660`
    Vmdk, Vhd, Vhdx, Qcow2, Vdi,  // Virtual machine disk formats
    Dmg,
}
```

### StoredHash (`containers/types.rs`)

```rust
pub struct StoredHash {
    pub algorithm: String,
    pub hash:      String,     // ⚠️ Field is `hash`, NOT `value`
    pub verified:  Option<bool>,
    pub timestamp: Option<String>,
    pub source:    Option<String>,
    pub offset:    Option<u64>,
    pub size:      Option<u64>,
}
```

### ProjectDbStats (`project_db.rs`)

```rust
#[serde(rename_all = "camelCase")]
pub struct ProjectDbStats {
    pub total_activities: i64,
    pub total_sessions: i64,
    pub total_users: i64,
    pub total_evidence_files: i64,     // ⚠️ `totalEvidenceFiles` in TS
    pub total_hashes: i64,
    pub total_verifications: i64,
    pub total_bookmarks: i64,
    pub total_notes: i64,
    pub total_tags: i64,
    pub total_reports: i64,
    pub total_saved_searches: i64,
    pub total_case_documents: i64,
    pub total_processed_databases: i64,
    pub total_axiom_cases: i64,
    pub total_artifact_categories: i64,
    // v3 stats
    pub total_exports: i64,
    pub total_custody_records: i64,
    pub total_classifications: i64,
    pub total_extractions: i64,
    pub total_viewer_history: i64,
    pub total_annotations: i64,
    pub total_relationships: i64,
    pub db_size_bytes: u64,            // ⚠️ u64, not i64
    pub schema_version: u32,           // ⚠️ u32, not i64
}
```

**⚠️ Key gotcha:** All `total_*` fields use `snake_case` in Rust but `camelCase` in TypeScript due to `#[serde(rename_all = "camelCase")]`. The TypeScript interface is in `src/types/projectDb.ts`.

### Rust Standard Library

| Trap | Reality |
|------|---------|
| `once_cell::sync::Lazy` | **Project uses `std::sync::LazyLock`** (Rust 1.80+). Do NOT add `once_cell` dependency. |
| `lazy_static!` | Same — use `std::sync::LazyLock` instead. |

---

## Verification Checklist

Before writing code that calls any crate API:

1. **Check this file** for the crate's actual method/field names
2. **If the crate isn't listed here**, run:
   ```bash
   grep -rn "pub fn\|pub struct\|pub enum" ~/.cargo/registry/src/index.crates.io-*/<crate>-<version>/src/ | head -50
   ```
3. **For internal types**, grep the project source:
   ```bash
   grep -rn "pub struct MyType\|pub enum MyType" src-tauri/src/
   ```
4. **After modifying any `#[serde(rename_all)]` Rust struct**, search for the matching TypeScript interface and update it too.
