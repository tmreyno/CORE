# AD1 V2 Quick Reference

## 🚀 Quick Start

```typescript
import { useAd1ContainerV2 } from '../hooks/useAd1ContainerV2';

const container = useAd1ContainerV2('/path/to/evidence.ad1');

// Get root files
const files = await container.getRootChildren();

// Verify all files
const results = await container.verifyAll('md5');

// Extract everything
const extraction = await container.extractAll('/output', true, true);
```

## 📋 Tauri Commands

| Command | Purpose | Returns |
|---------|---------|---------|
| `container_get_root_children_v2` | Get root-level children | `TreeEntryV2[]` |
| `container_get_children_at_addr_v2` | Load child nodes lazily | `TreeEntryV2[]` |
| `container_read_file_data_v2` | Read file contents | `number[]` (bytes) |
| `container_get_item_info_v2` | Get item metadata | `TreeEntryV2` |
| `container_verify_item_hash_v2` | Verify single file | `boolean` |
| `container_verify_all_v2` | Verify all files | `ItemVerifyResult[]` |
| `container_get_info_v2` | Get container info | `Ad1InfoV2` |
| `container_extract_all_v2` | Extract all files | `ExtractionResult` |
| `container_extract_item_v2` | Extract single file | `void` |

## 🔧 Rust API

### Open Container
```rust
use crate::ad1::reader_v2::SessionV2;

let session = SessionV2::open(Path::new("/path/to/evidence.ad1"))?;
```

### Get Root Children
```rust
use crate::ad1::operations_v2::get_root_children_v2;

let children = get_root_children_v2("/path/to/evidence.ad1").await?;
```

### Verify Files
```rust
use crate::ad1::hash_v2::{verify_all_items, HashType};

let results = verify_all_items(&session, HashType::Md5).await?;
```

### Extract All
```rust
use crate::ad1::extract_v2::{extract_all, ExtractOptions};

let options = ExtractOptions {
    apply_metadata: true,
    verify_hashes: true,
};
let result = extract_all(&session, Path::new("/output"), &options).await?;
```

## 📊 Data Types

### TreeEntryV2
```typescript
interface TreeEntryV2 {
  name: string;
  path: string;
  is_dir: boolean;
  size: number;
  item_type: number;
  first_child_addr?: number | null;
  data_addr?: number | null;
  item_addr?: number | null;
  md5_hash?: string | null;
  sha1_hash?: string | null;
  created?: string | null;
  accessed?: string | null;
  modified?: string | null;
}
```

### ItemVerifyResult
```typescript
interface ItemVerifyResult {
  path: string;
  name: string;
  is_dir: boolean;
  size: number;
  hash_type: 'Md5' | 'Sha1';
  result: 'Ok' | 'Mismatch' | 'NotFound' | { Error: string };
  stored_hash?: string;
  computed_hash?: string;
}
```

### ExtractionResult
```typescript
interface ExtractionResult {
  total_files: number;
  total_dirs: number;
  total_bytes: number;
  failed: string[];
  verified: number;
  verification_failed: string[];
}
```

### Ad1InfoV2
```typescript
interface Ad1InfoV2 {
  segment_header: SegmentHeaderInfo;
  logical_header: LogicalHeaderInfo;
  total_items: number;
  total_size: number;
  file_count: number;
  dir_count: number;
  tree?: TreeItem[];
}
```

## 🎨 UI Components

### EvidenceTreeV2
```tsx
<EvidenceTreeV2
  containerPath="/path/to/evidence.ad1"
  onFileSelect={(entry) => console.log(entry)}
  onError={(error) => console.error(error)}
/>
```

### Ad1OperationsV2
```tsx
<Ad1OperationsV2 />
```

## 💡 Common Patterns

### Load and Display Tree
```typescript
const container = useAd1ContainerV2(path);
const [nodes, setNodes] = createSignal([]);

async function loadRoot() {
  const root = await container.getRootChildren();
  setNodes(root);
}

async function expandNode(addr: number, path: string) {
  const children = await container.getChildrenAtAddr(addr, path);
  // Update UI with children
}
```

### Verify with Progress
```typescript
const container = useAd1ContainerV2(path);
const [progress, setProgress] = createSignal('');

async function verify() {
  setProgress('Starting verification...');
  const results = await container.verifyAll('md5');
  
  const passed = results.filter(r => r.result === 'Ok').length;
  const failed = results.filter(r => r.result === 'Mismatch').length;
  
  setProgress(`Complete: ${passed} passed, ${failed} failed`);
}
```

### Extract with Options
```typescript
const container = useAd1ContainerV2(path);

async function extract(outputDir: string) {
  const result = await container.extractAll(
    outputDir,
    true,  // Apply metadata (timestamps, attributes)
    true   // Verify hashes during extraction
  );
  
  console.log(`Extracted ${result.total_files} files`);
  console.log(`Verified ${result.verified} files`);
  
  if (result.failed.length > 0) {
    console.error('Failed:', result.failed);
  }
}
```

## ⚡ Performance Tips

1. **Lazy Loading**: Don't call `getInfo(true)` unless you need the full tree
2. **Caching**: Frontend automatically caches loaded nodes
3. **Streaming**: Large file reads are chunked automatically
4. **Parallel**: Multiple containers can be opened simultaneously
5. **Memory**: Close containers when done to free resources

## 🐛 Error Handling

```typescript
const container = useAd1ContainerV2(path);

try {
  const result = await container.verifyAll('md5');
  console.log('Success:', result);
} catch (e) {
  console.error('Error:', e);
  // container.error() signal contains error message
}

// Check loading state
if (container.isLoading()) {
  return <div>Loading...</div>;
}

// Check error state
if (container.error()) {
  return <div>Error: {container.error()}</div>;
}
```

## 📖 Documentation

- **Technical Details**: `docs/AD1_V2_IMPLEMENTATION.md`
- **Complete Guide**: `docs/AD1_V2_COMPLETE_GUIDE.md`
- **Project Summary**: `docs/AD1_V2_PROJECT_SUMMARY.md`

## 🔗 Related Files

### Rust
- `src-tauri/src/ad1/reader_v2.rs` - Core reader
- `src-tauri/src/ad1/operations_v2.rs` - Tree operations
- `src-tauri/src/ad1/hash_v2.rs` - Hash verification
- `src-tauri/src/ad1/extract_v2.rs` - File extraction
- `src-tauri/src/ad1/info_v2.rs` - Container info

### TypeScript
- `src/hooks/useAd1ContainerV2.ts` - React/Solid hook
- `src/components/EvidenceTreeV2.tsx` - Tree UI
- `src/components/Ad1OperationsV2.tsx` - Operations UI

## ✅ Feature Checklist

- [x] Multi-segment containers
- [x] Lazy tree loading
- [x] File reading with decompression
- [x] MD5 verification
- [x] SHA1 verification
- [x] File extraction
- [x] Directory extraction
- [x] Metadata preservation (timestamps)
- [x] Metadata preservation (attributes)
- [x] Container info display
- [x] Progress tracking
- [x] Error handling
- [x] TypeScript integration
- [x] React/Solid components
- [ ] Encryption/decryption (partial - header only)
- [ ] FUSE mounting (N/A for Tauri)

## 📞 Support

Questions? Check the comprehensive guides:
1. `AD1_V2_IMPLEMENTATION.md` for technical details
2. `AD1_V2_COMPLETE_GUIDE.md` for usage examples
3. `AD1_V2_PROJECT_SUMMARY.md` for overview

---

**CORE-FFX - Forensic File Explorer**  
Based on libad1 by Maxim Suhanov  
Licensed under MIT
