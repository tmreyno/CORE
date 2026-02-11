# Archive Tools User Guide

**Date:** February 6, 2026  
**Version:** CORE-FFX v0.1.0

---

## 🎯 Overview

Archive Tools provides forensic-grade operations for 7z archives:

- **Test** - Verify archive integrity without extraction
- **Repair** - Recover data from corrupted archives
- **Validate** - Detailed validation with error context
- **Extract Split** - Extract multi-volume archives

---

## 📍 Access Archive Tools

### From Export Panel

1. Open **Export & Archive** panel
2. Click **Archive Tools** button in header
3. Select desired operation tab

---

## 🔍 Test Archive

**Purpose:** Verify archive integrity without extracting files

### Steps:

1. Click **Test** tab
2. Browse or paste archive path
3. Enter password (if encrypted)
4. Click **Test Archive**

### Results:

- ✅ **Pass** - Archive is intact and valid
- ❌ **Fail** - Archive has integrity issues

### Use Cases:

- Verify evidence archives before analysis
- Check archives after transfer
- Validate archives before court presentation
- Quick integrity check (faster than full extraction)

---

## 🔧 Repair Archive

**Purpose:** Recover data from damaged or incomplete archives

### Steps:

1. Click **Repair** tab
2. Select corrupted archive path
3. Specify output path for repaired archive
4. Click **Repair Archive**
5. Monitor progress bar

### Results:

- **Success:** Repaired archive saved to output path
- **Partial:** Some files recovered (check logs)
- **Failure:** Archive too damaged to repair

### Progress Indicators:

- Real-time progress percentage
- Status messages (e.g., "Processing headers", "Recovering data")
- Estimated time remaining

### Use Cases:

- Recover data from damaged evidence
- Fix incomplete downloads
- Repair archives with corrupted headers
- Salvage data from failing drives

---

## ✓ Validate Archive

**Purpose:** Thorough validation with detailed error information

### Steps:

1. Click **Validate** tab
2. Browse or paste archive path
3. Click **Validate Archive**
4. Review detailed results

### Results:

- ✅ **Valid** - Archive passes all checks
- ⚠️ **Issues** - Problems detected with details:
  - Error code and message
  - File context (which file caused error)
  - Byte position of error
  - Suggested remediation

### Detailed Error Information:

```
Code: 2
Message: CRC mismatch
Context: file.txt (offset 0x1234)
Position: 4660
Suggestion: Try repair_7z_archive to recover data
```

### Use Cases:

- Deep integrity verification
- Troubleshoot failed extractions
- Forensic validation before court
- Identify specific corruption points

---

## 📦 Extract Split Archive

**Purpose:** Extract multi-volume archives (.001, .002, etc.)

### Steps:

1. Click **Extract Split** tab
2. Select **first volume** (.001 or .7z.001)
3. Select output directory
4. Enter password (if encrypted)
5. Click **Extract Archive**
6. Monitor progress

### Progress Indicators:

- Current volume being processed
- Overall extraction percentage
- Files extracted count

### Requirements:

- All volumes must be in same directory
- First volume must be selected
- Sequential naming (.001, .002, .003...)

### Use Cases:

- Extract large disk images split for storage
- Process archives from cloud backup (split for upload limits)
- Handle evidence split across multiple media
- Extract archives created with split size option

---

## 🔐 Encryption Support

### Testing Encrypted Archives:

- Provide password in Test tab
- Invalid password = test failure
- No password = "encrypted" error

### Extracting Encrypted Split Archives:

- Password required for extraction
- Same password for all volumes
- AES-256 encryption supported

---

## 📊 Performance Characteristics

| Operation | Speed | Memory |
|-----------|-------|--------|
| **Test** | Fast (~100ms-10s) | Low (~50MB) |
| **Repair** | Medium (1-5 min) | Medium (~250MB) |
| **Validate** | Medium (~2x test) | Low (~50MB) |
| **Extract Split** | Fast (streaming) | Low (~250MB) |

---

## 🔥 Forensic Workflow Examples

### 1. Evidence Verification Workflow

```
1. Receive evidence archive from source
2. Test archive integrity (30 seconds)
3. If test passes → Validate for detailed check
4. If validation passes → Proceed with extraction
5. If validation fails → Attempt repair
```

### 2. Corrupted Archive Recovery

```
1. Test archive → Fails
2. Validate archive → Check error details
3. Repair archive → Create recovered version
4. Test repaired archive → Verify fix
5. Extract repaired archive → Recover data
```

### 3. Multi-Volume Evidence Processing

```
1. Verify all volumes present (.001-.010)
2. Test first volume for integrity
3. Extract split archive to working directory
4. Verify extraction with hash comparison
5. Begin forensic analysis
```

---

## ⚠️ Error Messages

### Common Errors:

| Error | Meaning | Solution |
|-------|---------|----------|
| "CRC mismatch" | Data corruption | Try Repair |
| "Encrypted" | Password required | Provide password |
| "Incomplete archive" | Missing volumes | Locate all volumes |
| "Unknown format" | Not 7z/ZIP | Check file format |
| "Header corrupted" | Damaged header | Try Repair |

### Detailed Error Context:

When validation fails, the tool provides:
- **Error Code:** Numeric error identifier
- **Message:** Human-readable description
- **File Context:** Which file in archive caused error
- **Position:** Byte offset of error
- **Suggestion:** Recommended next steps

---

## 🎯 Best Practices

### Testing:

- ✅ Always test archives after creation
- ✅ Test after file transfer
- ✅ Test before court presentation
- ✅ Use password-protected testing for encrypted archives

### Repairing:

- ✅ Create backup of corrupted archive first
- ✅ Save repaired archive with different name
- ✅ Test repaired archive before trusting it
- ✅ Document repair attempts in case notes

### Validating:

- ✅ Use validation before critical operations
- ✅ Save error details for documentation
- ✅ Run validation on evidence copies, not originals
- ✅ Cross-reference validation results with extraction logs

### Extracting Split:

- ✅ Verify all volumes present before starting
- ✅ Check free space in output directory
- ✅ Use same password for all volumes
- ✅ Don't move/rename volumes during extraction

---

## 🔒 Security Notes

1. **Password Protection:**
   - Passwords transmitted securely to backend
   - Not stored in memory after operation
   - AES-256 encryption (industry standard)

2. **Read-Only Operations:**
   - Test and Validate are non-destructive
   - Original archives never modified
   - Forensic integrity maintained

3. **Evidence Integrity:**
   - Operations designed for forensic use
   - All actions logged for chain-of-custody
   - Hash verification supported

---

## 🐛 Troubleshooting

### "Test failed but file extracts fine"

- Archive may have non-critical issues
- Use Validate for detailed diagnosis
- Compare extracted files with manifests

### "Repair produces smaller archive"

- Damaged data was unrecoverable
- Check repair logs for details
- Partial recovery is common with severe corruption

### "Split extract fails on volume 3"

- Check all volumes present
- Verify sequential naming
- Test each volume individually
- Check for file system corruption

### "Password works in 7-Zip but not here"

- Ensure correct password encoding
- Try copy-paste instead of typing
- Check for hidden characters
- Verify archive compatibility

---

## 📚 Related Documentation

- **Export Panel Guide:** Creating archives
- **Archive Creation API:** `docs/SEVENZIP_INTEGRATION_COMPLETE.md`
- **Backend Commands:** `src-tauri/src/commands/archive.rs`
- **TypeScript API:** `src/api/archiveCreate.ts`

---

## 🆘 Support

For issues with Archive Tools:

1. Check error messages and suggestions
2. Review validation results for details
3. Try alternative operations (e.g., Repair if Test fails)
4. Document steps taken for support requests

---

**Note:** Archive Tools uses the same sevenzip-ffi library as archive creation, ensuring compatibility with all 7z archives created by CORE-FFX.
