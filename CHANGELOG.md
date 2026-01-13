# CORE-FFX Changelog

All notable changes to CORE-FFX are documented here. Format follows Keep a Changelog and Semantic Versioning.

## [Unreleased]

### Added

- Report wizard with preview and export (PDF, DOCX, HTML, Markdown; Typst optional)
- Expanded hash algorithm options (SHA-1/256/512, MD5, BLAKE3, BLAKE2b, XXH3, XXH64, CRC32)
- Processed database discovery for multiple tools (AXIOM parsing implemented)
- Archive metadata parsing and ZIP extraction utilities

### Changed

- Documentation sweep and consolidation
- Clarified format support (full parsing vs detection-only)

## [0.1.0] - 2024-01-01

### Added

- Initial release
- AD1 logical evidence container support
- E01/Ex01 Expert Witness Format support
- L01/Lx01 EnCase logical format support
- Raw disk image support (DD, IMG, RAW)
- UFED extraction support
- Hash verification (MD5, SHA1, SHA256)
- Hex and text viewers
- Project file support (.ffxproj)
- SQLite case database

---

[Unreleased]: https://github.com/CORE/AD1-tools/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/CORE/AD1-tools/releases/tag/v0.1.0
