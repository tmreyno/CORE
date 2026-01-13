# CORE-FFX Type Definitions

TypeScript domain types for the CORE-FFX frontend.

## Files

| File | Purpose |
|------|---------|
| `index.ts` | Central export point |
| `formats.ts` | Format definitions and detection helpers |
| `lifecycle.ts` | Evidence lifecycle stages and verification types |
| `processed.ts` | Processed database types |
| `project.ts` | Project file model (frontend) |

## Alignment With Rust

These types mirror backend structures where possible:

- `formats.ts` <-> `src-tauri/src/formats.rs`
- `lifecycle.ts` <-> `src-tauri/src/containers/traits.rs`
- `processed.ts` <-> `src-tauri/src/processed/types.rs`
- `report/types.ts` <-> `src-tauri/src/report/types.rs`

Note: the frontend project model is richer (v2) than the on-disk Rust project schema (v1). See `src-tauri/FFX_PROJECT_FORMAT.md`.

## Conventions

- Avoid `any`
- Prefer explicit unions for enums
- Document complex types with comments
