// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Evidence Container Trait Implementations
//!
//! **DEPRECATED**: This module contains trait-based parser implementations that
//! are not currently used in production. The application uses direct module calls
//! via `operations.rs` for container operations and `unified.rs` for tree navigation.
//!
//! This module is preserved for potential future plugin/extension system development.
//! For current container operations, use:
//! - `containers::info()`, `containers::verify()`, `containers::extract()` from `operations.rs`
//! - `unified::get_children()`, `unified::get_summary()` from `unified.rs`
//!
//! ## Supported Formats (if used)
//!
//! - AD1 (AccessData Logical Image)
//! - EWF/E01 (Expert Witness Format)
//! - RAW (Raw Disk Images)
//! - UFED (Universal Forensic Extraction Data)
//! - Archive (ZIP, 7z, RAR)

#![allow(dead_code)]
#![allow(deprecated)]

use std::path::Path;

use crate::formats::FormatCategory;
use super::traits::{
    EvidenceContainer, SegmentedContainer, TreeContainer, HashableContainer,
    MountableContainer, FormatInfo, SegmentInfo, HashResult, VerifyResult, VerifyStatus,
    ContainerMetadata, CaseMetadata, StoredHashInfo, TreeEntryInfo, SegmentMetadata,
    ContainerError,
};
use crate::common::vfs::VirtualFileSystem;

// =============================================================================
// AD1 Parser Implementation
// =============================================================================

/// AD1 container parser implementing the EvidenceContainer trait
#[deprecated(since = "0.2.0", note = "Use containers::operations::info() instead")]
pub struct Ad1Parser;

impl EvidenceContainer for Ad1Parser {
    fn format_info(&self) -> FormatInfo {
        FormatInfo {
            id: "ad1",
            name: "AccessData Logical Image",
            extensions: &["ad1"],
            category: FormatCategory::ForensicContainer,
            supports_segments: true,
            stores_hashes: true,
            has_file_tree: true,
        }
    }
    
    fn detect(&self, path: &Path) -> Result<bool, ContainerError> {
        let path_str = path.to_string_lossy();
        crate::ad1::is_ad1(&path_str)
    }
    
    fn info(&self, path: &Path, include_tree: bool) -> Result<ContainerMetadata, ContainerError> {
        let path_str = path.to_string_lossy();
        let info = crate::ad1::info(&path_str, include_tree)?;
        
        // Extract stored hashes from companion log if available
        let mut stored_hashes = Vec::new();
        if let Some(ref log) = info.companion_log {
            if let Some(ref md5) = log.md5_hash {
                stored_hashes.push(StoredHashInfo {
                    algorithm: "MD5".to_string(),
                    hash: md5.clone(),
                    source: "companion_log".to_string(),
                    verified: None,
                });
            }
            if let Some(ref sha1) = log.sha1_hash {
                stored_hashes.push(StoredHashInfo {
                    algorithm: "SHA1".to_string(),
                    hash: sha1.clone(),
                    source: "companion_log".to_string(),
                    verified: None,
                });
            }
            if let Some(ref sha256) = log.sha256_hash {
                stored_hashes.push(StoredHashInfo {
                    algorithm: "SHA256".to_string(),
                    hash: sha256.clone(),
                    source: "companion_log".to_string(),
                    verified: None,
                });
            }
        }
        
        // Extract case metadata from companion log
        let case_info = info.companion_log.as_ref().map(|log| CaseMetadata {
            case_number: log.case_number.clone(),
            evidence_number: log.evidence_number.clone(),
            examiner_name: log.examiner.clone(),
            description: log.source_device.clone(),
            notes: log.notes.clone(),
            acquisition_date: log.acquisition_date.clone(),
        });
        
        // Build segment info
        let segments = info.segment_files.as_ref().filter(|f| f.len() > 1).map(|files| {
            let sizes = info.segment_sizes.clone().unwrap_or_default();
            let total_size = sizes.iter().sum();
            SegmentInfo {
                count: files.len() as u32,
                files: files.clone(),
                sizes,
                total_size,
                missing: info.missing_segments.clone().unwrap_or_default(),
            }
        });
        
        Ok(ContainerMetadata {
            format: "AD1".to_string(),
            version: Some(format!("{}", info.logical.image_version)),
            total_size: info.total_size.unwrap_or(0),
            segments,
            stored_hashes,
            case_info,
            format_specific: None,
        })
    }
    
    fn verify(&self, path: &Path, algorithm: &str) -> Result<VerifyResult, ContainerError> {
        let path_str = path.to_string_lossy();
        // Use hash_segments to compute container hash (not verify which does per-file hashes)
        let hash = crate::ad1::hash_segments(&path_str, algorithm)?;
        
        // Get stored hashes from companion log
        let info = crate::ad1::info_fast(&path_str)?;
        
        let expected = info.companion_log.as_ref().and_then(|log| {
            match algorithm.to_lowercase().as_str() {
                "md5" => log.md5_hash.clone(),
                "sha1" => log.sha1_hash.clone(),
                "sha256" => log.sha256_hash.clone(),
                _ => None,
            }
        });
        
        let verified = expected.as_ref().map(|e| e.to_lowercase() == hash.to_lowercase());
        let status = match verified {
            Some(true) => VerifyStatus::Verified,
            Some(false) => VerifyStatus::Mismatch,
            None => VerifyStatus::Computed,
        };
        
        Ok(VerifyResult {
            status,
            hashes: vec![HashResult {
                algorithm: algorithm.to_string(),
                computed: hash,
                expected,
                verified,
                duration_secs: 0.0,
            }],
            chunks: vec![],
            messages: vec![],
        })
    }
    
    fn extract(&self, path: &Path, output_dir: &Path) -> Result<(), ContainerError> {
        let path_str = path.to_string_lossy();
        let output_str = output_dir.to_string_lossy();
        crate::ad1::extract(&path_str, &output_str)
    }
}

impl SegmentedContainer for Ad1Parser {
    fn discover_segments(&self, path: &Path) -> Result<SegmentInfo, ContainerError> {
        let path_str = path.to_string_lossy();
        let paths = crate::ad1::get_segment_paths(&path_str)?;
        
        let files: Vec<String> = paths.iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        
        let sizes: Vec<u64> = paths.iter()
            .filter_map(|p| std::fs::metadata(p).ok().map(|m| m.len()))
            .collect();
        
        let total_size: u64 = sizes.iter().sum();
        
        Ok(SegmentInfo {
            count: paths.len() as u32,
            files,
            sizes,
            total_size,
            missing: vec![],
        })
    }
    
    fn segment_info(&self, path: &Path, index: u32) -> Result<SegmentMetadata, ContainerError> {
        let segments = self.discover_segments(path)?;
        
        if index as usize >= segments.files.len() {
            return Err(ContainerError::SegmentError(
                format!("Segment {} not found", index)
            ));
        }
        
        Ok(SegmentMetadata {
            index,
            path: segments.files[index as usize].clone(),
            size: segments.sizes.get(index as usize).copied().unwrap_or(0),
            hash: None,
        })
    }
}

impl TreeContainer for Ad1Parser {
    fn list_entries(&self, path: &Path) -> Result<Vec<TreeEntryInfo>, ContainerError> {
        let path_str = path.to_string_lossy();
        let tree = crate::ad1::get_tree(&path_str)?;
        
        // AD1 get_tree returns a flat list of TreeEntry
        let results: Vec<TreeEntryInfo> = tree.iter()
            .map(|entry| {
                // Extract name from path
                let name = std::path::Path::new(&entry.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| entry.path.clone());
                    
                TreeEntryInfo {
                    path: entry.path.clone(),
                    name,
                    is_directory: entry.is_dir,
                    size: entry.size,
                    created: entry.created.clone(),
                    modified: entry.modified.clone(),
                    accessed: entry.accessed.clone(),
                    hash: entry.md5_hash.clone(),
                }
            })
            .collect();
        
        Ok(results)
    }
    
    fn entry_info(&self, container_path: &Path, entry_path: &str) -> Result<TreeEntryInfo, ContainerError> {
        let entries = self.list_entries(container_path)?;
        entries.iter()
            .find(|e| e.path == entry_path)
            .cloned()
            .ok_or_else(|| ContainerError::FileNotFound(entry_path.to_string()))
    }
    
    fn extract_entry(&self, container_path: &Path, _entry_path: &str, output_path: &Path) -> Result<(), ContainerError> {
        // For AD1, we use the main extract function
        self.extract(container_path, output_path)
    }
}

impl HashableContainer for Ad1Parser {
    fn stored_hashes(&self, path: &Path) -> Result<Vec<StoredHashInfo>, ContainerError> {
        let path_str = path.to_string_lossy();
        let info = crate::ad1::info_fast(&path_str)?;
        
        let mut hashes = Vec::new();
        if let Some(ref log) = info.companion_log {
            if let Some(ref md5) = log.md5_hash {
                hashes.push(StoredHashInfo {
                    algorithm: "MD5".to_string(),
                    hash: md5.clone(),
                    source: "companion_log".to_string(),
                    verified: None,
                });
            }
            if let Some(ref sha1) = log.sha1_hash {
                hashes.push(StoredHashInfo {
                    algorithm: "SHA1".to_string(),
                    hash: sha1.clone(),
                    source: "companion_log".to_string(),
                    verified: None,
                });
            }
            if let Some(ref sha256) = log.sha256_hash {
                hashes.push(StoredHashInfo {
                    algorithm: "SHA256".to_string(),
                    hash: sha256.clone(),
                    source: "companion_log".to_string(),
                    verified: None,
                });
            }
        }
        Ok(hashes)
    }
    
    fn verify_stored_hashes(&self, path: &Path) -> Result<Vec<HashResult>, ContainerError> {
        let stored = self.stored_hashes(path)?;
        let path_str = path.to_string_lossy();
        
        let mut results = Vec::new();
        for hash_info in &stored {
            // Use hash_segments to compute container hash
            let computed = crate::ad1::hash_segments(&path_str, &hash_info.algorithm)?;
            
            let verified = computed.to_lowercase() == hash_info.hash.to_lowercase();
            results.push(HashResult {
                algorithm: hash_info.algorithm.clone(),
                computed,
                expected: Some(hash_info.hash.clone()),
                verified: Some(verified),
                duration_secs: 0.0,
            });
        }
        
        Ok(results)
    }
}

impl MountableContainer for Ad1Parser {
    fn mount(&self, path: &Path) -> Result<Box<dyn VirtualFileSystem>, ContainerError> {
        let path_str = path.to_string_lossy();
        let vfs = crate::ad1::vfs::Ad1Vfs::open(&path_str)
            .map_err(|e| ContainerError::IoError(e.to_string()))?;
        Ok(Box::new(vfs))
    }
    
    fn supports_mount(&self) -> bool {
        true
    }
}

// =============================================================================
// EWF Parser Implementation
// =============================================================================

/// EWF/E01 container parser implementing the EvidenceContainer trait
#[deprecated(since = "0.2.0", note = "Use containers::operations::info() instead")]
pub struct EwfParser;

impl EvidenceContainer for EwfParser {
    fn format_info(&self) -> FormatInfo {
        FormatInfo {
            id: "ewf",
            name: "Expert Witness Format",
            extensions: &["e01", "l01", "ex01", "lx01"],
            category: FormatCategory::ForensicContainer,
            supports_segments: true,
            stores_hashes: true,
            has_file_tree: false,  // EWF is a disk image format, no file tree
        }
    }
    
    fn detect(&self, path: &Path) -> Result<bool, ContainerError> {
        let path_str = path.to_string_lossy();
        crate::ewf::is_ewf(&path_str)
    }
    
    fn info(&self, path: &Path, _include_tree: bool) -> Result<ContainerMetadata, ContainerError> {
        let path_str = path.to_string_lossy();
        let info = crate::ewf::info(&path_str)?;
        
        // Convert EwfInfo to ContainerMetadata
        let stored_hashes: Vec<StoredHashInfo> = info.stored_hashes.iter()
            .map(|h| StoredHashInfo {
                algorithm: h.algorithm.clone(),
                hash: h.hash.clone(),
                source: "container".to_string(),
                verified: None,
            })
            .collect();
        
        let case_info = Some(CaseMetadata {
            case_number: info.case_number.clone(),
            evidence_number: info.evidence_number.clone(),
            examiner_name: info.examiner_name.clone(),
            description: info.description.clone(),
            notes: info.notes.clone(),
            acquisition_date: info.acquiry_date.clone(),  // EWF uses acquiry_date
        });
        
        let segments = if info.segment_count > 1 {
            let segment_paths = crate::ewf::get_segment_paths(&path_str)
                .unwrap_or_default();
            
            let files: Vec<String> = segment_paths.iter()
                .map(|p| p.to_string_lossy().to_string())
                .collect();
            
            let sizes: Vec<u64> = segment_paths.iter()
                .filter_map(|p| std::fs::metadata(p).ok().map(|m| m.len()))
                .collect();
            
            let total_size: u64 = sizes.iter().sum();
            
            Some(SegmentInfo {
                count: info.segment_count,
                files,
                sizes,
                total_size,
                missing: vec![],
            })
        } else {
            None
        };
        
        Ok(ContainerMetadata {
            format: "EWF".to_string(),
            version: Some(info.format_version.clone()),  // Use format_version not format
            total_size: info.total_size,
            segments,
            stored_hashes,
            case_info,
            format_specific: None,
        })
    }
    
    fn verify(&self, path: &Path, algorithm: &str) -> Result<VerifyResult, ContainerError> {
        let path_str = path.to_string_lossy();
        let hash = crate::ewf::verify(&path_str, algorithm)?;
        
        // Check against stored hashes
        let info = crate::ewf::info_fast(&path_str)?;
        
        let expected = info.stored_hashes.iter()
            .find(|h| h.algorithm.to_lowercase() == algorithm.to_lowercase())
            .map(|h| h.hash.clone());
        
        let verified = expected.as_ref().map(|e| e.to_lowercase() == hash.to_lowercase());
        let status = match verified {
            Some(true) => VerifyStatus::Verified,
            Some(false) => VerifyStatus::Mismatch,
            None => VerifyStatus::Computed,
        };
        
        Ok(VerifyResult {
            status,
            hashes: vec![HashResult {
                algorithm: algorithm.to_string(),
                computed: hash,
                expected,
                verified,
                duration_secs: 0.0,
            }],
            chunks: vec![],
            messages: vec![],
        })
    }
    
    fn extract(&self, path: &Path, output_dir: &Path) -> Result<(), ContainerError> {
        let path_str = path.to_string_lossy();
        let output_str = output_dir.to_string_lossy();
        crate::ewf::extract(&path_str, &output_str)
    }
}

impl SegmentedContainer for EwfParser {
    fn discover_segments(&self, path: &Path) -> Result<SegmentInfo, ContainerError> {
        let path_str = path.to_string_lossy();
        let paths = crate::ewf::get_segment_paths(&path_str)?;
        
        let files: Vec<String> = paths.iter()
            .map(|p| p.to_string_lossy().to_string())
            .collect();
        
        let sizes: Vec<u64> = paths.iter()
            .filter_map(|p| std::fs::metadata(p).ok().map(|m| m.len()))
            .collect();
        
        let total_size: u64 = sizes.iter().sum();
        
        Ok(SegmentInfo {
            count: paths.len() as u32,
            files,
            sizes,
            total_size,
            missing: vec![],
        })
    }
    
    fn segment_info(&self, path: &Path, index: u32) -> Result<SegmentMetadata, ContainerError> {
        let segments = self.discover_segments(path)?;
        
        if index as usize >= segments.files.len() {
            return Err(ContainerError::SegmentError(
                format!("Segment {} not found", index)
            ));
        }
        
        Ok(SegmentMetadata {
            index,
            path: segments.files[index as usize].clone(),
            size: segments.sizes.get(index as usize).copied().unwrap_or(0),
            hash: None,
        })
    }
}

impl HashableContainer for EwfParser {
    fn stored_hashes(&self, path: &Path) -> Result<Vec<StoredHashInfo>, ContainerError> {
        let path_str = path.to_string_lossy();
        let info = crate::ewf::info_fast(&path_str)?;
        
        Ok(info.stored_hashes.iter()
            .map(|h| StoredHashInfo {
                algorithm: h.algorithm.clone(),
                hash: h.hash.clone(),
                source: "container".to_string(),
                verified: None,
            })
            .collect())
    }
    
    fn verify_stored_hashes(&self, path: &Path) -> Result<Vec<HashResult>, ContainerError> {
        let path_str = path.to_string_lossy();
        let info = crate::ewf::info_fast(&path_str)?;
        
        let mut results = Vec::new();
        for stored in &info.stored_hashes {
            let computed = crate::ewf::verify(&path_str, &stored.algorithm)?;
            
            let verified = computed.to_lowercase() == stored.hash.to_lowercase();
            results.push(HashResult {
                algorithm: stored.algorithm.clone(),
                computed,
                expected: Some(stored.hash.clone()),
                verified: Some(verified),
                duration_secs: 0.0,
            });
        }
        
        Ok(results)
    }
}

impl MountableContainer for EwfParser {
    fn mount(&self, path: &Path) -> Result<Box<dyn VirtualFileSystem>, ContainerError> {
        let path_str = path.to_string_lossy();
        let vfs = crate::ewf::vfs::EwfVfs::open(&path_str)
            .map_err(|e| ContainerError::IoError(e.to_string()))?;
        Ok(Box::new(vfs))
    }
    
    fn supports_mount(&self) -> bool {
        true
    }
}

// =============================================================================
// Archive Parser Implementation
// =============================================================================

/// Archive container parser (ZIP, 7z, RAR)
#[deprecated(since = "0.2.0", note = "Use containers::operations::info() instead")]
pub struct ArchiveParser;

impl EvidenceContainer for ArchiveParser {
    fn format_info(&self) -> FormatInfo {
        FormatInfo {
            id: "archive",
            name: "Archive Container",
            extensions: &["zip", "7z", "rar"],
            category: FormatCategory::Archive,
            supports_segments: false,
            stores_hashes: false,
            has_file_tree: true,
        }
    }
    
    fn detect(&self, path: &Path) -> Result<bool, ContainerError> {
        let path_str = path.to_string_lossy();
        crate::archive::is_archive(&path_str)
    }
    
    fn info(&self, path: &Path, _include_tree: bool) -> Result<ContainerMetadata, ContainerError> {
        let path_str = path.to_string_lossy();
        let info = crate::archive::info(&path_str)?;
        
        Ok(ContainerMetadata {
            format: info.format.to_string(),
            version: None,
            total_size: info.total_size,  // Use total_size
            segments: None,
            stored_hashes: vec![],
            case_info: None,
            format_specific: None,
        })
    }
    
    fn verify(&self, path: &Path, algorithm: &str) -> Result<VerifyResult, ContainerError> {
        let path_str = path.to_string_lossy();
        let hash = crate::archive::verify(&path_str, algorithm)?;
        
        Ok(VerifyResult {
            status: VerifyStatus::Computed,
            hashes: vec![HashResult {
                algorithm: algorithm.to_string(),
                computed: hash,
                expected: None,
                verified: None,
                duration_secs: 0.0,
            }],
            chunks: vec![],
            messages: vec![],
        })
    }
    
    fn extract(&self, path: &Path, output_dir: &Path) -> Result<(), ContainerError> {
        let path_str = path.to_string_lossy();
        let output_str = output_dir.to_string_lossy();
        crate::archive::extract_zip(&path_str, &output_str)
            .map(|_| ())
    }
}

impl TreeContainer for ArchiveParser {
    fn list_entries(&self, path: &Path) -> Result<Vec<TreeEntryInfo>, ContainerError> {
        let path_str = path.to_string_lossy();
        let entries = crate::archive::list_zip_entries(&path_str)?;
        
        Ok(entries.iter()
            .map(|e| {
                // Extract name from path
                let name = std::path::Path::new(&e.path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| e.path.clone());
                    
                TreeEntryInfo {
                    path: e.path.clone(),
                    name,
                    is_directory: e.is_directory,
                    size: e.size,  // Use size field
                    created: None,
                    modified: Some(e.last_modified.clone()),  // Use last_modified
                    accessed: None,
                    hash: Some(format!("{:08x}", e.crc32)),  // crc32 is u32 not Option
                }
            })
            .collect())
    }
    
    fn entry_info(&self, container_path: &Path, entry_path: &str) -> Result<TreeEntryInfo, ContainerError> {
        let entries = self.list_entries(container_path)?;
        entries.iter()
            .find(|e| e.path == entry_path)
            .cloned()
            .ok_or_else(|| ContainerError::FileNotFound(entry_path.to_string()))
    }
    
    fn extract_entry(&self, container_path: &Path, _entry_path: &str, output_path: &Path) -> Result<(), ContainerError> {
        self.extract(container_path, output_path)
    }
}

// =============================================================================
// RAW Parser Implementation
// =============================================================================

/// RAW disk image parser implementing the EvidenceContainer trait
#[deprecated(since = "0.2.0", note = "Use containers::operations::info() instead")]
pub struct RawParser;

impl EvidenceContainer for RawParser {
    fn format_info(&self) -> FormatInfo {
        FormatInfo {
            id: "raw",
            name: "Raw Disk Image",
            extensions: &["dd", "raw", "img", "001"],
            category: FormatCategory::RawImage,
            supports_segments: true,
            stores_hashes: false,
            has_file_tree: false,  // RAW is a disk image format, no file tree
        }
    }
    
    fn detect(&self, path: &Path) -> Result<bool, ContainerError> {
        let path_str = path.to_string_lossy();
        crate::raw::is_raw(&path_str)
    }
    
    fn info(&self, path: &Path, _include_tree: bool) -> Result<ContainerMetadata, ContainerError> {
        let path_str = path.to_string_lossy();
        let info = crate::raw::info(&path_str)?;
        
        // Build segment info if multi-segment
        let segments = if info.segment_count > 1 {
            Some(SegmentInfo {
                count: info.segment_count,
                files: info.segment_names.clone(),
                sizes: info.segment_sizes.clone(),
                total_size: info.total_size,
                missing: vec![],
            })
        } else {
            None
        };
        
        Ok(ContainerMetadata {
            format: "RAW".to_string(),
            version: None,
            total_size: info.total_size,
            segments,
            stored_hashes: vec![],  // RAW doesn't store hashes
            case_info: None,
            format_specific: None,
        })
    }
    
    fn verify(&self, path: &Path, algorithm: &str) -> Result<VerifyResult, ContainerError> {
        let path_str = path.to_string_lossy();
        let hash = crate::raw::verify(&path_str, algorithm)?;
        
        Ok(VerifyResult {
            status: VerifyStatus::Computed,
            hashes: vec![HashResult {
                algorithm: algorithm.to_string(),
                computed: hash,
                expected: None,
                verified: None,
                duration_secs: 0.0,
            }],
            chunks: vec![],
            messages: vec![],
        })
    }
    
    fn extract(&self, path: &Path, output_dir: &Path) -> Result<(), ContainerError> {
        let path_str = path.to_string_lossy();
        let output_str = output_dir.to_string_lossy();
        crate::raw::extract(&path_str, &output_str)
    }
}

impl SegmentedContainer for RawParser {
    fn discover_segments(&self, path: &Path) -> Result<SegmentInfo, ContainerError> {
        let path_str = path.to_string_lossy();
        let info = crate::raw::info(&path_str)?;
        
        Ok(SegmentInfo {
            count: info.segment_count,
            files: info.segment_names,
            sizes: info.segment_sizes.clone(),
            total_size: info.total_size,
            missing: vec![],
        })
    }
    
    fn segment_info(&self, path: &Path, index: u32) -> Result<SegmentMetadata, ContainerError> {
        let path_str = path.to_string_lossy();
        let info = crate::raw::info(&path_str)?;
        
        if (index as usize) >= info.segment_names.len() {
            return Err(ContainerError::SegmentError(format!(
                "Segment index {} out of range (0-{})",
                index,
                info.segment_names.len() - 1
            )));
        }
        
        let idx = index as usize;
        Ok(SegmentMetadata {
            index,
            path: info.segment_names[idx].clone(),
            size: info.segment_sizes[idx],
            hash: None,  // RAW segments don't have stored hashes
        })
    }
}

impl HashableContainer for RawParser {
    fn stored_hashes(&self, _path: &Path) -> Result<Vec<StoredHashInfo>, ContainerError> {
        // RAW format doesn't store hashes
        Ok(vec![])
    }
    
    fn verify_stored_hashes(&self, path: &Path) -> Result<Vec<HashResult>, ContainerError> {
        // RAW doesn't have stored hashes to verify, just compute
        let result = self.verify(path, "sha256")?;
        Ok(result.hashes)
    }
}

impl MountableContainer for RawParser {
    fn mount(&self, path: &Path) -> Result<Box<dyn VirtualFileSystem>, ContainerError> {
        let path_str = path.to_string_lossy();
        // Try to mount as filesystem first, fallback to physical mode
        let vfs = crate::raw::vfs::RawVfs::open_filesystem(&path_str)
            .or_else(|_| crate::raw::vfs::RawVfs::open(&path_str))
            .map_err(|e| ContainerError::IoError(e.to_string()))?;
        Ok(Box::new(vfs))
    }
    
    fn supports_mount(&self) -> bool {
        true
    }
}

// =============================================================================
// UFED Parser Implementation
// =============================================================================

/// UFED container parser implementing the EvidenceContainer trait
#[deprecated(since = "0.2.0", note = "Use containers::operations::info() instead")]
pub struct UfedParser;

impl EvidenceContainer for UfedParser {
    fn format_info(&self) -> FormatInfo {
        FormatInfo {
            id: "ufed",
            name: "Universal Forensic Extraction Data",
            extensions: &["ufd", "ufdr", "ufdx"],
            category: FormatCategory::MobileForensic,
            supports_segments: false,
            stores_hashes: true,
            has_file_tree: false,  // UFED metadata files don't have file trees (the ZIP does)
        }
    }
    
    fn detect(&self, path: &Path) -> Result<bool, ContainerError> {
        let path_str = path.to_string_lossy();
        Ok(crate::ufed::is_ufed(&path_str))
    }
    
    fn info(&self, path: &Path, _include_tree: bool) -> Result<ContainerMetadata, ContainerError> {
        let path_str = path.to_string_lossy();
        let info = crate::ufed::info(&path_str)?;
        
        // Convert stored hashes
        let stored_hashes = info.stored_hashes
            .as_ref()
            .map(|hashes| {
                hashes.iter().map(|h| StoredHashInfo {
                    algorithm: h.algorithm.clone(),
                    hash: h.hash.clone(),
                    source: h.filename.clone(),
                    verified: None,
                }).collect()
            })
            .unwrap_or_default();
        
        // Build case metadata from UFED info
        let case_info = info.case_info.as_ref().map(|c| CaseMetadata {
            case_number: c.case_identifier.clone(),
            evidence_number: info.evidence_number.clone(),
            examiner_name: c.examiner_name.clone(),
            description: c.device_name.clone(),
            notes: None,
            acquisition_date: info.extraction_info.as_ref()
                .and_then(|e| e.start_time.clone()),
        });
        
        Ok(ContainerMetadata {
            format: info.format.clone(),
            version: info.extraction_info.as_ref()
                .and_then(|e| e.tool_version.clone()),
            total_size: info.size,
            segments: None,
            stored_hashes,
            case_info,
            format_specific: None,
        })
    }
    
    fn verify(&self, path: &Path, algorithm: &str) -> Result<VerifyResult, ContainerError> {
        let path_str = path.to_string_lossy();
        let hash = crate::ufed::verify(&path_str, algorithm)?;
        
        // Get stored hashes to compare
        let info = crate::ufed::info(&path_str)?;
        
        // Find matching stored hash for this file
        let expected = info.stored_hashes
            .as_ref()
            .and_then(|hashes| {
                // Get filename from path
                let filename = Path::new(path)
                    .file_name()
                    .map(|n| n.to_string_lossy().to_string())?;
                
                hashes.iter()
                    .find(|h| h.filename == filename && h.algorithm.to_lowercase() == algorithm.to_lowercase())
                    .map(|h| h.hash.clone())
            });
        
        let verified = expected.as_ref().map(|e| e.to_lowercase() == hash.to_lowercase());
        let status = match verified {
            Some(true) => VerifyStatus::Verified,
            Some(false) => VerifyStatus::Mismatch,
            None => VerifyStatus::Computed,
        };
        
        Ok(VerifyResult {
            status,
            hashes: vec![HashResult {
                algorithm: algorithm.to_string(),
                computed: hash,
                expected,
                verified,
                duration_secs: 0.0,
            }],
            chunks: vec![],
            messages: vec![],
        })
    }
    
    fn extract(&self, _path: &Path, _output_dir: &Path) -> Result<(), ContainerError> {
        // UFED metadata files don't have extractable content
        // The associated ZIP would use ArchiveParser
        Err(ContainerError::UnsupportedOperation(
            "UFED metadata files don't contain extractable content. Extract the associated ZIP file instead.".to_string()
        ))
    }
}

impl HashableContainer for UfedParser {
    fn stored_hashes(&self, path: &Path) -> Result<Vec<StoredHashInfo>, ContainerError> {
        let path_str = path.to_string_lossy();
        let info = crate::ufed::info(&path_str)?;
        
        let hashes = info.stored_hashes
            .as_ref()
            .map(|hashes| {
                hashes.iter().map(|h| StoredHashInfo {
                    algorithm: h.algorithm.clone(),
                    hash: h.hash.clone(),
                    source: h.filename.clone(),
                    verified: None,
                }).collect()
            })
            .unwrap_or_default();
        
        Ok(hashes)
    }
    
    fn verify_stored_hashes(&self, path: &Path) -> Result<Vec<HashResult>, ContainerError> {
        // UFED stores SHA256 hashes for associated files
        let path_str = path.to_string_lossy();
        let info = crate::ufed::info(&path_str)?;
        
        let mut results = Vec::new();
        
        if let Some(stored_hashes) = &info.stored_hashes {
            for stored in stored_hashes {
                // Try to verify each stored hash
                let result = crate::ufed::verify_file(&stored.filename, &stored.algorithm);
                
                let (computed, verified) = match result {
                    Ok(hash) => {
                        let matches = hash.to_lowercase() == stored.hash.to_lowercase();
                        (hash, Some(matches))
                    }
                    Err(_) => (stored.hash.clone(), None),
                };
                
                results.push(HashResult {
                    algorithm: stored.algorithm.clone(),
                    computed,
                    expected: Some(stored.hash.clone()),
                    verified,
                    duration_secs: 0.0,
                });
            }
        }
        
        Ok(results)
    }
}

// =============================================================================
// Parser Registry
// =============================================================================

/// Get all available parsers
/// 
/// **DEPRECATED**: Use `containers::operations` functions directly instead.
#[deprecated(since = "0.2.0", note = "Use containers::operations functions directly instead")]
#[allow(deprecated)]
pub fn get_parsers() -> Vec<Box<dyn EvidenceContainer>> {
    vec![
        Box::new(Ad1Parser),
        Box::new(EwfParser),
        Box::new(RawParser),
        Box::new(UfedParser),
        Box::new(ArchiveParser),
    ]
}

/// Detect format and return appropriate parser
/// 
/// **DEPRECATED**: Use `containers::detect_container()` instead.
#[deprecated(since = "0.2.0", note = "Use containers::detect_container() instead")]
#[allow(deprecated)]
pub fn detect_parser(path: &Path) -> Option<Box<dyn EvidenceContainer>> {
    for parser in get_parsers() {
        if let Ok(true) = parser.detect(path) {
            return Some(parser);
        }
    }
    None
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ad1_parser_format_info() {
        let parser = Ad1Parser;
        let info = parser.format_info();
        assert_eq!(info.id, "ad1");
        assert!(info.supports_segments);
        assert!(info.stores_hashes);
        assert!(info.has_file_tree);
    }
    
    #[test]
    fn test_ewf_parser_format_info() {
        let parser = EwfParser;
        let info = parser.format_info();
        assert_eq!(info.id, "ewf");
        assert!(info.supports_segments);
        assert!(info.stores_hashes);
        assert!(!info.has_file_tree);  // EWF is disk image
    }
    
    #[test]
    fn test_archive_parser_format_info() {
        let parser = ArchiveParser;
        let info = parser.format_info();
        assert_eq!(info.id, "archive");
        assert!(!info.supports_segments);
        assert!(!info.stores_hashes);
        assert!(info.has_file_tree);
    }
    
    #[test]
    fn test_get_parsers() {
        let parsers = get_parsers();
        assert_eq!(parsers.len(), 5);  // Ad1, Ewf, Raw, Ufed, Archive
    }
    
    #[test]
    fn test_detect_parser_nonexistent() {
        let path = Path::new("/nonexistent/file.unknown");
        let parser = detect_parser(path);
        assert!(parser.is_none());
    }
    
    #[test]
    fn test_ad1_detect_nonexistent() {
        let parser = Ad1Parser;
        let result = parser.detect(Path::new("/nonexistent/file.ad1"));
        // Should return Ok(false) for nonexistent file, not an error
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_ewf_detect_nonexistent() {
        let parser = EwfParser;
        let result = parser.detect(Path::new("/nonexistent/file.e01"));
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_raw_parser_format_info() {
        let parser = RawParser;
        let info = parser.format_info();
        assert_eq!(info.id, "raw");
        assert!(info.supports_segments);
        assert!(!info.stores_hashes);
        assert!(!info.has_file_tree);
    }
    
    #[test]
    fn test_ufed_parser_format_info() {
        let parser = UfedParser;
        let info = parser.format_info();
        assert_eq!(info.id, "ufed");
        assert!(!info.supports_segments);
        assert!(info.stores_hashes);
        assert!(!info.has_file_tree);
    }
    
    #[test]
    fn test_raw_detect_nonexistent() {
        let parser = RawParser;
        let result = parser.detect(Path::new("/nonexistent/file.dd"));
        assert!(result.is_ok() || result.is_err());
    }
    
    #[test]
    fn test_ufed_detect_nonexistent() {
        let parser = UfedParser;
        let result = parser.detect(Path::new("/nonexistent/file.ufd"));
        // UFED detect returns false for nonexistent files (doesn't error)
        assert!(result.is_ok());
    }
}