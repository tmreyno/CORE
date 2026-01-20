# Third-Party Licenses

This document lists the third-party libraries used by CORE-FFX and their respective licenses.

## libarchive

CORE-FFX uses libarchive for reading various archive formats (7-Zip, RAR, ZIP, TAR, etc.).

- **License**: BSD 2-Clause License
- **Website**: https://www.libarchive.org/
- **Usage**: Dynamic linking via FFI (libarchive2 crate)

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

All crates are compatible with CORE-FFX's MIT license.

## System Libraries

CORE-FFX may dynamically link to system libraries:

- **libarchive** (BSD 2-Clause) - Archive format support
- **zlib** (zlib License) - Compression
- **liblzma** (Public Domain) - XZ/LZMA compression
- **libbz2** (BSD-style) - BZIP2 compression
- **libzstd** (BSD/GPLv2) - Zstandard compression (BSD used)
