# Common Utilities Module

Shared backend utilities used across CORE-FFX.

## Files

### Core Utilities

| File | Purpose |
|------|---------|
| `mod.rs` | Module exports |
| `hash.rs` | Hashing helpers (MD5, SHA-1, SHA-256, SHA-512, BLAKE2b, BLAKE3, XXH3, XXH64, CRC32) |
| `hex.rs` | Hex formatting |
| `binary.rs` | Little-endian binary readers |
| `entropy.rs` | Shannon entropy calculation |
| `magic.rs` | Magic-byte file type detection |
| `datetime.rs` | Date/time parsing and formatting utilities |
| `vfs.rs` | Virtual filesystem helpers (normalize paths, DirEntry, FileAttr) |
| `segments.rs` | Multi-segment file discovery (.E01/.E02, .ad1/.ad2, .001/.002) |
| `segment_hash.rs` | Segment-level hash computation |

### Path & Security

| File | Purpose |
|------|---------|
| `path_security.rs` | Path traversal sanitization and validation |
| `audit.rs` | Audit logging for evidence access |
| `container_detect.rs` | Container format detection from file extension/magic |

### I/O & Performance

| File | Purpose |
|------|---------|
| `io_adaptive.rs` | Adaptive I/O buffer sizing (512KB–32MB based on file size and operation type) |
| `io_pool.rs` | File handle pooling for concurrent I/O |
| `hash_cache.rs` | In-memory hash result caching |
| `hash_queue.rs` | Priority-based hash job queue with adaptive concurrency |
| `index_cache.rs` | SQLite-backed container index cache for fast repeated access |
| `index_worker.rs` | Background worker for automatic index building |
| `lazy_loading.rs` | Lazy loading utilities for container tree nodes |

### Monitoring & Observability

| File | Purpose |
|------|---------|
| `health.rs` | System health monitoring (CPU, memory, disk, queue, errors) with `QueueMetricsRegistry` for lock-free concurrent metrics |
| `metrics.rs` | Prometheus-style metrics registry (counters, gauges, histograms) |
| `profiler.rs` | CPU profiling instrumentation |
| `memory_profiler.rs` | Memory usage profiling |
| `progress.rs` | Progress tracking for long operations |
| `notifications.rs` | Notification helpers |
| `regression.rs` | Performance regression detection |

### Resilience

| File | Purpose |
|------|---------|
| `retry.rs` | Retry logic with exponential backoff |
| `recovery.rs` | Recovery utilities for failed operations |

### Filesystem Drivers (`filesystem/`)

Read-only filesystem drivers for disk image analysis:

| File | Purpose |
|------|---------|
| `mod.rs` | Driver module exports and filesystem detection |
| `traits.rs` | `FilesystemDriver`, `SeekableBlockDevice` traits |
| `partition.rs` | GPT/MBR partition table parsing |
| `apfs_driver.rs` | Apple APFS filesystem (directory listing + file extent reading) |
| `hfsplus_driver.rs` | HFS+ filesystem driver |
| `ntfs_driver.rs` | NTFS filesystem driver |
| `ext_driver.rs` | ext2/ext3/ext4 filesystem driver |
| `fat.rs` | FAT12/FAT16/FAT32 filesystem driver |
| `dmg_driver.rs` | macOS DMG container (with encrypted DMG detection) |

## Key Patterns

### Hash Queue (`hash_queue.rs`)

Priority-based scheduling for parallel batch hashing:

- **Quick** (< 100MB) → **Normal** (100MB–1GB) → **Low** (1–10GB) → **Background** (> 10GB)
- Adaptive concurrency: 1.5× CPU cores for I/O-bound workloads
- Real-time throughput tracking (MB/s) and ETA prediction

### Health Monitoring (`health.rs`)

- `QueueMetricsRegistry`: lock-free atomic metrics (jobs submitted/completed/failed, throughput)
- `collect_error_metrics()`: extracts per-type error breakdown from Prometheus-format metric keys
- Alerts for high queue depth (>100), stalled queues, and elevated error rates

### Adaptive I/O (`io_adaptive.rs`)

Dynamic buffer sizing based on file size and operation type:

- Hash operations: maximize throughput (larger buffers)
- Read operations: balance latency/throughput
- Stream operations: minimize latency (smaller buffers)

## Security

- Sanitize all path input before writing
- Avoid mutating evidence source files
- Emit audit entries for evidence access
