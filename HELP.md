# CORE-FFX Quick Help

A short, practical guide to the current UI and workflows.

## Getting Started

1) Open a directory (toolbar)
2) Scan for evidence
3) Load metadata ("Load All")
4) Select files to view metadata or hash results

## Toolbar Actions

- Open Directory: pick an evidence root
- Scan: scan the current path
- Recursive: include subfolders
- Hash: compute hashes for selected files
- Load All: fetch metadata for all discovered containers
- Save/Load: project file management
- Report: open the report wizard

## Evidence Formats (Primary)

- AD1
- E01/Ex01
- L01/Lx01
- Raw images (.dd, .raw, .img, .001)
- UFED (.ufd, .ufdr)
- Archives (ZIP/7z/RAR metadata; ZIP extraction)

## File List Navigation

- Arrow keys: move selection
- Enter: open active file/tab
- Space: toggle selection
- Home/End: jump to first/last
- Escape: clear filter

## Hashing Tips

- Choose algorithm from the dropdown (forensic vs fast)
- Hash matches are compared against stored hashes when available
- Incomplete containers will show a warning state

## Reports

Report Wizard steps:

1) Case Information
2) Evidence Selection
3) Findings
4) Preview
5) Export (PDF/DOCX/HTML/Markdown)

---

## Copy & Export Features

The Export Panel provides three modes for evidence transfer:

### Copy Mode
Direct file/folder copy with progress tracking:
- Shows current file being copied
- Displays speed and estimated time
- Preserves file timestamps
- No hashing overhead - fastest option

### Export Mode (Forensic)
Copy with full forensic documentation:
- **SHA-256 hash computed while copying** (shown in progress)
- Hash verification after copy
- JSON manifest file with:
  - Source and destination paths
  - File sizes and timestamps
  - SHA-256 hash for each file
  - Verification status
- Progress shows both copy and hash stages

### Archive Mode
Create 7z archives (see Archive Export Feature below).

### Progress Display

The Activity Panel shows detailed progress for all operations:

| Phase | What's Happening |
|-------|------------------|
| Calculating | Scanning files, counting total size |
| Copying + Hashing | Files being copied with inline hash computation |
| Verifying | Destination files being hash-verified |
| Complete | Operation finished with summary |

For forensic export, hashing happens **simultaneously with copying** (no separate pass), which is nearly as fast as plain copy while providing verification.

---

## Hashing Segmented Evidence Containers

### How Segment Hashing Works

When you hash an evidence container in the Evidence Tree, the system automatically handles segmented files:

| Container Type | Hashing Method |
|----------------|----------------|
| **AD1** (.ad1, .ad2, .ad3...) | Hashes all segment files sequentially to produce ONE hash |
| **E01** (.E01, .E02, .E03...) | Reads and decompresses all chunks, hashes uncompressed data |
| **L01** (.L01, .L02...) | Same as E01 - decompresses and hashes content |
| **Raw** (.001, .002...) | Hashes all segment files sequentially |

### AD1 Segment Hashing
- Reads each segment file (.ad1, .ad2, .ad3, etc.) in order
- Hashes the **raw segment bytes** (container-level hash)
- Produces a single hash representing the entire evidence container
- Compare against stored hashes from acquisition logs

### E01/L01 Segment Hashing  
- Opens all segment files (.E01, .E02, etc.)
- Decompresses zlib chunks across all segments
- Hashes the **uncompressed content** (image-level hash)
- Compares against embedded MD5/SHA1 stored in the container

### Progress Display for Segmented Containers

When hashing multi-segment containers:
- Progress shows total bytes across ALL segments
- Speed reflects actual read throughput
- Decompression progress shown for E01/L01 (chunks processed)

Example progress output:
```
Hashing: evidence.E01
Segment 1/5: 45% (chunks 234/512)
Speed: 180 MB/s
```

### Stored Hash Verification

Evidence containers may have embedded hashes from acquisition:

| Container | Stored Hashes |
|-----------|---------------|
| E01/L01 | MD5, SHA1 in header sections |
| AD1 | SHA1 in companion log files |

When stored hashes are present:
- Computed hash is compared automatically
- ✅ Green = Match (integrity verified)
- ❌ Red = Mismatch (possible tampering or corruption)
- ⚠️ Yellow = No stored hash available

---

## Archive Export Feature

The Archive Export feature creates forensic-grade 7z archives for secure evidence transfer and storage.

### Why 7z Format?

- **Industry Standard**: Compatible with 7-Zip, WinRAR, macOS Archive Utility, and all major tools
- **AES-256 Encryption**: Strong password protection for sensitive evidence
- **Split Volumes**: Break large archives into manageable pieces (DVD, USB, cloud upload sizes)
- **Integrity Verification**: Built-in CRC checks ensure data integrity
- **Cross-Platform**: Works on Windows, macOS, and Linux

### Compression Levels

| Level | Name | Speed | Best For |
|-------|------|-------|----------|
| **0** | **Store** | **~500+ MB/s** | **E01/AD1/L01 containers (already compressed)** |
| 1 | Fastest | ~180 MB/s | Text files, logs, uncompressed data |
| 3 | Fast | ~80 MB/s | Mixed content |
| 5 | Normal | ~22 MB/s | General purpose |
| 7 | Maximum | ~12 MB/s | Space-critical storage |
| 9 | Ultra | ~9 MB/s | Maximum compression (slow) |

### Recommended Settings for Forensics

**Default: Store (Level 0)** - This is intentional!

Forensic evidence containers are already internally compressed:

| Container | Internal Compression |
|-----------|---------------------|
| E01/EWF | zlib |
| AD1 | zlib |
| L01/Lx01 | zlib |
| AFF4 | zlib/lz4 |

Attempting to re-compress already-compressed data:
- ❌ Wastes CPU time (hours on large files)
- ❌ Produces the same output size
- ❌ No benefit whatsoever

Store mode packages files at disk I/O speed while still providing:
- ✅ Single archive file
- ✅ AES-256 encryption
- ✅ Split volumes
- ✅ Integrity checks

### When to Use Compression

Use **Fastest (Level 1)** or higher only for:
- Raw disk images (.dd, .raw) that aren't compressed
- Text files, logs, CSV exports
- Uncompressed document collections
- Memory dumps (.mem, .vmem)

### Encryption

Password protection uses AES-256-CBC encryption:
- Virtually zero performance overhead (<1%)
- Encrypts both file content and filenames
- Compatible with all 7z tools

**Best Practices:**
- Use strong passwords (12+ characters, mixed case, numbers, symbols)
- Document passwords securely in case notes
- Test extraction before deleting originals

### Split Archives

For large evidence, split volumes are enabled by default at **2 GB** - ideal for:
- Cloud upload limits (many services cap at 2-5 GB per file)
- USB flash drives (FAT32 has 4GB file limit)
- Email attachments (when zipped further)
- Easier re-transmission if transfer fails

Available split sizes:
- **700 MB**: CD-R size
- **2 GB**: Cloud/USB - **Default & Recommended**
- **4.7 GB**: DVD-R size
- **8.5 GB**: DVD DL size
- **25 GB**: Blu-ray size  
- **50 GB**: Blu-ray DL size
- **No Split**: Single file (for local storage)

Split archives create files like:
```
evidence.7z.001  (first volume)
evidence.7z.002  (second volume)
evidence.7z.003  (etc.)
```

To extract, open the `.001` file - 7z tools automatically read all volumes.

### Performance Expectations

| Evidence Size | Store Time | Fastest Time | Normal Time |
|---------------|------------|--------------|-------------|
| 10 GB | ~20 sec | ~55 sec | ~7.5 min |
| 50 GB | ~100 sec | ~4.5 min | ~38 min |
| 100 GB | ~3.5 min | ~9 min | ~75 min |
| 500 GB | ~17 min | ~46 min | ~6.3 hours |

*Times assume SSD storage. HDD will be disk I/O limited.*

### Activity Panel

While archiving, the right panel shows:
- Current file being processed
- Bytes processed / total
- Speed (MB/s)
- Estimated time remaining
- Progress percentage

### Verification

After archive creation:
1. Open the archive with 7-Zip or similar tool
2. Run "Test Archive" to verify integrity
3. Optionally extract to a test location and compare hashes

### Troubleshooting

**Archive creation is slow:**
- Check compression level (use Store for E01/AD1)
- Verify disk isn't the bottleneck (check Activity Monitor)

**Archive won't open:**
- Verify correct password
- Check all split volumes are present
- Ensure volumes weren't corrupted in transfer

**Out of memory on large archives:**
- Archives >8GB automatically use streaming mode
- Split volumes are auto-enabled for very large files

---

## Need More Detail?

- Application Guide: `APP_README.md`
- Code Directory: `CODE_BIBLE.md`
- Backend Docs: `src-tauri/src/README.md`
