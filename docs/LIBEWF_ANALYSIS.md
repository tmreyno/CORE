# libewf Analysis - Rust/TSX Structure Recommendations

## Overview

This document analyzes the libewf C library architecture to identify patterns and concepts 
that can be applied to the CORE-FFX Rust/TSX forensic container implementation.

## Status In CORE-FFX

CORE-FFX already implements a trait-based container abstraction in `src-tauri/src/containers/traits.rs`, including `EvidenceContainer` and `MountableContainer`. The mount interfaces are present but not yet wired to a filesystem layer.

## libewf Architecture Summary

### Core Components

#### 1. Handle System (`libewf_handle`)
The central abstraction for accessing EWF containers:

```c
struct libewf_internal_handle {
    libewf_io_handle_t *io_handle;           // I/O operations
    libewf_media_values_t *media_values;     // Media metadata
    libewf_segment_table_t *segment_table;   // Segment management
    libewf_chunk_table_t *chunk_table;       // Chunk-level access
    libewf_hash_sections_t *hash_sections;   // Hash storage
    libfvalue_table_t *header_values;        // Header metadata
    libfvalue_table_t *hash_values;          // Hash values
    libewf_single_files_t *single_files;     // Logical file tree (L01)
    libewf_read_io_handle_t *read_io_handle; // Read operations
    libewf_write_io_handle_t *write_io_handle; // Write operations
};
```

#### 2. File Entry System (`libewf_file_entry`)
For logical evidence containers (L01/Lx01):

```c
struct libewf_internal_file_entry {
    libewf_handle_t *handle;                  // Parent handle
    libewf_single_files_t *single_files;      // Files collection
    libcdata_tree_node_t *file_entry_tree_node; // Tree position
    libewf_lef_file_entry_t *lef_file_entry;  // Entry data
    off64_t offset;                           // Current read offset
};
```

#### 3. Mount System (ewftools)
For FUSE/Dokan filesystem mounting:

```c
struct mount_handle {
    mount_file_system_t *file_system;         // Mounted filesystem
    uint8_t input_format;                     // files | raw
    int maximum_number_of_open_handles;
};

struct mount_file_system {
    uint64_t mounted_timestamp;
    system_character_t *path_prefix;
    libewf_handle_t *ewf_handle;
};

struct mount_file_entry {
    mount_file_system_t *file_system;
    system_character_t *name;
    int type;  // UNKNOWN | FILE_ENTRY | HANDLE
    libewf_handle_t *ewf_handle;
    libewf_file_entry_t *ewf_file_entry;
};
```

#### 4. Tool Handles (ewftools)
Specialized handles for different operations:

- **info_handle** - Display container information
- **verification_handle** - Hash verification with MD5/SHA1/SHA256
- **export_handle** - Extract/convert to other formats
- **imaging_handle** - Create new containers

---

## Common Code Patterns Identified

### 1. Handle Pattern
All operations use a handle that encapsulates:
- File I/O state
- Metadata caching
- Progress tracking
- Error handling

**Rust Equivalent: Session struct with lifetime management**

### 2. Segment Table Pattern
Multi-segment containers managed through:
- Basename + numbering scheme
- Segment size limits
- File list with caching
- Cross-segment seeking

**Current Rust Implementation:** `common/segments.rs`

### 3. Value Table Pattern
Key-value metadata storage:
- Header values (case info, examiner, etc.)
- Hash values (MD5, SHA1, SHA256)
- Device information

**Rust Equivalent:** HashMap or struct with optional fields

### 4. File Entry Tree Pattern
For logical containers:
- Tree node structure with parent/child relationships
- Lazy loading of children
- Path-based and index-based access
- Attribute/permission groups

**Current Rust Implementation:** `TreeEntry` in each format module

### 5. Progress Callback Pattern
All long operations support:
```c
typedef void (*progress_callback)(size64_t bytes_read, size64_t bytes_total);
```

**Current Rust Implementation:** Progress callbacks via closures

---

## Recommended Unified Rust Structure

### Shared Traits (implemented)

```rust
// containers/traits.rs - Already exists
pub trait EvidenceContainer {
    fn format_info(&self) -> FormatInfo;
    fn detect(&self, path: &Path) -> Result<bool, ContainerError>;
    fn info(&self, path: &Path, include_tree: bool) -> Result<ContainerInfo, ContainerError>;
    fn verify(&self, path: &Path, algorithm: &str) -> Result<VerifyResult, ContainerError>;
    fn extract(&self, path: &Path, output: &Path) -> Result<(), ContainerError>;
}

pub trait SegmentedContainer: EvidenceContainer {
    fn segment_info(&self, path: &Path) -> Result<SegmentInfo, ContainerError>;
    fn discover_segments(&self, path: &Path) -> Result<Vec<PathBuf>, ContainerError>;
}

pub trait TreeContainer: EvidenceContainer {
    fn list_entries(&self, path: &Path) -> Result<Vec<TreeEntryInfo>, ContainerError>;
    fn entry_info(&self, container_path: &Path, entry_path: &str) -> Result<TreeEntryInfo, ContainerError>;
    fn read_entry(&self, container_path: &Path, entry_path: &str) -> Result<Vec<u8>, ContainerError>;
}
```

### MountableContainer Trait (implemented, not yet integrated)

```rust
/// Trait for containers that can be mounted as virtual filesystems
pub trait MountableContainer: TreeContainer {
    /// Create a mount handle for virtual filesystem access
    fn create_mount(&self, path: &Path) -> Result<Box<dyn MountHandle>, ContainerError>;
}

/// Mount handle for virtual filesystem operations
pub trait MountHandle: Send + Sync {
    /// Get file attributes (stat)
    fn getattr(&self, path: &str) -> Result<FileAttr, ContainerError>;
    
    /// Read directory contents
    fn readdir(&self, path: &str) -> Result<Vec<DirEntry>, ContainerError>;
    
    /// Read file data
    fn read(&self, path: &str, offset: u64, size: usize) -> Result<Vec<u8>, ContainerError>;
    
    /// Check if path is a file or directory
    fn is_dir(&self, path: &str) -> Result<bool, ContainerError>;
}

/// File attributes (similar to struct stat)
#[derive(Debug, Clone)]
pub struct FileAttr {
    pub size: u64,
    pub is_directory: bool,
    pub created: Option<i64>,
    pub modified: Option<i64>,
    pub accessed: Option<i64>,
    pub permissions: u16,
}

/// Directory entry
#[derive(Debug, Clone)]
pub struct DirEntry {
    pub name: String,
    pub is_directory: bool,
}
```

### NEW: Session Handle Pattern

```rust
/// Common session handle for stateful container access
/// Inspired by libewf_handle pattern
pub struct ContainerSession<T> {
    /// Container-specific handle
    handle: T,
    /// Path to primary segment
    path: PathBuf,
    /// Cached segment information
    segments: Option<SegmentInfo>,
    /// Cached metadata
    metadata: Option<ContainerMetadata>,
    /// Read position (for streaming)
    position: u64,
    /// Abort flag
    abort: AtomicBool,
}

impl<T> ContainerSession<T> {
    pub fn signal_abort(&self) {
        self.abort.store(true, Ordering::SeqCst);
    }
    
    pub fn is_aborted(&self) -> bool {
        self.abort.load(Ordering::SeqCst)
    }
}
```

### NEW: Streaming Reader Pattern

```rust
/// Streaming reader for large container data
/// Supports cross-segment reading (like libewf)
pub struct ContainerReader {
    session: Arc<ContainerSession>,
    position: u64,
    buffer: Vec<u8>,
    buffer_offset: u64,
}

impl std::io::Read for ContainerReader {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        // Handle cross-segment boundaries
        // Use session's segment table for navigation
    }
}

impl std::io::Seek for ContainerReader {
    fn seek(&mut self, pos: std::io::SeekFrom) -> std::io::Result<u64> {
        // Calculate position across segments
    }
}
```

---

## TSX Component Structure

Based on libewf's info/export/verify tools, the UI should have:

### Container Viewer Components

```tsx
// Components for unified container viewing

interface ContainerViewerProps {
    path: string;
    format: ContainerFormat;
}

// Main viewer
export const ContainerViewer: React.FC<ContainerViewerProps> = ({ path, format }) => {
    return (
        <div>
            <ContainerHeader path={path} format={format} />
            <Tabs>
                <Tab label="Info"><ContainerInfo path={path} /></Tab>
                <Tab label="Tree"><ContainerTree path={path} /></Tab>
                <Tab label="Verify"><ContainerVerify path={path} /></Tab>
                <Tab label="Hex"><ContainerHex path={path} /></Tab>
            </Tabs>
        </div>
    );
};

// Unified tree component (like ewfmount directory listing)
interface TreeViewProps {
    path: string;
    onSelect: (entryPath: string) => void;
    lazyLoad?: boolean;
}

export const ContainerTree: React.FC<TreeViewProps> = ({ path, onSelect, lazyLoad }) => {
    // Uses get_children() for lazy loading
    // Unified across AD1, L01, UFED, archives
};

// Verification panel (like ewfverify)
interface VerifyPanelProps {
    path: string;
    algorithms: string[];
}

export const ContainerVerify: React.FC<VerifyPanelProps> = ({ path, algorithms }) => {
    // Progress tracking
    // Stored vs computed hash comparison
    // Per-chunk verification status
};
```

### Shared Type Definitions

```tsx
// types.ts - Unified container types

export interface ContainerInfo {
    format: string;
    path: string;
    segments?: SegmentInfo;
    metadata?: ContainerMetadata;
    tree?: TreeEntry[];
}

export interface SegmentInfo {
    count: number;
    files: string[];
    sizes: number[];
    totalSize: number;
    missing: string[];
    isComplete: boolean;
}

export interface TreeEntry {
    path: string;
    name: string;
    isDirectory: boolean;
    size: number;
    created?: string;
    modified?: string;
    accessed?: string;
    hash?: {
        md5?: string;
        sha1?: string;
        sha256?: string;
    };
    // For lazy loading
    hasChildren: boolean;
    childrenLoaded: boolean;
}

export interface VerifyResult {
    status: 'verified' | 'computed' | 'mismatch' | 'error';
    hashes: HashResult[];
    chunks: ChunkResult[];
    messages: string[];
}

export interface HashResult {
    algorithm: string;
    computed: string;
    stored?: string;
    verified?: boolean;
    durationSecs: number;
}
```

---

## Common Code to Extract/Share

### 1. Segment Discovery (already in common/)
```rust
// common/segments.rs - Already implemented
pub fn discover_e01_segments(path: &str) -> (Vec<PathBuf>, Vec<u64>, Vec<String>);
pub fn discover_ad1_segments(path: &str, count: u32) -> (Vec<PathBuf>, Vec<u64>, Vec<String>);
pub fn discover_numbered_segments(path: &str) -> (Vec<PathBuf>, Vec<u64>, Vec<String>);
```

### 2. Hash Computation (already in common/)
```rust
// common/hash.rs - Already implemented
pub struct StreamingHasher { /* ... */ }
pub fn compute_hash(data: &[u8], algorithm: HashAlgorithm) -> String;
```

### 3. Companion Log Parsing (already in containers/)
```rust
// containers/companion.rs - Already implemented
pub fn find_companion_log(path: &str) -> Option<CompanionLogInfo>;
```

### 4. NEW: Cross-Segment Reader
```rust
// common/reader.rs - NEW
/// Read data that may span multiple segment files
pub struct SegmentedReader {
    segments: Vec<SegmentFile>,
    current_segment: usize,
    position: u64,
}

impl SegmentedReader {
    pub fn read_at(&mut self, offset: u64, buf: &mut [u8]) -> io::Result<usize> {
        // Calculate which segment contains offset
        // Handle boundary crossing
    }
}
```

### 5. NEW: Progress Tracker
```rust
// common/progress.rs - NEW
/// Unified progress tracking (like libewf process_status)
pub struct ProgressTracker {
    total: u64,
    current: AtomicU64,
    start_time: Instant,
    callback: Option<Box<dyn Fn(u64, u64) + Send + Sync>>,
}

impl ProgressTracker {
    pub fn update(&self, bytes: u64) {
        self.current.fetch_add(bytes, Ordering::SeqCst);
        if let Some(cb) = &self.callback {
            cb(self.current.load(Ordering::SeqCst), self.total);
        }
    }
    
    pub fn elapsed(&self) -> Duration {
        self.start_time.elapsed()
    }
    
    pub fn rate(&self) -> f64 {
        let current = self.current.load(Ordering::SeqCst);
        current as f64 / self.elapsed().as_secs_f64()
    }
}
```

---

## Implementation Priority

### Phase 1: Enhance Common Infrastructure
1. Add `common/reader.rs` - Cross-segment reader
2. Add `common/progress.rs` - Progress tracking
3. Enhance `common/segments.rs` - Add L01 patterns

### Phase 2: Unify Container Traits
1. Add `MountableContainer` trait
2. Add `MountHandle` trait for virtual FS
3. Standardize `TreeEntry` across formats

### Phase 3: TSX Components
1. Create unified `ContainerViewer` component
2. Create lazy-loading `ContainerTree` component
3. Create `VerifyPanel` with progress

### Phase 4: Format-Specific Enhancements
1. E01 - Add L01 (logical) support
2. AD1 - Already well-implemented
3. Archive - Enhance with virtual mount
4. UFED - Add tree navigation

---

## libewf Concepts NOT Needed in Rust

1. **Manual memory management** - Rust handles this
2. **Error number system** - Use Result<T, E> instead
3. **Type prefixes** (libewf_, uint8_t) - Rust has native types
4. **Separate UTF-8/UTF-16 functions** - Use Rust strings
5. **Thread locks in every struct** - Use Rust's ownership

---

## Summary

The libewf library provides a well-structured approach to forensic container handling:

1. **Handle-based architecture** - Session management for stateful operations
2. **Segment abstraction** - Unified multi-file container access
3. **File entry tree** - Logical container navigation
4. **Mount system** - Virtual filesystem presentation
5. **Tool separation** - info/verify/export/mount as distinct operations

The CORE-FFX Rust implementation already follows many of these patterns. The key additions would be:
- `MountableContainer` trait for virtual filesystem operations
- `SegmentedReader` for cross-segment data access
- Unified `ProgressTracker` for all operations
- Standard `MountHandle` abstraction for FUSE-like access
