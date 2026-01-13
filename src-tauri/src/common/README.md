# Common Utilities Module

Shared backend utilities used across CORE-FFX.

## Files

- `mod.rs` - Module exports
- `hash.rs` - Hashing helpers (SHA, BLAKE, XXH)
- `hex.rs` - Hex formatting
- `binary.rs` - Little-endian readers
- `entropy.rs` - Entropy calculation
- `magic.rs` - Magic-byte detection
- `path_security.rs` - Path traversal protection
- `audit.rs` - Audit logging
- `segments.rs` - Multi-segment discovery
- `io_pool.rs` - File handle pooling

## Security

- Sanitize all path input before writing
- Avoid mutating evidence source files
- Emit audit entries for evidence access
