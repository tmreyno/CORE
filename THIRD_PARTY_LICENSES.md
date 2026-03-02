# Third-Party Licenses

This document lists the third-party libraries used by CORE-FFX and their respective licenses.

## libarchive

CORE-FFX uses libarchive for reading various archive formats (7-Zip, RAR, ZIP, TAR, etc.).

- **License**: BSD 2-Clause License
- **Website**: https://www.libarchive.org/
- **Usage**: Static linking via FFI (libarchive2 crate)

```
Copyright (c) 2003-2018 Tim Kientzle
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

## Other Dependencies

The following Rust crates are used (see `Cargo.toml` for complete list):

| Crate | License | Purpose |
|-------|---------|---------|
| `zip` | MIT/Apache-2.0 | Pure-Rust ZIP reading/writing |
| `sevenz-rust` | MIT/Apache-2.0 | Pure-Rust 7-Zip reading (fallback) |
| `tar` | MIT/Apache-2.0 | TAR archive reading |
| `flate2` | MIT/Apache-2.0 | GZIP compression |
| `bzip2` | MIT/Apache-2.0 | BZIP2 compression |
| `xz2` | MIT/Apache-2.0 | XZ/LZMA compression |
| `zstd` | MIT | Zstandard compression |
| `unrar` | MIT | RAR reading (fallback, requires system unrar) |
| `tauri` | MIT/Apache-2.0 | Application framework |
| `serde` | MIT/Apache-2.0 | Serialization |
| `tokio` | MIT | Async runtime |
| `tracing` | MIT | Logging/diagnostics |
| `genpdf` | MIT/Apache-2.0 | PDF report generation |
| `docx-rs` | MIT | DOCX report generation |
| `pdfium-render` | MIT/Apache-2.0 | PDF rendering/viewing |
| `goblin` | MIT | Binary format parsing (PE/ELF/Mach-O) |
| `mail-parser` | MIT/Apache-2.0 | Email (EML/MBOX) parsing |
| `calamine` | MIT | Excel/ODS spreadsheet reading |
| `rusqlite` | MIT | SQLite database access |
| `lru` | MIT | LRU caching |

All crates are compatible with CORE-FFX's MIT license.

## Native C Libraries

CORE-FFX statically links the following C libraries:

| Library | License | Version | Purpose |
|---------|---------|---------|----------|
| libarchive | BSD 2-Clause | 3.7+ | Archive format reading (7z, RAR, ZIP, TAR, ISO, etc.) |
| libewf | LGPL-3.0 | 20251220 | EWF forensic image creation and metadata reading |
| LZMA SDK | Public Domain | 24.09 | 7z archive creation (via sevenzip-ffi) |
| zlib | zlib License | System | Compression (dependency of libarchive, libewf) |
| libbz2 | BSD-style | System | BZIP2 compression |

### libewf

CORE-FFX uses libewf via the `libewf-ffi` workspace crate for creating E01/Ex01 forensic disk images.

- **License**: LGPL-3.0
- **Website**: https://github.com/libyal/libewf
- **Version**: 20251220
- **Usage**: Static linking via Rust FFI wrapper. LGPL compliance is maintained — the complete corresponding source is available in the `libewf-ffi/` directory and via the upstream repository.

### LZMA SDK

CORE-FFX uses the LZMA SDK via the `sevenzip-ffi` workspace crate for creating 7z archives.

- **License**: Public Domain
- **Website**: https://www.7-zip.org/sdk.html
- **Version**: 24.09
- **Usage**: Compiled into `lib7z_ffi.a` static library. LZMA SDK is placed in the public domain by Igor Pavlov.
