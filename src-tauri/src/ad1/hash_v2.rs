// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! # AD1 Hash Verification V2 - Based on libad1
//!
//! Hash verification functionality matching libad1_hash.c implementation

use md5::{Md5, Digest as Md5Digest};
use sha1::Sha1;
use serde::Serialize;
use tracing::debug;

use super::reader_v2::{ItemHeader, SessionV2};
use super::types::*;

/// Hash verification result
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum HashResult {
    /// Hash matches stored value
    Ok,
    /// Hash doesn't match stored value
    Mismatch,
    /// No hash found in metadata
    NotFound,
    /// Error during verification
    Error(String),
}

/// Hash type for verification
#[derive(Debug, Clone, Copy, Serialize)]
pub enum HashType {
    Md5,
    Sha1,
}

/// Compute MD5 hash of data
pub fn md5_hash(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Compute SHA1 hash of data
pub fn sha1_hash(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let result = hasher.finalize();
    format!("{:x}", result)
}

/// Check MD5 hash of an item against metadata
///
/// Based on libad1's `check_md5()` function
pub fn check_md5(
    session: &SessionV2,
    item: &ItemHeader,
) -> Result<HashResult, Ad1Error> {
    // Read metadata chain
    let metadata = if item.first_metadata_addr != 0 {
        session.read_metadata_chain(item.first_metadata_addr)?
    } else {
        return Ok(HashResult::NotFound);
    };

    // Find MD5 hash in metadata (category=0x01, key=0x5001)
    let stored_hash = metadata.iter().find_map(|m| {
        if m.category == 0x01 && m.key == 0x5001 {
            Some(String::from_utf8_lossy(&m.data).to_lowercase())
        } else {
            None
        }
    });

    let stored_hash = match stored_hash {
        Some(h) => h,
        None => return Ok(HashResult::NotFound),
    };

    // Read and decompress file data
    let file_data = crate::ad1::operations_v2::decompress_file_data(session, item)
        .map_err(|e| Ad1Error::IoError(format!("Failed to read file data: {}", e)))?;

    // Compute MD5
    let computed_hash = md5_hash(&file_data);

    debug!(
        "MD5 check: {} (stored: {}, computed: {})",
        item.name, stored_hash, computed_hash
    );

    if stored_hash.eq_ignore_ascii_case(&computed_hash) {
        Ok(HashResult::Ok)
    } else {
        Ok(HashResult::Mismatch)
    }
}

/// Check SHA1 hash of an item against metadata
///
/// Based on libad1's `check_sha1()` function
pub fn check_sha1(
    session: &SessionV2,
    item: &ItemHeader,
) -> Result<HashResult, Ad1Error> {
    // Read metadata chain
    let metadata = if item.first_metadata_addr != 0 {
        session.read_metadata_chain(item.first_metadata_addr)?
    } else {
        return Ok(HashResult::NotFound);
    };

    // Find SHA1 hash in metadata (category=0x01, key=0x5002)
    let stored_hash = metadata.iter().find_map(|m| {
        if m.category == 0x01 && m.key == 0x5002 {
            Some(String::from_utf8_lossy(&m.data).to_lowercase())
        } else {
            None
        }
    });

    let stored_hash = match stored_hash {
        Some(h) => h,
        None => return Ok(HashResult::NotFound),
    };

    // Read and decompress file data
    let file_data = crate::ad1::operations_v2::decompress_file_data(session, item)
        .map_err(|e| Ad1Error::IoError(format!("Failed to read file data: {}", e)))?;

    // Compute SHA1
    let computed_hash = sha1_hash(&file_data);

    debug!(
        "SHA1 check: {} (stored: {}, computed: {})",
        item.name, stored_hash, computed_hash
    );

    if stored_hash.eq_ignore_ascii_case(&computed_hash) {
        Ok(HashResult::Ok)
    } else {
        Ok(HashResult::Mismatch)
    }
}

/// Verification result for a single item
#[derive(Debug, Clone, Serialize)]
pub struct ItemVerifyResult {
    pub path: String,
    pub name: String,
    pub is_dir: bool,
    pub size: u64,
    pub hash_type: HashType,
    pub result: HashResult,
    pub stored_hash: Option<String>,
    pub computed_hash: Option<String>,
}

/// Recursively verify all items in container
///
/// Based on libad1's `recurse_md5()` and `recurse_sha1()` functions
pub fn verify_all_items<P: AsRef<std::path::Path>>(
    path: P,
    hash_type: HashType,
) -> Result<Vec<ItemVerifyResult>, Ad1Error> {
    let session = SessionV2::open(path)?;
    
    let first_item_addr = session.logical_header.first_item_addr;
    if first_item_addr == 0 {
        return Ok(Vec::new());
    }

    let root_item = session.read_item_at(first_item_addr)?;
    
    let mut results = Vec::new();
    recurse_verify(&session, &root_item, "", hash_type, &mut results)?;
    
    Ok(results)
}

/// Recursive helper for verification
fn recurse_verify(
    session: &SessionV2,
    item: &ItemHeader,
    parent_path: &str,
    hash_type: HashType,
    results: &mut Vec<ItemVerifyResult>,
) -> Result<(), Ad1Error> {
    let item_path = if parent_path.is_empty() {
        item.name.clone()
    } else {
        format!("{}/{}", parent_path, item.name)
    };

    // Only verify files, not folders
    let is_dir = item.item_type == 0x05;
    
    if !is_dir && item.decompressed_size > 0 {
        let (result, stored_hash, computed_hash) = match hash_type {
            HashType::Md5 => {
                let res = check_md5(session, item)?;
                let metadata = if item.first_metadata_addr != 0 {
                    session.read_metadata_chain(item.first_metadata_addr).ok()
                } else {
                    None
                };
                let stored = metadata.as_ref().and_then(|m| {
                    m.iter().find_map(|e| {
                        if e.category == 0x01 && e.key == 0x5001 {
                            Some(String::from_utf8_lossy(&e.data).to_string())
                        } else {
                            None
                        }
                    })
                });
                let computed = if res == HashResult::Ok || res == HashResult::Mismatch {
                    // Re-compute to get the value
                    let data = crate::ad1::operations_v2::decompress_file_data(session, item).ok();
                    data.map(|d| md5_hash(&d))
                } else {
                    None
                };
                (res, stored, computed)
            }
            HashType::Sha1 => {
                let res = check_sha1(session, item)?;
                let metadata = if item.first_metadata_addr != 0 {
                    session.read_metadata_chain(item.first_metadata_addr).ok()
                } else {
                    None
                };
                let stored = metadata.as_ref().and_then(|m| {
                    m.iter().find_map(|e| {
                        if e.category == 0x01 && e.key == 0x5002 {
                            Some(String::from_utf8_lossy(&e.data).to_string())
                        } else {
                            None
                        }
                    })
                });
                let computed = if res == HashResult::Ok || res == HashResult::Mismatch {
                    let data = crate::ad1::operations_v2::decompress_file_data(session, item).ok();
                    data.map(|d| sha1_hash(&d))
                } else {
                    None
                };
                (res, stored, computed)
            }
        };

        results.push(ItemVerifyResult {
            path: item_path.clone(),
            name: item.name.clone(),
            is_dir,
            size: item.decompressed_size,
            hash_type,
            result,
            stored_hash,
            computed_hash,
        });
    }

    // Recurse into children
    if item.first_child_addr != 0 {
        let children = session.read_children_at(item.first_child_addr)?;
        for child in children {
            recurse_verify(session, &child, &item_path, hash_type, results)?;
        }
    }

    Ok(())
}

/// Verify a single item by address
pub fn verify_item_by_addr<P: AsRef<std::path::Path>>(
    path: P,
    addr: u64,
    hash_type: HashType,
) -> Result<ItemVerifyResult, Ad1Error> {
    let session = SessionV2::open(path)?;
    let item = session.read_item_at(addr)?;
    
    let (result, stored_hash, computed_hash) = match hash_type {
        HashType::Md5 => {
            let res = check_md5(&session, &item)?;
            let metadata = if item.first_metadata_addr != 0 {
                session.read_metadata_chain(item.first_metadata_addr).ok()
            } else {
                None
            };
            let stored = metadata.as_ref().and_then(|m| {
                m.iter().find_map(|e| {
                    if e.category == 0x01 && e.key == 0x5001 {
                        Some(String::from_utf8_lossy(&e.data).to_string())
                    } else {
                        None
                    }
                })
            });
            let computed = if res == HashResult::Ok || res == HashResult::Mismatch {
                let data = crate::ad1::operations_v2::decompress_file_data(&session, &item).ok();
                data.map(|d| md5_hash(&d))
            } else {
                None
            };
            (res, stored, computed)
        }
        HashType::Sha1 => {
            let res = check_sha1(&session, &item)?;
            let metadata = if item.first_metadata_addr != 0 {
                session.read_metadata_chain(item.first_metadata_addr).ok()
            } else {
                None
            };
            let stored = metadata.as_ref().and_then(|m| {
                m.iter().find_map(|e| {
                    if e.category == 0x01 && e.key == 0x5002 {
                        Some(String::from_utf8_lossy(&e.data).to_string())
                    } else {
                        None
                    }
                })
            });
            let computed = if res == HashResult::Ok || res == HashResult::Mismatch {
                let data = crate::ad1::operations_v2::decompress_file_data(&session, &item).ok();
                data.map(|d| sha1_hash(&d))
            } else {
                None
            };
            (res, stored, computed)
        }
    };

    Ok(ItemVerifyResult {
        path: item.name.clone(),
        name: item.name.clone(),
        is_dir: item.item_type == 0x05,
        size: item.decompressed_size,
        hash_type,
        result,
        stored_hash,
        computed_hash,
    })
}
