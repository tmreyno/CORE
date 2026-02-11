# FFX Project File Format (.cffx)

## Status

The Rust backend now fully supports **version 2** of the project schema. Both the frontend and backend use the same comprehensive data model, ensuring all project state is properly persisted across save/load operations.

## Default Location

The default project path is derived from the evidence root folder name and saved in the parent directory. For example: `/path/to/evidence-root/../evidence-root.cffx`

## Schema (v2)

The v2 schema includes comprehensive project state tracking.

### Metadata

- `version`: Project file format version (2)
- `project_id`: Unique project identifier
- `name`: Project name
- `description`: Optional project description
- `root_path`: Root directory path
- `created_at`: Project creation timestamp
- `saved_at`: Last save timestamp
- `created_by_version`: App version that created the project
- `saved_by_version`: App version that last saved the project

### Users and Sessions

- `users[]`: Array of users who have accessed the project
- `current_user`: Active user
- `sessions[]`: Session history with start/end times
- `current_session_id`: Active session identifier
- `activity_log[]`: Comprehensive activity tracking
- `activity_log_limit`: Maximum log entries (default: 1000)

### Evidence State

- `locations`: Project directory structure
- `open_directories[]`: Currently open directories
- `recent_directories[]`: Recently accessed directories
- `tabs[]`: Open evidence container tabs
- `active_tab_path`: Currently active tab
- `file_selection`: Selected files state
- `hash_history`: Complete hash computation history

### Processed Databases

- `processed_databases`: State for AXIOM, Cellebrite, etc.

### Bookmarks and Notes

- `bookmarks[]`: Bookmarked items
- `notes[]`: Project notes and annotations
- `tags[]`: Tag definitions

### Reports

- `reports[]`: Generated report history

### Searches

- `saved_searches[]`: Saved search queries
- `recent_searches[]`: Recent search history
- `filter_state`: Active filters and sort settings

### UI State

- `ui_state`: Complete UI restoration state

### Settings

- `settings`: Project-specific settings

### Custom Data

- `custom_data{}`: Extensible key-value storage

## Backward Compatibility

The v2 schema maintains backward compatibility with v1 project files:

- All v1 fields are preserved and automatically upgraded
- Missing v2 fields are initialized with sensible defaults
- The `app_version` field (v1 legacy) is mapped to `saved_by_version`
- Version migration is automatic and transparent
