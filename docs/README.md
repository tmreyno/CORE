# CORE-FFX Documentation

Technical documentation and references for CORE-FFX — a forensic file explorer built with Tauri v2 (Rust) + SolidJS (TypeScript).

---

## Core Capabilities

CORE-FFX provides the following major subsystems. Each is fully implemented and documented:

| Capability | Summary | Key Docs |
|-----------|---------|----------|
| **Evidence Containers** | Read-only browsing of AD1, E01/Ex01, L01, UFED, raw/DD, and 15+ archive formats | [Container ARCHITECTURE](../src-tauri/src/containers/ARCHITECTURE.md), [AD1 Structure](AD1_FILE_STRUCTURE_COMPLETE.md) |
| **Full-Text Search** | Tantivy 0.22 engine indexes filenames, paths, and file content across all containers | [SEARCH_ARCHITECTURE.md](SEARCH_ARCHITECTURE.md) |
| **Hash Verification** | Storage-aware parallel hashing with per-drive concurrency, 3-arm container routing, LRU cache | [HASH_VERIFICATION_MODEL.md](HASH_VERIFICATION_MODEL.md) |
| **Case Data Model** | Dual-file architecture (.cffx + .ffxdb), 35+ SQLite tables, COC immutability, FTS5 | [CASE_DATA_MODEL.md](CASE_DATA_MODEL.md) |
| **Evidence Collection** | Schema-driven forms (JSON templates), auto-enrichment from container metadata | [FORM_TEMPLATE_SYSTEM.md](FORM_TEMPLATE_SYSTEM.md) |
| **File Viewers** | PDF, Office, email, spreadsheet, plist, binary, hex, registry, database, image (EXIF) | [Viewer README](../src-tauri/src/viewer/README.md) |
| **Export** | E01 creation (libewf-ffi), L01 creation (pure-Rust), 7z archives (sevenzip-ffi), file copy | [SEVENZIP_FFI_API_REFERENCE.md](SEVENZIP_FFI_API_REFERENCE.md) |
| **Processed Databases** | Parse AXIOM, Cellebrite, Autopsy case databases | [Processed README](../src-tauri/src/processed/README.md), [AXIOM Bible](formats/AXIOM_DATABASE_BIBLE.md) |
| **Reports** | Wizard-driven report generation with COC, evidence collection, and findings sections | [Report README](../src-tauri/src/report/README.md) |
| **Project Merge** | Combine multiple .cffx/.ffxdb projects with examiner identification and conflict resolution | [copilot-instructions.md § Merge](../.github/copilot-instructions.md) |

---

## Recommended Reading Order

**New to the codebase? Start here:**

1. [`.github/copilot-instructions.md`](../.github/copilot-instructions.md) — **Primary source of truth.** Architecture overview, all invariants, "Do NOT" rules, complete hook/command reference.
2. [`CODE_BIBLE.md`](../CODE_BIBLE.md) — Authoritative codebase map and glossary.
3. [`CASE_DATA_MODEL.md`](CASE_DATA_MODEL.md) — Understand the dual-file data model (.cffx + .ffxdb schema).
4. [`CRATE_API_NOTES.md`](../CRATE_API_NOTES.md) — **Check before using any crate API.** Prevents 50% of compilation errors.
5. [`FRONTEND_API_NOTES.md`](../FRONTEND_API_NOTES.md) — SolidJS/TypeScript API notes.

**Then explore by topic:**

- Search & indexing → [`SEARCH_ARCHITECTURE.md`](SEARCH_ARCHITECTURE.md)
- Hashing & verification → [`HASH_VERIFICATION_MODEL.md`](HASH_VERIFICATION_MODEL.md)
- Forms & evidence collection → [`FORM_TEMPLATE_SYSTEM.md`](FORM_TEMPLATE_SYSTEM.md)
- Container formats → [Format Specifications](#format-specifications) section below
- UI styling → [`src/styles/README.md`](../src/styles/README.md)

---

## Start Here

| Document | Purpose |
| -------- | ------- |
| [`README.md`](../README.md) | Project overview, quick start, feature list |
| [`.github/copilot-instructions.md`](../.github/copilot-instructions.md) | AI coding agent guidance — **primary architecture reference** |
| [`CODE_BIBLE.md`](../CODE_BIBLE.md) | Authoritative codebase map and glossary |
| [`CRATE_API_NOTES.md`](../CRATE_API_NOTES.md) | Third-party crate API reference (prevents compilation errors) |
| [`FRONTEND_API_NOTES.md`](../FRONTEND_API_NOTES.md) | SolidJS/TypeScript API reference |
| [`HELP.md`](../HELP.md) | Quick help guide for UI and workflows |

## Core Architecture

| Document | Purpose |
| -------- | ------- |
| [`SEARCH_ARCHITECTURE.md`](SEARCH_ARCHITECTURE.md) | Tantivy search engine — schema, crawlers, query model, content extraction |
| [`HASH_VERIFICATION_MODEL.md`](HASH_VERIFICATION_MODEL.md) | Hash system — storage-aware scheduling, 3-arm routing, LRU cache |
| [`CASE_DATA_MODEL.md`](CASE_DATA_MODEL.md) | Data model — .cffx project file, .ffxdb schema (35+ tables), COC immutability |
| [`FORM_TEMPLATE_SYSTEM.md`](FORM_TEMPLATE_SYSTEM.md) | JSON-schema-driven forms — templates, hooks, persistence, auto-fill |
| [`MACOS_SIGNING.md`](MACOS_SIGNING.md) | macOS code signing and notarization |
| [`UI_CONSTANTS_REFERENCE.md`](UI_CONSTANTS_REFERENCE.md) | UI constants and theme tokens |

## Container Format References

| Document | Purpose |
| -------- | ------- |
| [`AD1_ARCHITECTURE_DIAGRAMS.md`](AD1_ARCHITECTURE_DIAGRAMS.md) | AD1 format architecture and data flow diagrams |
| [`AD1_FILE_STRUCTURE_COMPLETE.md`](AD1_FILE_STRUCTURE_COMPLETE.md) | Complete AD1 binary file structure reference |
| [`LIBEWF_ANALYSIS.md`](LIBEWF_ANALYSIS.md) | libewf (E01/EWF) library analysis |
| [`SEVENZIP_FFI_API_REFERENCE.md`](SEVENZIP_FFI_API_REFERENCE.md) | 7-Zip FFI C API reference (dict sizes, UTF-8, SDK 24.09) |

## Format Specifications

| Document | Purpose |
| -------- | ------- |
| [`formats/AD1.md`](formats/AD1.md) | AD1 container format notes |
| [`formats/L01.md`](formats/L01.md) | L01 container format notes |
| [`formats/ewfHex.md`](formats/ewfHex.md) | EWF/E01 hex structure reference |
| [`formats/7zip.md`](formats/7zip.md) | 7-Zip integration notes |
| [`formats/Archive_Reference.md`](formats/Archive_Reference.md) | Archive format reference |
| [`formats/zlib_compression.md`](formats/zlib_compression.md) | Zlib compression implementation |
| [`formats/AXIOM_DATABASE_BIBLE.md`](formats/AXIOM_DATABASE_BIBLE.md) | Magnet AXIOM database schema reference |
| [`formats/FFX_PROJECT_FORMAT.md`](formats/FFX_PROJECT_FORMAT.md) | .cffx project file schema |
| [`formats/FFX_REPORT_TEMPLATES.md`](formats/FFX_REPORT_TEMPLATES.md) | Report template format |

## Native FFI Libraries

| Library | Location | Purpose |
| ------- | -------- | ------- |
| **libewf-ffi** | [`libewf-ffi/`](../libewf-ffi/) | Safe Rust FFI bindings for libewf 20251220 — EWF image creation/reading |
| **sevenzip-ffi** | [`sevenzip-ffi/`](../sevenzip-ffi/) | C library + Rust FFI for 7z archive creation (LZMA SDK 24.09) |
| **libarchive** (patched) | [`patches/libarchive2-sys/`](../patches/libarchive2-sys/) | Archive reading (ZIP, 7z, TAR, RAR, ISO, etc.) |

## Backend Module READMEs

| Location | Content |
| -------- | ------- |
| [`src-tauri/src/README.md`](../src-tauri/src/README.md) | Backend module overview |
| [`src-tauri/src/ad1/README.md`](../src-tauri/src/ad1/README.md) | AD1 parser module |
| [`src-tauri/src/ewf/README.md`](../src-tauri/src/ewf/README.md) | EWF/E01 parser module (pure-Rust) |
| [`src-tauri/src/ufed/README.md`](../src-tauri/src/ufed/README.md) | UFED parser module |
| [`src-tauri/src/archive/README.md`](../src-tauri/src/archive/README.md) | Archive handling module |
| [`src-tauri/src/containers/README.md`](../src-tauri/src/containers/README.md) | Container abstraction layer |
| [`src-tauri/src/containers/ARCHITECTURE.md`](../src-tauri/src/containers/ARCHITECTURE.md) | Container architecture |
| [`src-tauri/src/viewer/README.md`](../src-tauri/src/viewer/README.md) | Viewer module (hex, document, etc.) |
| [`src-tauri/src/common/README.md`](../src-tauri/src/common/README.md) | Shared utilities (hash, binary, segments, filesystem drivers) |
| [`src-tauri/src/processed/README.md`](../src-tauri/src/processed/README.md) | Processed database parsers (AXIOM, Cellebrite, Autopsy) |
| [`src-tauri/src/report/README.md`](../src-tauri/src/report/README.md) | Report generation module |
| [`src-tauri/src/l01_writer/`](../src-tauri/src/l01_writer/) | Pure-Rust L01 logical evidence writer |
| [`src-tauri/src/search/`](../src-tauri/src/search/) | Tantivy full-text search engine |

## Frontend Module READMEs

| Location | Content |
| -------- | ------- |
| [`src/components/README.md`](../src/components/README.md) | Component catalog |
| [`src/components/project/README.md`](../src/components/project/README.md) | Project management components |
| [`src/components/report/README.md`](../src/components/report/README.md) | Report wizard components |
| [`src/hooks/README.md`](../src/hooks/README.md) | State management hooks |
| [`src/styles/README.md`](../src/styles/README.md) | Tailwind CSS styling guide |
| [`src/report/README.md`](../src/report/README.md) | Report API |
| [`src/extensions/README.md`](../src/extensions/README.md) | Extension system |
| [`src/utils/README.md`](../src/utils/README.md) | Utility functions |

## Project & Build

| Document | Purpose |
| -------- | ------- |
| [`CONTRIBUTING.md`](../CONTRIBUTING.md) | Developer workflow and standards |
| [`CHANGELOG.md`](../CHANGELOG.md) | Release history |
| [`WINDOWS_BUILD.md`](../WINDOWS_BUILD.md) | Windows cross-compilation instructions |
| [`SECURITY.md`](../SECURITY.md) | Security policy |
| [`THIRD_PARTY_LICENSES.md`](../THIRD_PARTY_LICENSES.md) | Third-party license attributions |

## Archive

Historical documentation from completed features, migrations, and fixes is organized in [`archive/`](archive/):

| Folder | Content |
| ------ | ------- |
| [`phases/`](archive/phases/) | Performance optimization phases 4-16 |
| [`2026-01-ad1-v2/`](archive/2026-01-ad1-v2/) | AD1 V2 parser migration (13 docs) |
| [`2026-01-implementation/`](archive/2026-01-implementation/) | Implementation summaries, code fixes, refactoring plans |
| [`2026-01-project-management/`](archive/2026-01-project-management/) | Project management system docs |
| [`2026-01-testing/`](archive/2026-01-testing/) | Test plans and results |
| [`2026-02-fixes/`](archive/2026-02-fixes/) | DMG and VFS bug fix documentation |
| [`consolidation/`](archive/consolidation/) | Code consolidation and migration summaries |
| [`hash-system/`](archive/hash-system/) | Hash system modernization docs |
| [`sevenzip/`](archive/sevenzip/) | 7-Zip integration status docs |

---

*Last updated: June 2025*
