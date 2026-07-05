# Common Shim

`src-tauri/src/common/mod.rs` is a compatibility shim. The real shared backend
utilities and forensic engine code live in `crates/ffx-common/src`.

Do not add implementation files here. Add or update shared code in
`crates/ffx-common/src`, then re-export it from `src-tauri/src/common/mod.rs`
only if an existing `crate::common::...` path needs to keep compiling.
