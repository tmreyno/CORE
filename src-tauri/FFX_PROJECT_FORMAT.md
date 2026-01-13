# FFX Project File Format (.ffxproj)

## Status

The Rust backend currently persists **version 1** of the project schema. The frontend uses a richer in-memory model (v2), but only the v1 subset is saved/loaded by the backend. Extra fields sent from the UI are ignored on load and not written back to disk.

## Default Location

The default project path is derived from the evidence root folder name and saved in the parent directory:

```
/path/to/evidence-root/..
└── evidence-root.ffxproj
```

## Schema (v1)

```json
{
  "version": 1,
  "name": "CaseFolder",
  "root_path": "/path/to/CaseFolder",
  "created_at": "2026-01-04T12:00:00Z",
  "saved_at": "2026-01-04T12:05:00Z",
  "app_version": "0.1.0",
  "tabs": [
    { "file_path": "/path/to/CaseFolder/case.E01", "order": 0 }
  ],
  "active_tab_path": "/path/to/CaseFolder/case.E01",
  "hash_history": {
    "files": {
      "/path/to/CaseFolder/case.E01": [
        {
          "algorithm": "SHA-256",
          "hash_value": "...",
          "computed_at": "2026-01-04T12:04:00Z",
          "verified": { "result": "match", "verified_at": "2026-01-04T12:04:05Z" }
        }
      ]
    }
  },
  "ui_state": {
    "panel_sizes": [],
    "expanded_paths": [],
    "scroll_positions": {}
  },
  "notes": null,
  "tags": []
}
```

## Field Notes (v1)

- `tabs`: only file paths and order are persisted
- `hash_history`: per-file history of computed hashes
- `ui_state`: lightweight UI state

## Frontend Model (v2)

The frontend constructs a richer project model that includes sessions, activity logs, processed database state, and UI filters. See `src/types/project.ts` for details. These fields are not currently persisted by the backend.
