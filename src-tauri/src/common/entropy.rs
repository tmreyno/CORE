// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Entropy analysis utilities
//!
//! Calculates Shannon entropy of data to detect encryption, compression,
//! or random data. Essential for forensic analysis to identify encrypted
//! volumes, compressed data, or areas of interest.

use serde::Serialize;

// =============================================================================
// Entropy Calculation
// =============================================================================

/// Calculate Shannon entropy of data (0.0 - 8.0 bits per byte)
///
/// - 0.0 = All bytes identical (e.g., all zeros)
/// - ~4.5 = English text
/// - ~7.5 = Compressed data
/// - ~7.9+ = Encrypted or truly random data
pub fn calculate_entropy(data: &[u8]) -> f64 {
    if data.is_empty() {
        return 0.0;
    }

    // Count byte frequencies
    let mut counts = [0u64; 256];
    for &byte in data {
        counts[byte as usize] += 1;
    }

    let len = data.len() as f64;
    let mut entropy = 0.0;

    for &count in &counts {
        if count > 0 {
            let probability = count as f64 / len;
            entropy -= probability * probability.log2();
        }
    }

    entropy
}

/// Calculate entropy for a slice of data with bounds checking
pub fn calculate_entropy_safe(data: &[u8], offset: usize, length: usize) -> Option<f64> {
    if offset >= data.len() {
        return None;
    }
    let end = (offset + length).min(data.len());
    Some(calculate_entropy(&data[offset..end]))
}

// =============================================================================
// Entropy Classification
// =============================================================================

/// Entropy classification result
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum EntropyClass {
    /// Very low entropy (< 1.0) - likely constant/sparse data
    Constant,
    /// Low entropy (1.0 - 4.0) - structured data, simple patterns
    Structured,
    /// Medium entropy (4.0 - 6.0) - text, code, structured binary
    Text,
    /// High entropy (6.0 - 7.5) - compressed data
    Compressed,
    /// Very high entropy (7.5 - 8.0) - encrypted or random
    Encrypted,
}

impl EntropyClass {
    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            EntropyClass::Constant => "Constant/sparse data (all similar bytes)",
            EntropyClass::Structured => "Structured binary data",
            EntropyClass::Text => "Text or code-like data",
            EntropyClass::Compressed => "Compressed data",
            EntropyClass::Encrypted => "Encrypted or random data",
        }
    }
}

/// Classify entropy value into categories
pub fn classify_entropy(entropy: f64) -> EntropyClass {
    if entropy < 1.0 {
        EntropyClass::Constant
    } else if entropy < 4.0 {
        EntropyClass::Structured
    } else if entropy < 6.0 {
        EntropyClass::Text
    } else if entropy < 7.5 {
        EntropyClass::Compressed
    } else {
        EntropyClass::Encrypted
    }
}

// =============================================================================
// Detection Helpers
// =============================================================================

/// Check if data is likely encrypted (entropy > 7.9)
///
/// Note: This is a heuristic. High entropy can also indicate:
/// - Truly random data
/// - Well-compressed data
/// - Media files (already compressed)
pub fn is_likely_encrypted(data: &[u8]) -> bool {
    if data.len() < 256 {
        return false; // Need enough data for reliable entropy
    }
    calculate_entropy(data) > 7.9
}

/// Check if data is likely compressed
pub fn is_likely_compressed(data: &[u8]) -> bool {
    if data.len() < 256 {
        return false;
    }
    let entropy = calculate_entropy(data);
    entropy > 7.0 && entropy <= 7.9
}

/// Check if data appears to be plaintext
pub fn is_likely_text(data: &[u8]) -> bool {
    if data.len() < 32 {
        return false;
    }
    let entropy = calculate_entropy(data);
    (3.5..=5.5).contains(&entropy)
}

/// Check if data is sparse (mostly zeros or single value)
pub fn is_sparse(data: &[u8]) -> bool {
    if data.is_empty() {
        return true;
    }
    calculate_entropy(data) < 0.5
}

// =============================================================================
// Block Analysis
// =============================================================================

/// Entropy analysis result for a data block
#[derive(Debug, Clone, Serialize)]
pub struct EntropyResult {
    /// Calculated entropy (0.0 - 8.0)
    pub entropy: f64,
    /// Classification
    pub class: EntropyClass,
    /// Human-readable description
    pub description: String,
    /// Size of analyzed data
    pub data_size: usize,
    /// Offset in source (if applicable)
    pub offset: Option<u64>,
}

impl EntropyResult {
    /// Create a new entropy result
    pub fn new(data: &[u8]) -> Self {
        let entropy = calculate_entropy(data);
        let class = classify_entropy(entropy);
        Self {
            entropy,
            class,
            description: class.description().to_string(),
            data_size: data.len(),
            offset: None,
        }
    }

    /// Add offset information
    pub fn with_offset(mut self, offset: u64) -> Self {
        self.offset = Some(offset);
        self
    }
}

/// Analyze entropy across blocks of data
///
/// Useful for finding encrypted regions, sparse areas, or data boundaries
/// in disk images or large files.
#[derive(Debug, Clone, Serialize)]
pub struct BlockEntropyAnalysis {
    /// Block size used for analysis
    pub block_size: usize,
    /// Total data size
    pub total_size: usize,
    /// Number of blocks analyzed
    pub block_count: usize,
    /// Per-block entropy values
    pub blocks: Vec<BlockEntropy>,
    /// Overall statistics
    pub stats: EntropyStats,
}

/// Entropy for a single block
#[derive(Debug, Clone, Serialize)]
pub struct BlockEntropy {
    /// Block index
    pub index: usize,
    /// Offset in data
    pub offset: u64,
    /// Entropy value
    pub entropy: f64,
    /// Classification
    pub class: EntropyClass,
}

/// Statistical summary of entropy analysis
#[derive(Debug, Clone, Serialize)]
pub struct EntropyStats {
    /// Minimum entropy found
    pub min: f64,
    /// Maximum entropy found
    pub max: f64,
    /// Average entropy
    pub mean: f64,
    /// Standard deviation
    pub std_dev: f64,
    /// Count of each classification
    pub class_counts: ClassCounts,
}

/// Count of blocks in each entropy class
#[derive(Debug, Clone, Default, Serialize)]
pub struct ClassCounts {
    pub constant: usize,
    pub structured: usize,
    pub text: usize,
    pub compressed: usize,
    pub encrypted: usize,
}

/// Analyze entropy across fixed-size blocks
pub fn analyze_blocks(data: &[u8], block_size: usize) -> BlockEntropyAnalysis {
    let block_size = block_size.max(64); // Minimum 64 bytes for meaningful entropy
    let mut blocks = Vec::new();
    let mut entropies = Vec::new();
    let mut class_counts = ClassCounts::default();

    for (i, chunk) in data.chunks(block_size).enumerate() {
        let entropy = calculate_entropy(chunk);
        let class = classify_entropy(entropy);
        
        blocks.push(BlockEntropy {
            index: i,
            offset: (i * block_size) as u64,
            entropy,
            class,
        });
        
        entropies.push(entropy);
        
        match class {
            EntropyClass::Constant => class_counts.constant += 1,
            EntropyClass::Structured => class_counts.structured += 1,
            EntropyClass::Text => class_counts.text += 1,
            EntropyClass::Compressed => class_counts.compressed += 1,
            EntropyClass::Encrypted => class_counts.encrypted += 1,
        }
    }

    // Calculate statistics
    let min = entropies.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = entropies.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = if entropies.is_empty() { 
        0.0 
    } else { 
        entropies.iter().sum::<f64>() / entropies.len() as f64 
    };
    
    let variance = if entropies.len() <= 1 {
        0.0
    } else {
        entropies.iter().map(|e| (e - mean).powi(2)).sum::<f64>() / (entropies.len() - 1) as f64
    };
    let std_dev = variance.sqrt();

    BlockEntropyAnalysis {
        block_size,
        total_size: data.len(),
        block_count: blocks.len(),
        blocks,
        stats: EntropyStats {
            min: if min.is_infinite() { 0.0 } else { min },
            max: if max.is_infinite() { 0.0 } else { max },
            mean,
            std_dev,
            class_counts,
        },
    }
}

/// Find regions of high entropy (potential encryption)
pub fn find_high_entropy_regions(data: &[u8], block_size: usize, threshold: f64) -> Vec<(u64, u64)> {
    let analysis = analyze_blocks(data, block_size);
    let mut regions = Vec::new();
    let mut region_start: Option<u64> = None;

    for block in &analysis.blocks {
        if block.entropy >= threshold {
            if region_start.is_none() {
                region_start = Some(block.offset);
            }
        } else if let Some(start) = region_start {
            regions.push((start, block.offset));
            region_start = None;
        }
    }

    // Close final region if needed
    if let Some(start) = region_start {
        regions.push((start, data.len() as u64));
    }

    regions
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_entropy_zeros() {
        let data = vec![0u8; 1000];
        let entropy = calculate_entropy(&data);
        assert!(entropy < 0.01, "All zeros should have ~0 entropy");
    }

    #[test]
    fn test_entropy_random() {
        // Pseudo-random data (not cryptographically random, but high entropy)
        let data: Vec<u8> = (0..1000).map(|i| ((i * 17 + 31) % 256) as u8).collect();
        let entropy = calculate_entropy(&data);
        assert!(entropy > 7.0, "Random-ish data should have high entropy");
    }

    #[test]
    fn test_entropy_text() {
        let data = b"The quick brown fox jumps over the lazy dog. This is sample English text.";
        let entropy = calculate_entropy(data);
        assert!(entropy > 3.5 && entropy < 5.5, "English text entropy ~4.5, got {}", entropy);
    }

    #[test]
    fn test_classify() {
        assert_eq!(classify_entropy(0.5), EntropyClass::Constant);
        assert_eq!(classify_entropy(3.0), EntropyClass::Structured);
        assert_eq!(classify_entropy(4.5), EntropyClass::Text);
        assert_eq!(classify_entropy(7.0), EntropyClass::Compressed);
        assert_eq!(classify_entropy(7.95), EntropyClass::Encrypted);
    }

    #[test]
    fn test_is_sparse() {
        assert!(is_sparse(&[0u8; 100]));
        assert!(is_sparse(&[0xFF; 100]));
        assert!(!is_sparse(b"Hello World"));
    }

    #[test]
    fn test_block_analysis() {
        // Create data with different entropy regions
        let mut data = Vec::new();
        data.extend_from_slice(&[0u8; 256]); // Low entropy
        data.extend_from_slice(b"The quick brown fox jumps over the lazy dog repeatedly many times to fill this block with text."); // Medium
        data.extend_from_slice(&(0u8..=255).collect::<Vec<u8>>()); // High entropy - all byte values
        
        let analysis = analyze_blocks(&data, 64);
        assert!(analysis.block_count > 0);
        assert!(analysis.stats.min < analysis.stats.max);
    }

    #[test]
    fn test_empty_data() {
        assert_eq!(calculate_entropy(&[]), 0.0);
        assert!(is_sparse(&[]));
    }
}
