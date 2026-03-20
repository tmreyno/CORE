# Mobile App Development

Living development document for the CORE mobile companion app. This file is intended to be updated during design, implementation, testing, and release work.

## Purpose

Build a mobile companion app for field work that complements CORE-FFX without attempting to parse or create forensic container formats on-device.

The mobile app should support:

- Viewing project-stored case data relevant to field work
- Creating and editing notes
- Filling evidence collection forms
- Capturing photos, location, and timestamps for evidence
- Managing chain of custody entries
- Generating simple field-facing reports and exports
- Returning mobile-collected data back into the desktop CORE workflow

The mobile app should not try to replace the desktop forensic workstation. It is a field capture and review tool.

## Working Name

- Product: CORE Field
- Repository target: `CORE-Field` or a mobile workspace under the main repo later if preferred
- iOS bundle identifier target: `com.core-ffx.field`

## Scope

### In Scope

- iPhone-first experience
- Local mobile project storage
- Notes, bookmarks, evidence collection records, chain of custody, simple reports
- Photo capture and attachment
- GPS capture for location-aware field entries
- Barcode and QR scanning for evidence IDs
- Optional OCR of labels, receipts, and handwritten or printed field sheets where practical
- Export or sync of mobile-generated data back to desktop CORE

### Out of Scope

- Parsing E01, Ex01, L01, AD1, AFF4, UFED, RAW, or archive forensic containers on-device
- Hash verification of full forensic images on-device
- Full desktop viewer parity
- Large-scale indexing, deduplication, or search-engine features from desktop CORE
- Real-time collaborative sync in the first implementation

## Product Principles

1. Mobile is for capture, review, and collection support.
2. Desktop remains the forensic authority for container handling and heavy analysis.
3. All field records must preserve timestamps, authorship, and provenance.
4. Sync back to desktop must be deterministic and auditable.
5. Mobile workflows should function offline first.

## Recommended Technical Direction

### Platform

- Primary target: iOS
- First device target: iPhone
- Optional later target: iPad with a layout pass

### App Stack

- Tauri Mobile v2
- SolidJS frontend
- Rust backend
- SQLite via `rusqlite` with bundled SQLite
- JSON serialization via `serde` and `serde_json`
- IDs via `uuid`
- Time handling via `chrono`

### Why This Direction

- Keeps frontend and backend development patterns aligned with desktop CORE
- Reuses Rust types, validation logic, and data-shape conventions where practical
- Minimizes context switching between desktop and mobile implementations
- Supports local-first storage and deterministic export

## Architecture Overview

```text
iPhone App (Tauri Mobile + SolidJS + Rust)
  ├── UI layer
  │   ├── Dashboard
  │   ├── Projects
  │   ├── Evidence Collection
  │   ├── Chain of Custody
  │   ├── Notes
  │   ├── Photos
  │   └── Reports
  ├── App services
  │   ├── local project store
  │   ├── media attachment manager
  │   ├── export package builder
  │   └── import/export validation
  ├── Mobile SQLite database
  └── File export/import bridge
        ├── AirDrop
        ├── Files/iCloud
        ├── USB transfer
        └── later: direct desktop sync

Desktop CORE
  ├── existing .cffx project
  ├── existing .ffxdb project database
  └── import/merge pipeline for mobile records
```

## Integration Model

The safest first version is export/import, not live sync.

Recommended initial flow:

1. Mobile creates or updates records locally.
2. Mobile exports a signed or structured sync package.
3. Desktop CORE imports the package.
4. Desktop writes imported records into the project `.ffxdb` using the existing project database model.
5. Desktop records import provenance in activity or audit tables.

This avoids early complexity around conflict resolution, account identity, and partial live synchronization.

## Data Strategy

### Storage Model

Use a mobile SQLite database that mirrors only the subset of desktop data required for field operations.

Recommended first-pass mobile tables:

- `notes`
- `bookmarks`
- `evidence_collections`
- `collected_items`
- `coc_items`
- `coc_transfers`
- `coc_amendments` if immutability is enforced on mobile in phase 2
- `coc_audit_log` if audit is enforced on mobile in phase 2
- `form_submissions`
- `reports`
- `activity_log`
- minimal `users` or local examiner identity table
- local attachment table for photos, scans, and document references

### Desktop Alignment

The desktop repo already has authoritative project DB types and schema in these files:

- [src-tauri/src/project_db/types.rs](/Users/terryreynolds/GitHub/CORE-1/src-tauri/src/project_db/types.rs)
- [src/types/projectDb.ts](/Users/terryreynolds/GitHub/CORE-1/src/types/projectDb.ts)
- [src-tauri/src/project_db/schema_tables.rs](/Users/terryreynolds/GitHub/CORE-1/src/project_db/schema_tables.rs)

The mobile app should reuse field names and semantics where possible, but it should not blindly copy every table or every desktop concern.

## Sync Package Design

### First Release Recommendation

Use a portable export package with:

- `manifest.json`
- `mobile.db` or structured JSON payloads
- attached media files under `attachments/`
- report exports under `reports/`

Example:

```text
core-field-export-2026-03-18/
  manifest.json
  mobile.db
  attachments/
    photo-001.jpg
    photo-002.jpg
    barcode-scan-001.json
  reports/
    field-summary.pdf
```

### Manifest Requirements

- app version
- schema version
- export timestamp
- device identifier
- examiner/user identifier
- project identifier
- attachment inventory
- checksums for package contents

### Import Rules

- Desktop import must validate manifest structure before writing anything
- Imported rows should preserve original IDs when safe
- Imported records should include source metadata such as `created_by`, `source_device`, and `imported_at`
- Conflicts should be surfaced, not silently overwritten

## Feature Areas

### 1. Project Workspace

The mobile app needs a lightweight project concept.

Required capabilities:

- Create local field project
- Open existing local field project
- Attach field work to a desktop case number
- Store examiner metadata
- Store agency or organization metadata

Recommended fields:

- project id
- case number
- case title
- examiner name
- organization
- created at
- updated at
- sync status

### 2. Notes

Support fast capture in the field.

Required capabilities:

- Plain text note creation and editing
- Priority or severity marker
- Timestamps
- Optional attachment linkage
- Optional geolocation snapshot
- Optional voice-to-text entry on iPhone

Future enhancements:

- note templates
- quick tags
- transcription cleanup workflow

### 3. Evidence Collection

This is a primary field workflow.

Required capabilities:

- Create evidence collection entries
- Add one or more collected items
- Capture item descriptions and identifiers
- Attach photos to items
- Record who collected the item and when
- Record where collection occurred

Use the desktop form template as the semantic reference:

- [src/templates/forms/evidence_collection.json](/Users/terryreynolds/GitHub/CORE-1/src/templates/forms/evidence_collection.json)

The mobile UI does not need to match desktop layout exactly, but the meaning of fields should remain aligned.

### 4. Chain of Custody

Required capabilities:

- Create COC records
- Add transfers
- Record relinquished by, received by, purpose, time, and location
- Preserve immutable history once locked

Recommended rule:

- Draft entries can be edited
- Locked entries become append-only
- Changes after lock create amendment or audit records

Relevant desktop references:

- [src-tauri/src/project_db/forensic.rs](/Users/terryreynolds/GitHub/CORE-1/src-tauri/src/project_db/forensic.rs)
- [src/components/report/wizard/cocConverters.ts](/Users/terryreynolds/GitHub/CORE-1/src/components/report/wizard/cocConverters.ts)

### 5. Photo and Media Capture

Required capabilities:

- Camera capture from inside an item or note workflow
- Attach existing photos from device library if policy allows
- Preserve original file metadata when possible
- Generate thumbnails for browsing

Recommended captured metadata:

- file name
- capture timestamp
- GPS coordinates if enabled
- orientation
- associated record ID
- optional caption

### 6. Simple Reports

The first version should generate lightweight field reports, not full desktop forensic reports.

Examples:

- Evidence collection summary
- Chain of custody handoff summary
- Field activity summary
- Photo log

Output formats:

- PDF
- shareable JSON export for desktop ingestion

### 7. Import Back Into Desktop CORE

Desktop CORE should later gain a dedicated import path for mobile exports.

Recommended desktop-side responsibilities:

- validate package
- preview changes before import
- merge notes and forms into `.ffxdb`
- copy attachments into the project evidence or support directory as configured
- create activity log entries documenting import operations

## iPhone Hardware Opportunities

These are practical capabilities worth using because they materially improve field workflows.

### Phase 1 Candidates

- Camera
- GPS location
- barcode and QR scanning
- share sheet export
- face ID or passcode app unlock

### Phase 2 Candidates

- OCR for labels and handwritten forms where usable
- speech-to-text for rapid note dictation
- digital signatures on handoff workflows

### Phase 3 Candidates

- NFC tag association for evidence packages
- LiDAR-assisted scene capture where device support exists
- on-device transcription cleanup or summarization

## Security Requirements

The mobile app will contain sensitive case data, so security cannot be deferred.

Required baseline controls:

- app launch protection with Face ID or passcode gate
- encrypted local database or encrypted sensitive fields if practical in phase 1
- encrypted attachment storage if feasible
- no automatic cloud sync unless explicitly designed and approved
- explicit export action required for data leaving device
- audit event for export generation

## Offline-First Requirements

The app must remain useful without network connectivity.

That means:

- all core data entry works locally
- attachments save locally
- report generation works locally
- export package generation works locally
- sync is deferred until user chooses a transport path

## UI Structure

Recommended primary navigation:

1. Dashboard
2. Projects
3. Collections
4. Chain of Custody
5. Notes
6. Reports
7. Settings

Recommended dashboard cards:

- active project
- recent notes
- pending sync exports
- recent collections
- quick capture actions

Recommended quick actions:

- New note
- New evidence item
- Scan barcode
- Take photo
- New custody transfer
- Export package

## Proposed Directory Structure

```text
CORE-Field/
  src/
    components/
      dashboard/
      projects/
      notes/
      collections/
      custody/
      reports/
      settings/
      media/
    hooks/
    api/
    types/
    utils/
    styles/
  src-tauri/
    src/
      commands/
      db/
      export/
      security/
      media/
      reports/
      sync/
      types/
    capabilities/
  docs/
```

## Development Phases

### Phase 0: Project Setup

- [ ] Create mobile repo or workspace
- [ ] Scaffold Tauri Mobile app with SolidJS
- [ ] Confirm iOS build pipeline works on local machine
- [ ] Add baseline SQLite integration
- [ ] Define initial mobile schema version
- [ ] Add signing, bundle ID, and app naming

Deliverable:

- App launches on iPhone simulator and physical device

### Phase 1: Local Project and Notes

- [ ] Create local project model
- [ ] Build project list and project detail screens
- [ ] Implement note CRUD
- [ ] Implement attachment support for notes
- [ ] Add export of notes-only package
- [ ] Add basic PDF or JSON report output

Deliverable:

- Usable field note-taking workflow

### Phase 2: Evidence Collection

- [ ] Port evidence collection field model
- [ ] Build collection list and collection detail screens
- [ ] Add repeatable collected item workflows
- [ ] Add barcode scan entry option
- [ ] Add camera attachment flow for items
- [ ] Add location capture option

Deliverable:

- End-to-end evidence collection from phone

### Phase 3: Chain of Custody

- [ ] Build draft COC entry workflow
- [ ] Add transfer entry workflow
- [ ] Add lock or finalize logic
- [ ] Add audit trail or amendment model
- [ ] Generate custody summary export

Deliverable:

- Mobile COC workflow suitable for field handoff records

### Phase 4: Reports and Export Packages

- [ ] Generate simple PDF field reports
- [ ] Build structured export package generator
- [ ] Add package checksums and manifest validation
- [ ] Add share sheet or Files export

Deliverable:

- Portable package importable by desktop tools later

### Phase 5: Desktop Import Path

- [ ] Add mobile import command path in desktop CORE
- [ ] Validate package manifest and schema version
- [ ] Map imported records into `.ffxdb`
- [ ] Copy attachments into project-managed storage
- [ ] Log all imports in activity records

Deliverable:

- Desktop CORE can ingest mobile-generated data

### Phase 6: Security Hardening

- [ ] Face ID or passcode gate
- [ ] encryption review
- [ ] attachment storage review
- [ ] export authorization guardrails
- [ ] tamper and audit review

Deliverable:

- Production-ready field data handling baseline

### Phase 7: Usability and Field Validation

- [ ] field test with realistic workflows
- [ ] optimize photo and note entry speed
- [ ] reduce taps in collection workflow
- [ ] test low-connectivity conditions
- [ ] refine import conflict UX on desktop

Deliverable:

- Fit-for-purpose field workflow

## Initial Command Surface

Likely mobile Tauri commands:

- `project_create_local`
- `project_list_local`
- `project_load_local`
- `note_upsert`
- `note_delete`
- `collection_upsert`
- `collection_list`
- `collected_item_upsert`
- `coc_item_upsert`
- `coc_transfer_upsert`
- `media_attach_file`
- `report_generate_summary`
- `export_build_package`
- `export_list_packages`

These are placeholders for planning. Final names should stay consistent with desktop conventions where practical.

## Data Compatibility Rules

1. Reuse desktop field names where semantics are the same.
2. Prefer additive schema changes over meaning changes.
3. Never silently reinterpret chain of custody fields.
4. Treat mobile attachment paths as temporary until imported into desktop-managed storage.
5. Preserve provenance on import.

## Risks

### Product Risks

- Trying to bring too much desktop complexity onto mobile
- Mixing field capture workflows with heavy forensic-analysis expectations
- Building sync too early before local workflows are stable

### Technical Risks

- Attachment management complexity
- iOS permissions and storage edge cases
- Import conflict handling between mobile and desktop records
- Security tradeoffs around offline local storage

### Mitigations

- Keep v1 focused on notes, collections, COC, and export
- use export/import before direct sync
- define schema contracts before UI expansion
- field test early with real workflows

## Current Decisions

- Mobile will not parse forensic containers in v1.
- Mobile is offline first.
- Export/import is the preferred first sync mechanism.
- Tauri Mobile + SolidJS + Rust is the recommended implementation path.
- Desktop CORE remains the system of record for forensic container analysis.

## Open Questions

- Should the mobile app be a separate repo or live in this repo as a sibling workspace?
- Should attachments import into project support directories or a dedicated mobile-import directory?
- Should PDF generation happen fully on-device in v1, or should JSON export be primary and PDF secondary?
- How strict should mobile-side COC immutability be in the first release?
- Should iPad layout be included in v1 or deferred?

## Update Rules For This Document

When implementation starts, update this file whenever one of these changes:

- scope changes
- schema changes
- sync package structure changes
- mobile command names change
- security model changes
- a phase is completed or split
- a risk is retired or a new risk appears

Add dated notes under a simple decision log section when major decisions are made.

## Decision Log

### 2026-03-18

- Created initial living plan for the CORE mobile companion app.
- Confirmed that mobile scope excludes forensic container parsing.
- Confirmed that evidence collection, notes, chain of custody, photos, and simple reports are primary v1 workflows.
- Chose export/import as the first sync model.