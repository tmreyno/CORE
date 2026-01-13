// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

// Shared binary reading utilities for forensic container parsers
//
// Provides consistent little-endian binary reading across all formats

use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use tracing::trace;

use crate::containers::ContainerError;

// =============================================================================
// Basic Read Functions (from current position)
// =============================================================================

/// Read a single byte from file at current position
pub fn read_u8(file: &mut File) -> Result<u8, ContainerError> {
    let mut buf = [0u8; 1];
    file.read_exact(&mut buf)
        .map_err(|e| ContainerError::IoError(format!("Failed to read u8: {}", e)))?;
    Ok(buf[0])
}

/// Read u16 little-endian from file at current position
pub fn read_u16_le(file: &mut File) -> Result<u16, ContainerError> {
    let mut buf = [0u8; 2];
    file.read_exact(&mut buf)
        .map_err(|e| ContainerError::IoError(format!("Failed to read u16: {}", e)))?;
    Ok(u16::from_le_bytes(buf))
}

/// Read u32 little-endian from file at current position
pub fn read_u32_le(file: &mut File) -> Result<u32, ContainerError> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)
        .map_err(|e| ContainerError::IoError(format!("Failed to read u32: {}", e)))?;
    Ok(u32::from_le_bytes(buf))
}

/// Read u64 little-endian from file at current position
pub fn read_u64_le(file: &mut File) -> Result<u64, ContainerError> {
    let mut buf = [0u8; 8];
    file.read_exact(&mut buf)
        .map_err(|e| ContainerError::IoError(format!("Failed to read u64: {}", e)))?;
    Ok(u64::from_le_bytes(buf))
}

/// Read u32 big-endian from file at current position
#[allow(dead_code)]
pub fn read_u32_be(file: &mut File) -> Result<u32, ContainerError> {
    let mut buf = [0u8; 4];
    file.read_exact(&mut buf)
        .map_err(|e| ContainerError::IoError(format!("Failed to read u32 BE: {}", e)))?;
    Ok(u32::from_be_bytes(buf))
}

/// Read u64 big-endian from file at current position
#[allow(dead_code)]
pub fn read_u64_be(file: &mut File) -> Result<u64, ContainerError> {
    let mut buf = [0u8; 8];
    file.read_exact(&mut buf)
        .map_err(|e| ContainerError::IoError(format!("Failed to read u64 BE: {}", e)))?;
    Ok(u64::from_be_bytes(buf))
}

// =============================================================================
// Read at Offset Functions (seek + read)
// =============================================================================

/// Read u8 at specific offset
pub fn read_u8_at(file: &mut File, offset: u64) -> Result<u8, ContainerError> {
    trace!(offset, "read_u8_at");
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| ContainerError::IoError(format!("Failed to seek to offset {}: {}", offset, e)))?;
    read_u8(file)
}

/// Read u16 little-endian at specific offset
pub fn read_u16_at(file: &mut File, offset: u64) -> Result<u16, ContainerError> {
    trace!(offset, "read_u16_at");
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| ContainerError::IoError(format!("Failed to seek to offset {}: {}", offset, e)))?;
    read_u16_le(file)
}

/// Read u32 little-endian at specific offset
pub fn read_u32_at(file: &mut File, offset: u64) -> Result<u32, ContainerError> {
    trace!(offset, "read_u32_at");
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| ContainerError::IoError(format!("Failed to seek to offset {}: {}", offset, e)))?;
    read_u32_le(file)
}

/// Read u64 little-endian at specific offset
pub fn read_u64_at(file: &mut File, offset: u64) -> Result<u64, ContainerError> {
    trace!(offset, "read_u64_at");
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| ContainerError::IoError(format!("Failed to seek to offset {}: {}", offset, e)))?;
    read_u64_le(file)
}

// =============================================================================
// String Reading
// =============================================================================

/// Read a fixed-length string at specific offset (null-terminated or fixed size)
pub fn read_string_at(file: &mut File, offset: u64, length: usize) -> Result<String, ContainerError> {
    if length == 0 {
        return Ok(String::new());
    }
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| ContainerError::IoError(format!("Failed to seek to offset {}: {}", offset, e)))?;
    read_string(file, length)
}

/// Read a fixed-length string from current position
pub fn read_string(file: &mut File, length: usize) -> Result<String, ContainerError> {
    if length == 0 {
        return Ok(String::new());
    }
    let mut buf = vec![0u8; length];
    file.read_exact(&mut buf)
        .map_err(|e| ContainerError::IoError(format!("Failed to read string: {}", e)))?;
    
    // Find null terminator
    let end = buf.iter().position(|&b| b == 0).unwrap_or(buf.len());
    Ok(String::from_utf8_lossy(&buf[..end]).to_string())
}

/// Read a length-prefixed string (u32 length + data)
pub fn read_string_with_u32_length(file: &mut File) -> Result<String, ContainerError> {
    let length = read_u32_le(file)? as usize;
    read_string(file, length)
}

// =============================================================================
// Byte Array Reading
// =============================================================================

/// Read exact bytes at specific offset
pub fn read_bytes_at(file: &mut File, offset: u64, length: usize) -> Result<Vec<u8>, ContainerError> {
    if length == 0 {
        return Ok(Vec::new());
    }
    file.seek(SeekFrom::Start(offset))
        .map_err(|e| ContainerError::IoError(format!("Failed to seek to offset {}: {}", offset, e)))?;
    read_bytes(file, length)
}

/// Read exact bytes from current position
pub fn read_bytes(file: &mut File, length: usize) -> Result<Vec<u8>, ContainerError> {
    if length == 0 {
        return Ok(Vec::new());
    }
    let mut buf = vec![0u8; length];
    file.read_exact(&mut buf)
        .map_err(|e| ContainerError::IoError(format!("Failed to read {} bytes: {}", length, e)))?;
    Ok(buf)
}

// =============================================================================
// Utility Functions
// =============================================================================

/// Convert bytes to null-terminated string
pub fn bytes_to_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).to_string()
}

/// Copy string bytes into fixed-size array
pub fn string_to_array<const N: usize>(value: &str) -> [u8; N] {
    let mut buf = [0u8; N];
    let bytes = value.as_bytes();
    let len = bytes.len().min(N);
    buf[..len].copy_from_slice(&bytes[..len]);
    buf
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_read_integers() {
        let mut temp = NamedTempFile::new().unwrap();
        
        // Write test data: u8, u16, u32, u64 in little-endian
        temp.write_all(&[0x42]).unwrap(); // u8
        temp.write_all(&[0x34, 0x12]).unwrap(); // u16 = 0x1234
        temp.write_all(&[0x78, 0x56, 0x34, 0x12]).unwrap(); // u32 = 0x12345678
        temp.write_all(&[0xEF, 0xCD, 0xAB, 0x90, 0x78, 0x56, 0x34, 0x12]).unwrap(); // u64 = 0x1234567890ABCDEF
        
        let mut file = File::open(temp.path()).unwrap();
        
        assert_eq!(read_u8(&mut file).unwrap(), 0x42);
        assert_eq!(read_u16_le(&mut file).unwrap(), 0x1234);
        assert_eq!(read_u32_le(&mut file).unwrap(), 0x12345678);
        assert_eq!(read_u64_le(&mut file).unwrap(), 0x1234567890ABCDEF);
    }

    #[test]
    fn test_read_at_offset() {
        let mut temp = NamedTempFile::new().unwrap();
        temp.write_all(&[0x00, 0x00, 0x00, 0x00, 0x42, 0x00, 0x00, 0x00]).unwrap();
        
        let mut file = File::open(temp.path()).unwrap();
        
        assert_eq!(read_u8_at(&mut file, 4).unwrap(), 0x42);
        assert_eq!(read_u32_at(&mut file, 4).unwrap(), 0x42);
    }

    #[test]
    fn test_read_string() {
        let mut temp = NamedTempFile::new().unwrap();
        temp.write_all(b"hello\x00world").unwrap();
        
        let mut file = File::open(temp.path()).unwrap();
        
        assert_eq!(read_string(&mut file, 11).unwrap(), "hello");
    }

    #[test]
    fn test_bytes_to_string() {
        assert_eq!(bytes_to_string(b"hello\x00world"), "hello");
        assert_eq!(bytes_to_string(b"no null"), "no null");
    }
}
