# CORE-FFX Update Review: 0.1.112 to 0.1.116

This review is bundled with CORE-FFX so the in-app update dialog can show release details directly when a new version is available.

<!-- release-note:start 0.1.116 -->
## CORE-FFX 0.1.116

### Updater Notes Repair
- **Readable update dialog:** The updater manifest now carries the full bundled release review instead of the generic nightly-build line, so users can read the app, engine, project-open, and validation changes directly in CORE-FFX before installing.
- **Immutable release recovery:** This patch supersedes `0.1.115` because GitHub immutable release assets prevented replacing the already-published `latest.json` file for `0.1.115`.
- **Same engine payload:** The evidence-loading, AD1 range-read, partial hashing, EWF/raw VFS reuse, project restore, and tab-sync fixes from `0.1.115` remain included.
- **Future bundled notes:** CORE-FFX now recognizes `0.1.116` in its bundled update-note selector so future update dialogs can include this repair note when appropriate.
<!-- release-note:end 0.1.116 -->

<!-- release-note:start 0.1.115 -->
## CORE-FFX 0.1.115

### Evidence Loading And Project Restore
- **AD1 range reads:** AD1 file reads now skip earlier declared chunks before reading a later requested range. This fixes incorrect failures when hashing, hex-viewing, or extracting later content from AD1 evidence.
- **Partial source hashing:** Hashing now continues through evidence byte sources that return smaller chunks than requested, while still rejecting true premature end-of-file conditions.
- **EWF and raw source reuse:** EWF and raw entry readers now reuse bounded virtual filesystem handles for repeated viewer, hashing, artifact extraction, and reporting reads. This reduces repeated image opens and lowers freeze risk when switching between evidence views.
- **Sibling evidence paths:** `.cffx` projects with evidence stored beside the project directory now round-trip paths as `../...` instead of invalid `./...` paths.
- **Post-load warnings:** Project load now keeps successfully loaded state when only saved UI restore or post-load setup fails, showing a warning instead of a false load failure.

### Viewer And UI Reliability
- **Evidence tab syncing:** Selecting evidence or entry tabs now keeps the active evidence context synchronized so the detail panel, tree, hashing, and viewer actions target the selected evidence.
- **Restored tab IDs:** Saved evidence, document, and processed tabs are normalized to current resolved paths, preventing stale IDs from creating duplicate or non-working tabs after a project is opened.
- **Recent projects keyboard access:** Recent project rows now support keyboard activation with Enter and Space.

### Validation
- **Seed project checks:** The project loader was checked against `/Users/terryreynolds/Cases/1827-1001/26-000.cffx` and `/Users/terryreynolds/Cases/1827-1001/1827-1001_Case_With_Data_.cffx`.
- **Local gates:** TypeScript checks, frontend regression tests, production frontend build, Rust formatting, focused Rust tests, and focused clippy checks passed.
- **GitHub gates:** PR #24 passed frontend tests, Rust backend tests on Ubuntu/macOS/Windows, performance regression checks, and application builds on Ubuntu/macOS/Windows.
- **Published updater build:** `0.1.115` is published with macOS, Windows, and Linux installer/updater artifacts plus `latest.json`.
<!-- release-note:end 0.1.115 -->

<!-- release-note:start 0.1.114 -->
## CORE-FFX 0.1.114

### Release Publishing Recovery
- **Published update version:** The corrected release is published as `0.1.114` because GitHub would not safely republish the deleted `v0.1.113` immutable release tag.
- **No extra runtime change:** Version `0.1.114` carries the `0.1.113` engine, viewer, project-open, artifact, and update-note fixes in a clean distributable build.
- **Manual release safety:** Manual release publishing now checks out the workflow commit instead of a not-yet-created release tag before generating updater metadata.
- **Release target alignment:** Draft releases now target the workflow commit when they are created so the eventual published tag points at the intended source revision.
- **Manifest generation:** The macOS architecture detector in `latest.json` generation now uses a valid regex and correctly maps the aarch64 updater bundle.

### Update Experience
- **Bundled notes:** Update availability now shows the `0.1.112`, `0.1.113`, and `0.1.114` review notes directly in CORE-FFX.
- **Version alignment:** `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, the root `Cargo.lock`, README badge, changelog references, and release documentation are aligned to `0.1.114`.
<!-- release-note:end 0.1.114 -->

<!-- release-note:start 0.1.113 -->
## CORE-FFX 0.1.113

### Evidence Project Stability
- **Project open repair:** Legacy `.cffx` and `.ffxdb` projects now repair generated evidence IDs to the resolved evidence paths used by the current app. This keeps evidence records, hash records, and source metadata connected when older case files are opened.
- **Seed project validation:** The migration path was checked against the supplied seed project files at `/Users/terryreynolds/Cases/1827-1001/26-000.cffx` and `/Users/terryreynolds/Cases/1827-1001/1827-1001_Case_With_Data_.cffx`.
- **Source fingerprint integrity:** Evidence sources now require full materialized source fingerprints before metadata is trusted, reducing mismatched source records during project open and review.
- **Partial VFS reads:** Virtual filesystem evidence reads now complete partial source reads instead of stopping early when a source segment is shorter than the requested window.

### Viewer And Navigation Reliability
- **Hex viewer bounds:** Deep offset navigation in large evidence files now reads bounded windows instead of requesting everything from the start of the evidence source. This reduces freezing and memory pressure when moving around large images or extracted files.
- **Buffered range reporting:** The hex viewer now reports loaded byte ranges from the active bounded window so the UI can describe what is actually available.
- **Scroll safety:** Viewer navigation tolerates environments where direct scroll APIs are not available, preventing viewer tests and some embedded contexts from failing.

### Update Experience
- **In-app release review:** The update availability dialog now shows this bundled `0.1.112` and `0.1.113` review directly in CORE-FFX, so users can read what changed before installing without opening the release website.
- **Version-aware notes:** Users updating from `0.1.111` or older see both `0.1.112` and `0.1.113`; users already on `0.1.112` see the `0.1.113` changes.
- **Release manifest notes:** The release workflow now uses this bundled review for the GitHub release body and `latest.json` updater notes so older installed apps can display the full update details.
- **Installer version alignment:** `src-tauri/tauri.conf.json` is aligned to `0.1.113`, and macOS release uploads now read the aarch64 bundle output path so installer and updater artifacts match the published version.

### Artifact And System Identity Engines
- **Driver artifacts:** Windows `.sys` and `.drv` files, plus Linux `.ko` kernel modules, are now collected as binary driver artifacts from loaded evidence trees. Directly viewed driver entries are also persisted through the binary artifact collector.
- **Windows system information:** The loaded-tree classifier now captures more Windows system and driver paths that are useful for machine identity and operating-system review.
- **macOS system information:** macOS boot plist data, kernel extension `Info.plist` files, and non-`/private` dsLocal identity paths are now collected as system identity artifacts.
- **Linux system information:** Linux NetworkManager and systemd-networkd configuration paths are now included in identity collection to improve network and host metadata extraction from Linux images.

### Validation
- **Frontend checks:** Targeted SolidJS and browser-viewer tests passed for the changed viewer and artifact paths.
- **Type checks:** `npx tsc --noEmit --pretty false` passed after the viewer and update-note changes.
- **Rust checks:** Rust formatting, clippy, and library tests passed for the backend engine changes, including the local `cargo test --lib --verbose` gate.
- **Seed project smoke test:** The ignored seed `.cffx` migration smoke test passed with both supplied case files.
<!-- release-note:end 0.1.113 -->

<!-- release-note:start 0.1.112 -->
## CORE-FFX 0.1.112

### Release Version Alignment
- **Version metadata sync:** `package.json`, `package-lock.json`, `src-tauri/Cargo.toml`, the root `Cargo.lock`, README badge, and changelog references were aligned to `0.1.112`.
- **Changelog compare links:** The `[Unreleased]` link now compares from the active release tag to `HEAD`, and `[0.1.112]` compares `v0.1.111...v0.1.112`.
- **Package lock refresh:** The top-level npm lockfile version fields now match `package.json` so installer and CI state match the shipped app version.

### Rust Workspace Lockfile Cleanup
- **Single Cargo lockfile:** The redundant tracked nested Tauri Cargo lockfile was removed. The root `Cargo.lock` is now the single Rust workspace lockfile source.
- **Release workflow reads:** Release workflow plugin-alignment checks now read the root `Cargo.lock` instead of the removed nested Tauri lockfile.
- **Cache keys:** Rust cache keys that previously hashed every Cargo lockfile now hash only the root `Cargo.lock`, which matches the workspace layout.

### Nightly Release Automation
- **Lockfiles in nightly bumps:** Nightly version bumps now refresh `package-lock.json` after `package.json` changes and refresh the workspace `Cargo.lock` after `src-tauri/Cargo.toml` changes.
- **Complete bump commits:** Nightly automation now commits both `package-lock.json` and `Cargo.lock` with the version bump so future releases do not drift.
- **Dry-run clarity:** Nightly dry-run output lists every file expected to change during the bump.

### Package Metadata
- **Repository URL:** `src-tauri/Cargo.toml` now points to `https://github.com/tmreyno/CORE` instead of the stale `https://github.com/CORE/AD1-tools` repository.

### Runtime Scope
- **No runtime contract change:** Version `0.1.112` was release-readiness and CI reliability work. It did not intentionally change IPC APIs, project schemas, UI workflows, or forensic engine behavior.
<!-- release-note:end 0.1.112 -->
