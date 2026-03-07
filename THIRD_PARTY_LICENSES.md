# Third-Party Licenses

CORE-FFX is licensed under the MIT License. This document lists third-party
software included in or linked by CORE-FFX and their respective licenses.

Last updated: 2025-07-14

---

## Table of Contents

1. [Native C Libraries](#native-c-libraries)
2. [Rust Crate Dependencies](#rust-crate-dependencies)
3. [Frontend (npm) Dependencies](#frontend-npm-dependencies)
4. [Fonts](#fonts)
5. [Full License Texts](#full-license-texts)

---

## Native C Libraries

CORE-FFX statically links the following C libraries:

| Library | License | Version | Purpose |
|---------|---------|---------|---------|
| libarchive | BSD 2-Clause | 3.7+ | Archive format reading (7z, RAR, ZIP, TAR, ISO, etc.) |
| libewf | **LGPL-3.0-or-later** | 20251220 | EWF forensic image creation and metadata reading |
| LZMA SDK | Public Domain | 24.09 | 7z archive creation (via sevenzip-ffi) |
| zlib | zlib License | System | Compression (dependency of libarchive, libewf) |
| libbz2 | BSD-style | System | BZIP2 compression (dependency of libarchive, libewf) |

### libarchive

CORE-FFX uses libarchive for reading various archive formats (7-Zip, RAR, ZIP, TAR, ISO, etc.).

- **License**: BSD 2-Clause License
- **Copyright**: (c) 2003-2024 Tim Kientzle and libarchive contributors
- **Website**: https://www.libarchive.org/
- **Source**: https://github.com/libarchive/libarchive
- **Usage**: Static linking via FFI (`libarchive2` crate, patched in `patches/libarchive2-sys/`)

Full license text: see [BSD 2-Clause (libarchive)](#bsd-2-clause-libarchive) below.

### libewf (LGPL-3.0-or-later)

CORE-FFX uses libewf via the `libewf-ffi` workspace crate for creating E01/Ex01 forensic disk images and reading EWF metadata.

- **License**: GNU Lesser General Public License v3.0 or later (LGPL-3.0-or-later)
- **Copyright**: (c) 2006-2025 Joachim Metz
- **Website**: https://github.com/libyal/libewf
- **Version**: 20251220
- **Usage**: Static linking via Rust FFI wrapper (`libewf-ffi/`)

**LGPL-3.0 Compliance Notice:**

The `libewf-ffi` Rust wrapper code is MIT-licensed. The underlying libewf C library
is licensed under the LGPL-3.0-or-later. Under the terms of the LGPL:

- The complete corresponding source code for libewf is available at:
  https://github.com/libyal/libewf
- Users may re-link CORE-FFX against a modified version of libewf
- Pre-built static libraries are provided in `libewf-ffi/prebuilt/`; the
  build workflow and instructions for rebuilding from source are in
  `.github/workflows/prebuild-native-deps.yml`

Full license text: https://www.gnu.org/licenses/lgpl-3.0.html

### LZMA SDK

CORE-FFX uses the LZMA SDK via the `sevenzip-ffi` workspace crate for creating 7z archives.

- **License**: Public Domain
- **Author**: Igor Pavlov
- **Website**: https://7-zip.org/sdk.html
- **Source**: https://github.com/ip7z/7zip
- **Version**: 24.09
- **Usage**: Compiled into `lib7z_ffi.a` static library. Source in `sevenzip-ffi/lzma/C/`.

*"LZMA SDK is placed in the public domain. Anyone is free to copy, modify,
publish, use, compile, sell, or distribute the original LZMA SDK code, either
in source code form or as a compiled binary, for any purpose, commercial or
non-commercial, and by any means."* — Igor Pavlov

---

## Rust Crate Dependencies

The following Rust crates are direct dependencies used by CORE-FFX. Transitive
dependencies inherit compatible licenses (primarily MIT and Apache-2.0).
See `src-tauri/Cargo.toml` and `Cargo.lock` for the complete dependency tree.

### Forensic Parsers & Filesystem Drivers

| Crate | License | Purpose |
|-------|---------|---------|
| `notatin` | Apache-2.0 | Windows Registry hive parser |
| `outlook-pst` | MIT | Outlook PST/OST email archive parser |
| `msg_parser` | MIT | Outlook MSG file parser |
| `mail-parser` | MIT OR Apache-2.0 | EML/MBOX email parsing |
| `ntfs` | MIT OR Apache-2.0 | NTFS filesystem driver |
| `fatfs` | MIT | FAT12/16/32 filesystem driver |
| `apple-dmg` | Apache-2.0 OR MIT | Apple DMG container parser |
| `goblin` | MIT | Binary format parsing (PE/ELF/Mach-O) |

### Document & Media Viewers

| Crate | License | Purpose |
|-------|---------|---------|
| `pdf-extract` | MIT | PDF text extraction |
| `lopdf` | MIT | PDF document parsing |
| `pdfium-render` | MIT OR Apache-2.0 | PDF rendering/viewing |
| `calamine` | MIT | Excel/ODS spreadsheet reading |
| `rust_xlsxwriter` | MIT OR Apache-2.0 | Excel XLSX report generation |
| `docx-rs` | MIT | DOCX report generation |
| `genpdf` | Apache-2.0 OR MIT | PDF report generation |
| `cfb` | MIT | Compound File Binary (OLE) parsing |
| `quick-xml` | MIT | XML parsing (DOCX, PPTX, UFED) |
| `plist` | MIT | Apple plist file parsing |
| `kamadak-exif` | BSD-2-Clause | EXIF metadata extraction |
| `image` | MIT OR Apache-2.0 | Image format decoding |

### Archive & Compression

| Crate | License | Purpose |
|-------|---------|---------|
| `zip` | MIT | Pure-Rust ZIP reading/writing |
| `sevenz-rust` | MIT | Pure-Rust 7-Zip reading (fallback) |
| `tar` | MIT OR Apache-2.0 | TAR archive reading |
| `flate2` | MIT OR Apache-2.0 | GZIP/Deflate compression |
| `bzip2` | MIT OR Apache-2.0 | BZIP2 compression |
| `xz2` | MIT OR Apache-2.0 | XZ/LZMA compression |
| `zstd` | MIT | Zstandard compression |
| `unrar` | MIT | RAR reading (requires system unrar) |
| `lz4_flex` | MIT | LZ4 compression |

### Framework & Runtime

| Crate | License | Purpose |
|-------|---------|---------|
| `tauri` | MIT OR Apache-2.0 | Application framework |
| `tauri-plugin-updater` | MIT OR Apache-2.0 | Auto-updater |
| `tauri-plugin-process` | MIT OR Apache-2.0 | Process management (relaunch) |
| `tauri-plugin-dialog` | MIT OR Apache-2.0 | Native file dialogs |
| `tauri-plugin-shell` | MIT OR Apache-2.0 | Shell command execution |
| `tokio` | MIT | Async runtime |
| `serde` | MIT OR Apache-2.0 | Serialization/deserialization |
| `serde_json` | MIT OR Apache-2.0 | JSON serialization |

### Cryptography & Hashing

| Crate | License | Purpose |
|-------|---------|---------|
| `md-5` | MIT OR Apache-2.0 | MD5 hashing |
| `sha1` | MIT OR Apache-2.0 | SHA-1 hashing |
| `sha2` | MIT OR Apache-2.0 | SHA-256/512 hashing |
| `blake3` | MIT OR Apache-2.0 | BLAKE3 hashing |
| `crc32fast` | MIT OR Apache-2.0 | CRC32 checksums |
| `aes` | MIT OR Apache-2.0 | AES encryption |

### Database & Storage

| Crate | License | Purpose |
|-------|---------|---------|
| `rusqlite` | MIT | SQLite database access |
| `lru` | MIT | LRU caching |

### Logging & Diagnostics

| Crate | License | Purpose |
|-------|---------|---------|
| `tracing` | MIT | Structured logging |
| `tracing-subscriber` | MIT | Log output formatting |
| `colored` | **MPL-2.0** | Terminal color output |

### Templating & Reporting

| Crate | License | Purpose |
|-------|---------|---------|
| `tera` | MIT | Template engine (reports) |
| `csv` | Unlicense OR MIT | CSV reading/writing |
| `chrono` | MIT OR Apache-2.0 | Date/time handling |
| `uuid` | MIT OR Apache-2.0 | UUID generation |

### AI Integration

| Crate | License | Purpose |
|-------|---------|---------|
| `ollama-rs` | MIT | Local AI model integration |

### Notable Non-MIT/Apache Transitive Dependencies

These crates have licenses that differ from the standard MIT/Apache-2.0:

| Crate | License | Notes |
|-------|---------|-------|
| `colored` | MPL-2.0 | Mozilla Public License 2.0. Compatible with MIT. |
| `cssparser` | MPL-2.0 | CSS parsing (transitive via HTML processing) |
| `selectors` | MPL-2.0 | CSS selector matching (transitive) |
| `dtoa-short` | MPL-2.0 | Float-to-string conversion (transitive) |
| `option-ext` | MPL-2.0 | Option extensions (transitive via `directories`) |
| `xxhash-rust` | BSL-1.0 | Boost Software License 1.0. Hash functions. |
| `ryu` | Apache-2.0 OR BSL-1.0 | Float formatting. Dual-licensed. |
| `webpki-root-certs` | CDLA-Permissive-2.0 | Root CA certificates. Permissive license. |
| `kamadak-exif` | BSD-2-Clause | EXIF metadata parsing |
| `notatin` | Apache-2.0 | Windows Registry parser |
| `csv` | Unlicense OR MIT | Dual-licensed, public domain option |

All listed licenses are permissive and compatible with the MIT License.
MPL-2.0 is file-level copyleft — it does not require CORE-FFX source to be
open-sourced, only modifications to MPL-2.0 files themselves.

---

## Frontend (npm) Dependencies

### Core Framework

| Package | License | Purpose |
|---------|---------|---------|
| `solid-js` | MIT | Reactive UI framework |
| `solid-icons` | MIT | Icon library (Heroicons) |
| `@solidjs/router` | MIT | Client-side routing |
| `@solid-primitives/storage` | MIT | Reactive storage utilities |
| `@solid-primitives/scheduled` | MIT | Scheduling primitives |

### Tauri Integration

| Package | License | Purpose |
|---------|---------|---------|
| `@tauri-apps/api` | MIT OR Apache-2.0 | Tauri IPC bridge |
| `@tauri-apps/plugin-dialog` | MIT OR Apache-2.0 | Native dialogs |
| `@tauri-apps/plugin-shell` | MIT OR Apache-2.0 | Shell commands |
| `@tauri-apps/plugin-updater` | MIT OR Apache-2.0 | Auto-updater client |
| `@tauri-apps/plugin-process` | MIT OR Apache-2.0 | Process management |

### Document Processing

| Package | License | Purpose |
|---------|---------|---------|
| `pdfjs-dist` | **Apache-2.0** | PDF rendering (Mozilla PDF.js) |
| `dompurify` | **MPL-2.0 OR Apache-2.0** | HTML sanitization (XSS prevention) |

### Styling

| Package | License | Purpose |
|---------|---------|---------|
| `tailwindcss` | MIT | Utility-first CSS framework |
| `postcss` | MIT | CSS processing |
| `autoprefixer` | MIT | CSS vendor prefixes |

---

## Fonts

CORE-FFX bundles the following fonts, both licensed under the SIL Open Font
License 1.1:

### Inter

- **License**: SIL Open Font License 1.1 (OFL-1.1)
- **Author**: Rasmus Andersson
- **Website**: https://rsms.me/inter/
- **Source**: https://github.com/rsms/inter

### JetBrains Mono

- **License**: SIL Open Font License 1.1 (OFL-1.1)
- **Author**: JetBrains s.r.o.
- **Website**: https://www.jetbrains.com/lp/mono/
- **Source**: https://github.com/JetBrains/JetBrainsMono

Under the OFL-1.1, these fonts may be freely used, modified, and redistributed
with any software, provided that the fonts are not sold by themselves and the
reserved font names are not used for modified versions.

Full license text: https://openfontlicense.org/

---

## Full License Texts

### BSD 2-Clause (libarchive)

```
Copyright (c) 2003-2024 Tim Kientzle
All rights reserved.

Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions
are met:

1. Redistributions of source code must retain the above copyright
   notice, this list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright
   notice, this list of conditions and the following disclaimer in the
   documentation and/or other materials provided with the distribution.

THIS SOFTWARE IS PROVIDED BY THE AUTHOR(S) ``AS IS'' AND ANY EXPRESS OR
IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES
OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED.
IN NO EVENT SHALL THE AUTHOR(S) BE LIABLE FOR ANY DIRECT, INDIRECT,
INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT
NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE,
DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY
THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT
(INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF
THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
```

### LGPL-3.0-or-later (libewf)

The full text of the GNU Lesser General Public License v3.0 is available at:
https://www.gnu.org/licenses/lgpl-3.0.html

libewf source code is available at: https://github.com/libyal/libewf

### Public Domain (LZMA SDK)

LZMA SDK is placed in the public domain by Igor Pavlov.
Full text available in `sevenzip-ffi/lzma/LICENSE` and at https://7-zip.org/sdk.html.

### Apache-2.0

The full text of the Apache License 2.0 is available at:
https://www.apache.org/licenses/LICENSE-2.0

Used by: `pdfjs-dist`, `notatin`, `apple-dmg`, various Rust crates (dual-licensed)

### MPL-2.0

The full text of the Mozilla Public License 2.0 is available at:
https://www.mozilla.org/en-US/MPL/2.0/

Used by: `colored`, `cssparser`, `selectors`, `dtoa-short`, `option-ext`
Also an option for: `dompurify` (MPL-2.0 OR Apache-2.0)

### SIL Open Font License 1.1 (OFL-1.1)

The full text of the SIL Open Font License 1.1 is available at:
https://openfontlicense.org/

Used by: Inter font, JetBrains Mono font

### BSL-1.0 (Boost Software License)

The full text of the Boost Software License 1.0 is available at:
https://www.boost.org/LICENSE_1_0.txt

Used by: `xxhash-rust`

---

*This file is maintained alongside the CORE-FFX source code. For the complete
and authoritative dependency list, see `src-tauri/Cargo.toml` (Rust),
`package.json` (npm), and the lock files in the repository.*
