# Hash Verification Model

> **Architecture:** Storage-aware parallel batch hashing with per-drive concurrency, in-memory LRU caching, and 3-arm container routing  
> **Scope:** All evidence container formats (E01, AD1, raw/dd) and ordinary files

---

## Table of Contents

1. [Overview](#overview)
2. [Storage-Aware Scheduling](#storage-aware-scheduling)
3. [Batch Hash Pipeline](#batch-hash-pipeline)
4. [Container Routing (3-Arm)](#container-routing-3-arm)
5. [Hash Cache](#hash-cache)
6. [Queue Management](#queue-management)
7. [Progress Reporting](#progress-reporting)
8. [Stored Hash Collection & Verification](#stored-hash-collection--verification)
9. [Frontend Integration](#frontend-integration)
10. [Key Files](#key-files)

---

## Overview

Hashing in CORE-FFX serves two purposes:

1. **Integrity computation** — compute MD5, SHA-1, SHA-256, or SHA-512 of evidence files
2. **Verification** — compare computed hashes against stored hashes embedded in container headers (E01 stored hashes, AD1 segment checksums)

The system is designed for forensic workloads where evidence files are large (10 GB – 2 TB), stored on mixed media (USB HDDs, internal SSDs, NAS), and must be hashed without mutual I/O interference.

---

## Storage-Aware Scheduling

### The Problem

Hashing is dominated by sequential I/O. Running too many concurrent hash operations on the same physical drive (especially HDDs) causes seek thrashing, reducing throughput dramatically. But files on **different drives** should hash in parallel since they use independent I/O channels.

### Storage Classification

Each evidence file's path is resolved to its mount point via `sysinfo::Disks`, then classified:

| Storage Class | Detection | Concurrency Limit | Rationale |
|--------------|-----------|-------------------|-----------|
| **Internal SSD** | `DiskKind::SSD` + not removable | 6 | NVMe/SATA SSDs handle parallel reads well |
| **Internal HDD** | `DiskKind::HDD` + not removable | 2 | Seek-limited; concurrent reads cause thrashing |
| **Removable** | `is_removable()` = true | 2 | Bus-limited regardless of media type |
| **Unknown** | Default fallback | 3 | Conservative default |

### Mount Point Resolution

`classify_storage()` uses **longest-prefix matching** against `sysinfo::Disks` mount points:

```text
File path: /Volumes/Evidence-USB/case001/image.E01

Mounts available:
  /                          → Internal SSD
  /Volumes/Evidence-USB      → Removable (USB HDD)  ← longest match

Result: StorageClass::Removable, concurrency = 2
```

### Per-Drive Semaphores

Files are grouped by mount point. Each unique mount point gets a `tokio::sync::Semaphore` with the drive's concurrency limit. Files on the same drive share a semaphore; files on different drives have independent semaphores and hash in full parallel.

```text
Drive /           (SSD, limit=6):  file1.E01, file2.E01
Drive /Volumes/USB (Rem, limit=2): file3.E01, file4.E01, file5.E01

→ 6 permit semaphore for /
→ 2 permit semaphore for /Volumes/USB
→ Files on / and /Volumes/USB hash independently
→ At most 2 USB files hash concurrently
```

---

## Batch Hash Pipeline

### End-to-End Flow

```text
Frontend: hashSelectedFiles() or hashAllFiles()
  1. Ensure all files have evidence_files records in .ffxdb (upsertEvidenceFile)
  2. invoke("batch_hash", { paths, algorithm })
  
Backend: batch_hash()
  3. Classify each file → StorageClass + mount point
  4. Emit "batch-drive-info" event (drive detection results)
  5. Create per-drive semaphores
  6. For each file, spawn tokio task:
     a. Wait while QUEUE_PAUSED
     b. Acquire drive semaphore permit
     c. Emit "started" progress event
     d. spawn_blocking:
        - Check hash_cache → if hit, return cached
        - Route to container-specific hash function (3-arm)
        - Cache computed result
     e. Emit "completed" / "error" progress event
  7. Collect all results
  8. Return Vec<BatchHashResult>

Frontend (on each progress event):
  9. Buffer events, flush to UI every 200ms
  10. On "completed": verify against stored hashes, persist to .ffxdb, record audit entry
```

### BatchHashResult

```rust
struct BatchHashResult {
    path: String,
    algorithm: String,
    hash: Option<String>,         // None if error
    error: Option<String>,        // None if success
    elapsed_ms: u64,
    drive_kind: Option<String>,   // "Internal SSD", "Removable", etc.
}
```

### Pre-Hash Checks

Before invoking `batch_hash`, the frontend ensures all files have `evidence_files` records in `.ffxdb` via `dbSync.upsertEvidenceFile()`. This prevents `FOREIGN KEY constraint failed` errors when `persistHashToDb` inserts into the `hashes` table (which has FK `file_id → evidence_files(id)`).

---

## Container Routing (3-Arm)

Inside each `spawn_blocking` task, the hash function is selected based on container format:

```text
            ┌── is_ewf_type? ──→ ewf::verify_with_progress()
            │                     (pure-Rust EWF parser)
file path ──┤
            ├── is_ad1_type? ──→ ad1::hash_segments_with_progress()
            │                     (AD1 segment hashing)
            │
            └── else ──────────→ raw::verify_with_progress()
                                  (byte-level sequential read)
```

| Arm | Formats | Method | Notes |
|-----|---------|--------|-------|
| **EWF** | .E01, .Ex01, .L01 | `ewf::verify_with_progress` | Reads EWF segments, decompresses chunks |
| **AD1** | .AD1 | `ad1::hash_segments_with_progress` | Hashes AD1 segment data |
| **Raw** | .dd, .raw, .img, .mem, .bin, everything else | `raw::verify_with_progress` | Sequential byte read with chunked updates |

All three arms accept a `progress_cb` closure for reporting bytes-processed/total.

---

## Hash Cache

### Purpose

Avoid re-hashing the same file when:
- The user switches tabs and comes back
- The same file is part of multiple batch operations
- A file was hashed in a previous session (within the app lifetime)

### Cache Key

```rust
struct HashCacheKey {
    path: PathBuf,      // Canonical (resolved symlinks)
    algorithm: String,  // Lowercase: "md5", "sha-256", etc.
    modified: SystemTime,
    size: u64,
}
```

The key includes **mtime + file size** as change-detection signals. If the file's metadata changes, the cached hash is automatically invalid because the key won't match.

### Cache Entry

```rust
struct HashCacheEntry {
    hash: String,
    cached_at: SystemTime,
    access_count: u64,
}
```

### Cache Behavior

| Operation | Description |
|-----------|-------------|
| **Lookup** | `get(key)` → returns hash string if found, increments `access_count` |
| **Insert** | `insert(key, hash)` → stores entry; if at capacity, evicts LRU entry first |
| **Invalidate** | `invalidate_path(path)` → removes ALL algorithm entries for a path |
| **Eviction** | LRU by `access_count` (lowest count evicted first) |
| **Capacity** | 1000 entries maximum |

### Thread Safety

`HashCache` uses `parking_lot::RwLock<HashMap>` for concurrent access. The global singleton is `GLOBAL_HASH_CACHE: LazyLock<HashCache>`.

### Integration Point

Inside `batch_hash`'s `spawn_blocking` closure:

```text
1. make_key(path, algorithm) → HashCacheKey
2. get_cached_hash(key)
   → Some(hash): skip computation, return immediately
   → None: proceed to 3-arm routing
3. Compute hash via container-specific function
4. cache_hash(key, computed_hash)
5. Return hash
```

### Convenience Functions

| Function | Purpose |
|----------|---------|
| `get_cached_hash(path, algorithm)` | Read-through: build key from file metadata, check cache |
| `cache_hash(path, algorithm, hash)` | Write-through: build key, insert into cache |
| `get_or_compute_hash(path, algorithm, compute_fn)` | Full cycle: check → compute → cache → return |

---

## Queue Management

The hash queue supports pause/resume for user-controlled scheduling:

| Command | Mechanism | Effect |
|---------|-----------|--------|
| `hash_queue_pause` | Sets `QUEUE_PAUSED` AtomicBool | Running tasks finish; queued tasks spin-wait |
| `hash_queue_resume` | Clears `QUEUE_PAUSED` | Waiting tasks proceed |
| `hash_queue_clear_completed` | Resets `QUEUE_METRICS` counters | Clears stats display |

### Queue Metrics

`QUEUE_METRICS` tracks aggregate statistics via atomic counters:

```rust
struct QueueMetrics {
    jobs_submitted: AtomicU64,
    jobs_completed: AtomicU64,
    jobs_failed: AtomicU64,
    bytes_processed: AtomicU64,
    processing_time_ms: AtomicU64,
}
```

---

## Progress Reporting

### Progress Reporter Thread

Each file gets a dedicated `spawn_progress_reporter()` thread that:

1. **Emits 0% immediately** (before the hash computation starts) — ensures the UI shows activity within milliseconds
2. **Polls every 250ms** with 0.5% granularity — avoids apparent stalls for large containers
3. **Heartbeat every 3 seconds** (1 second during startup while file handle opens) — proves the operation is alive during slow I/O

### Frontend Progress Buffering

To avoid reactive overhead from rapid event streams:

1. Progress events arrive via `listen("batch-progress")`
2. Events are buffered in a `pendingProgress` Map
3. A `setTimeout` flushes buffered events to the reactive UI **at most every 200ms**
4. Terminal events (`"completed"` / `"error"`) are processed immediately

### Safety Net

After `invoke("batch_hash")` returns, any files missing terminal events are marked as errors. This catches silent backend failures.

---

## Stored Hash Collection & Verification

### Stored Hashes

Container formats may embed hashes in their headers:

| Format | Hash Source | Algorithms |
|--------|-----------|------------|
| **E01/EWF** | EWF header `hash` section | MD5, SHA-1 |
| **AD1** | AD1 segment table | MD5, SHA-256 |
| **L01** | ltree `hash` sections | MD5, SHA-1 |

### Collection

`collectStoredHashes()` (in `hashUtils.ts`) gathers all vendor-embedded hashes from the container's metadata. This is the **single source of truth** for stored hash data.

### Verification

`determineVerification()` (in `hashUtils.ts`) compares a computed hash against stored hashes:

1. Look up stored hashes for the same algorithm
2. If found → compare (case-insensitive hex) → `"match"` or `"mismatch"`
3. If no stored hash for this algorithm → `"no_reference"`

### Persistence

After successful hash computation:

```text
handleHashCompleted(file, algorithm, computedHash)
  → determineVerification(storedHashes, algorithm, computedHash)
  → dbSync.insertHash({ file_id, algorithm, hash_value, source: "computed" })
  → dbSync.insertVerification({ hash_id, result, expected_hash, actual_hash })
  → dbSync.insertActivity({ action: "hash_computed" | "hash_verified", ... })
```

---

## Frontend Integration

### Hooks

| Hook | File | Purpose |
|------|------|---------|
| `useHashManager` | `src/hooks/useHashManager.ts` | Algorithm selection, single/batch/all-file hashing |
| `useHashComputation` | `src/hooks/useHashComputation.ts` | Core computation logic, progress tracking, completion handling |

### Key Functions

| Function | Description |
|----------|-------------|
| `hashSingleFile(file, algorithm)` | Hash one file (no batch, direct invoke) |
| `hashSelectedFiles(files, algorithm)` | Batch hash selected files (upsert evidence records → invoke batch_hash) |
| `hashAllFiles(files, algorithm)` | Hash all discovered files |
| `handleHashCompleted(file, algo, hash)` | Post-hash: verify + audit + persist |
| `persistHashToDb(file, algo, hash)` | DB write (insertHash + insertVerification) |
| `collectStoredHashes(file, infoMap)` | Gather vendor-embedded hashes |
| `determineVerification(stored, algo, hash)` | Compare against stored reference |

### Resource Budget (Approximate)

| Scenario | File Descriptors | Memory (Buffers) | Threads |
|----------|-----------------|-------------------|---------|
| 2 concurrent (USB HDD) | 32 (2 × 16) | ~128 MB | 6 |
| 6 concurrent (Internal SSD) | 96 (6 × 16) | ~384 MB | 18 |

---

## Key Files

### Backend (Rust)

| File | Purpose |
|------|---------|
| `src-tauri/src/commands/hash.rs` | `batch_hash`, `StorageClass`, per-drive semaphores, 3-arm routing, queue management |
| `src-tauri/src/common/hash_cache.rs` | `HashCache`, LRU eviction, `GLOBAL_HASH_CACHE`, convenience functions |
| `src-tauri/src/ewf/operations.rs` | `verify_with_progress` for E01/L01 |
| `src-tauri/src/ad1/mod.rs` | `hash_segments_with_progress` for AD1 |
| `src-tauri/src/commands/raw.rs` | `verify_with_progress` for raw images |

### Frontend (TypeScript)

| File | Purpose |
|------|---------|
| `src/hooks/useHashManager.ts` | Public API: algorithm, hash operations |
| `src/hooks/useHashComputation.ts` | Core logic: progress buffering, completion handling, DB persistence |
| `src/hooks/useHashUtils.ts` | `collectStoredHashes()`, `determineVerification()` |
| `src/types/hash.ts` | `StoredHash`, hash-related types |

---

*Last updated: June 2025*
