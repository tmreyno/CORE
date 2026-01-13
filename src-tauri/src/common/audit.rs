// =============================================================================
// CORE-FFX - Forensic File Explorer
// Copyright (c) 2024-2026 CORE-FFX Project Contributors
// Licensed under MIT License - see LICENSE file for details
// =============================================================================

//! Forensic Audit Logging
//!
//! Provides structured logging for forensic chain of custody compliance.
//! All evidence file operations are logged with timestamps and context.

use std::path::Path;
use tracing::{info, warn, span, Level};

/// Log evidence file access for audit trail
pub fn log_evidence_access(
    operation: &str,
    path: &Path,
    file_type: Option<&str>,
    file_size: Option<u64>,
) {
    let _span = span!(
        Level::INFO,
        "evidence_access",
        operation = operation,
        path = %path.display(),
    ).entered();
    
    info!(
        target: "forensic_audit",
        operation = operation,
        path = %path.display(),
        file_type = file_type.unwrap_or("unknown"),
        file_size = file_size.unwrap_or(0),
        timestamp = %chrono::Utc::now().to_rfc3339(),
        "Evidence file accessed"
    );
}

/// Log evidence file hash verification
pub fn log_hash_verification(
    path: &Path,
    algorithm: &str,
    computed_hash: &str,
    expected_hash: Option<&str>,
    verified: Option<bool>,
) {
    let status = match verified {
        Some(true) => "VERIFIED",
        Some(false) => "MISMATCH",
        None => "COMPUTED",
    };
    
    info!(
        target: "forensic_audit",
        operation = "hash_verification",
        path = %path.display(),
        algorithm = algorithm,
        computed_hash = computed_hash,
        expected_hash = expected_hash.unwrap_or("none"),
        status = status,
        timestamp = %chrono::Utc::now().to_rfc3339(),
        "Hash verification completed"
    );
}

/// Log evidence container opened
pub fn log_container_opened(
    path: &Path,
    container_type: &str,
    segments: usize,
) {
    info!(
        target: "forensic_audit",
        operation = "container_open",
        path = %path.display(),
        container_type = container_type,
        segments = segments,
        timestamp = %chrono::Utc::now().to_rfc3339(),
        "Evidence container opened"
    );
}

/// Log report generation
pub fn log_report_generation(
    case_number: &str,
    format: &str,
    output_path: &Path,
) {
    info!(
        target: "forensic_audit",
        operation = "report_generation",
        case_number = case_number,
        format = format,
        output_path = %output_path.display(),
        timestamp = %chrono::Utc::now().to_rfc3339(),
        "Forensic report generated"
    );
}

/// Log security event (blocked operation, validation failure, etc.)
pub fn log_security_event(
    event_type: &str,
    description: &str,
    path: Option<&Path>,
) {
    warn!(
        target: "forensic_audit",
        event_type = "security",
        security_event = event_type,
        description = description,
        path = path.map(|p| p.display().to_string()).unwrap_or_default(),
        timestamp = %chrono::Utc::now().to_rfc3339(),
        "Security event"
    );
}

/// Log data export operation
pub fn log_data_export(
    source: &Path,
    destination: &Path,
    bytes_exported: u64,
) {
    info!(
        target: "forensic_audit",
        operation = "data_export",
        source = %source.display(),
        destination = %destination.display(),
        bytes_exported = bytes_exported,
        timestamp = %chrono::Utc::now().to_rfc3339(),
        "Evidence data exported"
    );
}

/// Audit context for tracking operations on a single evidence item
pub struct EvidenceAuditContext {
    pub evidence_id: String,
    pub path: String,
    pub opened_at: chrono::DateTime<chrono::Utc>,
}

impl EvidenceAuditContext {
    pub fn new(evidence_id: impl Into<String>, path: impl Into<String>) -> Self {
        let ctx = Self {
            evidence_id: evidence_id.into(),
            path: path.into(),
            opened_at: chrono::Utc::now(),
        };
        
        info!(
            target: "forensic_audit",
            operation = "session_start",
            evidence_id = %ctx.evidence_id,
            path = %ctx.path,
            timestamp = %ctx.opened_at.to_rfc3339(),
            "Evidence audit session started"
        );
        
        ctx
    }
    
    pub fn log_operation(&self, operation: &str, details: &str) {
        info!(
            target: "forensic_audit",
            evidence_id = %self.evidence_id,
            operation = operation,
            details = details,
            timestamp = %chrono::Utc::now().to_rfc3339(),
            "Evidence operation"
        );
    }
}

impl Drop for EvidenceAuditContext {
    fn drop(&mut self) {
        let duration = chrono::Utc::now() - self.opened_at;
        info!(
            target: "forensic_audit",
            operation = "session_end",
            evidence_id = %self.evidence_id,
            path = %self.path,
            duration_secs = duration.num_seconds(),
            timestamp = %chrono::Utc::now().to_rfc3339(),
            "Evidence audit session ended"
        );
    }
}

// =============================================================================
// TESTS
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_audit_context_new() {
        let ctx = EvidenceAuditContext::new("EVD-001", "/path/to/evidence.E01");
        assert_eq!(ctx.evidence_id, "EVD-001");
        assert_eq!(ctx.path, "/path/to/evidence.E01");
        // opened_at should be recent (within last second)
        let elapsed = chrono::Utc::now() - ctx.opened_at;
        assert!(elapsed.num_seconds() < 1);
    }

    #[test]
    fn test_audit_context_log_operation() {
        let ctx = EvidenceAuditContext::new("EVD-002", "/test/file.ad1");
        // This should not panic - just logs
        ctx.log_operation("hash", "Computing SHA-256");
        ctx.log_operation("read", "Read 4096 bytes at offset 0");
    }

    #[test]
    fn test_log_evidence_access() {
        let path = PathBuf::from("/evidence/case/disk.E01");
        // Should not panic
        log_evidence_access("open", &path, Some("E01"), Some(1024 * 1024));
        log_evidence_access("close", &path, None, None);
    }

    #[test]
    fn test_log_hash_verification_computed() {
        let path = PathBuf::from("/evidence/test.ad1");
        log_hash_verification(
            &path,
            "SHA-256",
            "abc123def456",
            None,
            None,
        );
    }

    #[test]
    fn test_log_hash_verification_verified() {
        let path = PathBuf::from("/evidence/test.E01");
        log_hash_verification(
            &path,
            "MD5",
            "d41d8cd98f00b204e9800998ecf8427e",
            Some("d41d8cd98f00b204e9800998ecf8427e"),
            Some(true),
        );
    }

    #[test]
    fn test_log_hash_verification_mismatch() {
        let path = PathBuf::from("/evidence/corrupted.raw");
        log_hash_verification(
            &path,
            "SHA-1",
            "abc123",
            Some("def456"),
            Some(false),
        );
    }

    #[test]
    fn test_log_container_opened() {
        let path = PathBuf::from("/evidence/disk.E01");
        log_container_opened(&path, "EWF/E01", 5);
    }

    #[test]
    fn test_log_report_generation() {
        log_report_generation(
            "2024-001",
            "PDF",
            &PathBuf::from("/reports/case-2024-001.pdf"),
        );
    }

    #[test]
    fn test_log_security_event_with_path() {
        let path = PathBuf::from("/etc/passwd");
        log_security_event(
            "path_traversal",
            "Blocked path traversal attempt",
            Some(&path),
        );
    }

    #[test]
    fn test_log_security_event_without_path() {
        log_security_event(
            "invalid_input",
            "Invalid algorithm specified",
            None,
        );
    }

    #[test]
    fn test_log_data_export() {
        log_data_export(
            &PathBuf::from("/evidence/disk.E01"),
            &PathBuf::from("/export/disk.raw"),
            1024 * 1024 * 100, // 100 MB
        );
    }
}
