# CORE-FFX Documentation

Technical documentation and references for CORE-FFX.

## Project Root

| Document | Purpose |
| -------- | ------- |
| [`README.md`](../README.md) | Project overview, quick start, feature list |
| [`CODE_BIBLE.md`](../CODE_BIBLE.md) | Authoritative codebase map and glossary |
| [`HELP.md`](../HELP.md) | Quick help guide for UI and workflows |
| [`CONTRIBUTING.md`](../CONTRIBUTING.md) | Developer workflow and standards |
| [`CHANGELOG.md`](../CHANGELOG.md) | Release history |
| [`SECURITY.md`](../SECURITY.md) | Security policy |
| [`THIRD_PARTY_LICENSES.md`](../THIRD_PARTY_LICENSES.md) | Third-party license attributions |
| [`WINDOWS_BUILD.md`](../WINDOWS_BUILD.md) | Windows build instructions |
| [`.github/copilot-instructions.md`](../.github/copilot-instructions.md) | AI coding agent guidance |

## Architecture References

| Document | Purpose |
| -------- | ------- |
| [`AD1_ARCHITECTURE_DIAGRAMS.md`](AD1_ARCHITECTURE_DIAGRAMS.md) | AD1 format architecture and data flow diagrams |
| [`AD1_FILE_STRUCTURE_COMPLETE.md`](AD1_FILE_STRUCTURE_COMPLETE.md) | Complete AD1 binary file structure reference |
| [`LIBEWF_ANALYSIS.md`](LIBEWF_ANALYSIS.md) | libewf (E01/EWF) library analysis |
| [`SEVENZIP_FFI_API_REFERENCE.md`](SEVENZIP_FFI_API_REFERENCE.md) | 7-Zip FFI Rust API reference |
| [`UI_CONSTANTS_REFERENCE.md`](UI_CONSTANTS_REFERENCE.md) | UI constants and theme tokens |

## Format Specifications (`formats/`)

| Document | Purpose |
| -------- | ------- |
| [`AD1.md`](formats/AD1.md) | AD1 container format notes |
| [`L01.md`](formats/L01.md) | L01 container format notes |
| [`ewfHex.md`](formats/ewfHex.md) | EWF/E01 hex structure reference |
| [`7zip.md`](formats/7zip.md) | 7-Zip integration notes |
| [`Archive_Reference.md`](formats/Archive_Reference.md) | Archive format reference |
| [`zlib_compression.md`](formats/zlib_compression.md) | Zlib compression implementation |
| [`AXIOM_DATABASE_BIBLE.md`](formats/AXIOM_DATABASE_BIBLE.md) | Magnet AXIOM database schema reference |
| [`FFX_PROJECT_FORMAT.md`](formats/FFX_PROJECT_FORMAT.md) | .cffx project file schema |
| [`FFX_REPORT_TEMPLATES.md`](formats/FFX_REPORT_TEMPLATES.md) | Report template format |

## Backend Module READMEs

| Location | Content |
| -------- | ------- |
| [`src-tauri/src/README.md`](../src-tauri/src/README.md) | Backend module overview |
| [`src-tauri/src/ad1/README.md`](../src-tauri/src/ad1/README.md) | AD1 parser module |
| [`src-tauri/src/ewf/README.md`](../src-tauri/src/ewf/README.md) | EWF/E01 parser module |
| [`src-tauri/src/ufed/README.md`](../src-tauri/src/ufed/README.md) | UFED parser module |
| [`src-tauri/src/archive/README.md`](../src-tauri/src/archive/README.md) | Archive handling module |
| [`src-tauri/src/containers/README.md`](../src-tauri/src/containers/README.md) | Container abstraction layer |
| [`src-tauri/src/containers/ARCHITECTURE.md`](../src-tauri/src/containers/ARCHITECTURE.md) | Container architecture |
| [`src-tauri/src/viewer/README.md`](../src-tauri/src/viewer/README.md) | Viewer module (hex, document, etc.) |
| [`src-tauri/src/common/README.md`](../src-tauri/src/common/README.md) | Shared utilities |
| [`src-tauri/src/processed/README.md`](../src-tauri/src/processed/README.md) | Processed database module |
| [`src-tauri/src/report/README.md`](../src-tauri/src/report/README.md) | Report generation module |

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

## 7-Zip FFI Library

| Location | Content |
| -------- | ------- |
| [`sevenzip-ffi/rust/README.md`](../sevenzip-ffi/rust/README.md) | Library overview |
| [`sevenzip-ffi/rust/docs/guides/`](../sevenzip-ffi/rust/docs/guides/) | Build guide, quick start, how it works |
| [`sevenzip-ffi/rust/docs/api/`](../sevenzip-ffi/rust/docs/api/) | Rust API bindings reference |
| [`sevenzip-ffi/rust/docs/testing/`](../sevenzip-ffi/rust/docs/testing/) | Testing guides |
| [`sevenzip-ffi/rust/docs/development/`](../sevenzip-ffi/rust/docs/development/) | Development history |

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

*Last updated: February 11, 2026*
